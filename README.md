# paste-cleaner

Turn text copied out of a Claude terminal response into clean prose you can paste straight into a text message or email — undo the terminal's indent and word-wrap newlines, and strip markdown scaffolding (headings, bold/italic, horizontal rules, framing preamble) while keeping what a person actually types (paragraphs, lists, blockquotes, backticked code, emoji).

## Run

```bash
cargo test          # 10 adversarial cases (see TESTCASES.md)
echo "$CLIPBOARD" | cargo run --quiet          # clean stdin -> stdout
pbpaste | cargo run --quiet | pbcopy           # clean the macOS clipboard in place
```

## Design

`clean(input) -> String` is a pure, deterministic function (no LLM in v1). It splits the input into blank-line-separated blocks, reflows each prose block (single-newline breaks are word-wrap artifacts → joined with a space; blank lines are real paragraph breaks → kept), then de-chromes: drop horizontal rules, strip heading `#` markers (keep the text), strip only *matched* emphasis pairs so literal asterisks (`2 * 3`, `*.py`, `rm *`) survive. Reflow runs **before** emphasis stripping so `**` split across a wrap is rejoined first. Fenced code blocks are passed through byte-for-byte — no reflow, no indent or marker stripping. A leading framing preamble ("Here's a longer version:") is removed via a narrow allowlist that **fails open** — on any doubt it keeps the line, so it never eats real content like "Here is code:".

- **Reads:** stdin only. **Writes:** stdout only. **Executes:** nothing.
- **Kept out of its reach:** the filesystem, network, any LLM call, and the clipboard itself (piping is the caller's choice). Nothing is mutated in place; a bad input can only ever produce a different string, never corrupt a file.

See `spec.md` for the full contract and `TESTCASES.md` for the ten failure modes it guards against.
