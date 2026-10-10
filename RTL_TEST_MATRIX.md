# RTL Test Matrix — 2026-10-10

## BIDI-1 Fix Verification

`crates/fonts/src/lib.rs`: `direction_runs()` and `shape()` rewritten using
`unicode-bidi 0.3.18` (UAX #9). Glyphs now output in visual order.

### Unit Tests (`cargo test -p goharscribe-fonts`)

| Test | Input | Expected | Result |
|------|-------|----------|--------|
| `bidi_mixed_text_uses_proper_levels` | `اردو 123 English` | ≥2 runs, first RTL | ✅ |
| `bidi_pure_ltr_has_single_level` | `Hello World` | 1 run, level 0 | ✅ |
| `bidi_pure_rtl_has_single_level` | `اردو` | 1 run, odd level | ✅ |

### Manual Test Cases (for visual verification)

| # | Input (logical order) | Expected visual |
|---|----------------------|-----------------|
| 1 | `اردو` | اردو (RTL) |
| 2 | `Hello اردو` | Hello اردو |
| 3 | `اردو 123` | اردو 123 (numbers stay with RTL per UAX #9) |
| 4 | `اردو 123 English` | اردو 123 English |
| 5 | `(اردو)` | (اردو) — parens mirror |
| 6 | `https://example.com اردو` | URL stays LTR |

### Known Limitations

- **HIT-1:** `crates/layout/src/hit.rs` lacks logical↔visual mapping. Clicking in
  mixed-direction text may place the caret incorrectly. Shaping is fixed; hit
  testing is the remaining work.
- Caret navigation across direction boundaries not yet tested.
- Selection rendering for mixed text not yet verified visually.

### Fonts Tested

| Font | Script | Status |
|------|--------|--------|
| Noto Nastaliq Urdu | Urdu Nastaleeq | ✅ Bundled, loads, shapes |
| Noto Naskh Arabic | Arabic Naskh | ✅ Bundled, loads, shapes |
| Amiri Regular/Bold | Arabic (Quranic marks) | ✅ Bundled, loads, shapes |

Diacritic preservation: shaping uses HarfBuzz (`harfrust`); diacritics are
never stripped (verified in code — no normalization that drops marks).
