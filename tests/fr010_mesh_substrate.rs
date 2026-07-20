//! FR-010 — Agent Mesh / Shared Substrate
//! FR: FR-010
//!
//! AC-010.1 default subject prefix
//! AC-010.2 subject_for shape
//! AC-010.3 DeviceRecord round-trip; register without NATS fails loudly

use sharecli_fleet::{DeviceRecord, FleetRegistry, DEFAULT_SUBJECT_PREFIX};

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
