# Native watcher validation — 2026-09-06

Implemented the agreed changeCount gate in `src/main.rs`, with macOS-only
`objc2-app-kit` / `objc2` dependencies. The cleaner and clipboard decision functions
are unchanged. Startup records the count without reading existing content; each
observed change is consumed before pbpaste, including failed/skipped reads. Native
Objective-C exceptions skip the tick, autorelease pools drain each read, and a
successful pbcopy records its resulting count.

## Checks

- Baseline: 64 tests passed. Final: 68 passed, none skipped; the four new gate
  tests were first observed failing against a stub implementation.
- `cargo clippy --all-targets -- -D warnings`, release build, and local installation passed.
- Launch agent restarted with the installed binary; login persistence retained.
- PATH wrappers around real pbpaste/pbcopy observed zero calls during two idle
  seconds, then exactly one read and one write for a dirty copy, with no further
  calls during another idle second. This used an isolated watcher with the launch
  agent temporarily unloaded and restored afterward.
- Native AppKit timing probe: 20 copies per version, 1 ms observation polling.
  Old: median 406.5 ms, range 320.1–470.6 ms. New installed launch agent:
  median 254.4 ms, range 156.9–307.2 ms (37% median reduction).
- Same subprocess-based timing probe before/after (20 copies each): median
  414.9 → 264.1 ms; maxima 524.2 → 409.6 ms. This includes subprocess overhead.
- First timing attempt immediately after restart timed out because the first
  copy preceded watcher initialization; the subsequent 20-copy run passed.

## Installed scheduling fix

The same new binary run directly measured median 65.2 ms (58.6–109.3 ms),
isolating the remaining delay to launch-agent scheduling. Changed the installed
`~/Library/LaunchAgents/com.smolkai.termpaste-watch.plist` ProcessType from
`Background` to `Interactive`, unloaded/reloaded it, and repeated the native probe:
**20/20 copies cleaned; median 61.0 ms, range 57.3–97.9 ms**. This is an 85%
median reduction versus the original installed watcher. The original plist is
backed up in `/tmp/termpaste-watcher-check/original-agent.plist`.

The installed watcher now meets the under-150 ms target in this sample. The 50 ms
native gate and no-idle-fork requirements are met; content still travels through
pbpaste/pbcopy as specified. Scheduling and system load can affect latency; a fast
copy/paste race remains. Real iTerm selection was not driven.
Temporary probes are in `/tmp/termpaste-watcher-check/`; they keep clipboard
contents out of their output. Cargo.lock remains ignored per repository convention.

## Unicode replacement repair after user-reported failure

The latency checks above used ASCII text and did not establish Unicode replacement
correctness. The running agent silently skipped an actual 259-byte UTF-8 clipboard
value, including when that same value was recopied. With launchd's locale-free
environment, pbpaste returned 255 bytes of non-UTF-8 data; explicit
`LC_ALL=en_US.UTF-8` returned the correct 259 UTF-8 bytes. The unchanged text-only
safety guard therefore treated valid copied prose as binary. Background scheduling
was a separate latency issue, not the cause of this replacement failure.

Both pbpaste and pbcopy now explicitly receive `LC_ALL=en_US.UTF-8`, overriding
absent or conflicting inherited locales. The regression test first failed against
the previous implementation, then passed with the fix. All 69 tests pass (none
skipped), Clippy is clean, and the release binary was installed and the launch agent
restarted.

Live verification through the installed agent confirmed exact cleaned bytes for:
the previously stuck clipboard, the user's quoted paragraph, curly apostrophes,
terminal glyphs plus accented text and emoji, Hebrew/Chinese text, and a repeated
dirty copy. The cleaned version of the original clipboard was restored afterward.
No real iTerm selection gesture was automated; the reproduced encoding failure and
actual installed-agent replacement were verified directly on the general pasteboard.
