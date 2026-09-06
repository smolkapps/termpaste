# spec.md — Terminal-Paste Markdown Cleaner (v1)

Derived from `PRD.md`. This is the agreed build contract (per AGENTS.md). v1 is **deterministic-only** — no LLM call.

## Objective

`clean(input: &str) -> String`: turn text copied from an AI coding-agent terminal response into clean prose a technical user can paste into a text message, email, or browser chat. Three transforms: **reflow** (undo terminal indent + wrap newlines), **de-chrome** (strip headings, bold, italic, horizontal rules, blockquote markers; optionally assistant preamble), and **terminal-gutter removal** (strip known Claude Code/Codex presentation glyphs at the start of prose lines).

The `termpaste` executable reads stdin and writes stdout by default. On macOS, an explicit `--watch-clipboard` mode may watch the clipboard and replace a newly copied value with `clean(value)`, allowing select/copy → Cmd+V to work without a pipe. It is opt-in, runs until interrupted, and touches the clipboard only through the macOS pasteboard: it reads `NSPasteboard.generalPasteboard.changeCount` to detect a new copy and invokes `pbpaste`/`pbcopy` only to read/write the value. To keep pasted text current before a fast Cmd+V, the watcher must react within a small fraction of a second of a copy.

## Hard constraints

1. **Deterministic.** Pure function, no network, no LLM in v1. Same input → same output.
2. **Idempotent.** `clean(clean(x)) == clean(x)`.
3. **Fail-safe / recoverable.** Never delete content on ambiguity. Preamble stripping is allowlist-only and fails open (keeps the line when unsure).
4. **Code is sacred.** Inside a fenced code block **and inside inline `` `...` `` code spans**: no reflow, no indent stripping, no marker stripping — verbatim passthrough.
5. **Markdown-aware emphasis stripping.** Strip only *matched* `**`/`*` emphasis pairs with non-space inner boundaries. Never touch lone/arithmetic/glob asterisks (`2 * 3`, `*.py`, `rm *`). **Underscore emphasis (`__x__`/`_x_`) is NOT stripped** — it collides with code identifiers (`__init__`, `my_var`), and Claude output uses asterisks anyway (fail-safe: preserve).
6. **Order:** reflow BEFORE emphasis stripping (so emphasis split across a wrap is rejoined first).
7. **Clipboard authority is explicit and bounded.** Default mode never touches the clipboard. `--watch-clipboard` reads only the macOS clipboard and writes it only when the deterministic cleaner changes a newly observed value. It never reads or writes files, the network, shell configuration, or browser state.
8. **Watcher is resilient and text-only.** Clipboard subprocesses explicitly use UTF-8 (`LC_ALL=en_US.UTF-8`) regardless of the parent locale, including launchd with no locale; valid Unicode clipboard text must not be mistaken for binary or corrupted on write. The watch loop only ever acts on valid-UTF-8 text. A non-text / non-UTF-8 clipboard (image, binary, a non-UTF-8 encoding) is **skipped** — never a decode error, never clobbered. A transient `pbpaste`/`pbcopy` failure is non-fatal: the watcher logs nothing fatal and continues to the next tick rather than exiting. The per-tick decision is the pure function `clipboard_action(last_seen, raw_bytes)`.
9. **Change-count-gated polling (responsiveness + no fork storm).** The watcher polls the pasteboard `changeCount` (a native, subprocess-free read) at a short cadence (~50 ms) and forks `pbpaste` to read the value **only when `changeCount` has advanced** since the last tick. An idle clipboard forks nothing. Every observed change updates the tracked `changeCount` exactly once, so a single copy triggers at most one read regardless of action taken (including a skipped non-text clipboard). Target end-to-end latency from copy to cleaned clipboard: well under ~150 ms.
10. **No self-triggered rewrite loop.** The watcher's own `pbcopy` write advances `changeCount`; the watcher must not treat that as a new user copy. Guarded by (a) recording the `changeCount` after its own write and (b) the existing `last_seen`/idempotency guard in `clipboard_action` (`clean(cleaned) == cleaned` → Adopt/Skip). A copy identical to a value the watcher already cleaned is a no-op.
11. **Clipboard-read authority is unchanged in kind.** Adding the native `changeCount` read introduces a link against AppKit but **no new capability**: the watcher still only reads/writes the general pasteboard, never files, network, shell, or browser state. `changeCount` is a monotonic integer — no clipboard *content* is read through the native path; content is still read via `pbpaste` and written via `pbcopy`.

## Transform rules

- **Reflow:** blocks are separated by blank lines. Within a prose block, single-newline-separated lines are wrap artifacts → join with a single space (trim each line's surrounding whitespace). Blank line → paragraph break (preserved, one blank line).
- **Structural lines never join across their boundary:** headings, horizontal rules, list items (`-`/`*`/`+`/`N.`), code fences. A non-structural line following a list item is a wrapped continuation → joins into that item.
- **De-chrome:** headings → strip `#`+space, keep text as a plain line. Setext underline (`===`, 2+) → drop the underline, keep the title line above. HR (`---`/`***`/`___`, 3+) → drop the line. Bold/italic → strip markers, keep inner text. **Blockquote → strip a leading run of `>` (including nested `> >`) plus the whitespace after each, then reflow the inner text as prose.** A `>` is presentation chrome, not content: the target paste surfaces (SMS, email, Docs, Gemini/ChatGPT web) render a leading `>` as a literal character, never as a quote, and a `>` gutter on a soft-wrapped terminal line is the common real artifact. Only the marker is removed; the inner text is always preserved. Keep: list markers, backticks/inline code verbatim, emoji, links (keep text / bare URL).
- **Preamble (allowlist, fail-open):** drop a leading line matching a small set of obvious framing openers (`Here's ...:`, `Sure, here's ...`, `Here is ...:`) only when it is the first line and clearly framing. Otherwise keep.
- **Terminal gutter:** outside fenced code, strip one leading known presentation glyph plus following whitespace (`⏺`, `⎿`, `❯`, `•`, or `│`), or a glyph alone on its line. This targets terminal UI chrome; ordinary Markdown list markers remain unchanged. A glyph glued to text (`•nospace`) is left alone (fail-open).
- **ANSI escapes:** outside fenced code, strip ANSI CSI sequences (e.g. SGR color codes) that survive copies from raw terminal buffers. Removed before gutter detection so a glyph hidden behind a color code is still found.
- **BOM / zero-width:** remove `U+FEFF` and `U+200B` globally so they don't cling to the first word.
- **Box-drawing rule:** a line of box-drawing horizontals (`─ ━ ═ ╌ ╍ ┄ ┅`, 3+) is dropped like an HR (Claude Code separators).
- **Tables:** rows starting with `|` are kept one-per-line (never joined into gibberish); a delimiter row (`| --- | :--: |`) is dropped. Not rendered to prose (out of scope).

## Failure modes (guarded)

- Over-join eating paragraph breaks → guarded by blank-line = hard break.
- Corrupting code indentation → guarded by fence passthrough.
- Corrupting literal asterisks → guarded by non-space matched-pair emphasis regex.
- Emphasis markers orphaned by a wrap → guarded by reflow-before-strip ordering.
- Deleting real content as "preamble" → guarded by allowlist + fail-open.
- Flattening a deliberate blockquote → accepted by design, bounded to marker-only removal: the `>` is dropped but the inner text is always preserved (never deleted), so content survives; only the (unrendered-on-target) quote styling is lost. Slack-style `>` quoting is sacrificed so the common terminal-wrap `>` artifact cleans correctly.
- Non-idempotent output → guarded by idempotency test.
- Watching an unchanged clipboard forever / repeatedly rewriting it → guarded by tracking the last observed clipboard value and writing only a changed cleaned result.
- Mutating sensitive copied content unexpectedly → guarded by opt-in watcher mode; default stdin mode has no clipboard authority.
- Launchd lacks a UTF-8 locale, causing valid Unicode prose to be skipped or corrupted → guarded by explicit UTF-8 encoding for both pbpaste and pbcopy, tested with absent and conflicting parent locales.
- Watcher crashing on a non-text / non-UTF-8 clipboard (image, binary, non-UTF-8 text), especially fatal under a KeepAlive launch agent (crash-loop) → guarded by `clipboard_action` skipping any clipboard value that is not valid UTF-8 (never a decode error, never clobbered) and by treating transient `pbpaste`/`pbcopy` errors as non-fatal so the watcher keeps running.
- Fork storm / battery drain from forking `pbpaste` on every tick → guarded by the `changeCount` gate: an idle clipboard forks nothing; `pbpaste` runs only when `changeCount` advances.
- Losing a copy to a fast Cmd+V (pasting before the watcher cleans) → mitigated (not eliminated) by the ~50 ms cadence + change-count gate bringing latency well under ~150 ms; a sub-cadence race remains possible if the user pastes within a few tens of ms of copying. Fully race-free cleaning is out of scope for the watcher (would require an on-demand clean-at-paste action).
- Watcher reprocessing its own `pbcopy` write in a loop → guarded by recording `changeCount` after the write and by `clipboard_action` idempotency (constraint 10).
- `changeCount` read failing / AppKit unavailable → treated like a transient read error: skip the tick and continue (never fatal).

## Out of scope (v1)

LLM newline classifier; recovering lost code fences from Type-A paste; full markdown→prose rendering; trailing sign-off stripping (Q9); browser automation; automatic background-service installation.
