use std::process::Command;
use tempfile::TempDir;

fn bin() -> Command { Command::new(env!("CARGO_BIN_EXE_sharecli")) }

#[test]
fn session_list_accepts_explicit_database() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("sessions.sqlite");
    let out = bin()
        .args(["session", "list", "--db", db.to_str().expect("utf8 path")])
        .output()
        .expect("spawn session list");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "[]");
}
