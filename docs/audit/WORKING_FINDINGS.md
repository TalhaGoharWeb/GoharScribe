# GoharScribe Audit — Working Findings

Started 2026-10-09. Baseline CI running.

## Confirmed findings

### BIDI-1 (High): No UAX #9 visual reordering in layout
- **File:** `crates/fonts/src/lib.rs:87-111` (`direction_runs`), `crates/layout/src/para.rs` (line breaking)
- **Evidence:** `direction_runs()` splits only on strong R/AL vs L; neutrals join the current run. No `unicode-bidi` `BidiInfo` usage. `crates/fonts/src/lib.rs:115-117` comment claims "line layout reorders them visually" but grep finds zero reordering code in `crates/layout/src/`. Line breaking (`break_lines`, para.rs:562) starts at `left` and advances right unconditionally.
- **Impact:** Mixed RTL/LTR text displays in wrong visual order. Urdu flagship feature is broken for bidirectional text.
- **Fix:** Phase 2a (full UAX #9). Not attempted in this audit — requires architectural change.
- **Status:** Documented, unresolved.

### INPUT-1 (Low): `urdu_to_roman_char` returns `""` for unmapped, handled correctly
- **File:** `crates/input/src/lib.rs`
- **Evidence:** Code review. The `""` sentinel is checked by caller. No bug.
- **Status:** No action needed.

## Baseline CI (2026-10-09)
- `cargo xtask ci`: fmt OK, clippy OK, test OK (all passed), assets OK, **layers FAILED**, wasm SKIPPED
- Layers failure: `goharscribe-input` UNREGISTERED (my new crate). **FIXED** in commit 474da5c.
- Test counts: ~250+ tests across workspace, 0 failures.

## Fixed in this audit
### LAYERS-1: goharscribe-input unregistered
- **Severity:** Medium (CI gate broken)
- **Fix:** Added `("input", Class::Layer(1))` to `xtask/src/layers.rs` TABLE.
- **Verification:** `cargo xtask layers` → "OK: 17 crates, no layering violations."
- **Commit:** 474da5c
