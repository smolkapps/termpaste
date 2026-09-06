//! CLI entry point. Default mode is a stdin/stdout filter; clipboard access is
//! explicit through `--clipboard` or `--watch-clipboard` on macOS.
use std::{
    env,
    io::{Read, Write},
    process::{Command, Stdio},
};
use termpaste::{clipboard_action, looks_like_terminal_output, ClipboardAction};

fn main() {
    match env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [] => filter_stdin(),
        [flag] if flag == "--clipboard" => clean_clipboard_once(false),
        [flag] if flag == "--clipboard-terminal" => clean_clipboard_once(true),
        [flag] if flag == "--watch-clipboard" => watch_clipboard(),
        [flag] if flag == "--help" || flag == "-h" => print_usage(),
        _ => {
            eprintln!("Unknown arguments. Use --help for usage.");
            std::process::exit(2);
        }
    }
}

fn filter_stdin() {
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        std::process::exit(1);
    }
    let out = termpaste::clean(&input);
    let _ = std::io::stdout().write_all(out.as_bytes());
}

fn clean_clipboard_once(terminal_only: bool) {
    let raw = read_clipboard_bytes().unwrap_or_else(|error| clipboard_error(error));
    // In terminal-only mode, act only when the clipboard looks like agent/terminal
    // output — so an always-on caller never rewrites deliberately-copied text.
    if terminal_only {
        match std::str::from_utf8(&raw) {
            Ok(text) if looks_like_terminal_output(text) => {}
            _ => return,
        }
    }
    // Non-text / already-clean clipboards yield Skip/Adopt → do nothing.
    if let ClipboardAction::Replace(cleaned) = clipboard_action("", &raw) {
        write_clipboard(&cleaned).unwrap_or_else(|error| clipboard_error(error));
    }
}

#[cfg(not(target_os = "macos"))]
fn watch_clipboard() {
    clipboard_error("Clipboard watching requires macOS".to_string());
}

#[cfg(target_os = "macos")]
fn clipboard_change_count() -> Option<isize> {
    // Drain autoreleased AppKit objects each tick in this long-running CLI.
    objc2::rc::autoreleasepool(|_| {
        objc2::exception::catch(|| objc2_app_kit::NSPasteboard::generalPasteboard().changeCount())
            .ok()
    })
}

#[cfg(target_os = "macos")]
fn watch_clipboard() {
    eprintln!("Watching the macOS clipboard. Press Ctrl-C to stop.");
    // Observe only future copies, without reading existing clipboard content.
    let mut last_count = clipboard_change_count();
    let mut last_seen = String::new();

    loop {
        std::thread::sleep(std::time::Duration::from_millis(50));
        if !observe_change(&mut last_count, clipboard_change_count()) {
            continue;
        }
        // A transient read failure is non-fatal: skip this tick, keep watching.
        let raw = match read_clipboard_bytes() {
            Ok(raw) => raw,
            Err(_) => continue,
        };
        match clipboard_action(&last_seen, &raw) {
            ClipboardAction::Skip => {}
            ClipboardAction::Replace(cleaned) => {
                if write_clipboard(&cleaned).is_ok() {
                    // Consume our own write; if the native read fails, retain
                    // the prior count and let the idempotency guard handle it.
                    if let Some(count) = clipboard_change_count() {
                        last_count = Some(count);
                    }
                    eprintln!("Cleaned copied terminal text.");
                    last_seen = cleaned;
                }
            }
            ClipboardAction::Adopt(text) => last_seen = text,
        }
    }
}

fn read_clipboard_bytes() -> Result<Vec<u8>, String> {
    let output = Command::new("pbpaste")
        .output()
        .map_err(|error| format!("Could not run pbpaste: {error}"))?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err("pbpaste failed".to_string())
    }
}

fn write_clipboard(value: &str) -> Result<(), String> {
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Could not run pbcopy: {error}"))?;
    child
        .stdin
        .take()
        .ok_or_else(|| "Could not open pbcopy input".to_string())?
        .write_all(value.as_bytes())
        .map_err(|error| format!("Could not write to pbcopy: {error}"))?;
    if child
        .wait()
        .map_err(|error| format!("Could not wait for pbcopy: {error}"))?
        .success()
    {
        Ok(())
    } else {
        Err("pbcopy failed".to_string())
    }
}

fn clipboard_error(error: String) -> ! {
    eprintln!("termpaste: {error}");
    std::process::exit(1);
}

fn print_usage() {
    println!("Usage: termpaste [--clipboard | --clipboard-terminal | --watch-clipboard]\n\nWithout an option, reads stdin and writes cleaned text to stdout.\n--clipboard           Clean the current macOS clipboard once.\n--clipboard-terminal  Clean the clipboard once, only if it looks like terminal output.\n--watch-clipboard     Keep the clipboard clean after each new copy; Ctrl-C stops it.");
}

#[cfg(any(target_os = "macos", test))]
fn observe_change(last_count: &mut Option<isize>, current: Option<isize>) -> bool {
    let Some(current) = current else {
        return false;
    };
    // Consume before pbpaste, including skipped content and failed reads.
    last_count
        .replace(current)
        .is_some_and(|previous| previous != current)
}

#[cfg(test)]
mod tests {
    use super::observe_change;

    #[test]
    fn startup_and_idle_do_not_read_clipboard() {
        let mut count = None;
        assert!(!observe_change(&mut count, Some(10)));
        assert_eq!(count, Some(10));
        assert!(!observe_change(&mut count, Some(10)));
    }

    #[test]
    fn each_change_is_consumed_even_if_content_read_fails() {
        let mut count = Some(10);
        assert!(observe_change(&mut count, Some(11)));
        assert!(!observe_change(&mut count, Some(11)));
        assert!(observe_change(&mut count, Some(14)));
    }

    #[test]
    fn native_failure_preserves_baseline_until_recovery() {
        let mut count = Some(10);
        assert!(!observe_change(&mut count, None));
        assert_eq!(count, Some(10));
        assert!(observe_change(&mut count, Some(11)));
    }

    #[test]
    fn recorded_self_write_does_not_trigger_another_read() {
        let mut count = Some(12);
        assert!(!observe_change(&mut count, Some(12)));
        assert!(observe_change(&mut count, Some(13)));
    }
}
