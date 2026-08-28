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

## Round 2 — 10 more edge cases (found from a real Type-B `/copy` paste)

| # | Name | Input (abridged) | Expected | Why it's an edge |
|---|------|------------------|----------|------------------|
| 11 | `inline_code_emphasis_preserved` | `` The `**markers**` stay `` | `` The `**markers**` stay `` | **BUG:** v1 stripped `**` inside inline code. Code spans must be verbatim. |
| 12 | `underscore_identifiers_preserved` | `Call __init__ and my_var` | `Call __init__ and my_var` | **BUG:** underscore emphasis regex ate `__init__`. Code identifiers must survive. |
| 13 | `wrapped_numbered_list` | `1. First item that\nwraps\n2. Second` | items stay separate; wrap joins into item 1 | list-boundary vs wrap-continuation |
| 14 | `multiple_blank_lines_collapse` | `A\n\n\n\nB` | `A\n\nB` | over-blank input must collapse to one paragraph gap |
| 15 | `leading_trailing_blank_and_space_trim` | `\n\n  Hello world  \n` | `Hello world` | surrounding blank lines + stray spaces trimmed |
| 16 | `setext_heading_underline_dropped` | `Project Title\n=============` | `Project Title` | **BUG:** `===` underline joined into the title as text |
| 17 | `crlf_line_endings` | `line that\r\nwrapped\r\n\r\nnext` | `line that wrapped\n\nnext` | Windows CRLF must reflow like LF |
| 18 | `asterisk_bullets_kept` | `* one\n* two` | `* one\n* two` | `*` bullets must not be mistaken for italic markers |
| 19 | `code_fence_with_markdown_inside` | fence containing `# h`, `- i`, `**b**`, `2 * 3` | verbatim | de-chrome must not reach inside fences |
| 20 | `empty_and_whitespace_input` | `   \n\n\t\n` | `` (empty) | degenerate input → empty, no panic |
| 21 | `nested_blockquote_collapsed` | `> > a\n> > b` | `> a b` | nested quote levels collapse to one, wrap joins |
