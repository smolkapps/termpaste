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

#[derive(PartialEq)]
enum Kind {
    Plain,
    List,
    Quote,
}

/// Reflow a prose block into logical lines: soft (single-newline) breaks join;
/// structural lines (heading/HR/list/quote) bound the join.
fn render_prose_block(lines: &[&str]) -> Option<String> {
    let mut logical: Vec<String> = Vec::new();
    let mut cur: Option<String> = None;
    let mut cur_kind = Kind::Plain;

    for &raw in lines {
        // Claude Code and Codex draw response/output rows with presentation
        // glyphs, sometimes wrapped in ANSI color escapes. Both are terminal
        // chrome, not text the recipient needs. ANSI is removed first so a glyph
        // hidden behind a color code is still recognized. This happens only
        // after fenced blocks have been excluded above.
        let deansi = strip_ansi(raw);
        let degutter = strip_terminal_gutter(&deansi);
        let t = degutter.trim();

        // A line that became empty (e.g. a lone response glyph) is pure chrome.
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
            cur_kind = Kind::List;
            continue;
        }
        if is_blockquote(t) {
            let inner = strip_quote(t);
            match cur.as_mut() {
                Some(s) if cur_kind == Kind::Quote => {
                    s.push(' ');
                    s.push_str(&inner);
                }
                _ => {
                    if let Some(s) = cur.take() {
                        logical.push(s);
                    }
                    cur = Some(format!("> {}", inner));
                    cur_kind = Kind::Quote;
                }
            }
            continue;
        }

        // Plain line: wrapped continuation of the open logical line, or a new one.
        if let Some(s) = cur.as_mut() {
            s.push(' ');
            s.push_str(t);
        } else {
            cur = Some(t.to_string());
            cur_kind = Kind::Plain;
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

fn is_blockquote(t: &str) -> bool {
    t.starts_with('>')
}

fn strip_quote(t: &str) -> String {
    let mut s = t;
    while let Some(r) = s.strip_prefix('>') {
        s = r.trim_start();
    }
    s.to_string()
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
