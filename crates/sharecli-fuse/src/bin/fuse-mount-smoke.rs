//! Privileged mount smoke binary (AC-009.8 / AC-009.16) for container matrix cells.
//! Prefer this over full `sharecli` package tests inside Containerfile.fuse-smoke.

fn main() {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use sharecli_fuse::{fuse_mount_smoke_enabled, run_mount_smoke, ENV_FUSE_MOUNT_SMOKE};
        if !fuse_mount_smoke_enabled() {
            eprintln!(
                "fuse-mount-smoke: set {ENV_FUSE_MOUNT_SMOKE}=1 to run privileged smoke"
            );
            std::process::exit(2);
        }
        let dir = tempfile::tempdir().expect("tempdir");
        if let Err(e) = run_mount_smoke(dir.path()) {
            eprintln!("fuse-mount-smoke: FAIL: {e:#}");
            std::process::exit(1);
        }
        println!("fuse-mount-smoke: PASS");
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        eprintln!(
            "fuse-mount-smoke: unsupported on this platform (need linux/macos; windows via WinFsp AC-009.25)"
        );
        std::process::exit(2);
    }
}
