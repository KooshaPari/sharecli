//! FR-010 — Agent Mesh / Shared Substrate
//! FR: FR-010
//!
//! AC-010.1 default subject prefix
//! AC-010.2 subject_for shape
//! AC-010.3 DeviceRecord round-trip; register without NATS fails loudly
//! AC-010.4 Maildir enqueue → claim → ack lifecycle
//! AC-010.5 Maildir priority ordering (lower first)
//! AC-010.6 Maildir nack returns task to new/

use sharecli_fleet::{DeviceRecord, FleetRegistry, DEFAULT_SUBJECT_PREFIX};
use sharecli_mesh::MaildirQueue;
use serde_json::json;
use tempfile::TempDir;

/// FR-010 / AC-010.1 — disconnected registry uses default mesh prefix.
#[test]
fn fr010_default_subject_prefix() {
    let reg = FleetRegistry::disconnected();
    assert_eq!(DEFAULT_SUBJECT_PREFIX, "sharecli.fleet");
    assert_eq!(
        reg.subject_for("dev-x"),
        format!("{DEFAULT_SUBJECT_PREFIX}.devices.dev-x")
    );
}

/// FR-010 / AC-010.2 — custom prefix still yields devices subject.
#[test]
fn fr010_subject_for_with_custom_prefix() {
    let reg = FleetRegistry::disconnected().with_subject_prefix("mesh.lab");
    assert_eq!(reg.subject_for("agent-1"), "mesh.lab.devices.agent-1");
}

/// FR-010 / AC-010.3 — device record JSON round-trips; disconnected register fails.
#[tokio::test]
async fn fr010_device_record_and_register_requires_nats() {
    let rec = DeviceRecord {
        device_id: "dev-mesh-1".into(),
        hostname: "host-a".into(),
        os: "darwin".into(),
        available_slots: 2,
    };
    let json = serde_json::to_string(&rec).expect("serialize");
    let parsed: DeviceRecord = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed, rec);
    for key in ["device_id", "hostname", "os", "available_slots"] {
        assert!(json.contains(key), "missing {key}");
    }

    let reg = FleetRegistry::disconnected();
    let err = reg
        .register(rec)
        .await
        .expect_err("register without NATS MUST fail loudly");
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("not connected") || msg.contains("nats"),
        "expected loud NATS/connect failure, got {msg}"
    );
}


/// FR-010 / AC-010.4 — Maildir enqueue/claim/ack lifecycle (tmp→new→cur).
#[test]
fn fr010_maildir_enqueue_claim_ack() {
    let dir = TempDir::new().expect("tempdir");
    let q = MaildirQueue::open(dir.path()).expect("open");
    let id = q.enqueue(json!({"op": "mesh-task"}), 5).expect("enqueue");
    assert!(
        dir.path().join("new").join(&id).exists(),
        "AC-010.4: enqueue MUST land in new/"
    );
    let claimed = q.claim(Some("worker-1")).expect("claim").expect("some");
    assert_eq!(claimed.id, id);
    assert_eq!(claimed.attempts, 1);
    assert!(dir.path().join("cur").join(&id).exists());
    q.ack(&id).expect("ack");
    assert!(
        q.list_pending().expect("list").is_empty(),
        "AC-010.4: ack MUST remove from cur/"
    );
}

/// FR-010 / AC-010.5 — lower priority number claimed first.
#[test]
fn fr010_maildir_priority_order() {
    let dir = TempDir::new().expect("tempdir");
    let q = MaildirQueue::open(dir.path()).expect("open");
    q.enqueue(json!("low"), 8).expect("enq low");
    q.enqueue(json!("high"), 1).expect("enq high");
    let first = q.claim(None).expect("claim").expect("some");
    assert_eq!(first.priority, 1, "AC-010.5: priority 1 before 8");
    assert_eq!(first.payload, json!("high"));
}

/// FR-010 / AC-010.6 — nack returns claimed task to new/ for retry.
#[test]
fn fr010_maildir_nack_requeues() {
    let dir = TempDir::new().expect("tempdir");
    let q = MaildirQueue::open(dir.path()).expect("open");
    let id = q.enqueue(json!({}), 4).expect("enq");
    q.claim(None).expect("claim").expect("some");
    q.nack(&id).expect("nack");
    assert!(
        dir.path().join("new").join(&id).exists(),
        "AC-010.6: nack MUST restore to new/"
    );
    assert!(!dir.path().join("cur").join(&id).exists());
}
