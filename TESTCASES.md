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
| 7 | `blockquote_markers_stripped_and_reflowed` | `> a long quote that\n> wrapped` | `a long quote that wrapped` | contract v2: `>` is chrome (paste targets render it literally) → strip the markers and reflow into prose |
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
| 21 | `nested_blockquote_markers_stripped` | `> > a\n> > b` | `a b` | contract v2: a leading run of `>` (incl. nested `> >`) is all chrome → stripped, wrap joins |

## Round 3 — copied terminal UI chrome

These twelve cases cover the select/copy → paste path for Claude Code and Codex terminal output.

| # | Name | Guard |
|---|------|-------|
| 22 | `claude_response_gutter_removed` | removes Claude response glyph `⏺` before reflow |
| 23 | `codex_response_gutter_removed` | removes Codex-style response glyph `•` before reflow |
| 24 | `claude_continuation_gutter_removed` | removes Claude continuation glyph `⎿` |
| 25 | `terminal_prompt_gutter_removed` | removes prompt glyph `❯` |
| 26 | `vertical_terminal_gutter_removed` | removes vertical gutter `│` |
| 27 | `terminal_gutter_with_indent_removed` | handles terminal indentation before the glyph |
| 28 | `gutter_requires_following_whitespace` | fails open for ordinary text beginning with a glyph |
| 29 | `markdown_bullets_are_preserved` | keeps ordinary Markdown list structure |
| 30 | `gutter_inside_fence_is_verbatim` | never changes code blocks |
| 31 | `inline_gutter_is_not_removed` | preserves inline code/text symbols |
| 32 | `gutter_cleaning_is_idempotent` | running on an already-clean result is safe |
| 33 | `gutter_and_markdown_clean_together` | applies terminal and Markdown cleanup together |

## Round 4 — 12 more agent-terminal domain cases (5 revealed gaps, now fixed)

| # | Name | Guard | Was |
|---|------|-------|-----|
| 34 | `ansi_sgr_codes_stripped` | remove ANSI color escapes from raw-buffer copies | **GAP→fixed** |
| 35 | `box_drawing_rule_dropped` | drop `────` separators (not just ASCII `---`) | **GAP→fixed** |
| 36 | `markdown_table_delimiter_dropped_rows_kept` | keep table rows one-per-line, drop `\|---\|` | **GAP→fixed** |
| 37 | `bold_label_list_item` | `- **Label:** text` → `- Label: text` | lock |
| 38 | `nbsp_indent_reflowed` | NBSP (U+00A0) wrap indent reflows | lock |
| 39 | `tab_indented_wrap_reflowed` | tab wrap indent reflows | lock |
| 40 | `lone_gutter_glyph_line_dropped` | a glyph alone on a line is pure chrome | **GAP→fixed** |
| 41 | `gutter_then_blockquote_both_stripped` | `⏺ > quote` → `quote` | v2 |
| 42 | `bom_and_zero_width_stripped` | strip U+FEFF / U+200B | **GAP→fixed** |
| 43 | `url_with_underscore_and_asterisk_preserved` | `a_b?x=1*2` in a URL survives | lock |
| 44 | `paren_numbered_list_wrapped` | `1)`/`2)` list wraps join per item | lock |
| 45 | `checkbox_task_list_kept` | `- [ ]` / `- [x]` preserved | lock |

## Round 5 — blockquote de-chrome (contract v2)

v2 reclassifies a leading `>` from a kept blockquote to presentation chrome: strip the markers (incl. nested `> >`) and reflow the inner text as prose. The paste targets (SMS/email/Docs/browser chat) render `>` literally, and a `>` gutter on a soft-wrapped terminal line is the common artifact. Only the marker is removed; content is preserved. This changed cases 7, 21, and 41 above from their v1 keep-`>` expectations.

| # | Name | Input (abridged) | Expected | Why it's an edge |
|---|------|------------------|----------|------------------|
| 46 | `codex_wrapped_blockquote_prefix_reflowed` | `…locally,\n  > before Cmd+V.”` | one line, `>` gone | the real reported case: a soft-wrapped continuation carrying a `>` gutter must reflow into the sentence, not survive as a quote |
| 47 | `standalone_blockquote_marker_stripped` | `> A deliberate quote line` | `A deliberate quote line` | `>` is chrome even when it begins its own block — no positional exception |
| 48 | `blockquote_after_prose_joins` | `They said:\n> ship it Friday` | `They said: ship it Friday` | a `>` line glued under a lead-in prose line is a wrap artifact → joins |
| 49 | `blockquote_stripped_but_inline_code_gt_preserved` | `` > use `a > b` in the guard `` | `` use `a > b` in the guard `` | only the LEADING marker is chrome; a `>` inside inline code is sacred |
| 50 | `blockquote_inside_fence_is_verbatim` | fence containing `> stays a quote` | verbatim | inside a fence `>` is code, not chrome (guard passes v1 and v2) |
| 51 | `blockquote_strip_is_idempotent` | `> one\n> two` | `one two`, stable on re-run | stripping must be idempotent |
