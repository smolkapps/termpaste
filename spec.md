# spec.md — Terminal-Paste Markdown Cleaner (v1)

Derived from `PRD.md`. This is the agreed build contract (per AGENTS.md). v1 is **deterministic-only** — no LLM call.

## Objective

`clean(input: &str) -> String`: turn text copied from an AI coding-agent terminal response into clean prose a technical user can paste into a text message, email, or browser chat. Three transforms: **reflow** (undo terminal indent + wrap newlines), **de-chrome** (strip headings, bold, italic, horizontal rules; optionally assistant preamble), and **terminal-gutter removal** (strip known Claude Code/Codex presentation glyphs at the start of prose lines).

The default executable reads stdin and writes stdout. On macOS, an explicit `--watch-clipboard` mode may watch `pbpaste` and replace a newly copied value with `clean(value)`, allowing select/copy → Cmd+V to work without a pipe. It is opt-in, runs until interrupted, and only invokes `pbpaste`/`pbcopy`.

## Hard constraints

1. **Deterministic.** Pure function, no network, no LLM in v1. Same input → same output.
2. **Idempotent.** `clean(clean(x)) == clean(x)`.
3. **Fail-safe / recoverable.** Never delete content on ambiguity. Preamble stripping is allowlist-only and fails open (keeps the line when unsure).
4. **Code is sacred.** Inside a fenced code block **and inside inline `` `...` `` code spans**: no reflow, no indent stripping, no marker stripping — verbatim passthrough.
5. **Markdown-aware emphasis stripping.** Strip only *matched* `**`/`*` emphasis pairs with non-space inner boundaries. Never touch lone/arithmetic/glob asterisks (`2 * 3`, `*.py`, `rm *`). **Underscore emphasis (`__x__`/`_x_`) is NOT stripped** — it collides with code identifiers (`__init__`, `my_var`), and Claude output uses asterisks anyway (fail-safe: preserve).
6. **Order:** reflow BEFORE emphasis stripping (so emphasis split across a wrap is rejoined first).
7. **Clipboard authority is explicit and bounded.** Default mode never touches the clipboard. `--watch-clipboard` reads only the macOS clipboard and writes it only when the deterministic cleaner changes a newly observed value. It never reads or writes files, the network, shell configuration, or browser state.

## Transform rules

- **Reflow:** blocks are separated by blank lines. Within a prose block, single-newline-separated lines are wrap artifacts → join with a single space (trim each line's surrounding whitespace). Blank line → paragraph break (preserved, one blank line).
- **Structural lines never join across their boundary:** headings, horizontal rules, list items (`-`/`*`/`+`/`N.`), blockquote (`>`), code fences. A non-structural line following a list item / blockquote line is a wrapped continuation → joins into that item/quote.
- **De-chrome:** headings → strip `#`+space, keep text as a plain line. Setext underline (`===`, 2+) → drop the underline, keep the title line above. HR (`---`/`***`/`___`, 3+) → drop the line. Bold/italic → strip markers, keep inner text. Keep: blockquote `>`, list markers, backticks/inline code verbatim, emoji, links (keep text / bare URL).
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
- Non-idempotent output → guarded by idempotency test.
- Watching an unchanged clipboard forever / repeatedly rewriting it → guarded by tracking the last observed clipboard value and writing only a changed cleaned result.
- Mutating sensitive copied content unexpectedly → guarded by opt-in watcher mode; default stdin mode has no clipboard authority.

## Out of scope (v1)

LLM newline classifier; recovering lost code fences from Type-A paste; full markdown→prose rendering; trailing sign-off stripping (Q9); browser automation; automatic background-service installation.
