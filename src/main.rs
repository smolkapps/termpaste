//! CLI entry point. Default mode is a stdin/stdout filter; clipboard access is
//! explicit through `--clipboard` or `--watch-clipboard` on macOS.
use std::{
    env,
    io::{Read, Write},
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use termpaste::{clipboard_action, ClipboardAction};

fn main() {
    match env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [] => filter_stdin(),
        [flag] if flag == "--clipboard" => clean_clipboard_once(),
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

fn clean_clipboard_once() {
    let raw = read_clipboard_bytes().unwrap_or_else(|error| clipboard_error(error));
    // Non-text / already-clean clipboards yield Skip/Adopt → do nothing.
    if let ClipboardAction::Replace(cleaned) = clipboard_action("", &raw) {
        write_clipboard(&cleaned).unwrap_or_else(|error| clipboard_error(error));
    }
}

fn watch_clipboard() {
    eprintln!("Watching the macOS clipboard. Press Ctrl-C to stop.");
    // Seed the baseline from whatever is on the clipboard now (empty if it is
    // unreadable or non-text — the loop picks up the next real text copy).
    let mut last_seen = read_clipboard_bytes()
        .ok()
        .and_then(|raw| String::from_utf8(raw).ok())
        .unwrap_or_default();

    loop {
        thread::sleep(Duration::from_millis(200));
        // A transient read failure is non-fatal: skip this tick, keep watching.
        let raw = match read_clipboard_bytes() {
            Ok(raw) => raw,
            Err(_) => continue,
        };
        match clipboard_action(&last_seen, &raw) {
            ClipboardAction::Skip => {}
            ClipboardAction::Replace(cleaned) => {
                if write_clipboard(&cleaned).is_ok() {
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
    println!("Usage: termpaste [--clipboard | --watch-clipboard]\n\nWithout an option, reads stdin and writes cleaned text to stdout.\n--clipboard        Clean the current macOS clipboard once.\n--watch-clipboard  Keep the clipboard clean after each new copy; Ctrl-C stops it.");
}
