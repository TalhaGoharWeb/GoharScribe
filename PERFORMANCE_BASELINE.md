# Performance Baseline — 2026-10-10

## Build Times (this environment)

| Build | Time |
|-------|------|
| `cargo build -p goharscribe-cli` (debug) | 4m 27s |
| `cargo build --target x86_64-pc-windows-gnu -p goharscribe-cli` | 15m 33s |
| `cargo build -p goharscribe` (GUI, Linux) | did not finish (linking stall, >28 min) |
| `cargo test -p goharscribe-server` (incl. deps) | ~7 min first build, 0.38s test run |

## Binary Sizes

| Binary | Size |
|--------|------|
| `goharscribe-cli` (Linux debug) | 419MB |
| `goharscribe-cli.exe` (Windows debug) | 491MB → 105MB zipped |
| PDF from Urdu test doc | 12KB |

## Test Suite Times

| Suite | Tests | Time |
|-------|-------|------|
| engine lib | 41 | <1s |
| fonts lib | 20 | <1s |
| formats lib | 33 | 0.31s |
| server integration | 9 | 0.38s |

## Known Performance Issues

1. **Font dropdown:** per-font `snippet()` rendering on UI thread froze the app
   (Noto Nastaliq Urdu). Workaround: previews disabled. Proper fix needs
   background-thread rendering.
2. **Debug binary size:** 419–491MB debug symbols; release builds strip to ~1/5.
3. **GUI link time:** Linux desktop link stalled >28 min in this sandbox
   (environment limitation, not a code defect).

## Not Yet Measured

Typing latency, document load time for large docs, pagination of long Urdu books,
PDF export time for 100+ pages, memory usage under sustained editing.
