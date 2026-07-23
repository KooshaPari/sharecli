use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn session_list_accepts_explicit_database() {
    let db = NamedTempFile::new().expect("database path");
    let output = Command::new(env!("CARGO_BIN_EXE_sharecli"))
        .args(["session", "list", "--db"])
        .arg(db.path())
        .output()
        .expect("sharecli session list");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "[]");
}
