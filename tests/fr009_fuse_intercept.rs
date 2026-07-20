//! FR-009 — FUSE IO Intercept
//! FR: FR-009
//!
//! AC-009.1 InterceptFs::new over backing path (no privileged mount)
//! AC-009.2 mount API fails loudly on unsupported platforms

use std::path::Path;
use tempfile::TempDir;

/// FR-009 / AC-009.1 — construct InterceptFs without mounting.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn fr009_intercept_fs_constructs_over_backing() {
    let dir = TempDir::new().expect("tempdir");
    let _fs = sharecli_fuse::InterceptFs::new(dir.path());
    // Construction MUST succeed; mount privileges are a separate operator step.
    assert!(dir.path().is_dir());
}

/// FR-009 / AC-009.2 — unsupported platforms reject mount with a clear error.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[test]
fn fr009_mount_unsupported_platform_fails_loudly() {
    let err = sharecli_fuse::mount(Path::new("/tmp/mp"), Path::new("/tmp/back"))
        .expect_err("unsupported platform MUST err");
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("unsupported") || msg.contains("only supported"),
        "expected unsupported platform message, got {msg}"
    );
}

/// FR-009 / AC-009.2 (linux/macos) — mount is exported; calling without FUSE
/// privileges may fail, but the API surface MUST exist (compile-time check).
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn fr009_mount_api_is_exported() {
    // Type-check the public mount symbol; do not require a successful mount.
    let _f: fn(&Path, &Path) -> anyhow::Result<()> = sharecli_fuse::mount;
}
