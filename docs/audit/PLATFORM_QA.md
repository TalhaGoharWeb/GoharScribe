# GoharScribe Cross-Platform QA Report — 2026-10-09

## Platform Matrix

| Target | Build | Tests | Runtime | Notes |
|--------|-------|-------|---------|-------|
| Linux x86_64 | ✅ | ✅ 250+ pass | ✅ Verified | Primary dev platform |
| Windows x86_64 | ✅ CI | ✅ CI | ⚠️ Unverified | Release workflow builds; no manual test |
| Windows ARM64 | ✅ CI | ✅ CI | ⚠️ Unverified | Dedicated workflow |
| macOS | ✅ CI | ✅ CI | ⚠️ Unverified | Signing/notarization in release env only |
| FreeBSD | ✅ CI | ✅ CI | ⚠️ Unverified | Dedicated workflow |
| WebAssembly | ⚠️ Target not installed | — | — | `wasm32-unknown-unknown` not in CI env |

## CI Configuration

**Workflows:**
- `release.yml` — multi-platform release builds, signing, draft releases
- `windows-arm64.yml` — Windows ARM64 specific
- `freebsd.yml` — FreeBSD builds
- `packaging-lint.yml` — package metadata checks

**Checks (via `cargo xtask ci`):**
- `fmt` — rustfmt check
- `clippy` — with `-D warnings`
- `test` — full workspace
- `assets` — ATTRIBUTION.md completeness
- `layers` — architecture layering
- `wasm` — WebAssembly build (skipped if target missing)

## Toolchain

- **Declared:** `rust-version = "1.95"` (updated from 1.90)
- **CI uses:** `dtolnay/rust-toolchain@stable` (currently 1.99)
- **Verified with:** Rust 1.99.0

The 1.90 declaration was incorrect — dependencies require 1.95+.

## Cross-Platform Code

**Platform-specific implementations:**
- TTS: macOS (`say`), Windows (PowerShell), Linux (espeak) — `crates/engine/src/cmd/tools.rs:798-815`
- All use structured args, no shell injection

**Font discovery:**
- Windows: `%WINDIR%/Fonts`
- macOS: `/System/Library/Fonts`, `~/Library/Fonts`
- Linux: fontconfig
- Bundled fonts work on all platforms

**File paths:**
- Uses `std::path::Path` throughout (platform-correct)
- Temp files use `std::env::temp_dir()` (platform-correct)

## Known Limitations

1. **WebAssembly target** not installed in current environment — cannot verify WASM build
2. **Manual runtime testing** only on Linux — Windows/macOS/FreeBSD rely on CI
3. **Font rendering** may differ by platform due to system font availability
4. **IME integration** — platform-specific, not fully tested on all targets
