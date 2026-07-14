#!/usr/bin/env bash
# Soft guard: fail if tracked files under crates/ look like build artifacts.
# Intended for CI (continue-on-error) and local pre-push hygiene.
set -euo pipefail

violations=()

while IFS= read -r path; do
  case "$path" in
    crates/*/.zig-cache/*|crates/*/zig-cache/*|crates/*/zig-out/*)
      violations+=("$path")
      ;;
    crates/*/*.o|crates/*/*.a|crates/*/*.so|crates/*/*.dylib|crates/*/*.dll|crates/*/*.exe)
      violations+=("$path")
      ;;
  esac
done < <(git ls-files 'crates/')

if ((${#violations[@]} > 0)); then
  echo "check-no-build-artifacts: tracked build artifacts under crates/:" >&2
  printf '  %s\n' "${violations[@]}" >&2
  echo "Remove with: git rm -r --cached <path>  (sources stay on disk)" >&2
  exit 1
fi

echo "check-no-build-artifacts: OK — no tracked zig caches or binary artifacts under crates/"
