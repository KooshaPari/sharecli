#!/usr/bin/env bash
# sharecli DevEx bootstrap — Rust stable, Zig 0.14.1, just, nextest.
set -euo pipefail

ZIG_VERSION="0.14.1"

echo ">> rustup: stable + rustfmt + clippy"
rustup default stable
rustup component add rustfmt clippy

echo ">> install just + cargo-nextest (if missing)"
command -v just >/dev/null 2>&1 || cargo install --locked just
command -v cargo-nextest >/dev/null 2>&1 || cargo install --locked cargo-nextest

if command -v zig >/dev/null 2>&1; then
  echo ">> zig already on PATH: $(zig version)"
else
  arch="$(uname -m)"
  case "${arch}" in
    x86_64|amd64) zig_arch="x86_64" ;;
    aarch64|arm64) zig_arch="aarch64" ;;
    *)
      echo "unsupported arch for Zig bootstrap: ${arch}" >&2
      exit 1
      ;;
  esac
  zig_dir="zig-linux-${zig_arch}-${ZIG_VERSION}"
  zig_tarball="${zig_dir}.tar.xz"
  echo ">> installing Zig ${ZIG_VERSION} (${zig_arch})"
  curl -fsSL "https://ziglang.org/download/${ZIG_VERSION}/${zig_tarball}" -o "/tmp/${zig_tarball}"
  sudo tar -xJf "/tmp/${zig_tarball}" -C /usr/local
  sudo ln -sfn "/usr/local/${zig_dir}/zig" /usr/local/bin/zig
  rm -f "/tmp/${zig_tarball}"
  echo ">> zig $(zig version)"
fi

echo ">> cargo fetch (warm registry)"
cargo fetch --locked || cargo fetch

echo ">> post-create complete — try: just dev"
