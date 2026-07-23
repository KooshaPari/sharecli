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

1. Install Tart: `brew trust cirruslabs/cli && brew install cirruslabs/cli/tart`
   (Homebrew may refuse the tap until trusted).
2. Create/clone a macOS VM (`tart create` / `tart clone`); inside the **guest**:
   - Install macFUSE (pkg from https://osxfuse.github.io or brew in guest).
   - Enable **System Settings → General → Login Items & Extensions → Driver Extensions**
     for macFUSE; reboot **the guest** once (not the host).
3. Snapshot/name the VM `sharecli-fuse-macos` (or set `SHARECLI_TART_FUSE_IMAGE`).
4. Ensure SSH works (`tart ip` + SSH keys) and the sharecli checkout is reachable
   at the same path (or adjust the remote `cd` in the runner).

**Host status (2026-07-22):** Tart is not installed by default on Phenotype Macs;
`macos_vm_tart` loud-fails with `tooling_missing` until steps 1–4 complete.
`mac_host_linux_colima` remains the no-reboot green path.

### macOS native Driver Extension (`macos_native`)

1. Install macFUSE if missing.
2. Open **System Settings → General → Login Items & Extensions → Driver Extensions**,
   enable macFUSE, then reboot the Mac (or a macOS guest).
3. Confirm `/dev/macfuse*` exists, then: `just fuse-smoke -- --cell macos_native`.

Without the extension, the cell fails loudly with `driver_missing` (never a soft pass).

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
