# sharecli — task runner
# Spec: https://github.com/casey/just
# Use `just` (no args) to list available recipes.

set shell := ["bash", "-uc"]
set dotenv-load := true
set positional-arguments := true

# -------- project metadata --------
app      := "sharecli"
registry := env_var_or_default("CARGO_REGISTRY", "crates-io")

# -------- default recipe (lists everything) --------
default:
    @just --list --unsorted

# -------- setup --------
[group: 'setup']
install-tools:
    @echo ">> installing cargo extensions (deny, audit, tarpaulin, typos, nextest)"
    @command -v cargo-deny    >/dev/null 2>&1 || cargo install --locked cargo-deny
    @command -v cargo-audit   >/dev/null 2>&1 || cargo install --locked cargo-audit
    @command -v cargo-llvm-cov >/dev/null 2>&1 || cargo install --locked cargo-llvm-cov
    @command -v typos         >/dev/null 2>&1 || cargo install --locked typos-cli
    @command -v cargo-nextest >/dev/null 2>&1 || cargo install --locked cargo-nextest
    @echo ">> tools ready"

# -------- formatting --------
[group: 'fmt']
fmt:
    @echo ">> cargo fmt"
    @cargo fmt --all

[group: 'fmt']
fmt-check:
    @echo ">> cargo fmt --check"
    @cargo fmt --all -- --check

# -------- linting --------
[group: 'lint']
lint:
    @echo ">> cargo clippy (all targets, all features, deny warnings)"
    @cargo clippy --all-targets --all-features --locked -- -D warnings

[group: 'lint']
lint-pedantic:
    @echo ">> cargo clippy pedantic"
    @cargo clippy --all-targets --all-features --locked -- -W clippy::pedantic -D warnings

# -------- build --------
[group: 'build']
build:
    @echo ">> cargo build (debug)"
    @cargo build --locked --all-features

[group: 'build']
build-release:
    @echo ">> cargo build --release"
    @cargo build --release --locked --all-features

# -------- testing --------
[group: 'test']
test:
    @echo ">> cargo test (all features, all targets)"
    @cargo test --locked --all-features --all-targets

[group: 'test']
test-nextest:
    @echo ">> cargo nextest run (faster, JUnit XML)"
    @command -v cargo-nextest >/dev/null 2>&1 || cargo install --locked cargo-nextest
    @cargo nextest run --locked --all-features --profile ci

[group: 'test']
test-doc:
    @echo ">> cargo test --doc"
    @cargo test --doc --locked --all-features

# -------- coverage --------
[group: 'coverage']
coverage:
    @echo ">> cargo llvm-cov (lcov)"
    @command -v cargo-llvm-cov >/dev/null 2>&1 || cargo install --locked cargo-llvm-cov
    @cargo llvm-cov --locked --all-features --workspace --lcov --output-path lcov.info
    @echo ">> coverage written to lcov.info"

# -------- security --------
[group: 'security']
audit:
    @echo ">> cargo audit (RustSec advisories)"
    @command -v cargo-audit >/dev/null 2>&1 || cargo install --locked cargo-audit
    @cargo audit

[group: 'security']
deny:
    @echo ">> cargo deny check (licenses, bans, advisories, sources)"
    @command -v cargo-deny >/dev/null 2>&1 || cargo install --locked cargo-deny
    @cargo deny check

# -------- doc --------
[group: 'doc']
doc:
    @echo ">> cargo doc (open in browser)"
    @cargo doc --no-deps --locked --all-features --open

[group: 'doc']
doc-build:
    @echo ">> cargo doc (build only)"
    @cargo doc --no-deps --locked --all-features

# -------- quality gates --------
[group: 'gate']
gate: lint test audit deny fmt-check
    @echo ">> all gates green"

[group: 'gate']
gate-release: lint-pedantic test audit deny fmt-check build-release
    @echo ">> release gate green"

# -------- repo hygiene --------
[group: 'hygiene']
typos:
    @echo ">> typos (spellcheck)"
    @command -v typos >/dev/null 2>&1 || cargo install --locked typos-cli
    @typos --config _typos.toml

[group: 'hygiene']
clean:
    @echo ">> cargo clean (target/)"
    @cargo clean

[group: 'hygiene']
outdated:
    @echo ">> cargo outdated"
    @command -v cargo-outdated >/dev/null 2>&1 || cargo install --locked cargo-outdated
    @cargo outdated --workspace

# -------- observability --------
[group: 'hygiene']
grade:
    @echo ">> audit scorecard snapshot"
    @test -f audit_scorecard.json && cat audit_scorecard.json | jq '{repo, overall, grade}' \
        || echo "no audit_scorecard.json present"

# -------- local CI simulation --------
# Mirrors .github/workflows/ci.yml — useful for `act` or local debugging
[group: 'ci']
ci: install-tools fmt-check lint test build audit deny
    @echo ">> local CI pipeline green"

[group: 'ci']
ci-fast: fmt-check lint test build
    @echo ">> local CI fast lane green"

# -------- release --------
[group: 'release']
version:
    @grep '^version' Cargo.toml | head -1 | cut -d'"' -f2

[group: 'release']
changelog:
    @echo ">> generating CHANGELOG.md via git-cliff"
    @command -v git-cliff >/dev/null 2>&1 || cargo install --locked git-cliff
    @git-cliff --tag "$(just version)" --output CHANGELOG.md

[group: 'release']
publish: build-release
    @echo ">> cargo publish --dry-run to {{ registry }}"
    @cargo publish --dry-run --locked

# C06 L52 — bit-identical release binary (unix only; skips on Windows).
[group: 'release']
repro-check:
    @echo ">> reproducible build digest check (SOURCE_DATE_EPOCH)"
    @bash scripts/repro-check.sh

# -------- C07 DevEx (append-only) --------
[group: 'devex']
dev: install-tools
    @echo ">> one-command local bootstrap (Rust + Zig + smoke)"
    @command -v zig >/dev/null 2>&1 || { echo "error: zig 0.14.1 required (see .devcontainer/post-create.sh)"; exit 1; }
    @cargo build --locked --all-features
    @cargo run --locked --quiet -- --help >/dev/null
    @echo ">> ready — next: just test-nextest | just gate"

[group: 'devex']
mutants:
    @echo ">> cargo-mutants smoke (not a CI gate yet)"
    @command -v cargo-mutants >/dev/null 2>&1 || cargo install --locked cargo-mutants
    @cargo mutants --timeout 60 --jobs 2 -- --locked --all-features

[group: 'devex']
fuzz:
    @echo ">> cargo fuzz toml_lite (30s; requires nightly + cargo-fuzz)"
    @command -v cargo-fuzz >/dev/null 2>&1 || cargo install --locked cargo-fuzz
    @rustup toolchain install nightly --profile minimal
    @cargo +nightly fuzz run toml_lite -- -max_total_time=30
