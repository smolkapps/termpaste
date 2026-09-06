//! A launch agent has no shell locale. Unicode prose must still be cleaned and
//! written intact, rather than skipped as binary or corrupted on output.
#![cfg(unix)]
use std::{fs, os::unix::fs::PermissionsExt, process::Command};

#[test]
fn clipboard_commands_use_utf8_without_relying_on_the_parent_locale() {
    let dir = std::env::temp_dir().join(format!("termpaste-locale-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let read = dir.join("pbpaste");
    fs::write(&read, "#!/bin/sh\nif [ \"$LC_ALL\" = en_US.UTF-8 ]; then\n  printf 'I’ll copy café 👋\\n  and continue'\nelse\n  printf '\\377'\nfi\n").unwrap();
    let write = dir.join("pbcopy");
    fs::write(
        &write,
        "#!/bin/sh\n[ \"$LC_ALL\" = en_US.UTF-8 ] || exit 1\n/bin/cat > \"$OUTPUT\"\n",
    )
    .unwrap();
    for path in [&read, &write] {
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let output = dir.join("output");
    for locale in [None, Some("C")] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_termpaste"));
        command
            .env_clear()
            .env("PATH", &dir)
            .env("OUTPUT", &output)
            .arg("--clipboard");
        if let Some(locale) = locale {
            command.env("LC_ALL", locale);
        }
        assert!(command.status().unwrap().success());
        assert_eq!(
            fs::read_to_string(&output).ok().as_deref(),
            Some("I’ll copy café 👋 and continue")
        );
        fs::remove_file(&output).unwrap();
    }
    fs::remove_dir_all(dir).unwrap();
}
