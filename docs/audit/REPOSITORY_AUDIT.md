# GoharScribe Repository Audit

**Date:** 2026-10-09
**Repository:** https://github.com/TalhaGoharWeb/GoharScribe
**Commit audited:** 778b4ae (main) + 474da5c (layers fix)
**Auditor:** Gohar (senior Rust engineer)

## Executive summary

GoharScribe is a ~44k-line pure-Rust word processor (13 crates, 3 apps) forked from WordCraft v0.3.0,
rebranded and refocused on Urdu/Arabic RTL writing. The codebase is well-structured with strong
engineering discipline: no-panic policy, hostile-input fuzzing, layering enforcement, and comprehensive tests.

**Baseline CI:** fmt ✓, clippy ✓, tests ✓ (250+, 0 failures), assets ✓, layers ✗ (fixed), wasm (pending)

**Critical finding:** The flagship Urdu/Arabic RTL support lacks UAX #9 visual reordering. Mixed-direction
text displays in incorrect visual order. This is a Phase 2a architectural item.

## Crate inventory

| Crate | Lines | Layer | Purpose |
|-------|-------|-------|---------|
| geom | 244 | L0 | Units, rects, measurement parsing |
| doc | 4,507 | L1 | Document model, edit primitives |
| fonts | 1,502 | L1 | Font DB, shaping (harfrust), Word font substitution |
| input | 248 | L1 | Urdu phonetic keyboard + transliteration (new) |
| proof | 1,224 | L1 | Spell check, hyphenation |
| layout | 4,019 | L2 | Line breaking, pagination, tables, hit testing |
| docx | 4,517 | L2 | OOXML read/write |
| formats | 7,171 | L2 | RTF/ODT/HTML/MD/TXT import/export |
| render | 560 | L3 | vello_cpu rasterizer |
| pdf | 1,244 | L3 | PDF export (krilla) |
| engine | 9,200 | L4 | Session, undo, command registry (389 commands) |
| mcp | 822 | L5 | MCP server |
| ui-egui | 6,585 | L6 | egui frontend |

## Findings

See SECURITY_FINDINGS.md for security issues.
See REMEDIATION_LOG.md for fixes applied.

### BIDI-1 (High): No UAX #9 visual reordering
**Status:** Confirmed, unresolved (Phase 2a scope)

The shaping layer (`crates/fonts/src/lib.rs`) splits text into directional runs using only strong
directional characters (R/AL vs L). Neutrals join the current run. This is not UAX #9 compliant:
no embedding levels, no neutral resolution (N0-N2), no number handling (EN/AN).

The layout layer (`crates/layout/src/para.rs`) performs line breaking left-to-right unconditionally.
The comment at `crates/fonts/src/lib.rs:115-117` claims "line layout reorders them visually" but no
such code exists in the layout crate.

**Impact:** Mixed RTL/LTR text (e.g., Urdu with English terms, numbers in Urdu text) displays in
wrong visual order. This breaks the flagship Urdu/Arabic feature.

**Evidence:**
- `crates/fonts/src/lib.rs:87-111` — `direction_runs()` implementation
- `crates/layout/src/para.rs:562` — `break_lines()` starts at `left`, advances right
- Zero occurrences of "visual", "reorder", or "bidi" in `crates/layout/src/`

**Remediation:** Implement UAX #9 using `unicode-bidi` 0.3.18 (already a dependency):
1. Replace `direction_runs()` with `BidiInfo`-based visual run computation
2. Reorder clusters in layout according to visual runs
3. Update hit testing (`crates/layout/src/hit.rs`) for visual↔logical mapping
4. Update caret movement in `crates/engine/src/cmd/caret.rs`

This is estimated at 8-12 hours and touches 4 crates. Deferred to Phase 2a.

### LAYERS-1 (Medium): goharscribe-input unregistered in layer table
**Status:** Fixed in 474da5c

The new `goharscribe-input` crate was not in `xtask/src/layers.rs` TABLE, causing
`cargo xtask ci` to fail at the layers step.

**Fix:** Added `("input", Class::Layer(1))`.

## Test results

See TEST_RESULTS.md.

## Platform limitations

- WASM check skipped in baseline (CI failed before reaching it). To be verified.
- Native GUI not tested (headless environment).
