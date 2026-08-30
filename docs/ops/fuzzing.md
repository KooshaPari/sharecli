# Fuzzing Operations

## Overview

sharecli uses `cargo-fuzz` with libFuzzer for continuous fuzz testing of parser and protocol handling code. The fuzz harness runs nightly via GitHub Actions with artifact upload for crash reproduction.

## Fuzz targets

| Target | Module | Description |
|--------|--------|-------------|
| `toml_lite` | `sharecli::util::toml_lite` | TOML config parser |
| `dns_query_parser` | DNS wire-format parser | Protocol parser |
| `snmpv3_msg` | SNMPv3 message decoder | Protocol parser |
| `ssh_packet` | SSH packet scanner | Protocol parser |
| `coap_option_parse` | CoAP option decoder | Protocol parser |
| `ldap_filter` | LDAP filter parser | Protocol parser |

## Running locally

```bash
# Install cargo-fuzz (nightly required)
cargo install --locked cargo-fuzz

# Fuzz a specific target for 60 seconds
cargo +nightly fuzz run toml_lite -- -max_total_time=60

# Fuzz with seed corpus
cargo +nightly fuzz run toml_lite --fuzz-corpus fuzz/corpora/toml_lite/

# Fuzz all targets sequentially (5 min each)
for t in toml_lite dns_query_parser snmpv3_msg ssh_packet coap_option_parse ldap_filter; do
  cargo +nightly fuzz run $t -- -max_total_time=300
done
```

## Corpus management

- **Seed corpus**: `fuzz/corpora/<target>/seed-01.dict` — minimal valid input per target
- **Accumulated corpus**: `fuzz/corpus/<target>/` — grown by libFuzzer during runs
- **Crash artifacts**: `fuzz/artifacts/<target>/` — inputs that trigger panics/hangs

### Adding a new seed

1. Create a minimal valid input file in `fuzz/corpora/<target>/`
2. The filename should be descriptive (e.g., `valid-toml.toml`)
3. Commit the seed to the repository

## CI integration

- **Nightly soft job**: `.github/workflows/fuzz-soft.yml` — runs all 6 targets for 300s on `ubuntu-24.04`
- **Crash artifacts**: uploaded with 14-day retention
- **Corpus seeds**: uploaded with 30-day retention
- **continue-on-error: true**: fuzz failures are advisory, not blocking

## Triage

1. Check `fuzz/artifacts/<target>/` for crash reproducer files
2. Reproduce locally: `cargo +nightly fuzz run <target> fuzz/artifacts/<target>/crash-<hash>`
3. Minimize: `cargo +nightly fuzz tmin <target> fuzz/artifacts/<target>/crash-<hash>`
4. Fix the root cause in the target module
5. Add the minimized reproducer to `fuzz/corpora/<target>/` as a regression test

## FR-003 traceability

- `tests/c07_l67_fuzz.rs` — acceptance gates asserting:
  - Fuzz workspace (`fuzz/Cargo.toml`) exists and is properly configured
  - All 6 fuzz targets are registered in `Cargo.toml` as `[[bin]]` entries
  - Fuzz target source files exist with `fuzz_target!` macro
  - Seed corpus directories exist for each target
  - CI workflow exists with correct matrix and artifact upload
  - Fuzz directory structure matches expected layout
