# GoharScribe Test Results

**Date:** 2026-10-09
**Final commit:** 9c92d74

## Final CI (`cargo xtask ci` minus wasm)

| Step | Result |
|------|--------|
| fmt (`cargo fmt --all -- --check`) | PASS |
| clippy (`-D warnings`) | PASS |
| test (`cargo test --workspace`) | PASS (260+ tests, 0 failures) |
| assets | PASS |
| layers | PASS |
| wasm | SKIPPED (target not installed — platform limitation) |

## Tests added in this audit

**Security (4):**
- `sec1_para_set_rejects_too_many_tabs`
- `sec2_pagesetup_rejects_hostile_section`
- `sec3_format_set_rejects_absurd_values`
- `sec4_base64_decode_bounds_input`

**QA (3):**
- `m1_sort_keeps_section_break_on_last_para`
- `m2_selection_breaks_typing_undo_group`
- `m3_reject_all_removes_inserted_paragraph`

All 7 new tests pass.

## Baseline (before audit)

| Step | Result |
|------|--------|
| fmt | PASS |
| clippy | PASS |
| test | PASS (250+ tests, 0 failures) |
| assets | PASS |
| layers | **FAIL** (goharscribe-input unregistered) |
| wasm | SKIPPED |
