# GoharScribe Remediation Log

**Date:** 2026-10-09

## Fixed (Security Audit Round 2)

### H2: MCP filesystem jail bypass (High)
- **File:** `crates/mcp/src/tools.rs`
- **Issue:** 7 path-taking commands (`insert.textFromFile`, `insert.picture`, `picture.change`, `review.compare`, `review.combine`, `mailings.recipients`, `mailings.finish`) were not in `PATH_COMMANDS`, allowing jail escape via MCP.
- **Fix:** Added all 7 to `PATH_COMMANDS`. Commit 12ef0ae.
- **Regression tests:** `h2_jail_covers_all_path_commands`, `h2_jail_blocks_textfromfile_outside`

### H3: picture.* adjust decodes before 80 Mpx guard (High)
- **File:** `crates/engine/src/cmd/objects.rs`
- **Issue:** `image::load_from_memory` decoded full bitmap before checking dimensions → hostile PNG could force multi-GB allocation → OOM.
- **Fix:** Check dimensions via `ImageReader::into_dimensions()` before decode. Commit 12ef0ae.

### LAYERS-1: Register goharscribe-input in layer table
- **Severity:** Medium
- **Commit:** 474da5c
- **Change:** Added `("input", Class::Layer(1))` to `xtask/src/layers.rs` TABLE.
- **Verification:** `cargo xtask layers` → "OK: 17 crates, no layering violations."

### SEC-1: para.set bypasses 64-tab cap (High)
- **Commit:** a5852dd
- **Regression test:** `sec1_para_set_rejects_too_many_tabs`

### SEC-2: layout.pageSetup section bypasses validation (High)
- **Commit:** a5852dd
- **Regression test:** `sec2_pagesetup_rejects_hostile_section`

### SEC-3: format.set unbounded strings/sizes (Medium)
- **Commit:** a5852dd
- **Regression test:** `sec3_format_set_rejects_absurd_values`

### SEC-4: base64_decode preallocates before validation (Medium)
- **Commit:** a5852dd
- **Regression test:** `sec4_base64_decode_bounds_input`

### H-1: Keyboard shortcuts silently dead (High)
- **File:** `crates/ui-egui/src/keys.rs`
- **Issue:** `key_name()` lacked match arms for F4, F8, Num2/3/4/6/7, F1/2/6/10/11. Broke `edit.repeat` (F4), `select.extend` (F8), `para.double` (Mod+2), etc.
- **Fix:** Added missing key mappings. Commit d7ea923.

### M-1: para.sort moves section breaks (Medium)
- **File:** `crates/engine/src/cmd/para.rs`
- **Issue:** Sorting moved `Paragraph.section` with the paragraph, applying wrong page setup to wrong content (silent corruption).
- **Fix:** Save section break before sort, clear from all, restore to new last paragraph. Commit d7ea923.
- **Regression test:** `m1_sort_keeps_section_break_on_last_para`

### M-2: Selection doesn't break typing undo group (Medium)
- **File:** `crates/engine/src/session.rs`
- **Issue:** `select.*` commands kept `typing_open=true`, so undo removed more than expected.
- **Fix:** `select.*` now breaks the typing group like `caret.*` does. Commit d7ea923.
- **Regression test:** `m2_selection_breaks_typing_undo_group`

### M-3: review.rejectAll leaves phantom paragraph (Medium)
- **File:** `crates/engine/src/cmd/review.rs`
- **Issue:** Rejecting a tracked paragraph insertion cleared `mark.ins` but left the empty paragraph.
- **Fix:** `resolve_all` now deletes paragraphs with `mark.ins` on reject. Commit d7ea923.
- **Regression test:** `m3_reject_all_removes_inserted_paragraph`

### F-1: ODT ignores table:number-columns-repeated (High)
- **File:** `crates/formats/src/odt.rs`
- **Issue:** LibreOffice collapses empty cells with `table:number-columns-repeated`; importer created 1 cell instead of N.
- **Fix:** Track repeat count, expand at end-tag. Commit d7ea923.

### F-2: ODT ignores table:number-rows-repeated (High)
- **File:** `crates/formats/src/odt.rs`
- **Issue:** Same for rows.
- **Fix:** Track in TableB, expand at row end. Commit d7ea923.

### F-3: RTF ignores \\itapN (Medium)
- **File:** `crates/formats/src/rtf.rs`
- **Issue:** Nested table content leaked into outer table.
- **Fix:** Track `itap` level, skip content at itap>1. Commit d7ea923.

### F-4: Markdown export drops images (Medium)
- **File:** `crates/formats/src/markdown.rs`
- **Issue:** Export wrote `![alt](image.png)` (dangling path); import only accepted data URIs. Silent image loss.
- **Fix:** Export embeds as `data:` URI. Commit d7ea923.

## In progress

(None.)

## Deferred (requires architectural decision)

### BIDI-1: UAX #9 visual reordering
- **Severity:** High
- **Blocker:** Requires changes to fonts, layout, hit testing, and caret movement across 4 crates.
- **Plan:** Phase 2a. Use `unicode-bidi` 0.3.18 for visual runs, reorder clusters in layout.
- **Estimate:** 8-12 hours.
