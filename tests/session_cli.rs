use std::process::Command;
use tempfile::TempDir;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sharecli"))
}

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

#[test]
fn session_layout_save_and_list_accept_explicit_database() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("sessions.sqlite");
    let snapshot = dir.path().join("layout.json");
    std::fs::write(
        &snapshot,
        r#"{"id":"daily","terminal":"ghostty","captured_at":"2026-08-01T00:00:00Z","root":{"Pane":{"surface_id":"ghostty:1"}}}"#,
    )
    .expect("write layout");

    let save = bin()
        .args([
            "session",
            "layout-save",
            snapshot.to_str().expect("utf8 path"),
            "--db",
            db.to_str().expect("utf8 path"),
        ])
        .output()
        .expect("spawn layout save");
    assert!(save.status.success(), "layout save stderr: {}", String::from_utf8_lossy(&save.stderr));

    let list = bin()
        .args(["session", "layout-list", "--db", db.to_str().expect("utf8 path")])
        .output()
        .expect("spawn layout list");
    assert!(list.status.success(), "layout list stderr: {}", String::from_utf8_lossy(&list.stderr));
    assert!(String::from_utf8_lossy(&list.stdout).contains("daily"));
}
