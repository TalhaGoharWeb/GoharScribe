# GoharScribe Codebase Audit Report — 2026-10-10

Consolidates: `docs/audit/REPOSITORY_AUDIT.md`, `docs/audit/TYPOGRAPHY_AUDIT.md`,
`docs/audit/RELIABILITY_AUDIT.md`, `docs/audit/PLATFORM_QA.md`, `docs/audit/FINAL_VERIFICATION.md`,
`docs/ROADMAP_PRODUCT_QUALITY.md`.

## 1. Repository Inventory

**Workspace:** 19 crates (18 + new `server`), 3 apps, 1 xtask.

| Layer | Crates | Purpose |
|-------|--------|---------|
| L0 | geom | units, rects |
| L1 | doc, fonts, input, proof | document model, font DB/shaping, Urdu input, proofing |
| L2 | layout, docx, formats | line breaking/pagination, OOXML I/O, TXT/MD/HTML/RTF/ODT |
| L3 | render, pdf | rasterizer, PDF export |
| L4 | engine | Session, undo, 389-command registry, catalog |
| L5 | mcp, **server** | MCP server, **SaaS backend (new)** |
| L6 | ui-egui | egui desktop frontend |

**Apps:** `goharscribe` (desktop GUI), `goharscribe-cli`, `goharscribe-web` (WASM).

## 2. Baseline Test Results (2026-10-10)

| Suite | Result |
|-------|--------|
| `cargo fmt --all -- --check` | ✅ Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ Pass |
| `cargo xtask layers` | ✅ 18 crates, no violations |
| `goharscribe-engine` lib | ✅ 41 passed |
| `goharscribe-fonts` lib | ✅ 20 passed (17 + 3 BIDI) |
| `goharscribe-formats` lib | ✅ 33 passed |
| `goharscribe-server` integration | ✅ 9 passed |
| Full `cargo test --workspace` | ⚠️ Not run (18+ min; engine/fonts/formats/server verified individually) |
| WASM build | ⚠️ Target not installed |

## 3. Feature Inventory (verified, not from README)

- **389 commands** registered, all with implementations (verified via `CommandSpec::new` count).
- **354/405** catalog features have commands (87.4%). Missing 51 documented in `docs/parity.md`.
- **Draw tab: 0/11** — ink/drawing tools entirely absent.
- **Proofing:** English only (`en-us.dic`). No Urdu/Arabic dictionaries.
- **Fonts:** 17 bundled OFL families including Noto Nastaliq Urdu, Noto Naskh Arabic, Amiri.

## 4. Workflows Tested via CLI

- ✅ Create doc with Urdu text → save JSON → convert to PDF (12KB, renders)
- ✅ DOCX round-trip preserves `Hello اردو 123 English`
- ✅ `text.insert`, `format.font` commands execute headlessly
- ✅ SaaS: signup → login → create → get → update → delete → convert

## 5. Defects Fixed (this audit cycle)

| ID | Severity | Description | Fix |
|----|----------|-------------|-----|
| BIDI-1 | High | No UAX #9 visual reordering; mixed RTL/LTR rendered wrong | Rewrote `direction_runs()` + `shape()` using `unicode-bidi 0.3.18`; 3 regression tests |
| UI-HANG-1 | High | Font dropdown froze on Noto Nastaliq Urdu preview | Disabled per-font preview rendering (plain labels) |
| UNDO-1 | High | Removing `.pure()` from undo/redo broke checkpoint logic | Excluded undo/redo from checkpoint creation in `session.rs` |
| PASTE-1 | Medium | `edit.pasteMerge` label promised "Merge Formatting", did plain paste | Removed misleading ribbon button |
| PASTE-2 | Medium | `edit.paste` silently discarded `image` param | Now returns honest error |
| PURE-1 | Low | `edit.undo`/`edit.redo` marked `.pure()` despite mutating | Removed incorrect markers |
| TOOLCHAIN-1 | Low | `rust-version = "1.90"` but deps need 1.95+ | Updated to 1.95 |

## 6. Remaining Known Issues

| ID | Severity | Description |
|----|----------|-------------|
| HIT-1 | High | Hit testing lacks logical↔visual mapping for BIDI text |
| PROOF-1 | High | No Urdu/Arabic spellcheck dictionaries |
| PREV-1 | Medium | Font previews disabled (workaround, not a proper fix) |
| DRAW-1 | Medium | Draw tab 0/11 implemented |
| WASM-1 | Low | WASM target not installed; cannot verify web build |

## 7. Dead Code Removed

- `edit.pasteMerge` ribbon button (duplicate of `pasteText` with misleading label).
- Unused `byte_pos` variable, unused `info` variable in BIDI rewrite (clippy).
- No other dead code found: 0 TODOs/FIXMEs in production, all 389 commands reachable.

## 8. Architecture Assessment

**Strengths:** Clean layering (enforced by xtask), command pattern with undo, thorough security hardening (MCP jail, input validation, bounded parsing).

**Bottlenecks:** `Previews::snippet()` runs sync layout+raster on UI thread. Font DB `include_bytes!` adds ~2MB for Urdu/Arabic fonts (acceptable).

**New:** `crates/server` (L5) — Axum REST API, JWT+Argon2 auth, SQLite storage, document CRUD, conversion, headless command execution. 9 integration tests pass.
