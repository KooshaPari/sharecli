# WAVE0 Failure Matrix — PR #198 @ 196429e

**Generated:** 2026-07-10
**CI run:** 29046155250 (CI), 29046155320 (Quality Gate), 29046155244 (Audit), 29046155243 (Coverage)

## Summary

| Check | Status | Owner lane | Notes |
|-------|--------|------------|-------|
| cargo fmt | PASS | — | |
| cargo build | PASS | — | |
| cargo deny | PASS | — | |
| cargo clippy | FAIL | ci-clippy | -D warnings / clippy lints |
| cargo test | FAIL | ci-test | compile or runtime |
| Unit Tests | FAIL | ci-test | quality-gate |
| coverage | FAIL | ci-test / follow-on | often cascade |
| cargo audit | FAIL | ci-rustsec | RUSTSEC-2026-0194, 0195 |
| FR Annotation | FAIL | ci-fr | tests/integration_cli.rs |
| CI Success | FAIL | (aggregator) | truthful |

## Clippy fingerprints
```
2026-07-09T19:58:24.1483132Z    [1m[94m--> [0msrc/x12_edi_segment.rs:146:41

```

## cargo test fingerprints
```
2026-07-09T19:58:58.1849123Z [1m[33mwarning[0m: build failed, waiting for other jobs to finish...

```

## Unit Tests fingerprints
```
2026-07-09T19:58:58.8180902Z test result: ok. 1287 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
2026-07-09T20:03:19.2770237Z test result: ok. 1287 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.87s
2026-07-09T20:03:19.7662766Z   fail_ci_if_error: false
2026-07-09T20:03:20.6706350Z Could not pull latest version information: SyntaxError: Unexpected token '<', "<!DOCTYPE "... is not valid JSON
2026-07-09T20:03:21.0828121Z yaml.parser.ParserError: while parsing a block mapping
2026-07-09T20:03:21.0832007Z [PYI-17442:ERROR] Failed to execute script 'main' due to unhandled exception!
2026-07-09T20:03:21.1253368Z ##[warning]Codecov: Failed to properly create commit: The process '/home/runner/work/_actions/codecov/codecov-action/v4/dist/codecov' failed with exit code 1

```

## RustSec fingerprints
```
2026-07-09T19:57:27.1031423Z Complete job name: cargo audit (RustSec)
2026-07-09T19:57:41.2816855Z [1m[92m  Downloaded[0m rustsec v0.33.0
2026-07-09T19:59:31.9315172Z [1m[92m   Compiling[0m rustsec v0.33.0
2026-07-09T19:59:50.3127269Z [0m[0m[1m[32m    Fetching[0m advisory database from `https://github.com/RustSec/advisory-db.git`
2026-07-09T19:59:54.7779492Z [0m[0m[1m[31mCrate:    [0m quick-xml
2026-07-09T19:59:54.7780417Z [0m[0m[1m[31mTitle:    [0m Quadratic run time when checking a start tag for duplicate attribute names
2026-07-09T19:59:54.7781322Z [0m[0m[1m[31mID:       [0m RUSTSEC-2026-0194
2026-07-09T19:59:54.7781812Z [0m[0m[1m[31mURL:      [0m https://rustsec.org/advisories/RUSTSEC-2026-0194
2026-07-09T19:59:54.7782462Z [0m[0m[1m[31mSolution: [0m Upgrade to >=0.41.0
2026-07-09T19:59:54.7782742Z [0m[0m[1m[31mCrate:    [0m quick-xml
2026-07-09T19:59:54.7783997Z [0m[0m[1m[31mTitle:    [0m Unbounded namespace-declaration allocation in `NsReader` enables memory-exhaustion denial of service
2026-07-09T19:59:54.7785066Z [0m[0m[1m[31mID:       [0m RUSTSEC-2026-0195
2026-07-09T19:59:54.7786084Z [0m[0m[1m[31mURL:      [0m https://rustsec.org/advisories/RUSTSEC-2026-0195
2026-07-09T19:59:54.7786959Z [0m[0m[1m[31mSolution: [0m Upgrade to >=0.41.0

```

## Coverage fingerprints
```
2026-07-09T19:57:23.1470104Z [36;1m  printf '::error::install-action: %s\n' "$*"[0m
2026-07-09T20:00:44.9777620Z test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
2026-07-09T20:00:44.9795563Z test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
2026-07-09T20:00:44.9807263Z test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
2026-07-09T20:00:44.9822438Z test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
2026-07-09T20:00:44.9845628Z test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
2026-07-09T20:00:45.6806189Z test result: ok. 1287 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
2026-07-09T20:00:45.7939250Z test result: ok. 209 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
2026-07-09T20:00:45.7973483Z test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
2026-07-09T20:00:45.8000543Z test send_reports_failed_when_ghostty_missing ... ok
2026-07-09T20:00:45.8010737Z test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
2026-07-09T20:00:45.8035038Z test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
2026-07-09T20:00:45.8058225Z test fr_cast_004_send_returns_failed_when_pane_not_resolved ... ok
2026-07-09T20:00:45.8058941Z test fr_cast_004_send_returns_failed_when_send_text_exits_non_zero ... ok
2026-07-09T20:00:45.8059818Z test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

## Lane assignment
- **ci-clippy** → Cargo.toml [lints] + src lint fixes
- **ci-test** → sharecli-ipc + failing tests
- **ci-rustsec** → lockfile / audit ignores for 0194/0195
- **ci-fr** → FR comments in tests/integration_cli.rs
