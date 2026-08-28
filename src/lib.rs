//! Terminal-paste markdown cleaner. See spec.md.
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
    let lines: Vec<&str> = input.split('\n').collect();
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
        let t = raw.trim();

        if is_hr(t) {
            if let Some(s) = cur.take() {
                logical.push(s);
            }
            continue; // drop the rule
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

fn is_hr(t: &str) -> bool {
    let compact: String = t.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.len() < 3 {
        return false;
    }
    let first = compact.chars().next().unwrap();
    matches!(first, '-' | '*' | '_') && compact.chars().all(|c| c == first)
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

/// Strip only matched emphasis pairs with non-space inner boundaries, so lone
/// asterisks (`2 * 3`, `*.py`, `rm *`) and snake_case underscores survive.
fn strip_emphasis(s: &str) -> String {
    static BOLD_STAR: OnceLock<Regex> = OnceLock::new();
    static ITAL_STAR: OnceLock<Regex> = OnceLock::new();
    static BOLD_US: OnceLock<Regex> = OnceLock::new();
    static ITAL_US: OnceLock<Regex> = OnceLock::new();

    let bold_star = BOLD_STAR.get_or_init(|| Regex::new(r"\*\*(\S|\S.*?\S)\*\*").unwrap());
    let ital_star = ITAL_STAR.get_or_init(|| Regex::new(r"\*(\S|\S.*?\S)\*").unwrap());
    let bold_us =
        BOLD_US.get_or_init(|| Regex::new(r"(^|[^\w])__(\S|\S.*?\S)__([^\w]|$)").unwrap());
    let ital_us = ITAL_US.get_or_init(|| Regex::new(r"(^|[^\w])_(\S|\S.*?\S)_([^\w]|$)").unwrap());

    let s = bold_star.replace_all(s, "$1").into_owned();
    let s = ital_star.replace_all(&s, "$1").into_owned();
    let s = bold_us.replace_all(&s, "${1}${2}${3}").into_owned();
    let s = ital_us.replace_all(&s, "${1}${2}${3}").into_owned();
    s
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
