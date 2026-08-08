use sharecli_session::{ResolutionConfidence, SessionObservation, SessionStore};

fn observation(id: &str, session_id: &str, confidence: ResolutionConfidence) -> SessionObservation {
    SessionObservation::new(
        id,
        session_id,
        "surface-1",
        "2026-08-08T00:00:00Z",
        confidence,
        "terminal process and harness metadata",
    )
}

#[test]
fn observations_survive_store_reopen() {
    let path = std::env::temp_dir().join(format!(
        "sharecli-session-ledger-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));

    {
        let store = SessionStore::open(&path).unwrap();
        store
            .append_observation(&observation("obs-1", "codex:abc", ResolutionConfidence::Exact))
            .unwrap();
    }

    let reopened = SessionStore::open(&path).unwrap();
    let rows = reopened.observations("codex:abc").unwrap();
    assert_eq!(rows.len(), 1);
    assert!(rows[0].resumable);
    assert_eq!(rows[0].confidence, ResolutionConfidence::Exact);

    std::fs::remove_file(&path).unwrap();
}

#[test]
fn heuristic_observations_are_persisted_but_not_resumable() {
    let store = SessionStore::open_memory().unwrap();
    store
        .append_observation(&observation(
            "obs-heuristic",
            "codex:ambiguous",
            ResolutionConfidence::Heuristic,
        ))
        .unwrap();

    let rows = store.observations("codex:ambiguous").unwrap();
    assert_eq!(rows.len(), 1);
    assert!(!rows[0].resumable);
    assert_eq!(rows[0].confidence, ResolutionConfidence::Heuristic);
}
