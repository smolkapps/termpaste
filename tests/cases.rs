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
