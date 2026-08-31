//! Watcher per-tick decision logic (`termpaste::clipboard_action`). See spec.md
//! §7-8: the watch loop must be resilient and text-only — a non-UTF-8 clipboard
//! is skipped, never a crash.
use termpaste::{clipboard_action, ClipboardAction};

#[test]
fn non_utf8_clipboard_is_skipped_not_a_crash() {
    // The real bug: an image / binary / non-UTF-8 clipboard used to exit the
    // watcher (and crash-loop under a KeepAlive launch agent). It must be skipped.
    let raw = [0xff, 0xfe, 0x00, 0x41]; // not valid UTF-8
    assert_eq!(clipboard_action("prev", &raw), ClipboardAction::Skip);
}

#[test]
fn unchanged_clipboard_is_skipped() {
    assert_eq!(
        clipboard_action("hello world", b"hello world"),
        ClipboardAction::Skip
    );
}

#[test]
fn dirty_text_is_replaced_with_cleaned() {
    let raw = "wrapped line one\n  and its continuation".as_bytes();
    assert_eq!(
        clipboard_action("", raw),
        ClipboardAction::Replace("wrapped line one and its continuation".to_string())
    );
}

#[test]
fn already_clean_new_text_is_adopted_not_rewritten() {
    assert_eq!(
        clipboard_action("old value", b"already clean prose."),
        ClipboardAction::Adopt("already clean prose.".to_string())
    );
}
