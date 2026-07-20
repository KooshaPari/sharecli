//! C01 — broad-workspace coverage lift toward 85% (FR-003).
//!
//! FR: FR-003

use std::process::Command;

use sharecli::config::{
    CastConfig, Config, DefaultHarnessConfig, MonitoringConfig, PoolConfig, ServeConfig,
    SpawnPolicyConfig,
};
use sharecli::error::{ErrorCode, SharecliError, EXIT_IO, EXIT_SPAWN, EXIT_SERVE};

/// FR-003 / C01 — default config embeds registered projects and harness presets.
#[test]
fn fr003_config_default_projects_and_harness_presets() {
    let cfg = Config::default();
    assert!(cfg.projects.contains_key("helios-cli"));
    assert!(cfg.projects.contains_key("portage"));
    assert!(cfg.defaults.contains_key("claude"));
    assert!(cfg.defaults.contains_key("node"));
    assert_eq!(cfg.defaults["claude"].max_instances, 11);
    assert_eq!(cfg.defaults["node"].memory_limit_mb, 256);
}

/// FR-003 / C01 — nested sub-config defaults round-trip through TOML.
#[test]
fn fr003_config_subconfig_defaults_roundtrip() {
    let cfg = Config {
        cast: CastConfig {
            default_transport: "ghostty".into(),
            pane_map_path: Some("/tmp/panes.toml".into()),
            handshake_timeout_ms: 500,
            max_retry_attempts: 5,
            retry_backoff_ms: 400,
        },
        pool: PoolConfig {
            enabled: false,
            max_per_type: 2,
            idle_timeout_secs: 120,
            max_age_secs: 900,
            spawn_delay_ms: 50,
        },
        monitoring: MonitoringConfig {
            health_check_interval_secs: 15,
            idle_threshold_secs: 60,
            high_memory_threshold_mb: 8192,
            idle_process_threshold: 3,
            per_process_warn_memory_bytes: 512 * 1024 * 1024,
        },
        serve: ServeConfig {
            bearer_token: Some("test-token".into()),
            auth_mode: Some("bearer".into()),
            rate_limit_max: Some(100),
            rate_limit_window_secs: Some(30),
            ..ServeConfig::default()
        },
        spawn_policy: SpawnPolicyConfig {
            nice_level: 5,
            max_concurrent_builds: 4,
            use_sccache: true,
        },
        ..Config::default()
    };

    let toml_text = toml::to_string_pretty(&cfg).expect("serialize config");
    let loaded: Config = toml::from_str(&toml_text).expect("deserialize config");
    assert_eq!(loaded.cast.default_transport, "ghostty");
    assert_eq!(loaded.pool.max_per_type, 2);
    assert_eq!(loaded.monitoring.health_check_interval_secs, 15);
    assert_eq!(loaded.serve.bearer_token.as_deref(), Some("test-token"));
    assert!(loaded.spawn_policy.use_sccache);
}

/// FR-003 / C01 — `cast list` prints empty-state guidance when no panes exist.
#[test]
fn fr003_cast_list_empty_state() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config_home = tmp.path().join("config");
    std::fs::create_dir_all(&config_home).expect("config home");
    let bin = env!("CARGO_BIN_EXE_sharecli");

    let output = Command::new(bin)
        .env("XDG_CONFIG_HOME", &config_home)
        .env("APPDATA", &config_home)
        .args(["cast", "list"])
        .output()
        .expect("run cast list");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No panes registered"));
}

/// FR-003 / C01 — `cast send` rejects empty stdin payload.
#[test]
fn fr003_cast_send_rejects_empty_text() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config_home = tmp.path().join("config");
    std::fs::create_dir_all(&config_home).expect("config home");
    let bin = env!("CARGO_BIN_EXE_sharecli");

    Command::new(bin)
        .env("XDG_CONFIG_HOME", &config_home)
        .env("APPDATA", &config_home)
        .args(["cast", "register", "demo", "mbp:local:0:0"])
        .status()
        .expect("register pane");

    let output = Command::new(bin)
        .env("XDG_CONFIG_HOME", &config_home)
        .env("APPDATA", &config_home)
        .args(["cast", "send", "demo"])
        .stdin(std::process::Stdio::piped())
        .output()
        .expect("run cast send empty");
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("empty text") || combined.contains("refusing"),
        "expected empty-text refusal, got: {combined}"
    );
}

/// FR-003 / C01 — `cast where` resolves pane-map path for hermetic config home.
#[test]
fn fr003_cast_where_prints_pane_map_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config_home = tmp.path().join("config");
    std::fs::create_dir_all(&config_home).expect("config home");
    let bin = env!("CARGO_BIN_EXE_sharecli");

    let output = Command::new(bin)
        .env("XDG_CONFIG_HOME", &config_home)
        .env("APPDATA", &config_home)
        .args(["cast", "where"])
        .output()
        .expect("run cast where");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pane-map"));
}

/// FR-003 / C01 — default harness config serde preserves numeric caps.
#[test]
fn fr003_default_harness_config_roundtrip() {
    let preset = DefaultHarnessConfig {
        enabled: false,
        max_instances: 7,
        memory_limit_mb: 128,
    };
    let json = serde_json::to_string(&preset).expect("serialize");
    let parsed: DefaultHarnessConfig = serde_json::from_str(&json).expect("deserialize");
    assert!(!parsed.enabled);
    assert_eq!(parsed.max_instances, 7);
    assert_eq!(parsed.memory_limit_mb, 128);
}

/// FR-003 / C01 — domain error helpers map to stable exit codes.
#[test]
fn fr003_error_constructors_and_io_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing file");
    let err = SharecliError::io("read failed", io_err);
    assert_eq!(err.code(), ErrorCode::Io);
    assert_eq!(err.exit_code(), EXIT_IO);

    assert_eq!(SharecliError::spawn("bad spawn").code(), ErrorCode::Spawn);
    assert_eq!(SharecliError::spawn("bad spawn").exit_code(), EXIT_SPAWN);
    assert_eq!(SharecliError::serve("down").code(), ErrorCode::Serve);
    assert_eq!(SharecliError::serve("down").exit_code(), EXIT_SERVE);

    let from_io: SharecliError = std::io::Error::other("disk").into();
    assert_eq!(from_io.code(), ErrorCode::Io);
}
