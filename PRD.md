# PRD — Terminal-Paste Markdown Cleaner

**Status:** Draft · **Owner:** Michael · **Date:** 2026-08-28
**One-liner:** Turn text copied out of a Claude terminal response into clean prose a person can paste straight into a text message or email.

## 0. Primary user & use case

A **technical user** who talks to Claude in the terminal, then wants to **paste the output into a text message or email**. They don't want it to look like it came from an AI or a markdown document — they want it to read the way _they_ would have written it: flowing prose, no formatting scaffolding, emoji and lightweight punctuation intact. This use case, not "preserve markdown fidelity," is the north star. Two transforms serve it:

1. **Reflow** — undo terminal indent + wrap newlines (the §1 case).
2. **De-chrome** — strip document/AI scaffolding that has no place in a message: headings, bold/italic, horizontal rules, and (open) the assistant's framing preamble (the §7 "Hey Mom" case).

---

## 1. Problem

When you copy text from a terminal, two artifacts get introduced:

1. **Leading indent** — every line gains a fixed left margin (e.g. two spaces) from the way the response is rendered.
2. **Wrap newlines** — the terminal inserts a hard `\n` wherever it word-wrapped a line to fit the window width.

The result is text that is broken into fragments that don't correspond to the author's real sentences or paragraphs. We want to reconstruct the text **as it read in the console**: one flowing body, with genuine structure (paragraphs, blockquotes, lists, code) preserved, and rendering artifacts removed.

**Illustrative case:**

```
INPUT
deterministic code is
  required wherever possible and LLMs are only for bounded judgment calls.

EXPECTED
deterministic code is required wherever possible and LLMs are only for bounded judgment calls.
```

The two source lines were one sentence, split by a wrap. The fix joins them and drops the indent.

## 2. Goals

- Reconstruct flowing text from terminal-copied output: **un-indent** + **un-wrap soft line breaks**.
- Preserve the lightweight structure a person actually uses in a message: paragraph breaks, blockquotes (`>`), lists, code (backticks), emoji.
- **De-chrome:** remove document/AI scaffolding a person wouldn't type into a message — headings, bold, italic, horizontal rules.
- Be **idempotent**: running it on already-clean text is a no-op.
- Handle **both input types** (see §4).

## 3. Non-goals

- Not a full markdown→plaintext renderer (we keep markdown).
- Not a lossless round-trip (un-wrapping is inherently lossy; goal is readability, not exact reversal).
- Not reconstructing the original terminal width (unrecoverable from a paste; we infer breaks from grammar instead).
- Interface (CLI/clipboard/lib/extension) is **deliberately unspecified** in this PRD — decided at build time.

## 4. Inputs

Two input types, both supported. They differ in a way that materially changes markdown handling (see §6, Asymmetry).

| Type                           | Description                                                                                                                                                                                                                                                         | Priority  |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- |
| **A — Terminal-selected text** | User selects part of a rendered response in the terminal and copies. Has leading indent + wrap newlines. **Emphasis markers (`**`, `*`) are already gone** — bold/italic rendered as ANSI styling, which is stripped on copy. This is the primary, motivating case. | Primary   |
| **B — `/copy` markdown**       | Claude's `/copy` yields the clean markdown source: no indent, no wrap newlines, but **all markdown markers present** (`**bold**`, backticks, etc.).                                                                                                                 | Secondary |

The tool must detect or gracefully handle both without the user declaring which.

## 5. The core: soft- vs hard-newline classification

The load-bearing decision. A newline is either:

- **Soft (wrap artifact)** → delete it, join the two lines with a single space.
- **Hard (intentional)** → keep it (paragraph break, list item boundary, blockquote line, code line, heading).

**Deterministic criterion (default):** join across a newline when the break is _not_ a grammatical/structural boundary — judged by the rules of English grammar. Concretely, treat a break as **soft** when the line before it does not end a grammatical unit and the next line continues it. Heuristic signals:

- Previous line does **not** end in sentence-final punctuation (`.`, `?`, `!`, `:`) → likely soft.
- Next line begins lowercase / mid-clause → likely soft.
- A blank line between them → **hard** (paragraph break), never join.
- Either line is a list item, heading, blockquote, or inside a code fence → **hard**, never join across the boundary.

**Bounded LLM fallback (optional):** for genuinely ambiguous breaks the deterministic heuristic can't resolve, an LLM may classify _soft vs hard for that single break only_. It is bounded (per AGENTS.md): it decides join-or-not on one boundary, never rewrites content, and there is always a deterministic default if the LLM is unavailable.

## 6. Markdown cleanup rules

Governing rule: **output the way the user would write it in a message or email.** A person doesn't type `##`, `**`, or `---` into iMessage; they do use backticks for code, emoji, line breaks, and the occasional list. Rules below follow from that.

**Strip document/AI scaffolding; keep what a person types.**

| Element                                 | Action                                                        | Rationale                                                    |
| --------------------------------------- | ------------------------------------------------------------- | ------------------------------------------------------------ |
| **Bold** `**x**` / `__x__`              | Strip markers, keep `x`                                       | Explicitly requested; not message-native.                    |
| **Italic** `*x*` / `_x_`                | Strip markers, keep `x`                                       | Same class of noise.                                         |
| **Headings** `#`, `##`                  | Strip markers, keep the text as a plain line                  | A person writes the heading text, not the `#`.               |
| **Horizontal rule** `---`, `***`, `___` | Remove the line entirely                                      | Renders as literal dashes in a message = noise (§7 example). |
| **Blockquote** `>`                      | Keep                                                          | A person uses quoting.                                       |
| **Lists** `-`, `1.`                     | Keep markers                                                  | Message-native structure.                                    |
| **Code fences / inline code**           | Keep verbatim + backticks, **no reflow, no marker stripping** | Backticks kept (confirmed); whitespace is semantic.          |
| **Links** `[text](url)`                 | Keep `text`; keep a bare/real URL as-is                       | A person pastes the words, or a plain URL — not `[..](..)`.  |
| **Emoji** `💙`                          | Keep                                                          | A person types emoji.                                        |

**Assistant preamble / framing chrome (NEW — the highest-value, highest-risk decision).**

Claude responses are commonly wrapped in conversational framing: `Here's a longer version:`, `Sure! Here's a draft:`, a trailing `Let me know if you'd like changes.`. When the user is pasting into a text message, that framing is the _first_ thing they'd delete by hand. Stripping it is arguably the single most valuable transform for the messaging use case — **and the riskiest**, because deciding "is this line Claude's framing or actual content?" is a semantic judgment. A wrong guess deletes real message text.

- This is a **bounded LLM judgment call** (per AGENTS.md): classify a leading/trailing line as _framing vs content_, remove only if framing. Never rewrite the body.
- Deterministic-only fallback: match a small allowlist of obvious opener/closer patterns (`Here's ...:`, `Sure, here's ...`) anchored at the very start/end, and do nothing otherwise (fail safe = keep).
- **Open (Q8):** in v1, or
- deferred? Fail-open (keep on doubt) is mandatory either way.

**Edge cases to resolve (raised by the "remove bolding / still markdown" vagueness):**

1. **Input asymmetry.** In **Type A** (terminal-selected), bold/italic markers are _already stripped by the terminal_ — there is nothing to remove. In **Type B** (`/copy`), they're present. So "remove bolding" only bites on Type B. The tool's markdown-cleanup must be a no-op-safe pass, not assume markers are present.
2. **Bold-as-heading.** A line like `**Overview**` used as a pseudo-heading becomes a bare line indistinguishable from body text once bold is stripped — the structural signal is lost. Do we detect standalone-emphasis-line and promote to a heading, or leave it flat?
3. **Literal asterisks that aren't emphasis.** `2 * 3`, glob `*.py`, `rm *`. Naive `*`-stripping corrupts these. Stripping must be markdown-aware (matched pairs, not lone `*`), or scoped to code-excluded regions.
4. **Emphasis spanning a (soft) wrap.** `**foo` / `bar**` split across a wrapped break: markers must be reconciled _after_ un-wrapping, not before, or they won't match.
5. **Inline code vs prose.** Stripping backticks around `` `rm -rf` `` blends a command into a sentence and loses the "this is code" signal. Default keeps backticks — confirm.
6. **Nested/mixed emphasis** `***x***`, `**_x_**` — define precedence so partial stripping doesn't leave orphan markers.

## 7. Success criterion & acceptance tests

**Criterion:** output reads as flowing text that obeys the rules of English grammar — wrapped sentence fragments are rejoined, real breaks preserved — while remaining valid, de-noised markdown. Idempotent on clean input.

Acceptance cases (to become the test suite):

- **AC1 (motivating):** the §1 example → single joined sentence, indent removed.
- **AC2 (paragraphs):** two paragraphs separated by a blank line stay two paragraphs.
- **AC3 (list):** wrapped list items rejoin within an item; item boundaries preserved.
- **AC4 (code):** a fenced code block passes through byte-for-byte (no reflow, no stripping).
- **AC5 (blockquote):** `>` lines preserved as a blockquote; wrapped quote text rejoined.
- **AC6 (idempotency):** clean input == output.
- **AC7 (Type B bold):** `**bold**` → `bold`; `2 * 3` untouched.
- **AC8 (Type A):** indented+wrapped terminal paste with no markers reflows correctly.
- **AC9 (horizontal rule):** a `---` line is removed; surrounding paragraphs preserved (§7 "Hey Mom" example).
- **AC10 (preamble):** `Here's a longer version:` opener is removed (if Q8=in-scope) while both message paragraphs and the trailing `💙` are preserved verbatim; fail-open if uncertain.

## 8. Edge cases & failure modes (carry-forward)

1. **Code blocks lose their fence markers** in terminal-rendered paste (fences render as nothing/box), so reflow can't tell it's inside code → corrupts indentation-significant languages. Need best-effort code detection for Type A.
2. **Hyphen/join ambiguity** — joining with a space vs. nothing (`state-\nof` → `state- of` vs `state-of`). Default join char is a single space; hyphen-ended lines are an open case.
3. **Over-merge / under-merge of paragraphs** — eating a real single-newline paragraph break, or leaving spurious breaks. Blank-line rule mitigates but single-newline breaks are ambiguous.
4. **CJK/emoji/ANSI width** — not width-based, so largely dodged; but stray ANSI residue on copy must be stripped first.
5. **Data that looks like formatting** — literal leading spaces, literal `>`, asterisks in content (see §6.3).

## 9. Determinism boundary (per AGENTS.md)

- **Deterministic** (required): indent stripping, blank-line paragraph detection, structure/code detection, markdown marker cleanup, the punctuation+capitalization soft-newline heuristic.
- **Bounded LLM** (optional, judgment only): classify a _single ambiguous newline_ as soft/hard when heuristics tie. Never rewrites, never invents text; deterministic fallback always exists.

## 10. Out of scope

- Full markdown→prose rendering (tables→sentences, images, HTML).
- Reconstructing exact terminal width or exact original line breaks.
- Lossless round-trip / reversibility.
- Non-Claude terminal sources (vim yanks, `less`, tmux capture) — input contract is Claude output, Types A and B.
- Interface/packaging decision (CLI, clipboard hook, library, extension).

## 11. Open questions (need a decision)

**Resolved** (per "the way a user would write"): Q1 italic → strip · Q2 headings → strip markers, keep text · Q3 links → keep text / bare URL as-is · Q4 backticks → keep · Q5 bold-as-heading → flatten.

**Still open:**

- **Q6** Join character for hyphen-ended wraps (§8.2)? (default: single space)
- **Q7** Is the bounded-LLM newline-classifier fallback in v1, or deterministic-only first?
- **Q8** Assistant-preamble stripping (§6): in v1 or deferred? (fail-open regardless)
- **Q9** For the messaging use case, do we also strip a trailing sign-off/offer (`Let me know if…`)? Same class as Q8.
