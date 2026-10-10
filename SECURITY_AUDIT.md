# GoharScribe Security Audit — 2026-10-10

Consolidates `docs/audit/SECURITY_FINDINGS.md` and `docs/audit/REMEDIATION_LOG.md`
(3 prior remediation rounds) plus the new SaaS server review.

## Remediation History

| Round | Date | Fixes | Commit |
|-------|------|-------|--------|
| 1 | 2026-10-09 | 13 issues: 4 DoS bypasses, 3 QA bugs, 4 format bugs | `bf5aed1e` |
| 2 | 2026-10-09 | H2 MCP jail bypass (7 commands), H3 picture OOM | `2e52115d` |
| 3 | 2026-10-09 | H1 quick-xml 0.38→0.41, H4/H5/M1/M3/M6/M7/M8/M9/M10 | `a2969d52` |

## Findings by Severity

### High (all remediated)

| ID | Component | Description | Status |
|----|-----------|-------------|--------|
| H1 | `crates/docx` | quick-xml 0.38 had 2 DoS advisories (entity expansion) | ✅ Upgraded to 0.41 |
| H2 | `crates/mcp` | Filesystem jail bypass via 7 unlisted commands | ✅ Added to `PATH_COMMANDS` |
| H3 | render/pdf/rtf | Picture adjust OOM (no dimension pre-check) | ✅ Pre-check added |

### Medium (all remediated)

| ID | Description | Status |
|----|-------------|--------|
| M1 | DOCX attribute cap missing | ✅ |
| M3 | Comment-ID O(n) complexity | ✅ |
| M6/M7 | Macro/batch caps missing | ✅ |
| M8/M10 | Insecure temp files | ✅ Secure temp |
| M9 | FIFOs/devices accepted in `open_path` | ✅ Rejected |
| H4 | Markdown table unbounded (DoS) | ✅ 64-col cap |
| H5 | Bracket scan unbounded | ✅ 10k bound |

## New: SaaS Server Security Review (2026-10-10)

| Check | Result |
|-------|--------|
| Password storage | ✅ Argon2 (OWASP recommended), never plaintext |
| Auth tokens | ✅ JWT, 24h expiry; secret from `GOHARSCRIBE_JWT_SECRET` env, never committed |
| SQL injection | ✅ All queries parameterized (`rusqlite::params!`) |
| Auth bypass | ✅ Every `/api/docs` route requires valid Bearer token; 9 tests incl. cross-user isolation |
| Error leakage | ✅ `ApiError::Internal` returns generic message; DB errors never exposed |
| Input validation | ✅ Email format, password 8–128 chars, title ≤200 chars, format whitelist |
| Panics | ✅ `#![deny(clippy::unwrap_used...)]`, `#![forbid(unsafe_code)]`; `Mutex` poison → `ApiError::Internal` |
| CORS | ⚠️ `allow_origin(Any)` — acceptable for MVP, tighten for production |

## Remaining Risks

1. **CORS `Any`** — tighten to specific origins before production.
2. **JWT secret management** — documented via env var; no rotation mechanism yet.
3. **Rate limiting** — not implemented; add for auth endpoints before public exposure.
4. **`cargo audit`** — could not run (install failed compiling `wasmparser`); dependency advisories not checked this cycle.

## Verification

- `cargo clippy --workspace --all-targets -- -D warnings` ✅
- `hostile_params_never_panic` test ✅ (engine)
- 9 SaaS integration tests ✅ (auth, isolation, CRUD)
