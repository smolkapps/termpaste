//! TermPaste: clean terminal output before pasting. See spec.md.
//! v1: deterministic-only. Public surface is `clean`.
//!
//! Pipeline: split into blocks (blank-line separated; fenced code passed through
//! verbatim), reflow each prose block into logical lines, de-chrome (drop HR,
//! strip heading markers), strip matched emphasis, then drop a leading framing
//! preamble (allowlist, fail-open). Blocks are re-joined with one blank line.

use regex::Regex;
use std::sync::OnceLock;

/// Clean terminal-pasted Claude output into message-ready prose.
pub fn clean(input: &str) -> String {
    // Strip BOM / zero-width chars that survive some terminal copies and would
    // otherwise cling to the first word.
    let normalized = input.replace(['\u{feff}', '\u{200b}'], "");
    let lines: Vec<&str> = normalized.split('\n').collect();
    let mut blocks: Vec<String> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let ts = line.trim_start();

        // Fenced code: passthrough verbatim, including the fence lines.
        if ts.starts_with("```") || ts.starts_with("~~~") {
            let mut code = vec![line];
            i += 1;
            while i < lines.len() {
                let l = lines[i];
                code.push(l);
                i += 1;
                let ct = l.trim_start();
                if ct.starts_with("```") || ct.starts_with("~~~") {
                    break;
                }
            }
            blocks.push(code.join("\n"));
            continue;
        }

        // Blank line: block separator.
        if line.trim().is_empty() {
            i += 1;
            continue;
        }

        // Prose block: consecutive non-blank, non-fence lines.
        let mut para: Vec<&str> = Vec::new();
        while i < lines.len() {
            let l = lines[i];
            let lts = l.trim_start();
            if l.trim().is_empty() || lts.starts_with("```") || lts.starts_with("~~~") {
                break;
            }
            para.push(l);
            i += 1;
        }
        if let Some(r) = render_prose_block(&para) {
            blocks.push(r);
        }
    }

    // Preamble strip: leading framing only, fail-open.
    if !blocks.is_empty() && is_preamble(&blocks[0]) {
        blocks.remove(0);
    }

    blocks.join("\n\n")
}

/// What the clipboard watcher should do for one observed clipboard value.
#[derive(Debug, PartialEq, Eq)]
pub enum ClipboardAction {
    /// Do nothing: the value is unchanged, or it is non-text / non-UTF-8 content
    /// that must not be touched.
    Skip,
    /// Overwrite the clipboard with this cleaned value.
    Replace(String),
    /// Already clean; adopt it as the new last-seen baseline without rewriting.
    Adopt(String),
}

/// Decide the watcher's action for a freshly observed clipboard, given the last
/// value it acted on and the raw bytes now on the clipboard. A non-UTF-8 clipboard
/// (image, binary, a non-UTF-8 encoding) is skipped — never decoded with an error
/// and never clobbered — so the watch loop cannot crash on it (which, under a
/// KeepAlive launch agent, would otherwise crash-loop). Keeping this a pure
/// function makes the watcher's core logic testable without touching pbpaste.
pub fn clipboard_action(last_seen: &str, raw: &[u8]) -> ClipboardAction {
    let text = match std::str::from_utf8(raw) {
        Ok(t) => t,
        Err(_) => return ClipboardAction::Skip,
    };
    if text == last_seen {
        return ClipboardAction::Skip;
    }
    let cleaned = clean(text);
    if cleaned != text {
        ClipboardAction::Replace(cleaned)
    } else {
        ClipboardAction::Adopt(text.to_string())
    }
}

/// Deterministic pre-gate for the menu-bar app: does this text look like agent /
/// terminal output worth cleaning? Fail-safe — returns false when unsure, so an
/// always-on watcher never rewrites ordinary copied text (a markdown snippet, plain
/// prose). See spec-menubar.md. The app applies this only in its default
/// "terminal-only" mode; a "Clean everything" toggle bypasses it.
pub fn looks_like_terminal_output(text: &str) -> bool {
    // ANSI escape anywhere is a strong terminal signal.
    if text.contains('\u{1b}') {
        return true;
    }
    let lines: Vec<&str> = text.split('\n').collect();
    for (i, raw) in lines.iter().enumerate() {
        let t = raw.trim_start();
        // A response/gutter glyph at the start of a line.
        if let Some(first) = t.chars().next() {
            if matches!(first, '⏺' | '⎿' | '❯' | '•' | '│') {
                return true;
            }
        }
        // A box-drawing rule line (Claude Code separators).
        let compact: String = t.chars().filter(|c| !c.is_whitespace()).collect();
        if compact.chars().count() >= 3
            && compact
                .chars()
                .all(|c| matches!(c, '─' | '━' | '═' | '╌' | '╍' | '┄' | '┅'))
        {
            return true;
        }
        // Soft-wrap artifact: this line does not end a sentence and the next line
        // (single newline) is indented with content — the terminal wrap pattern.
        if let Some(next) = lines.get(i + 1) {
            let ends_sentence = raw
                .trim_end()
                .chars()
                .last()
                .is_none_or(|c| matches!(c, '.' | '!' | '?'));
            let next_indented = next.starts_with([' ', '\t']) && !next.trim().is_empty();
            if !ends_sentence && next_indented {
                return true;
            }
        }
    }
    false
}

/// Reflow a prose block into logical lines: soft (single-newline) breaks join;
/// structural lines (heading/HR/list) bound the join.
fn render_prose_block(lines: &[&str]) -> Option<String> {
    let mut logical: Vec<String> = Vec::new();
    let mut cur: Option<String> = None;

    for &raw in lines {
        // Claude Code and Codex draw response/output rows with presentation
        // glyphs, sometimes wrapped in ANSI color escapes. Both are terminal
        // chrome, not text the recipient needs. ANSI is removed first so a glyph
        // hidden behind a color code is still recognized. This happens only
        // after fenced blocks have been excluded above.
        let deansi = strip_ansi(raw);
        let degutter = strip_terminal_gutter(&deansi);
        // A leading run of blockquote markers (`>`, incl. nested `> >`) is
        // presentation chrome, not content: the target paste surfaces render `>`
        // literally, and a `>` gutter on a soft-wrapped terminal line is the
        // common artifact. Strip the markers so the inner text reflows as ordinary
        // prose. Fenced code is excluded above and an inline-code `>` is never at
        // line start, so only real leading quote markers are removed here.
        let dequote = strip_leading_blockquotes(degutter);
        let t = dequote.trim();

        // A line that became empty (a lone response glyph or bare `>`) is chrome.
        if t.is_empty() {
            continue;
        }

        if is_hr(t) || is_box_rule(t) {
            if let Some(s) = cur.take() {
                logical.push(s);
            }
            continue; // drop the rule
        }
        if is_setext_underline(t) {
            if let Some(s) = cur.take() {
                logical.push(s);
            }
            continue; // keep the title line above, drop the `===` underline
        }
        if is_table_row(t) {
            if let Some(s) = cur.take() {
                logical.push(s);
            }
            if !is_table_delimiter(t) {
                logical.push(t.to_string()); // keep data rows, drop the `|---|` separator
            }
            continue;
        }
        if is_heading(t) {
            if let Some(s) = cur.take() {
                logical.push(s);
            }
            logical.push(strip_heading(t));
            continue;
        }
        if is_list_item(t) {
            if let Some(s) = cur.take() {
                logical.push(s);
            }
            cur = Some(t.to_string());
            continue;
        }

        // Plain line (now including de-chromed blockquote text): a wrapped
        // continuation of the open logical line, or the start of a new one.
        if let Some(s) = cur.as_mut() {
            s.push(' ');
            s.push_str(t);
        } else {
            cur = Some(t.to_string());
        }
    }
    if let Some(s) = cur.take() {
        logical.push(s);
    }
    if logical.is_empty() {
        return None;
    }
    let stripped: Vec<String> = logical.iter().map(|l| strip_emphasis(l)).collect();
    Some(stripped.join("\n"))
}

/// Remove one recognized terminal UI gutter from a prose line. Requiring
/// whitespace after the glyph keeps identifiers and ordinary text fail-open.
fn strip_terminal_gutter(raw: &str) -> &str {
    let trimmed = raw.trim_start();
    const GUTTERS: [char; 5] = ['⏺', '⎿', '❯', '•', '│'];
    for gutter in GUTTERS {
        if let Some(rest) = trimmed.strip_prefix(gutter) {
            // Strip when followed by whitespace, or when the glyph is alone on
            // the line (a bare response marker). A glyph glued to text like
            // "•nospace" is left alone (fail-open).
            if rest.is_empty() || rest.chars().next().is_some_and(char::is_whitespace) {
                return rest.trim_start();
            }
        }
    }
    trimmed
}

/// Remove ANSI CSI escape sequences (e.g. SGR color codes) that survive copies
/// from raw terminal buffers.
fn strip_ansi(s: &str) -> String {
    static ANSI: OnceLock<Regex> = OnceLock::new();
    let ansi = ANSI.get_or_init(|| Regex::new(r"\x1b\[[0-9;?=]*[A-Za-z]").unwrap());
    ansi.replace_all(s, "").into_owned()
}

/// Box-drawing horizontal line (Claude Code separators) — dropped like an HR.
fn is_box_rule(t: &str) -> bool {
    let compact: String = t.chars().filter(|c| !c.is_whitespace()).collect();
    compact.chars().count() >= 3
        && compact
            .chars()
            .all(|c| matches!(c, '─' | '━' | '═' | '╌' | '╍' | '┄' | '┅'))
}

fn is_table_row(t: &str) -> bool {
    t.starts_with('|')
}

/// A table delimiter row like `| --- | :--: |` — presentation only, dropped.
fn is_table_delimiter(t: &str) -> bool {
    t.starts_with('|') && t.contains('-') && t.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
}

fn is_hr(t: &str) -> bool {
    let compact: String = t.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.len() < 3 {
        return false;
    }
    let first = compact.chars().next().unwrap();
    matches!(first, '-' | '*' | '_') && compact.chars().all(|c| c == first)
}

fn is_setext_underline(t: &str) -> bool {
    t.len() >= 2 && t.chars().all(|c| c == '=')
}

fn is_heading(t: &str) -> bool {
    let hashes = t.chars().take_while(|&c| c == '#').count();
    (1..=6).contains(&hashes) && t[hashes..].starts_with(' ')
}

fn strip_heading(t: &str) -> String {
    t.trim_start_matches('#').trim_start().to_string()
}

fn is_list_item(t: &str) -> bool {
    if t.strip_prefix("- ").is_some()
        || t.strip_prefix("* ").is_some()
        || t.strip_prefix("+ ").is_some()
    {
        return true;
    }
    let digits = t.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0 {
        let after = &t[digits..];
        return after.starts_with(". ") || after.starts_with(") ");
    }
    false
}

/// Remove a leading run of blockquote markers (`>`, including nested `> >`) plus
/// the whitespace after each, returning the inner text as a subslice (no
/// allocation). Only the marker is removed, never the content. Callers apply this
/// only outside fenced code, and an inline-code `>` is never at line start, so
/// this touches only real leading quote markers.
fn strip_leading_blockquotes(s: &str) -> &str {
    let mut cur = s.trim_start();
    while let Some(rest) = cur.strip_prefix('>') {
        cur = rest.trim_start();
    }
    cur
}

/// Strip matched `*`/`**` emphasis, but keep inline-code (backtick) spans
/// verbatim. Underscore emphasis is intentionally NOT stripped — it collides
/// with code identifiers (`__init__`, `my_var`), and Claude uses `*`/`**` anyway.
fn strip_emphasis(s: &str) -> String {
    let mut out = String::new();
    // Even segments are outside inline code; odd segments are inside it.
    for (i, part) in s.split('`').enumerate() {
        if i > 0 {
            out.push('`');
        }
        if i % 2 == 0 {
            out.push_str(&strip_star_emphasis(part));
        } else {
            out.push_str(part); // inside inline code: verbatim
        }
    }
    out
}

/// Strip matched `*`/`**` pairs with non-space inner boundaries, so lone
/// asterisks (`2 * 3`, `*.py`, `rm *`) survive.
fn strip_star_emphasis(s: &str) -> String {
    static BOLD_STAR: OnceLock<Regex> = OnceLock::new();
    static ITAL_STAR: OnceLock<Regex> = OnceLock::new();
    let bold_star = BOLD_STAR.get_or_init(|| Regex::new(r"\*\*(\S|\S.*?\S)\*\*").unwrap());
    let ital_star = ITAL_STAR.get_or_init(|| Regex::new(r"\*(\S|\S.*?\S)\*").unwrap());
    let s = bold_star.replace_all(s, "$1").into_owned();
    ital_star.replace_all(&s, "$1").into_owned()
}

/// Leading framing preamble detector. Narrow allowlist, fails open (keeps the
/// line on any doubt) so it never deletes real content like "Here is code:".
fn is_preamble(block: &str) -> bool {
    if block.contains('\n') {
        return false;
    }
    let b = block.trim().to_lowercase();
    const MARKERS: [&str; 11] = [
        "longer version",
        "shorter version",
        "revised version",
        "updated version",
        "cleaned up version",
        "cleaned-up version",
        "here's a draft",
        "here's the rewrite",
        "here you go",
        "sure, here's",
        "sure! here's",
    ];
    MARKERS.iter().any(|m| b.contains(m))
}
