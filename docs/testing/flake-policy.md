# Flake policy

How sharecli treats intermittent test failures (C07 / L68).

## Goals

- Keep CI signal high: a red job means a real regression, not noise.
- Prefer root-cause fixes over retries.
- When a flake is confirmed, quarantine it with an owner and expiry — never delete the test.

## Detection

1. **CI retries** — `.config/nextest.toml` `[profile.ci]` retries failed tests twice with a fixed 1s delay. Retries are a safety net, not a license to ignore flakes.
2. **Local reproduction** — re-run the failing filter under load:

   ```bash
   cargo nextest run --profile ci -E 'test(<name>)' --run-ignored all
   # or hammer a suspect:
   for i in $(seq 1 20); do cargo test --locked --all-features <name> || break; done
   ```

3. **Symptoms that count as flakes** — pass/fail flip without code change; order-dependent failures; timing/sleep races; shared temp-dir / port collisions.

## Quarantine process

1. Open an issue titled `flake: <test name>` with OS, nextest/cargo output, and last green SHA.
2. Rename or gate the test so it is easy to filter (prefer a `#[ignore = "flake: #NNNN"]` with the issue number, or a name containing `_quarantine_`).
3. Add a nextest override only when needed (see `.config/nextest.toml` — keep overrides documented and temporary).
4. Assign an owner and a **remove-by** date (default: 14 days). Quarantine without an expiry is not allowed.
5. Fix or delete the quarantine entry in the same PR that lands the root-cause fix.

## What not to do

- Do not widen CI retries globally to hide a known flake.
- Do not skip the entire integration suite because one test is flaky.
- Do not disable `RUSTFLAGS=-D warnings` or quality gates to “make green.”

## Related recipes

| Command | Role |
|---------|------|
| `just test-nextest` | Local run with `--profile ci` (retries + JUnit) |
| `just test` | Plain `cargo test` (no nextest retries) |
| `just test-doc` | Doctests (nextest does not run these) |

Windows CI matrix expansion is deferred; flake triage on Windows remains manual until that matrix lands.
