# spec-menubar.md — TermPaste menu-bar app (v1)

Extends `spec.md`. The app is a macOS menu-bar shell around the same deterministic
`clean()` core. It adds zero cleaning behavior — it only decides *when* to clean and
provides a no-terminal UX. Deterministic-only; no network, no LLM.

## Objective

A menu-bar app (`LSUIElement`, no Dock icon, no window) that watches the macOS
clipboard and, when a newly copied value *looks like agent/terminal output*, replaces
it with `clean(value)`. Result: the user copies in iTerm/Terminal and an ordinary
Cmd+V pastes clean prose, with nothing to run and no terminal open. Distributable to
anyone as a signed, notarized `.dmg`.

## Hard constraints

1. **Reuse `clean()` and `clipboard_action` verbatim** from the `termpaste` lib (path
   dependency). The app never re-implements cleaning.
2. **Local-only, deterministic.** No network, no LLM, no files, no shell config.
3. **Clipboard authority is opt-in and bounded.** Watching is a menu toggle. The app
   writes the clipboard only when it changed a value it decided to act on; it never
   logs, persists, or transmits clipboard contents. Non-text / non-UTF-8 clipboards
   are skipped (inherited from `clipboard_action`).
4. **Terminal-only pre-gate (default on).** In watch mode the app cleans a new value
   only when `looks_like_terminal_output(text)` is true, so it never rewrites a
   markdown snippet or prose the user pasted for its own sake. A menu toggle
   "Clean everything" disables the pre-gate. The pre-gate is deterministic
   (`termpaste` lib), no heurist'y LLM.
5. **No fight, no loop.** After the app writes a cleaned value it records it as
   last-seen and skips it (`clipboard_action` already returns `Skip` when the new
   value equals last-seen and when `clean(x) == x`), so it cannot oscillate.

## Pre-gate rule — `looks_like_terminal_output(text) -> bool`

True if ANY of these deterministic signals is present (outside fenced code is not
distinguished here — this is a gate, not a transform):

- A known response/gutter glyph (`⏺ ⎿ ❯ • │`) at the start of a trimmed line.
- An ANSI CSI escape sequence.
- A box-drawing rule line (3+ of `─ ━ ═ ╌ ╍ ┄ ┅`).
- A **soft-wrap artifact**: a line that ends on a non-terminal character (a lowercase
  letter, digit, comma, or a leading blockquote continuation) immediately followed by
  a single-newline line that begins with whitespace — the terminal wrap pattern.

False for ordinary single-line text, deliberately hand-authored markdown with blank-
line paragraph breaks, and clean prose. Fail-safe: when unsure, return false (don't
clean), so the app never surprises the user; the toggle covers the rest.

## Menu

- **Enable / Disable cleaning** (checkable) — suspends/resumes the watch loop.
- **Clean everything** (checkable) — toggles the terminal-only pre-gate off/on.
- **Clean clipboard now** — one-shot `clean()` of the current clipboard.
- Status line (non-interactive): "Watching" / "Paused"; never shows clipboard text.
- **Quit.**

## Failure modes (guarded)

- Cleaning content the user wanted left alone → guarded by the terminal-only pre-gate
  (default) + reversibility (re-copy the source) + opt-in watching.
- Rewrite loop / fighting the source app → guarded by last-seen tracking + idempotent
  `clean()` (`clipboard_action` returns `Skip` on our own output).
- Crash on non-text clipboard → guarded by `clipboard_action` skipping non-UTF-8.

## Build / distribution

- App is a separate crate `app/` (macOS-only deps), path-dependent on `termpaste`, so
  the CLI + lib keep building on Linux CI. Bundled as a `.app` (`LSUIElement`).
- Ship signed with a Developer ID Application cert + hardened runtime, notarized with
  `notarytool`, stapled — so it opens on a normal double-click. (Signing is the final
  stage; an unsigned local build runs for development/demo.)

## Out of scope (v1)

Auto-update engine; Windows/Linux tray; App Store; login-item autostart (add after v1
via `SMAppService`); the pre-gate learning/anything non-deterministic.
