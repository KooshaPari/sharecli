use sharecli::session::{GhosttyAdapter, GhosttyCapabilities, ZmxCommand, ZmxSessionAdapter};

#[test]
fn zmx_commands_are_shell_free() {
    let adapter = ZmxSessionAdapter::new("zmx");
    assert_eq!(
        adapter.attach("chat one", &["codex", "resume", "id"]),
        ZmxCommand::new("zmx", ["attach", "chat one", "codex", "resume", "id"])
    );
    assert_eq!(
        adapter.send("chat one", "hello\n"),
        ZmxCommand::new("zmx", ["send", "chat one", "hello\n"])
    );
    assert_eq!(
        adapter.tail("chat one", Some(40)),
        ZmxCommand::new("zmx", ["tail", "--lines", "40", "chat one"])
    );
    assert_eq!(
        adapter.history("chat one", true),
        ZmxCommand::new("zmx", ["history", "--vt", "chat one"])
    );
}

#[test]
fn missing_zmx_is_degraded() {
    let caps = ZmxSessionAdapter::new("definitely-not-installed-zmx").capabilities(false);
    assert!(!caps.available && !caps.durable_pty);
}

#[test]
fn ghostty_adapter_never_claims_private_rpc() {
    let caps = GhosttyCapabilities::from_probe(true, false, false);
    assert!(caps.apple_events && !caps.app_intents && !caps.control_socket);
    assert_eq!(GhosttyAdapter::degraded_reason(&caps), Some("native RPC unavailable"));
}
