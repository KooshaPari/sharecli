//! FR-009 — FUSE IO Intercept
//! FR: FR-009
//!
//! AC-009.1 InterceptFs::new over backing path (no privileged mount)
//! AC-009.2 mount API fails loudly on unsupported platforms
//! AC-009.3 inode map / path resolution (pure)
//! AC-009.4 in-process read coalesce cache hit/miss meters
//! AC-009.5 write-serialize per-path lock + CoW stub API

use std::path::{Path, PathBuf};
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

/// FR-009 / AC-009.3 — inode map resolves nested parents without a mount.
#[test]
fn fr009_inode_map_path_resolution() {
    use std::ffi::OsStr;
    use sharecli_fuse::{InodeMap, ROOT_INO};

    let mut map = InodeMap::new();
    assert_eq!(map.resolve(ROOT_INO), Some(Path::new("")));
    let (dir_ino, _) = map
        .lookup_or_alloc(ROOT_INO, OsStr::new("src"))
        .expect("dir");
    let (file_ino, rel) = map
        .lookup_or_alloc(dir_ino, OsStr::new("main.rs"))
        .expect("file");
    assert_eq!(rel, PathBuf::from("src/main.rs"));
    assert_eq!(map.resolve(file_ino), Some(Path::new("src/main.rs")));
}

/// FR-009 / AC-009.4 — read coalesce hit/miss without privileged mount.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn fr009_read_coalesce_hit_miss_meters() {
    use std::fs;
    use std::io::Write;

    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("hot.txt");
    {
        let mut f = fs::File::create(&file).expect("create");
        f.write_all(b"coalesce-me").expect("write");
    }

    let fs = sharecli_fuse::InterceptFs::new(dir.path());
    let a = fs
        .read_coalesced_rel(Path::new("hot.txt"))
        .expect("first read");
    assert_eq!(a, b"coalesce-me");
    let m1 = fs.cache_meters();
    assert_eq!(m1.misses, 1);
    assert_eq!(m1.hits, 0);

    let b = fs
        .read_coalesced_rel(Path::new("hot.txt"))
        .expect("second read");
    assert_eq!(b, a);
    let m2 = fs.cache_meters();
    assert_eq!(m2.misses, 1);
    assert_eq!(m2.hits, 1);
}

/// FR-009 / AC-009.4 — ReadContentCache unit surface (all platforms).
#[test]
fn fr009_read_content_cache_direct() {
    use std::io::Write;
    use sharecli_fuse::ReadContentCache;
    use tempfile::NamedTempFile;

    let mut tmp = NamedTempFile::new().expect("tmp");
    write!(tmp, "x").expect("w");
    tmp.flush().expect("flush");
    let mut cache = ReadContentCache::new();
    let _ = cache.read_coalesced(tmp.path()).expect("miss");
    let _ = cache.read_coalesced(tmp.path()).expect("hit");
    let m = cache.meters();
    assert_eq!((m.hits, m.misses), (1, 1));
}

/// FR-009 / AC-009.5 — write serialize stubs + passthrough write invalidates cache.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn fr009_write_passthrough_invalidates_cache() {
    use std::fs;
    use std::io::Write;
    use sharecli_fuse::{WriteSerialize, WriteSerializeError};

    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("rw.txt");
    {
        let mut f = fs::File::create(&file).expect("create");
        f.write_all(b"aaaa").expect("write");
    }

    let fs = sharecli_fuse::InterceptFs::new(dir.path());
    let _ = fs.read_coalesced_rel(Path::new("rw.txt")).expect("warm");
    assert_eq!(fs.cache_meters().misses, 1);

    fs.write_rel(Path::new("rw.txt"), 0, b"bbbb")
        .expect("passthrough write must not ENOSYS");
    let after = fs.read_coalesced_rel(Path::new("rw.txt")).expect("reload");
    assert_eq!(after, b"bbbb");
    // Invalidate + reload => second miss.
    assert!(fs.cache_meters().misses >= 2);

    let ws = WriteSerialize::new();
    assert!(matches!(
        ws.commit_pending(Path::new("rw.txt")),
        Err(WriteSerializeError::CommitTodo(_))
    ));
}
