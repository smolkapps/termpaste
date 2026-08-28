# 10 test cases that break naive implementations

Naive baseline #1: "strip 2 leading spaces from every line, replace every `\n` with a space."
Naive baseline #2: "regex-replace all `*`, `#`, `-` markers."

Each case below breaks at least one naive baseline. These map 1:1 to `tests/cases.rs`.

| # | Name | Input (abridged) | Expected | Why naive code fails |
|---|------|------------------|----------|----------------------|
| 1 | `soft_wrap_join` | `deterministic code is\n  required wherever...` | one joined sentence | naive "keep newlines" leaves the split; also must drop the continuation's 2-space indent |
| 2 | `paragraph_break_preserved` | two paragraphs separated by a blank line | two paragraphs | naive "replace every `\n` with space" merges paragraphs into one wall of text |
| 3 | `code_fence_verbatim` | fenced Python with indentation + blank line | code unchanged | naive indent-strip + line-join destroys significant whitespace and merges statements |
| 4 | `bold_stripped_literal_asterisks_kept` | `**bold** and 2 * 3 and *.py` | `bold and 2 * 3 and *.py` | naive "remove all `*`" corrupts arithmetic and globs |
| 5 | `horizontal_rule_removed` | `Para one\n\n---\n\nPara two` | two paras, no `---` | naive keeps `---`; in a text message it's literal dashes = noise |
| 6 | `heading_markers_stripped_text_kept` | `## Overview\n\nBody` | `Overview\n\nBody` | naive either keeps `##` or deletes the whole heading line (losing the text) |
| 7 | `blockquote_wrapped_rejoined` | `> a long quote that\n> wrapped` | `> a long quote that wrapped` | naive join merges into one line and drops/duplicates `>`; naive keep leaves the wrap |
| 8 | `emphasis_spanning_wrap` | `This is **very\n  important** text` | `This is very important text` | strip-before-reflow leaves unmatched `**`; ordering matters |
| 9 | `idempotent_on_clean_prose` | already-clean prose | identical output | naive "always strip 2 chars" eats real leading characters on a second pass |
| 10 | `preamble_and_emoji` | `Here's a longer version:\n\n---\n\nHey Mom ... 💙` | preamble + `---` gone, message + emoji kept verbatim | naive keeps AI framing and `---`; or an over-eager stripper eats the emoji / real text |
