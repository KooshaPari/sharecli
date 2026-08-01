use sharecli_session::{
    ProcessEvidence, SessionStateProvider, SidecarStateProvider, SurfaceRecord,
};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn surface(id: &str, pid: Option<u32>) -> SurfaceRecord {
    SurfaceRecord {
        id: id.into(),
        terminal: "ghostty".into(),
        title: None,
        cwd: PathBuf::from("/tmp/project"),
        process: Some(ProcessEvidence {
            pid,
            tty: None,
            cwd: PathBuf::from("/tmp/project"),
            argv: vec!["codex".into()],
            started_at: None,
        }),
    }
}

fn temp_sidecar() -> PathBuf {
    let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("sharecli-sidecar-{suffix}.jsonl"))
}

#[test]
fn sidecar_returns_latest_exact_surface_and_pid_match() {
    let path = temp_sidecar();
    fs::write(
        &path,
        "{\"surface_id\":\"pane-1\",\"harness\":\"codex\",\"session_id\":\"old\",\"pid\":41}\n{\"surface_id\":\"pane-1\",\"harness\":\"codex\",\"session_id\":\"new\",\"pid\":42}\n",
    )
    .unwrap();
    let provider = SidecarStateProvider::new(&path);
    assert_eq!(
        provider.session_id(&surface("pane-1", Some(42)), "codex").unwrap(),
        Some("new".into())
    );
    let _ = fs::remove_file(path);
}

#[test]
fn sidecar_refuses_stale_pid_or_harness_matches() {
    let path = temp_sidecar();
    fs::write(
        &path,
        "{\"surface_id\":\"pane-1\",\"harness\":\"codex\",\"session_id\":\"id\",\"pid\":42}\n",
    )
    .unwrap();
    let provider = SidecarStateProvider::new(&path);
    assert_eq!(provider.session_id(&surface("pane-1", Some(7)), "codex").unwrap(), None);
    assert_eq!(provider.session_id(&surface("pane-1", Some(42)), "forge").unwrap(), None);
    let _ = fs::remove_file(path);
}

#[test]
fn newer_pid_mismatch_supersedes_an_older_matching_record() {
    let path = temp_sidecar();
    fs::write(
        &path,
        "{\"surface_id\":\"pane-1\",\"harness\":\"codex\",\"session_id\":\"old\",\"pid\":42}\n{\"surface_id\":\"pane-1\",\"harness\":\"codex\",\"session_id\":\"recycled\",\"pid\":99}\n",
    )
    .unwrap();
    let provider = SidecarStateProvider::new(&path);
    assert_eq!(provider.session_id(&surface("pane-1", Some(42)), "codex").unwrap(), None);
    let _ = fs::remove_file(path);
}

#[test]
fn malformed_sidecar_fails_closed() {
    let path = temp_sidecar();
    fs::write(&path, "not-json\n").unwrap();
    let error = SidecarStateProvider::new(&path)
        .session_id(&surface("pane-1", Some(42)), "codex")
        .unwrap_err();
    assert!(error.to_string().contains("parse sidecar"));
    let _ = fs::remove_file(path);
}
