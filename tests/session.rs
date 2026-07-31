use sharecli::session::{
    GhosttyAdapter, GhosttyCapabilities, GhosttyControlClient, ZmxCommand, ZmxSessionAdapter,
};

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

#[test]
fn ghostty_control_client_requires_configured_socket() {
    let client = GhosttyControlClient::new("/tmp/sharecli-no-ghostty.sock", Some("token".into()));
    assert!(client.request("surface.list", serde_json::json!({})).is_err());
    let caps = GhosttyCapabilities::from_probe(false, false, false).with_control_socket(true);
    assert!(GhosttyAdapter::degraded_reason(&caps).is_none());
}

#[cfg(unix)]
#[test]
fn ghostty_control_client_round_trips_authenticated_io_request() {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;
    use std::time::{SystemTime, UNIX_EPOCH};

    let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let socket = std::env::temp_dir().join(format!("sharecli-ghostty-{suffix}.sock"));
    let listener = UnixListener::bind(&socket).unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut line = String::new();
        BufReader::new(stream.try_clone().unwrap()).read_line(&mut line).unwrap();
        let request: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(request["method"], "surface.io.send");
        assert_eq!(request["token"], "test-token");
        stream.write_all(b"{\"id\":1,\"result\":{\"accepted\":true}}\n").unwrap();
    });

    let client = GhosttyControlClient::new(&socket, Some("test-token".into()));
    client.send_text("ghostty:1", "hello\n").unwrap();
    server.join().unwrap();
    let _ = std::fs::remove_file(socket);
}
