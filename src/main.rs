//! Thin entry point: read stdin, write cleaned text to stdout.
use std::io::{Read, Write};

fn main() {
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        std::process::exit(1);
    }
    let out = paste_cleaner::clean(&input);
    let _ = std::io::stdout().write_all(out.as_bytes());
}
