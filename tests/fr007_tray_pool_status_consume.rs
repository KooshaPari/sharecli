//! FR-007 — tray refresh consumes `pool.status` + `status.snapshot` (AC-007.69)
//! FR: FR-007
//!
//! Tray/desktop refresh loops enrich operator panels from supplementary `pool.status` /
//! `status.snapshot` IPC round-trips alongside the primary `monitoring.report` refresh
//! (AC-007.48 / AC-007.51); MUST NOT replace monitoring.report as gate/host_watch source.

use sharecli_tray_linux::ipc::{PoolSnapshot, StatusSnapshot};
use sharecli_tray_linux::operator_display as linux_display;
use sharecli_tray_windows::operator_display as win_display;

fn sample_pool() -> PoolSnapshot {
    PoolSnapshot {
        node_total: 2,
        node_idle: 1,
        bun_total: 1,
        bun_idle: 0,
        max_per_type: 4,
        healthy: true,
        issues: vec![],
        gate: sharecli_tray_linux::ipc::GateStatusSnapshot {
            thermal_pressure: "GREEN".into(),
            detected_agents: 0,
            agent_total_rss_bytes: 0,
            agent_contention: "OK".into(),
            gate_decision: "ADMIT".into(),
        },
        host_watch: sharecli_tray_linux::ipc::HostResourceWatchJson {
            fd_count: 1,
            net_rx_bytes: 2,
            net_tx_bytes: 3,
            mem_rss_bytes: 4,
            load_1m: 0.5,
        },
    }
}

fn sample_status() -> StatusSnapshot {
    StatusSnapshot {
        total_processes: 2,
        agents: vec![sharecli_tray_linux::ipc::AgentProcRow {
            pid: 99,
            family: "claude".into(),
            comm: "claude".into(),
            state: "S".into(),
            mem_rss_bytes: 4096,
            mem_rss: "4.0M".into(),
            fd_count: Some(12),
        }],
        scanned: 50,
        watched: 1,
        gate: sharecli_tray_linux::ipc::GateStatusSnapshot {
            thermal_pressure: "GREEN".into(),
            detected_agents: 1,
            agent_total_rss_bytes: 4096,
            agent_contention: "OK".into(),
            gate_decision: "ADMIT".into(),
        },
        host_watch: sharecli_tray_linux::ipc::HostResourceWatchJson {
            fd_count: 1,
            net_rx_bytes: 2,
            net_tx_bytes: 3,
            mem_rss_bytes: 4,
            load_1m: 0.5,
        },
    }
}

/// FR-007 / AC-007.69 — live IPC dispatch decodes pool/status for tray operator formatters.
#[tokio::test]
async fn fr007_tray_pool_status_consume_live() {
    let handler = sharecli_ipc::handler::Handler::new().await.expect("Handler::new");

    let pool_resp = handler
        .dispatch(r#"{"id":1,"method":"pool.status","params":{}}"#)
        .await;
    assert!(
        pool_resp.error.is_none(),
        "pool.status MUST succeed (AC-007.69); err={:?}",
        pool_resp.error
    );
    let pool_raw = serde_json::to_string(&pool_resp.result).expect("serialize pool.status result");
    let pool: PoolSnapshot =
        serde_json::from_str(&pool_raw).expect("decode PoolSnapshot for tray consume");
    assert!(!pool.gate.gate_decision.is_empty());

    let status_resp = handler
        .dispatch(r#"{"id":2,"method":"status.snapshot","params":{}}"#)
        .await;
    assert!(
        status_resp.error.is_none(),
        "status.snapshot MUST succeed (AC-007.69); err={:?}",
        status_resp.error
    );
    let status_raw =
        serde_json::to_string(&status_resp.result).expect("serialize status.snapshot result");
    let status: StatusSnapshot =
        serde_json::from_str(&status_raw).expect("decode StatusSnapshot for tray consume");

    let lines = linux_display::format_pool_status_operator_lines(&pool, &status);
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("Pool node"));
    assert!(lines[1].starts_with("Proc scan"));
}

/// FR-007 / AC-007.69 — Linux tray format helpers map pool + status snapshot fields.
#[test]
fn fr007_tray_pool_status_consume_linux_format() {
    let pool = sample_pool();
    let status = sample_status();

    let pool_line = linux_display::format_pool_tray_line(&pool);
    assert_eq!(pool_line, "Pool node 2/1 idle · bun 1/0 idle · max 4 · healthy");

    let status_line = linux_display::format_status_snapshot_tray_line(&status);
    assert_eq!(status_line, "Proc scan 50 · watched 1 · 2 managed · 1 agent row(s)");

    let lines = linux_display::format_pool_status_operator_lines(&pool, &status);
    assert_eq!(lines.len(), 2);
}

/// FR-007 / AC-007.69 — Windows tray Rust helpers stay byte-identical to Linux.
#[test]
fn fr007_tray_pool_status_consume_windows_linux_parity() {
    use sharecli_tray_windows::ipc::{
        AgentProcRow as WinAgent, GateStatusSnapshot as WinGate, HostResourceWatchJson as WinHost,
        PoolSnapshot as WinPool, StatusSnapshot as WinStatus,
    };

    let pool = sample_pool();
    let status = sample_status();

    let win_pool = WinPool {
        node_total: pool.node_total,
        node_idle: pool.node_idle,
        bun_total: pool.bun_total,
        bun_idle: pool.bun_idle,
        max_per_type: pool.max_per_type,
        healthy: pool.healthy,
        issues: pool.issues.clone(),
        gate: WinGate {
            thermal_pressure: pool.gate.thermal_pressure.clone(),
            detected_agents: pool.gate.detected_agents,
            agent_total_rss_bytes: pool.gate.agent_total_rss_bytes,
            agent_contention: pool.gate.agent_contention.clone(),
            gate_decision: pool.gate.gate_decision.clone(),
        },
        host_watch: WinHost {
            fd_count: pool.host_watch.fd_count,
            net_rx_bytes: pool.host_watch.net_rx_bytes,
            net_tx_bytes: pool.host_watch.net_tx_bytes,
            mem_rss_bytes: pool.host_watch.mem_rss_bytes,
            load_1m: pool.host_watch.load_1m,
        },
    };
    let win_status = WinStatus {
        total_processes: status.total_processes,
        agents: vec![WinAgent {
            pid: status.agents[0].pid,
            family: status.agents[0].family.clone(),
            comm: status.agents[0].comm.clone(),
            state: status.agents[0].state.clone(),
            mem_rss_bytes: status.agents[0].mem_rss_bytes,
            mem_rss: status.agents[0].mem_rss.clone(),
            fd_count: status.agents[0].fd_count,
        }],
        scanned: status.scanned,
        watched: status.watched,
        gate: WinGate {
            thermal_pressure: status.gate.thermal_pressure.clone(),
            detected_agents: status.gate.detected_agents,
            agent_total_rss_bytes: status.gate.agent_total_rss_bytes,
            agent_contention: status.gate.agent_contention.clone(),
            gate_decision: status.gate.gate_decision.clone(),
        },
        host_watch: WinHost {
            fd_count: status.host_watch.fd_count,
            net_rx_bytes: status.host_watch.net_rx_bytes,
            net_tx_bytes: status.host_watch.net_tx_bytes,
            mem_rss_bytes: status.host_watch.mem_rss_bytes,
            load_1m: status.host_watch.load_1m,
        },
    };

    assert_eq!(
        linux_display::format_pool_tray_line(&pool),
        win_display::format_pool_tray_line(&win_pool),
    );
    assert_eq!(
        linux_display::format_status_snapshot_tray_line(&status),
        win_display::format_status_snapshot_tray_line(&win_status),
    );
}

/// FR-007 / AC-007.69 — Linux tray refresh wires supplementary pool/status IPC after monitoring.report.
#[test]
fn fr007_tray_pool_status_consume_linux_main_wires_refresh() {
    let main_rs = include_str!("../crates/sharecli-tray-linux/src/main.rs");
    assert!(
        main_rs.contains("ipc::pool_status()"),
        "Linux tray refresh MUST call pool.status (AC-007.69)"
    );
    assert!(
        main_rs.contains("ipc::status_snapshot()"),
        "Linux tray refresh MUST call status.snapshot (AC-007.69)"
    );
    assert!(
        main_rs.contains("ipc::monitoring_report()"),
        "Linux tray MUST keep monitoring.report primary refresh (AC-007.69)"
    );
    assert!(
        main_rs.contains("format_pool_status_operator_lines"),
        "Linux tray menu MUST surface pool/status operator lines (AC-007.69)"
    );
}

/// FR-007 / AC-007.69 — Swift AppState + views consume pool/status alongside monitoring.report.
#[test]
fn fr007_tray_pool_status_consume_swift_wires_refresh() {
    let app_state = include_str!("../desktop/ShareCLITray/Sources/ShareCLICore/AppState.swift");
    assert!(
        app_state.contains("poolStatus"),
        "AppState MUST publish poolStatus (AC-007.69)"
    );
    assert!(
        app_state.contains("statusSnapshot"),
        "AppState MUST publish statusSnapshot (AC-007.69)"
    );
    assert!(
        app_state.contains("client.poolStatus()"),
        "AppState refresh MUST call pool.status (AC-007.69)"
    );
    assert!(
        app_state.contains("client.statusSnapshot()"),
        "AppState refresh MUST call status.snapshot (AC-007.69)"
    );
    assert!(
        app_state.contains("monitoringReport()"),
        "AppState MUST keep monitoring.report primary refresh (AC-007.69)"
    );

    let op = include_str!("../desktop/ShareCLITray/Sources/ShareCLICore/OperatorDisplay.swift");
    assert!(
        op.contains("formatPoolTrayLine"),
        "Swift OperatorDisplay MUST expose pool tray formatter (AC-007.69)"
    );
    assert!(
        op.contains("formatStatusSnapshotTrayLine"),
        "Swift OperatorDisplay MUST expose status snapshot formatter (AC-007.69)"
    );

    let popover = include_str!("../desktop/ShareCLITray/Sources/ShareCLITray/TrayPopoverView.swift");
    assert!(
        popover.contains("poolStatusSection"),
        "Tray popover MUST surface pool/status section (AC-007.69)"
    );

    let dashboard = include_str!("../desktop/ShareCLITray/Sources/ShareCLITray/DashboardView.swift");
    assert!(
        dashboard.contains("formatPoolStatusOperatorLines"),
        "Dashboard Health view MUST surface pool/status lines (AC-007.69)"
    );
}

/// FR-007 / AC-007.69 — WinUI tray surfaces pool/status operator rows after monitoring.report.
#[test]
fn fr007_tray_pool_status_consume_windows_wires_refresh() {
    let op = include_str!("../windows/ShareCLITray/OperatorDisplay.cs");
    assert!(
        op.contains("FormatPoolTrayLine"),
        "C# OperatorDisplay MUST expose pool tray formatter (AC-007.69)"
    );
    assert!(
        op.contains("FormatStatusSnapshotTrayLine"),
        "C# OperatorDisplay MUST expose status snapshot formatter (AC-007.69)"
    );

    let xaml = include_str!("../windows/ShareCLITray/TrayWindow.xaml");
    assert!(
        xaml.contains("PoolStatusText"),
        "TrayWindow XAML MUST include pool status row (AC-007.69)"
    );
    assert!(
        xaml.contains("StatusSnapshotText"),
        "TrayWindow XAML MUST include status snapshot row (AC-007.69)"
    );

    let code = include_str!("../windows/ShareCLITray/TrayWindow.xaml.cs");
    assert!(
        code.contains("pool.status"),
        "TrayWindow MUST call pool.status IPC (AC-007.69)"
    );
    assert!(
        code.contains("status.snapshot"),
        "TrayWindow MUST call status.snapshot IPC (AC-007.69)"
    );
    assert!(
        code.contains("monitoring.report"),
        "TrayWindow MUST keep monitoring.report primary refresh (AC-007.69)"
    );
    assert!(
        code.contains("FormatPoolTrayLine"),
        "TrayWindow MUST bind pool operator strings (AC-007.69)"
    );
}
