# Windows WinFsp FUSE mount (AC-009.25 / AC-009.27)

On Windows, `sharecli fuse mount` uses **WinFsp** (`crates/sharecli-fuse/src/winfsp_mount.rs`)
instead of libfuse/`fuser`. CoW uses the shared [`CowMountHandle`](../../crates/sharecli-fuse/src/cow_session.rs).

## Requirements

1. Install [WinFsp](https://winfsp.dev) including **Developer** files.
2. Loud fail reason when missing: `winfsp_missing`.
3. Write provenance uses NTFS ADS/EA via the `xattr` crate — failures are never
   silent.

## Smoke

```text
set SHARECLI_FUSE_MOUNT_SMOKE=1
cargo run -p sharecli-fuse --bin fuse-mount-smoke
# or matrix cell:
fuse-smoke --cell windows_winfsp
```

GHA `fuse-mount-smoke.yml` installs WinFsp via MSI on `windows-latest` (soft /
`continue-on-error` while Actions billing is exhausted).

## CoW (AC-009.27)

```text
sharecli fuse mount BACKING MOUNTPOINT --cow [--agent ID]
sharecli fuse commit path.txt --agent ID
sharecli fuse discard path.txt --agent ID
```

Same per-agent staging under `{cow_root}/{agent}/` as Linux/macOS.

## Host proof checklist (no Windows VM on this Mac)

- [ ] Windows 10/11 machine or VM with WinFsp Developer installed
- [ ] `fuse-mount-smoke` prints `PASS`
- [ ] `fuse mount --cow` + stage/commit round-trip

See also: [`fuse-mount-smoke-matrix.md`](fuse-mount-smoke-matrix.md).
