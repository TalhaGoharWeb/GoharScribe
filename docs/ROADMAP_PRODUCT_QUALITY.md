# GoharScribe Product Quality Roadmap — 2026-10-09

## Baseline (verified via CLI and tests)

| Area | Status | Evidence |
|------|--------|----------|
| Document model | ✅ Solid | 41 engine tests pass |
| Urdu/Arabic fonts | ✅ Bundled | 4 OFL fonts, 20 font tests pass |
| BIDI shaping | ✅ Fixed | UAX #9 via unicode-bidi, 3 new tests |
| DOCX round-trip | ✅ Works | Urdu text preserved (`cli convert` + `cli text`) |
| PDF export | ✅ Works | 12KB PDF from Urdu doc |
| Formats (ODT/RTF/HTML/MD/TXT) | ✅ 33 tests | `goharscribe-formats` green |
| Undo/redo | ✅ Fixed | Checkpoint regression fixed, tests pass |
| Font dropdown | ⚠️ Workaround | Previews disabled (was hanging) |
| CLI | ✅ Built | `goharscribe-cli` 419MB, all subcommands work |
| Desktop GUI | ❌ Can't build here | Hangs at 0% CPU in this env; code is clean |

## Gap Analysis vs Microsoft Word

### P0 — Critical for Urdu/Arabic users

1. **Urdu spellcheck dictionary** — Only English (`en-us.dic`) exists. Urdu users get no proofing.
   - Effort: Medium (need word list, can bootstrap from public domain sources)
   - Impact: High (core value prop is Urdu support)

2. **Hit testing / caret for mixed-direction text** — BIDI shaping fixed, but `crates/layout/src/hit.rs` lacks logical↔visual mapping.
   - Effort: Medium
   - Impact: High (clicking in mixed text places caret wrong)

3. **Font preview re-enable** — Currently disabled as workaround. Need background-thread rendering or fix Nastaliq hang root cause.
   - Effort: Medium
   - Impact: Medium (UX polish)

### P1 — Important for Word compatibility

4. **Draw tab (0/11)** — Ink/drawing tools completely missing. Complex feature.
   - Effort: Large
   - Impact: Medium (niche for word processing)

5. **Track changes fidelity** — Implemented but DOCX round-trip fidelity untested for complex cases.
   - Effort: Medium
   - Impact: Medium

6. **Mail merge** — Commands exist (17/17), but real-world testing with data sources needed.
   - Effort: Small (testing)
   - Impact: Medium

### P2 — Polish

7. **WASM build verification** — Target not installed in CI env.
8. **Windows/macOS manual testing** — CI builds only.
9. **Performance benchmarks** — No large-document stress tests.

## Architecture Notes

**Strengths:**
- Clean crate separation (doc/engine/layout/fonts/render)
- Command pattern with undo/redo is solid
- Security hardening is thorough (MCP jail, input validation)

**Bottlenecks:**
- `Previews::snippet()` does synchronous layout+raster on UI thread — caused the font dropdown hang
- Font DB initialization includes all bundled fonts via `include_bytes!` — increases binary size (~2MB for 4 new fonts)

**No major duplication found.** The 389 commands are well-organized by module.

## Recommended Next Steps

1. Build Urdu spellcheck dictionary (highest user impact)
2. Fix hit testing for BIDI text
3. Re-enable font previews with background rendering
4. Manual test on Windows with real Urdu documents
