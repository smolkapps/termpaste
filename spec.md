# spec.md — Terminal-Paste Markdown Cleaner (v1)

Derived from `PRD.md`. This is the agreed build contract (per AGENTS.md). v1 is **deterministic-only** — no LLM call.

## Objective

`clean(input: &str) -> String`: turn text copied from a Claude terminal response into clean prose a technical user can paste into a text message or email. Two transforms: **reflow** (undo terminal indent + wrap newlines) and **de-chrome** (strip headings, bold, italic, horizontal rules; optionally assistant preamble).

## Hard constraints

1. **Deterministic.** Pure function, no network, no LLM in v1. Same input → same output.
2. **Idempotent.** `clean(clean(x)) == clean(x)`.
3. **Fail-safe / recoverable.** Never delete content on ambiguity. Preamble stripping is allowlist-only and fails open (keeps the line when unsure).
4. **Code is sacred.** Inside a fenced code block: no reflow, no indent stripping, no marker stripping — byte-for-byte passthrough of the inner lines.
5. **Markdown-aware emphasis stripping.** Strip only *matched* `**`/`__`/`*`/`_` emphasis pairs with non-space inner boundaries. Never touch lone/arithmetic/glob asterisks (`2 * 3`, `*.py`, `rm *`).
6. **Order:** reflow BEFORE emphasis stripping (so emphasis split across a wrap is rejoined first).

## Transform rules

- **Reflow:** blocks are separated by blank lines. Within a prose block, single-newline-separated lines are wrap artifacts → join with a single space (trim each line's surrounding whitespace). Blank line → paragraph break (preserved, one blank line).
- **Structural lines never join across their boundary:** headings, horizontal rules, list items (`-`/`*`/`+`/`N.`), blockquote (`>`), code fences. A non-structural line following a list item / blockquote line is a wrapped continuation → joins into that item/quote.
- **De-chrome:** headings → strip `#`+space, keep text as a plain line. HR (`---`/`***`/`___`, 3+) → drop the line. Bold/italic → strip markers, keep inner text. Keep: blockquote `>`, list markers, backticks/inline code verbatim, emoji, links (keep text / bare URL).
- **Preamble (allowlist, fail-open):** drop a leading line matching a small set of obvious framing openers (`Here's ...:`, `Sure, here's ...`, `Here is ...:`) only when it is the first line and clearly framing. Otherwise keep.

## Failure modes (guarded)

- Over-join eating paragraph breaks → guarded by blank-line = hard break.
- Corrupting code indentation → guarded by fence passthrough.
- Corrupting literal asterisks → guarded by non-space matched-pair emphasis regex.
- Emphasis markers orphaned by a wrap → guarded by reflow-before-strip ordering.
- Deleting real content as "preamble" → guarded by allowlist + fail-open.
- Non-idempotent output → guarded by idempotency test.

## Out of scope (v1)

LLM newline classifier; recovering lost code fences from Type-A paste; full markdown→prose rendering; trailing sign-off stripping (Q9); interface/packaging.
