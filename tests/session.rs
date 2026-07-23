use sharecli::session::{GhosttyAdapter, GhosttyCapabilities, ZmxCommand, ZmxSessionAdapter};

#[test]
fn zmx_commands_are_shell_free_and_preserve_arguments() {
    let adapter = ZmxSessionAdapter::new("zmx");
    assert_eq!(adapter.attach("chat one", &["codex", "resume", "id"]),
        ZmxCommand::new("zmx", ["attach", "chat one", "codex", "resume", "id"]));
    assert_eq!(adapter.send("chat one", "hello\n"),
        ZmxCommand::new("zmx", ["send", "chat one", "hello\n"]));
    assert_eq!(adapter.tail("chat one", Some(40)),
        ZmxCommand::new("zmx", ["tail", "--lines", "40", "chat one"]));
    assert_eq!(adapter.history("chat one", true),
        ZmxCommand::new("zmx", ["history", "--vt", "chat one"]));
}

#[test]
fn zmx_capability_probe_is_degraded_when_binary_missing() {
    let adapter = ZmxSessionAdapter::new("definitely-not-installed-zmx");
    assert!(!adapter.capabilities(false).available);
    assert!(!adapter.capabilities(false).durable_pty);
}

#[test]
fn ghostty_capabilities_never_claim_a_private_control_socket() {
    let caps = GhosttyCapabilities::from_probe(true, false, false);
    assert!(caps.apple_events);
    assert!(!caps.app_intents);
    assert!(!caps.control_socket);
    assert_eq!(GhosttyAdapter::degraded_reason(&caps), Some("native RPC unavailable"));
}

