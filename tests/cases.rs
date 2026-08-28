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
