# TermPaste

**Paste terminal output cleanly.** Select a response in Claude Code or Codex within iTerm, then paste clean prose straight into Gemini, Messages, or email. TermPaste removes terminal response glyphs and Markdown chrome, joins terminal-wrapped lines, and preserves paragraphs, lists, links, emoji, and code.

## Make copy → paste work naturally (macOS)

Install once from this checkout, then start the watcher:

```bash
cargo install --path .
termpaste --watch-clipboard
```

Leave that command running. Now select a Claude Code or Codex response in iTerm; when iTerm copies it, TermPaste cleans the clipboard and your normal Cmd+V pastes the cleaned result. Press Ctrl-C to stop it. The watcher is opt-in and changes only newly copied text when cleaning actually changes it.

## One-off use

```bash
termpaste --clipboard                     # clean the current macOS clipboard once
pbpaste | termpaste | pbcopy              # equivalent shell pipeline
cargo test                                # 45 deterministic regression tests
```

## Design

`clean(input) -> String` is a pure, deterministic function (no LLM). It splits input into blank-line-separated blocks, reflows terminal-wrap lines, removes known Claude Code/Codex start-of-line presentation glyphs, then de-chromes Markdown. Reflow runs before emphasis stripping. Fenced code blocks pass through byte-for-byte — no reflow, indent stripping, or marker stripping. The default command reads stdin and writes stdout only; macOS clipboard modes may invoke only `pbpaste` and `pbcopy`.

- **Reads:** stdin only. **Writes:** stdout only. **Executes:** nothing.
- **Kept out of its reach:** the filesystem, network, any LLM call, and the clipboard itself (piping is the caller's choice). Nothing is mutated in place; a bad input can only ever produce a different string, never corrupt a file.

See `spec.md` for the full contract and `TESTCASES.md` for the regression-case rationale.
