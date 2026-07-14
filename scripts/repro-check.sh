#!/usr/bin/env bash
# Lightweight reproducibility gate (C06 L52 / FR-002 build determinism).
# Builds the release `sharecli` binary twice with SOURCE_DATE_EPOCH and
# compares SHA-256 digests. Mirrors release.yml flags (no cross-target).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

# Windows hosts: skip locally; unix CI is the enforcement surface.
case "$(uname -s 2>/dev/null || echo unknown)" in
  MINGW*|MSYS*|CYGWIN*|Windows*)
    echo "repro-check: skipped on Windows (unix CI gate only)" >&2
    exit 0
    ;;
esac

export SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-0}"
# Match release.yml — do not inherit CI RUSTFLAGS (-D warnings).
unset RUSTFLAGS

BIN_NAME="sharecli"
CARGO_BIN=(cargo build --release --locked --bin "${BIN_NAME}")

digest_in_dir() {
  local target_dir="$1"
  rm -rf "${target_dir}"
  mkdir -p "${target_dir}"
  CARGO_TARGET_DIR="${target_dir}" SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH}" \
    "${CARGO_BIN[@]}" >/dev/null
  local bin="${target_dir}/release/${BIN_NAME}"
  if [[ ! -f "${bin}" ]]; then
    bin="${target_dir}/release/${BIN_NAME}.exe"
  fi
  if [[ ! -f "${bin}" ]]; then
    echo "repro-check: binary not found under ${target_dir}/release" >&2
    exit 1
  fi
  sha256sum "${bin}" | awk '{print $1}'
}

echo ">> repro-check: SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH}"
TMP_BASE="$(mktemp -d)"
trap 'rm -rf "${TMP_BASE}"' EXIT

D1="$(digest_in_dir "${TMP_BASE}/build-a")"
D2="$(digest_in_dir "${TMP_BASE}/build-b")"

echo ">> digest-1: ${D1}"
echo ">> digest-2: ${D2}"

if [[ "${D1}" != "${D2}" ]]; then
  echo "repro-check: FAIL — digests differ" >&2
  exit 1
fi

echo "repro-check: PASS — bit-identical release binary"
