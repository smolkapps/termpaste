# TermPaste

**Paste terminal output cleanly.** Select a response in Claude Code or Codex within iTerm, then paste clean prose straight into Gemini, Messages, or email. TermPaste removes terminal response glyphs and Markdown chrome — headings, emphasis, and blockquote `>` markers — joins terminal-wrapped lines, and preserves paragraphs, lists, links, emoji, and code.

## Download and install (macOS)

TermPaste currently installs **from source**. There is no prebuilt `.dmg`, `.pkg`, or GitHub release download yet.

1. Install Apple's Command Line Tools if needed: `xcode-select --install`.
2. Install Rust and Cargo using [rustup](https://rustup.rs/), then open a new terminal.
3. Download the source and install:

```bash
git clone https://github.com/smolkapps/termpaste.git
cd termpaste
cargo install --path . --force
```

Alternatively, choose **Code → Download ZIP** on GitHub, extract it, open a terminal in the extracted folder, and run the same `cargo install` command.

## Start automatic cleaning

```bash
"$HOME/.cargo/bin/termpaste" --watch-clipboard
```

Leave that command running. Copy a Claude Code or Codex response from your terminal, then paste normally with Cmd+V. In iTerm, selecting text also copies it if **Copy to pasteboard on selection** is enabled; otherwise use Cmd+C. TermPaste cleans newly copied text when cleaning changes it, including Unicode punctuation, accented text, and emoji. Press Ctrl-C to stop it. Starting this command does not configure login autostart.

The watcher checks native `NSPasteboard.changeCount` every 50 ms and runs `pbpaste` only after a change. Idle polling starts no subprocesses. When configuring a launch agent, use `ProcessType = Interactive`; `Background` scheduling can delay cleaning beyond the polling interval. Cleaning is asynchronous: an immediate Cmd+V can still beat it. See [watcher validation](WATCHER-VALIDATION-2026-09-06.md) for measured latency and Unicode replacement checks.

Prefer a menu-bar app? See the [app build instructions](app/README.md). It uses terminal-only detection by default; the CLI watcher above cleans all copied text that the cleaner changes. Run one watcher at a time.

## Update an existing installation

**The September 6, 2026 fix is essential for background users:** older builds could appear to run normally while silently failing to clean text containing Unicode characters. The fix explicitly uses UTF-8 for clipboard reads and writes.

From your existing source checkout:

```bash
git pull --ff-only
cargo install --path . --force
```

**Restart the running watcher after updating.** For a terminal watcher, press Ctrl-C and run the start command again. If you already configured the `com.smolkai.termpaste-watch` launch agent, restart it with:

```bash
launchctl kickstart -k "gui/$(id -u)/com.smolkai.termpaste-watch"
```

For the menu-bar app, quit it, rebuild it using the linked instructions, and reopen it; it bundles its own copy of the CLI.

## One-off use

```bash
termpaste --clipboard                     # clean the current macOS clipboard once
LC_ALL=en_US.UTF-8 pbpaste | termpaste | LC_ALL=en_US.UTF-8 pbcopy
cargo test                                # 69 tests covering cleaning, detection, watcher gating, and locale handling
```

## Design

`clean(input) -> String` is a pure, deterministic function (no LLM). It splits input into blank-line-separated blocks, reflows terminal-wrap lines, removes known Claude Code/Codex start-of-line presentation glyphs, then de-chromes Markdown (strips heading, emphasis, and blockquote `>` markers; drops horizontal rules). Reflow runs before emphasis stripping. Fenced code blocks pass through byte-for-byte — no reflow, indent stripping, or marker stripping. The default command reads stdin and writes stdout only; macOS clipboard modes may invoke only `pbpaste` and `pbcopy`.

- **Reads:** stdin only. **Writes:** stdout only. **Executes:** nothing.
- **Kept out of its reach:** the filesystem, network, any LLM call, and the clipboard itself (piping is the caller's choice). Nothing is mutated in place; a bad input can only ever produce a different string, never corrupt a file.

## How this was built

TermPaste was built with an AI coding agent driven through a spec-first, test-driven loop. The contract lives in `AGENTS.md`: no code without an agreed `spec.md`, deterministic code over model calls, and the model bounded to judgment calls only. Each change lands as a failing test first and then the implementation, so the RED/GREEN commits in the git history are one iteration each, and the 69-test suite is the verification signal the agent can't satisfy without actually meeting the spec.

See `spec.md` for the full contract and `TESTCASES.md` for the regression-case rationale.
