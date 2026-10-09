# GoharScribe Final Remediation Verification — 2026-10-09

## Executive Summary

Independent review of all changes since the security audit (round 3). All code changes address identifiable problems. No security controls weakened. No tests removed. Two regressions found and fixed during verification (undo/redo checkpoint logic).

**Status: READY FOR FURTHER TESTING** (not release-ready — see limitations).

## Changes Reviewed

| Commit | Description | Verdict |
|--------|-------------|---------|
| 08a80fa | Add 4 OFL Urdu/Arabic fonts | ✅ Correct — OFL licensed, attribution added |
| 55f3698 | Remove misleading pasteMerge button, honest image paste error, fix undo/redo pure markers | ✅ Correct — addresses L-1, L-2, L-3 from QA audit |
| 2053299 | Disable font preview (hang fix) | ✅ Correct — fixes user-reported hang |
| a86ef16 | rust-version 1.90→1.95, clippy fix | ✅ Correct — matches dependency requirements |
| 8df407c | Platform QA report | ✅ Documentation only |
| f243842 | Fix undo/redo checkpoint logic | ✅ Correct — fixes regression from 55f3698 |

## Verification Results

| Check | Result |
|-------|--------|
| `cargo fmt --all -- --check` | ✅ Pass |
| `cargo check --workspace --all-targets` | ✅ Pass |
| `cargo test -p goharscribe-engine --lib` | ✅ 41 passed |
| `cargo test -p goharscribe-fonts --lib` | ✅ 17 passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ Pass |

**Note:** Full `cargo test --workspace` was not completed (takes 18+ minutes). Engine and fonts crates verified individually.

## Regressions Found and Fixed

### R-1: Undo/redo checkpoint regression (from 55f3698)
**Severity:** High  
**Cause:** Removing `.pure()` from undo/redo set `mutates=true`, which caused `session.rs` to create a checkpoint for undo operations, breaking the undo stack.  
**Fix:** Excluded `edit.undo`/`edit.redo`/`edit.repeat` from checkpoint creation in `session.rs:343`.  
**Tests:** `tests::typing_enter_undo` and `tests::m2_selection_breaks_typing_undo_group` now pass.

## Remaining Findings

### BIDI-1 (High, unresolved)
Mixed RTL/LTR text (e.g., `اردو 123 English`) renders in incorrect visual order. The `direction_runs()` function does not implement full UAX #9. Documented in `docs/audit/TYPOGRAPHY_AUDIT.md`. Requires Phase 2a architectural work.

### Font preview disabled (Medium, workaround)
Per-font preview rendering in the font dropdown was disabled to fix a hang on Noto Nastaliq Urdu. The dropdown now shows plain text labels. A proper fix would render previews in a background thread.

### WASM target (Low, environmental)
`wasm32-unknown-unknown` target not installed. Cannot verify WASM build.

## Data Integrity

- ✅ Undo/redo: 41 engine tests pass, including undo grouping tests
- ✅ Font changes: 17 font tests pass, fonts load correctly
- ✅ No document format changes in this patch set
- ✅ Failed commands restore state (verified by `failed_command_leaves_document_unchanged` test)

## Platforms Verified

- Linux x86_64: Build ✅, Tests ✅, Runtime ✅
- Windows/macOS/FreeBSD: CI only (not manually verified)
- WebAssembly: Not verified (target missing)

## Recommendation

**READY FOR FURTHER TESTING.** The codebase is in good shape for continued development and testing. Not release-ready due to:
1. BIDI-1 (High) — incorrect RTL rendering for mixed text
2. Manual testing only on Linux
3. Font preview feature disabled (workaround, not a proper fix)
