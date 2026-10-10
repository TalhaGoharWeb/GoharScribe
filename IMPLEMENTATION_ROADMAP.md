# GoharScribe Implementation Roadmap — 2026-10-10

## Completed (2026-10-09 → 2026-10-10)

- [x] Security hardening: 3 rounds, 20+ fixes (`SECURITY_AUDIT.md`)
- [x] BIDI-1: UAX #9 visual reordering (`unicode-bidi 0.3.18`)
- [x] Font dropdown hang workaround
- [x] Undo/redo checkpoint fix
- [x] Dead-code audit: 389 commands verified
- [x] Urdu/Arabic fonts bundled (Noto Nastaliq, Naskh, Amiri)
- [x] SaaS backend: `crates/server` (L5) — auth, document API, conversion
- [x] Audit docs: `AUDIT_REPORT.md`, `SECURITY_AUDIT.md`, `RTL_TEST_MATRIX.md`,
      `DOCX_COMPATIBILITY.md`, `PERFORMANCE_BASELINE.md`, this roadmap

## Phase A — Baseline ✅ DONE
Establish baseline, map codebase, document verified findings.

## Phase B — Security & Crashes ✅ DONE
Critical security issues fixed (3 rounds). Crash prevention: `Session::run`
panic guard, `hostile_params_never_panic` green.

## Phase C — RTL/Unicode & DOCX (IN PROGRESS)

| Item | Status |
|------|--------|
| BIDI shaping (UAX #9) | ✅ Done |
| Hit testing logical↔visual (HIT-1) | ❌ Next priority |
| Urdu/Arabic proofing dictionaries (PROOF-1) | ❌ Not started |
| DOCX external-editor verification | ❌ Blocked (no Word/LibreOffice in env) |

## Phase D — Pagination, Printing, Recovery

- Footnote continuation across pages — verify
- Table row splitting — verify
- Autosave/recovery — implement (currently absent)
- Atomic save with flush/sync — verify

## Phase E — Scholarly Workflows

- TOC with page numbers — check existing commands
- Bibliography/citations — check existing commands
- Urdu/Arabic/English templates — create
- Cross-references auto-update — verify

## Phase F — Word Features & Accessibility

- Draw tab (0/11) — implement ink tools
- Font previews: proper background rendering (PREV-1)
- Keyboard navigation audit
- RTL interface layout

## Phase G — Performance & Packaging

- Benchmark suite for typing/load/pagination/export
- Release builds for Windows/macOS/Linux
- Installer packaging

## Phase H — AI & Differentiation

- AI proofreading (Urdu/Arabic grammar)
- AI-assisted writing (optional, offline-capable)

## SaaS Track (NEW)

- [x] Backend API (auth, CRUD, convert, exec)
- [ ] Rate limiting on auth endpoints
- [ ] CORS tighten for production
- [ ] Web frontend wired to API (WASM app currently standalone)
- [ ] Billing/subscriptions (deferred)
