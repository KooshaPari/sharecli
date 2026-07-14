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

# -------- MVP finality / OS parity (docs/deploy/FINALITY.md) --------
[group: 'parity']
build-cli: build-release
    @echo ">> CLI release binary ready (lane A / GA)"

[group: 'parity']
[windows]
build-cli-windows:
    @echo ">> Windows CLI native release (lane A / GA)"
    @cargo build --release --locked --bin sharecli

[group: 'parity']
[unix]
build-cli-windows:
    @echo ">> Windows CLI cross-compile (optional; CI uses windows-latest)"
    @echo "    rustup target add x86_64-pc-windows-msvc  # plus appropriate linker"
    -cargo build --release --locked --bin sharecli --target x86_64-pc-windows-msvc

[group: 'parity']
build-tray-linux:
    @echo ">> Linux StatusNotifier tray (lane B / beta)"
    @cargo build -p sharecli-tray-linux --release --locked

[group: 'parity']
build-tray-macos:
    @echo ">> macOS Swift tray + desktop (lane B+C / beta)"
    @cargo build -p sharecli-ffi --release --locked
    @cd desktop/ShareCLITray && swift build -c release \
        -Xlinker -L -Xlinker ../../target/release \
        -Xlinker -lsharecli_ffi

[group: 'parity']
build-desktop-macos: build-tray-macos
    @echo ">> alias: ShareCLITray includes DashboardView"

[group: 'parity']
build-tray-windows:
    @echo ">> Windows WinUI tray (lane B / beta)"
    @dotnet build windows/ShareCLITray/ShareCLITray.csproj -c Release

[group: 'parity']
wsl-parity-check:
    @echo ">> WSL parity checklist (see docs/deploy/FINALITY.md)"
    @grep -q 'WSL bridge' docs/deploy/FINALITY.md
    @grep -q '127.0.0.1:9000' docs/deploy/FINALITY.md
    @grep -q 'WSL (CLI in WSL' README.md
    @grep -q 'x86_64-pc-windows-msvc' .github/workflows/release.yml
    @grep -q 'tray-macos' .github/workflows/release.yml
    @echo "1) WSL: sharecli serve --bind 0.0.0.0:9000"
    @echo "2) Windows: curl http://127.0.0.1:9000/healthz  OR  ShareCLITray"
    @echo "3) Optional WSLg: sharecli-tray on Linux session"
    @echo ">> wsl-parity-check OK"

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

# Soft hermetic: fetch then offline build (C06 L54). See docs/ops/hermetic-builds.md
[group: 'gate']
hermetic:
    @echo ">> cargo fetch --locked && cargo build --locked --offline -p sharecli"
    @cargo fetch --locked
    @cargo build --locked --offline -p sharecli
    @echo ">> hermetic soft gate green"

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

# -------- observability (C05 L45) --------
[group: 'observability']
pyro-push-sample:
    @echo ">> curl pprof from sharecli serve and push to Pyroscope (see docs/ops/pyroscope.md)"
    @test -n "${SHARECLI_PPROF:-}" || { echo "error: set SHARECLI_PPROF=1 and run sharecli serve first"; exit 1; }
    @PYRO_URL="${SHARECLI_PYROSCOPE_URL:-http://127.0.0.1:4040}"; \
     APP="${SHARECLI_PYROSCOPE_APP_NAME:-sharecli}"; \
     PPROF_URL="${SHARECLI_PPROF_URL:-http://127.0.0.1:9000/debug/pprof/profile}"; \
     SECS="${SHARECLI_PPROF_SECONDS:-10}"; \
     OUT="/tmp/sharecli-profile-$$.pb"; \
     CURL_AUTH=(); \
     if [ -n "${SHARECLI_SERVE_TOKEN:-}" ]; then CURL_AUTH=(-H "Authorization: Bearer ${SHARECLI_SERVE_TOKEN}"); fi; \
     echo ">> sampling ${PPROF_URL}?seconds=${SECS}"; \
     curl -fsS "${CURL_AUTH[@]}" -o "$$OUT" "$${PPROF_URL}?seconds=$${SECS}"; \
     if command -v pyroscope >/dev/null 2>&1; then \
       echo ">> pyroscope push -> $${PYRO_URL}"; \
       pyroscope push "$$OUT" --application-name="$${APP}" --server-address="$${PYRO_URL}"; \
     elif [ -n "$${SHARECLI_PYROSCOPE_USER:-}" ] && [ -n "$${SHARECLI_PYROSCOPE_PASSWORD:-}" ]; then \
       echo ">> curl ingest -> $${PYRO_URL}/ingest"; \
       curl -fsS -u "$${SHARECLI_PYROSCOPE_USER}:$${SHARECLI_PYROSCOPE_PASSWORD}" \
         -H "Content-Type: application/vnd.google.protobuf" \
         --data-binary @"$$OUT" \
         "$${PYRO_URL}/ingest?format=pprof&name=$${APP}"; \
     else \
       echo ">> no pyroscope CLI or Grafana Cloud creds — saved $$OUT"; \
       echo ">> install pyroscope CLI or set SHARECLI_PYROSCOPE_USER/PASSWORD; see docs/ops/pyroscope.md"; \
       go tool pprof -top "$$OUT" 2>/dev/null || true; \
     fi; \
     rm -f "$$OUT"

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

# Soft: validate docs/eval/corpus scenario JSON
eval-corpus:
    bash scripts/eval/run-corpus.sh

# -------- C05 load (soft) --------
[group: 'ops']
load-soft:
    @echo ">> soft healthz burst (requires curl; starts serve on :9000)"
    @cargo build --locked --release -p sharecli
    @bash -c 'set -euo pipefail; ./target/release/sharecli serve --bind 127.0.0.1:9000 & pid=$!; trap "kill $$pid 2>/dev/null || true" EXIT; for i in $$(seq 1 30); do curl -sf -o /dev/null http://127.0.0.1:9000/healthz && break; sleep 1; done; SHARECLI_LOAD_URL=http://127.0.0.1:9000/healthz SHARECLI_LOAD_N=50 bash scripts/load/healthz_burst.sh'

