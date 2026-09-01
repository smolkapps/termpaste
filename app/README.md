# TermPaste.app — macOS menu-bar app

Auto-cleans terminal output the moment you copy it, so you copy in your terminal and
an ordinary Cmd+V pastes clean prose — no command to run, no window. It wraps the same
deterministic `clean()` core as the CLI (all cleaning logic and the terminal-only
pre-gate live in the tested Rust crate; this app only decides *when* to run it).

## Build

Requires macOS with Xcode command-line tools (`swiftc`), plus the `termpaste` CLI it
bundles:

```bash
cargo install --path ..     # build + install the termpaste CLI (bundled into the app)
./build.sh                  # produce TermPaste.app (self-contained)
open TermPaste.app          # or double-click it in Finder
```

The menu bar shows **✂︎** with: **Enable cleaning**, **Clean everything (not just
terminal output)**, **Clean clipboard now**, and **Quit**. The default is terminal-only
— it acts only on copies that look like agent/terminal output (response glyphs, ANSI,
box rules, the soft-wrap pattern), so it never rewrites deliberately-copied markdown or
prose. Toggle **Clean everything** to clean every copy.

## How it decides (no fight, no surprise)

- Polls `NSPasteboard.changeCount`; on a real change it runs `termpaste
  --clipboard-terminal` (or `--clipboard` in "everything" mode).
- `clean()` is idempotent and only rewrites when it changes the value, so the app
  never loops on its own output.
- Non-text clipboards (images, files) are skipped.

## Status / distribution

The core, the pre-gate, and the `--clipboard-terminal` path are covered by the suite
and verified end to end. The bundle is ad-hoc signed today, so a first launch may need
**right-click → Open** (Gatekeeper). Developer ID signing + notarization — so it opens
on a plain double-click for anyone, and can auto-start at login via `SMAppService` — is
the remaining distribution step. A launch/heartbeat log is written to
`~/Library/Logs/termpaste-app.log`.
