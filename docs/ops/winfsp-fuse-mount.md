# Windows WinFsp FUSE mount (AC-009.25)

On Windows, `sharecli fuse mount` uses **WinFsp** (`crates/sharecli-fuse/src/winfsp_mount.rs`)
instead of libfuse/`fuser`.

## Requirements

1. Install [WinFsp](https://winfsp.dev) including **Developer** files.
2. Loud fail reason when missing: `winfsp_missing`.
3. Write provenance uses NTFS ADS/EA via the `xattr` crate — failures are never
   silent.

## Smoke

```text
set SHARECLI_FUSE_MOUNT_SMOKE=1
cargo run -p sharecli-fuse --bin fuse-mount-smoke
# or matrix cell (from AC-009.22+):
fuse-smoke --cell windows_winfsp
```

## Limits (loud)

- `--cow` / `fuse commit|discard` require InterceptFs (Linux/macOS). Windows mounts
  are passthrough + provenance until CoW is ported behind WinFsp.

See also: matrix doc on the smoke-matrix branch (`docs/ops/fuse-mount-smoke-matrix.md`).
