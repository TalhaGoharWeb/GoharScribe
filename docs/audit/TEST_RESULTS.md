# GoharScribe Test Results

**Date:** 2026-10-09
**Baseline commit:** 778b4ae

## Baseline CI (`cargo xtask ci`)

| Step | Result |
|------|--------|
| fmt (`cargo fmt --all -- --check`) | PASS |
| clippy (`-D warnings`) | PASS |
| test (`cargo test --workspace`) | PASS (250+ tests, 0 failures) |
| assets | PASS |
| layers | **FAIL** (1 violation: goharscribe-input unregistered) |
| wasm | SKIPPED (CI stopped at layers) |

## Post-fix verification

| Step | Result |
|------|--------|
| layers (`cargo xtask layers`) | PASS ("OK: 17 crates, no layering violations.") |

## Full CI re-run

(Pending — waiting for subagents to finish to avoid resource contention.)
