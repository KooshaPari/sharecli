//! FR-009 — Hypervisor FUSE write-provenance session wiring
//! FR: FR-009
//!
//! AC-009.12 Hypervisor cache-miss FUSE mounts derive session id from coalesce CommandKey

use sharecli_core::fuse_session_id_for_command_key;
use sharecli_fuse::{read_provenance, InterceptFs};
use sharecli_ipc::command_key;
use std::path::Path;
use tempfile::TempDir;

/// FR-009 / AC-009.12 — coalesce key maps to FUSE session id stamped on writes.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn fr009_hypervisor_fuse_session_stamps_provenance() {
    use std::fs;
    use std::io::Write;

    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("artifact.txt");
    {
        let mut f = fs::File::create(&file).expect("create");
        f.write_all(b"seed").expect("write");
    }

    let key = command_key(&["cargo".into(), "build".into()], dir.path(), &[]);
    let session = fuse_session_id_for_command_key(&key);
    assert!(session.starts_with("hv-"));
    assert_eq!(session.len(), 19, "hv- + 16 hex chars from CommandKey");

    let fs = InterceptFs::with_session(dir.path(), &session);
    assert_eq!(fs.session_id(), session);

    fs.write_rel(Path::new("artifact.txt"), 0, b"built")
        .expect("write_rel");
    let prov = read_provenance(&file)
        .expect("read_provenance")
        .expect("provenance xattrs");
    assert_eq!(prov.session_id, session);
}

/// FR-009 / AC-009.12 — different argv → different FUSE session ids.
#[test]
fn fr009_hypervisor_fuse_session_differs_by_command_key() {
    let cwd = Path::new("/workspace");
    let k1 = command_key(&["a".into()], cwd, &[]);
    let k2 = command_key(&["b".into()], cwd, &[]);
    assert_ne!(
        fuse_session_id_for_command_key(&k1),
        fuse_session_id_for_command_key(&k2)
    );
}
