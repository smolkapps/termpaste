//! CLI entry point. Default mode is a stdin/stdout filter; clipboard access is
//! explicit through `--clipboard` or `--watch-clipboard` on macOS.
use std::{
    env,
    io::{Read, Write},
    process::{Command, Stdio},
    thread,
    time::Duration,
};

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
    let out = paste_cleaner::clean(&input);
    let _ = std::io::stdout().write_all(out.as_bytes());
}

fn clean_clipboard_once() {
    let original = read_clipboard().unwrap_or_else(|error| clipboard_error(error));
    let cleaned = paste_cleaner::clean(&original);
    if cleaned != original {
        write_clipboard(&cleaned).unwrap_or_else(|error| clipboard_error(error));
    }
}

fn watch_clipboard() {
    let mut last_seen = read_clipboard().unwrap_or_else(|error| clipboard_error(error));
    eprintln!("Watching the macOS clipboard. Press Ctrl-C to stop.");

    loop {
        thread::sleep(Duration::from_millis(200));
        let original = read_clipboard().unwrap_or_else(|error| clipboard_error(error));
        if original == last_seen {
            continue;
        }
        let cleaned = paste_cleaner::clean(&original);
        if cleaned != original {
            write_clipboard(&cleaned).unwrap_or_else(|error| clipboard_error(error));
            eprintln!("Cleaned copied terminal text.");
            last_seen = cleaned;
        } else {
            last_seen = original;
        }
    }
}

fn read_clipboard() -> Result<String, String> {
    let output = Command::new("pbpaste")
        .output()
        .map_err(|error| format!("Could not run pbpaste: {error}"))?;
    if output.status.success() {
        String::from_utf8(output.stdout)
            .map_err(|error| format!("Clipboard was not UTF-8: {error}"))
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
    eprintln!("paste-cleaner: {error}");
    std::process::exit(1);
}

fn print_usage() {
    println!("Usage: paste-cleaner [--clipboard | --watch-clipboard]\n\nWithout an option, reads stdin and writes cleaned text to stdout.\n--clipboard        Clean the current macOS clipboard once.\n--watch-clipboard  Keep the clipboard clean after each new copy; Ctrl-C stops it.");
}
