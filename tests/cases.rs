//! The 10 adversarial cases from TESTCASES.md. See spec.md for rules.
use paste_cleaner::clean;

#[test]
fn case01_soft_wrap_join() {
    let input = "deterministic code is\n  required wherever possible and LLMs are only for bounded judgment calls.";
    let expected =
        "deterministic code is required wherever possible and LLMs are only for bounded judgment calls.";
    assert_eq!(clean(input), expected);
}

#[test]
fn case02_paragraph_break_preserved() {
    let input = "First paragraph that got\nwrapped here.\n\nSecond paragraph also\nwrapped.";
    let expected = "First paragraph that got wrapped here.\n\nSecond paragraph also wrapped.";
    assert_eq!(clean(input), expected);
}

#[test]
fn case03_code_fence_verbatim() {
    let input =
        "Here is code:\n\n```python\ndef f(x):\n    if x:\n        return 1\n\n    return 2\n```";
    let expected =
        "Here is code:\n\n```python\ndef f(x):\n    if x:\n        return 1\n\n    return 2\n```";
    assert_eq!(clean(input), expected);
}

#[test]
fn case04_bold_stripped_literal_asterisks_kept() {
    let input = "**bold** and 2 * 3 and *.py globs and rm *";
    let expected = "bold and 2 * 3 and *.py globs and rm *";
    assert_eq!(clean(input), expected);
}

#[test]
fn case05_horizontal_rule_removed() {
    let input = "Para one\n\n---\n\nPara two";
    let expected = "Para one\n\nPara two";
    assert_eq!(clean(input), expected);
}

#[test]
fn case06_heading_markers_stripped_text_kept() {
    let input = "## Overview\n\nSome body text here.";
    let expected = "Overview\n\nSome body text here.";
    assert_eq!(clean(input), expected);
}

#[test]
fn case07_blockquote_wrapped_rejoined() {
    let input = "> This is a long quote that\n> wrapped across two lines";
    let expected = "> This is a long quote that wrapped across two lines";
    assert_eq!(clean(input), expected);
}

#[test]
fn case08_emphasis_spanning_wrap() {
    let input = "This is **very\n  important** text to keep";
    let expected = "This is very important text to keep";
    assert_eq!(clean(input), expected);
}

#[test]
fn case09_idempotent_on_clean_prose() {
    let clean_text = "Hey Mom, hope you have a great weekend.\n\nTalk soon.";
    let once = clean(clean_text);
    assert_eq!(once, clean_text);
    assert_eq!(clean(&once), once);
}

#[test]
fn case10_preamble_and_emoji() {
    let input = "Here's a longer version:\n\n---\n\nHey Mom, I hope you have an\nabsolutely amazing weekend!\n\nLove you! 💙";
    let expected = "Hey Mom, I hope you have an absolutely amazing weekend!\n\nLove you! 💙";
    assert_eq!(clean(input), expected);
}

// ---- Round 2: 10 more edge cases ----

#[test]
fn case11_inline_code_emphasis_preserved() {
    let input = "The `**markers**` stay verbatim.";
    let expected = "The `**markers**` stay verbatim.";
    assert_eq!(clean(input), expected);
}

#[test]
fn case12_underscore_identifiers_preserved() {
    let input = "Call the __init__ constructor and read my_var here.";
    let expected = "Call the __init__ constructor and read my_var here.";
    assert_eq!(clean(input), expected);
}

#[test]
fn case13_wrapped_numbered_list() {
    let input = "1. First item that\nwraps here\n2. Second item";
    let expected = "1. First item that wraps here\n2. Second item";
    assert_eq!(clean(input), expected);
}

#[test]
fn case14_multiple_blank_lines_collapse() {
    let input = "Para A\n\n\n\nPara B";
    let expected = "Para A\n\nPara B";
    assert_eq!(clean(input), expected);
}

#[test]
fn case15_leading_trailing_blank_and_space_trim() {
    let input = "\n\n  Hello world  \n";
    let expected = "Hello world";
    assert_eq!(clean(input), expected);
}

#[test]
fn case16_setext_heading_underline_dropped() {
    let input = "Project Title\n=============\n\nBody text follows.";
    let expected = "Project Title\n\nBody text follows.";
    assert_eq!(clean(input), expected);
}

#[test]
fn case17_crlf_line_endings() {
    let input = "line that\r\nwrapped across\r\n\r\nnext paragraph";
    let expected = "line that wrapped across\n\nnext paragraph";
    assert_eq!(clean(input), expected);
}

#[test]
fn case18_asterisk_bullets_kept() {
    let input = "* one\n* two";
    let expected = "* one\n* two";
    assert_eq!(clean(input), expected);
}

#[test]
fn case19_code_fence_with_markdown_inside() {
    let input = "```sh\n# not a heading\n- not a list\n**not bold** and 2 * 3\n```";
    let expected = "```sh\n# not a heading\n- not a list\n**not bold** and 2 * 3\n```";
    assert_eq!(clean(input), expected);
}

#[test]
fn case20_empty_and_whitespace_input() {
    assert_eq!(clean(""), "");
    assert_eq!(clean("   \n\n\t\n"), "");
}

#[test]
fn case21_nested_blockquote_collapsed() {
    let input = "> > deeply nested\n> > quote line";
    let expected = "> deeply nested quote line";
    assert_eq!(clean(input), expected);
}

// ---- Clipboard-terminal regression cases ----

#[test]
fn case22_claude_response_gutter_removed() {
    assert_eq!(
        clean("⏺ Here is the cleaned\n  message."),
        "Here is the cleaned message."
    );
}

#[test]
fn case23_codex_response_gutter_removed() {
    assert_eq!(
        clean("• First wrapped\n• response line"),
        "First wrapped response line"
    );
}

#[test]
fn case24_claude_continuation_gutter_removed() {
    assert_eq!(
        clean("⎿ A terminal continuation\n⎿ becomes normal prose."),
        "A terminal continuation becomes normal prose."
    );
}

#[test]
fn case25_terminal_prompt_gutter_removed() {
    assert_eq!(
        clean("❯ A copied answer\n  is ready to paste."),
        "A copied answer is ready to paste."
    );
}

#[test]
fn case26_vertical_terminal_gutter_removed() {
    assert_eq!(
        clean("│ A rendered response\n│ keeps its words."),
        "A rendered response keeps its words."
    );
}

#[test]
fn case27_terminal_gutter_with_indent_removed() {
    assert_eq!(
        clean("  ⏺ Indented terminal\n    output"),
        "Indented terminal output"
    );
}

#[test]
fn case28_gutter_requires_following_whitespace() {
    assert_eq!(clean("•not a terminal gutter"), "•not a terminal gutter");
}

#[test]
fn case29_markdown_bullets_are_preserved() {
    assert_eq!(
        clean("- One useful point\n- Another point"),
        "- One useful point\n- Another point"
    );
}

#[test]
fn case30_gutter_inside_fence_is_verbatim() {
    let input = "```text\n⏺ do not alter this\n• or this\n```";
    assert_eq!(clean(input), input);
}

#[test]
fn case31_inline_gutter_is_not_removed() {
    assert_eq!(
        clean("Keep the inline symbol `⏺` exactly."),
        "Keep the inline symbol `⏺` exactly."
    );
}

#[test]
fn case32_gutter_cleaning_is_idempotent() {
    let once = clean("⏺ A response\n⏺ ready to paste.");
    assert_eq!(clean(&once), once);
}

#[test]
fn case33_gutter_and_markdown_clean_together() {
    assert_eq!(
        clean("⏺ ## Update\n⏺ **Everything** is ready."),
        "Update\nEverything is ready."
    );
}

// ---- Round 4: 12 more agent-terminal domain cases ----

#[test]
fn case34_ansi_sgr_codes_stripped() {
    // Copied from a raw buffer/tmux where color escapes survive the copy.
    let input = "Build \u{1b}[1;32mPASSED\u{1b}[0m in 2.1s";
    let expected = "Build PASSED in 2.1s";
    assert_eq!(clean(input), expected);
}

#[test]
fn case35_box_drawing_rule_dropped() {
    // Claude Code draws separators with box-drawing chars, not ASCII dashes.
    let input =
        "First section.\n\n\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\n\nSecond section.";
    let expected = "First section.\n\nSecond section.";
    assert_eq!(clean(input), expected);
}

#[test]
fn case36_markdown_table_delimiter_dropped_rows_kept() {
    // A whole table joined into one line is gibberish; keep rows, drop the `---` delimiter.
    let input = "| Feature | Status |\n| --- | --- |\n| Reflow | Done |";
    let expected = "| Feature | Status |\n| Reflow | Done |";
    assert_eq!(clean(input), expected);
}

#[test]
fn case37_bold_label_list_item() {
    // Extremely common Claude output shape: "- **Label:** text".
    let input = "- **Reflow:** joins wrapped lines\n- **De-chrome:** strips markers";
    let expected = "- Reflow: joins wrapped lines\n- De-chrome: strips markers";
    assert_eq!(clean(input), expected);
}

#[test]
fn case38_nbsp_indent_reflowed() {
    // Some terminals paste the wrap indent as non-breaking spaces (U+00A0).
    let input = "\u{a0}\u{a0}wrapped by the\nterminal gutter";
    let expected = "wrapped by the terminal gutter";
    assert_eq!(clean(input), expected);
}

#[test]
fn case39_tab_indented_wrap_reflowed() {
    let input = "\tindented output that\ncontinues here";
    let expected = "indented output that continues here";
    assert_eq!(clean(input), expected);
}

#[test]
fn case40_lone_gutter_glyph_line_dropped() {
    // A response glyph on its own line (no following text) is pure chrome.
    let input = "⏺\n⏺ The actual response text here";
    let expected = "The actual response text here";
    assert_eq!(clean(input), expected);
}

#[test]
fn case41_gutter_then_blockquote() {
    let input = "⏺ > Quoted tool output here";
    let expected = "> Quoted tool output here";
    assert_eq!(clean(input), expected);
}

#[test]
fn case42_bom_and_zero_width_stripped() {
    let input = "\u{feff}First real line that\nwrapped";
    let expected = "First real line that wrapped";
    assert_eq!(clean(input), expected);
}

#[test]
fn case43_url_with_underscore_and_asterisk_preserved() {
    let input = "See https://example.com/a_b?x=1*2 for the docs";
    let expected = "See https://example.com/a_b?x=1*2 for the docs";
    assert_eq!(clean(input), expected);
}

#[test]
fn case44_paren_numbered_list_wrapped() {
    let input = "1) Install the deps which\ntakes a while\n2) Run the build";
    let expected = "1) Install the deps which takes a while\n2) Run the build";
    assert_eq!(clean(input), expected);
}

#[test]
fn case45_checkbox_task_list_kept() {
    let input = "- [ ] Write more tests\n- [x] Fix the bug";
    let expected = "- [ ] Write more tests\n- [x] Fix the bug";
    assert_eq!(clean(input), expected);
}
