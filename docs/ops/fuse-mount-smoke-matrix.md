# FUSE mount-smoke matrix (AC-009.22+)

Operator / agent guide for privileged FUSE smoke across Linux, macOS, WSL2, and
Windows (WinFsp) — including Mac-hosted Linux VMs/containers **without** a host
macFUSE reboot.

## Quick start

```bash
# From sharecli repo root
cargo build -p fuse-smoke-runner --locked
cargo run -p fuse-smoke-runner --locked -- --json

# Or via just
just fuse-smoke
just fuse-smoke -- --cell mac_host_linux_colima --json
```

Environment:

| Variable | Role |
|----------|------|
| `SHARECLI_FUSE_MOUNT_SMOKE=1` | Opt-in for in-process / cargo test smoke |
| `SHARECLI_TART_FUSE_IMAGE` | Tart VM name (default `sharecli-fuse-macos`) |
| `SHARECLI_TART_SSH_USER` | SSH user inside Tart guest (default `admin`) |

## Cells

| Cell id | What runs | Green without host macFUSE reboot? |
|---------|-----------|--------------------------------------|
| `linux_native` | Host cargo smoke + `/dev/fuse` | Yes (on Linux) |
| `linux_container` | `Containerfile.fuse-smoke` + Docker `--device /dev/fuse` | Yes (needs Docker daemon) |
| `mac_host_linux_colima` | Starts Colima if needed, then Linux container smoke | **Yes — preferred on Mac** |
| `macos_native` | Host cargo smoke + macFUSE | Needs Driver Extension |
| `macos_vm_tart` | SSH into Tart macOS VM with macFUSE prebaked | Yes on host if Tart image ready |
| `wsl2` | `wsl.exe` + fuse3 + cargo smoke | Windows + WSL2 |
| `windows_winfsp` | WinFsp + cargo smoke | Windows + WinFsp installed |

Loud fail reasons (never silent skip when a cell is selected):
`driver_missing`, `winfsp_missing`, `no_fuse_device`, `unsupported_arch`,
`tooling_missing`, `host_os_mismatch`, `smoke_failed`.

## Mac without reboot

1. `brew install colima docker qemu` (qemu required for working FUSE mounts).
2. Prefer QEMU VM type (Apple Virtualization / `vz` often returns **ENOSYS** for FUSE):

```bash
colima delete -f
colima start --vm-type qemu --runtime docker --cpu 2 --memory 4
just fuse-smoke-colima
```

`fuse-smoke` auto-picks `--vm-type qemu` when `qemu-img` is on PATH.

### Tart macOS VM bake (optional `macos_vm_tart`)

Host reboot stays skipped; bake macFUSE **once inside the guest image**:

1. `brew install cirruslabs/cli/tart`
2. Create/clone a macOS VM; inside the guest install macFUSE and enable
   **Driver Extensions**, then reboot **the guest** once.
3. Snapshot/name the VM `sharecli-fuse-macos` (or set `SHARECLI_TART_FUSE_IMAGE`).
4. Ensure SSH works and the sharecli checkout is reachable at the same path
   (or adjust the remote `cd` in the runner).

## WSL2

On Windows: install WSL2 + `fuse3`/`libfuse3-dev` inside the distro, then:

```text
fuse-smoke --cell wsl2
```

## Windows native (WinFsp)

Install [WinFsp](https://winfsp.dev) with **Developer** files. Then:

```text
fuse-smoke --cell windows_winfsp
```

Backend lands under AC-009.25 (`sharecli-fuse` WinFsp adapter).

## Production Containerfile

Do **not** add FUSE to the hardened serve [`Containerfile`](../../Containerfile).
Smoke uses [`Containerfile.fuse-smoke`](../../Containerfile.fuse-smoke) only.

## Local gate vs CI

KooshaPari Actions billing often fails immediately — treat
`just fuse-smoke` / `fuse-smoke --json` as the merge gate. Workflow
`.github/workflows/fuse-mount-smoke.yml` (AC-009.26) mirrors cells for when
runners work.
