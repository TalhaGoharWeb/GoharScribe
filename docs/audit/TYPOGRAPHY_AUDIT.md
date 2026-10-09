# GoharScribe Urdu/Arabic Typography Audit — 2026-10-09

## Executive Summary

**Status:** Shaping engine works. Bidi algorithm is fundamentally broken (BIDI-1).
**Impact:** Mixed Urdu/English text renders in wrong visual order. Flagship feature non-functional.

## Shaping Engine: WORKING

**Engine:** rustybuzz 0.20.1 (HarfBuzz port) via `crates/fonts/src/lib.rs:144-180`

Verified:
- Arabic joining forms (isolated/initial/medial/final) — handled by rustybuzz
- Ligatures (lam-alef, etc.) — handled by rustybuzz
- Mark positioning (tashkeel, shadda) — handled by rustybuzz
- `UnicodeBuffer::guess_segment_properties()` + explicit direction setting

The shaping itself is correct. The problem is WHAT ORDER the text is fed to the shaper.

## Bidirectional Algorithm: BROKEN (BIDI-1)

**File:** `crates/fonts/src/lib.rs:87-111`

### Current Implementation

```rust
fn direction_runs(text: &str) -> Vec<(Range<usize>, bool)> {
    // Only splits on strong R/AL vs L. No UAX #9.
}
```

### What's Wrong

1. **No embedding levels** — UAX #9 requires computing levels 0-125
2. **No number handling** — EN (European Number) and AN (Arabic Number) have special rules
3. **No neutral resolution** — punctuation between LTR/RTL follows complex rules (W1-W7, N0-N2)
4. **No visual reordering** — layout renders in logical order, not visual

### Example Failure

Input (logical): `اردو 123 English`
- Expected visual (RTL para): `English 123 اردو` (numbers stay LTR)
- Actual: Broken — numbers may appear in wrong position

Input: `Hello (اردو) world`
- Parentheses are neutrals — UAX #9 has specific rules
- Current code: treats `(` as joining the surrounding run (wrong)

### Impact

- `crates/fonts/src/lib.rs:117-141` — `shape()` uses broken `direction_runs`
- `crates/layout/src/para.rs` — line breaking assumes LTR visual order
- `crates/layout/src/hit.rs` — hit testing doesn't map visual↔logical
- Caret movement is logical, not visual (user confusion)

## Font Discovery: WORKING

**Mechanism:** `crates/fonts/src/fontdb.rs` — system font database

- Windows: `%WINDIR%/Fonts`
- macOS: `/System/Library/Fonts`, `~/Library/Fonts`
- Linux: fontconfig via `fc-list`
- Bundled: `assets/fonts/` (Inter, Source Sans/Serif, JetBrains Mono)

**No Urdu fonts bundled.** Available on test system:
- Noto Sans Arabic (multiple weights) — good for Arabic, acceptable for Urdu Naskh
- NO Jameel Noori Nastaleeq (not installed, licensing unclear)

## Nastaleeq: UNTESTED

Cannot verify Nastaleeq shaping without the font. Jameel Noori Nastaleeq:
- Not bundled (licensing — not OFL)
- Not on test system
- Cannot verify joining, ligatures, or metrics

**Recommendation:** Bundle an OFL Nastaleeq font (e.g., from Google Fonts) or
document that users must install Jameel Noori separately.

## DOCX Round-Trip: PARTIAL

**Preserved:**
- `w:rtl` on runs (`crates/docx/src/write/props.rs:88`)
- `w:bidi` on paragraphs (`props.rs:216`, `story.rs:675-676`)

**Not verified:**
- Visual order in Word (depends on Word's bidi, not ours)
- Font metadata for RTL fonts

## PDF Export: UNVERIFIED

`crates/pdf/src/lib.rs` has no bidi handling. Uses glyph IDs directly.
If glyphs are in wrong visual order (due to BIDI-1), PDF will be wrong.

## Required Fix: Phase 2a (8-12 hours)

1. **Use `unicode-bidi` properly** (already a dependency, v0.3.18):
   ```rust
   let bidi = BidiInfo::new(text, Some(ParagraphDirection::Auto));
   let (levels, runs) = bidi.visual_runs(paragraph, line_range);
   ```

2. **Shape in visual order**, keep cluster→logical mapping

3. **Layout in visual order** — line breaking, x positions

4. **Hit testing**: visual x → logical cluster

5. **Caret**: visual movement (arrow keys), logical selection

6. **Preserve**: Do not reverse strings. Do not strip diacritics.

## Test Corpus Needed

- Urdu paragraph with English technical terms
- Arabic with tashkeel + English citations
- Mixed numbers: `اردو 123 English 456`
- Parentheses: `Hello (اردو) world`
- URLs, emails in RTL text
- Tables with mixed-direction cells
- Footnotes with mixed text

## Honest Limitations

1. **BIDI-1 is architectural** — cannot be fixed with a small patch
2. **No Nastaleeq font** — cannot verify without legal font
3. **PDF visual order** — unverified, likely broken due to BIDI-1
4. **Caret behavior** — logical, not visual (confusing for RTL users)

Do not claim Urdu support works until Phase 2a is complete.
