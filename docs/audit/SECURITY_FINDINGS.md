# GoharScribe Security Findings

**Date:** 2026-10-09
**Scope:** Full repository security audit

## Summary

Phase 1 hardening (2026-10-09) addressed: image decompression bombs, MCP filesystem jail,
control-server authentication. This audit found 4 additional DoS bypasses, all fixed.

## Verified Phase 1 fixes

### Image decompression bombs — VERIFIED
- `crates/pdf/src/lib.rs:946-949` — dimension check before decode
- `crates/render/src/lib.rs` — dimension check before decode
- `crates/formats/src/rtf.rs:163` — MAX_IMAGE_SIDE check
- Regression tests with hostile 50k×50k PNG headers present.

### MCP filesystem jail — VERIFIED
- `crates/mcp/src/server.rs:55-56` — `--jail` / `GOHARSCRIBE_MCP_JAIL`
- Canonicalization blocks `..` and symlink escapes.

### Control-server authentication — VERIFIED
- `apps/goharscribe/src/control_server.rs:17-45` — bearer token required
- Constant-time comparison.

## New findings (all fixed)

### SEC-1 (High): para.set bypasses 64-tab cap
**Status:** Fixed in a5852dd. See REMEDIATION_LOG.md.

### SEC-2 (High): layout.pageSetup section bypasses validation
**Status:** Fixed in a5852dd.

### SEC-3 (Medium): format.set unbounded strings/sizes
**Status:** Fixed in a5852dd.

### SEC-4 (Medium): base64_decode preallocates before validation
**Status:** Fixed in a5852dd.

## Dependency audit (from security subagent)

**quick-xml 0.38.4** has two High-severity DoS advisories:
- RUSTSEC-2026-0194: quadratic duplicate-attribute check
- RUSTSEC-2026-0195: (details in subagent report)

**Recommendation:** Upgrade quick-xml to patched version. (Not done in this audit — requires dependency update and testing.)
