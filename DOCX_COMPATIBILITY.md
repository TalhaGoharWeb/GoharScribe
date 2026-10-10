# DOCX Compatibility — 2026-10-10

## Verified via CLI

| Test | Command | Result |
|------|---------|--------|
| Urdu round-trip | `cli run --cmd 'text.insert={"text":"Hello اردو 123 English"}' --save d.json` → `cli convert d.json d.docx` → `cli text d.docx` | ✅ Text preserved exactly |
| PDF export | `cli convert d.json d.pdf` | ✅ 12KB, renders |
| Format suite | `cargo test -p goharscribe-formats` | ✅ 33 passed |

## Coverage (from `docs/parity.md`)

354/405 catalog features have commands (87.4%).

## Known Gaps

1. **Draw tab 0/11** — ink features absent (no DOCX ink round-trip).
2. **Complex tracked-changes** — commands exist; DOCX fidelity for complex cases untested.
3. **External editor testing** — output not yet opened in Microsoft Word or LibreOffice
   (not available in this environment). Structural validity via `goharscribe-docx`
   write path; relationship/content-type validation not independently verified.
4. **Repeated save/reopen** — no automated cycle test yet.

## Format-Specific Notes

- **DOCX:** read via `goharscribe_docx::read`, write via `goharscribe_docx::write`.
- **PDF:** `goharscribe_pdf::export` with `PdfOptions::default()`.
- **ODT/RTF/HTML/MD/TXT:** via `goharscribe_formats::export(ext, doc)`.
- **SaaS conversion endpoint** (`POST /api/docs/:id/convert`) reuses `io::save_bytes`
  which dispatches to the same paths; format whitelist enforced.
