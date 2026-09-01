//! Terminal-output pre-gate for the menu-bar app (`looks_like_terminal_output`).
//! See spec-menubar.md §pre-gate. Fail-safe: false when unsure.
use termpaste::looks_like_terminal_output;

#[test]
fn plain_single_line_is_not_terminal() {
    assert!(!looks_like_terminal_output(
        "Just a normal sentence I copied."
    ));
}

#[test]
fn deliberate_markdown_paragraphs_are_not_terminal() {
    // Hand-authored markdown, blank-line paragraph breaks — leave it alone.
    assert!(!looks_like_terminal_output(
        "# My notes\n\nA paragraph here.\n\nAnother paragraph."
    ));
}

#[test]
fn empty_or_blank_is_not_terminal() {
    assert!(!looks_like_terminal_output(""));
    assert!(!looks_like_terminal_output("   \n\n"));
}

#[test]
fn response_glyph_is_terminal() {
    assert!(looks_like_terminal_output("⏺ Here is the response"));
    assert!(looks_like_terminal_output("  ❯ a prompt line"));
}

#[test]
fn ansi_escape_is_terminal() {
    assert!(looks_like_terminal_output(
        "Build \u{1b}[1;32mPASSED\u{1b}[0m in 2.1s"
    ));
}

#[test]
fn box_drawing_rule_is_terminal() {
    assert!(looks_like_terminal_output(
        "Section one\n\u{2500}\u{2500}\u{2500}\u{2500}\nSection two"
    ));
}

#[test]
fn soft_wrap_artifact_is_terminal() {
    // Line ends mid-sentence (comma) and the next line is indented: the wrap pattern.
    assert!(looks_like_terminal_output(
        "fixes it automatically, locally,\n  before Cmd+V."
    ));
}

#[test]
fn blockquote_wrap_continuation_is_terminal() {
    // The exact reported case shape.
    assert!(looks_like_terminal_output(
        "...automatically, locally,\n  > before Cmd+V."
    ));
}

#[test]
fn two_complete_sentences_indented_are_not_wrap() {
    // First line ends a sentence, so an indented next line is not a wrap artifact.
    assert!(!looks_like_terminal_output(
        "First thought.\n  Second thought."
    ));
}
