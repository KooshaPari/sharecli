//! FR-007 — tray thermal gate visual indicator parity (AC-007.57)
//! FR: FR-007
//!
//! Linux, Swift, and Windows tray icon/badge/color MUST derive from
//! `gate.thermal_pressure` + `gate_decision` using shared severity→visual helpers
//! (parity with dashboard thermal/gate styling).

use sharecli_tray_linux::ipc::GateStatusSnapshot;
use sharecli_tray_linux::operator_display as linux_display;
use sharecli_tray_linux::operator_display::TrayGateSeverity;
use sharecli_tray_windows::operator_display as win_display;

/// FR-007 / AC-007.57 — golden severity matrix for ADMIT/GREEN (normal).
#[test]
fn fr007_tray_thermal_visual_admit_green_normal() {
    let v = linux_display::resolve_tray_gate_visual("GREEN", "ADMIT", true);
    assert_eq!(v.severity, TrayGateSeverity::Normal);
    assert_eq!(v.decision_class, "gate-admit");
    assert_eq!(v.thermal_class, "");
    assert_eq!(v.color_hex, "#3fb950");
    assert_eq!(v.badge_label, "Normal");
    assert_eq!(v.linux_icon_name, "utilities-system-monitor");
    assert_eq!(v.swift_symbol_name, "cpu");
}

/// FR-007 / AC-007.57 — YELLOW thermal elevates to warning even when ADMIT.
#[test]
fn fr007_tray_thermal_visual_yellow_warning() {
    let v = linux_display::resolve_tray_gate_visual("YELLOW", "ADMIT", true);
    assert_eq!(v.severity, TrayGateSeverity::Warning);
    assert_eq!(v.decision_class, "gate-admit");
    assert_eq!(v.thermal_class, "warning");
    assert_eq!(v.badge_label, "Warning");
    assert_eq!(v.linux_icon_name, "dialog-warning");
}

/// FR-007 / AC-007.57 — DENY + RED maps to critical (dashboard gate-deny + critical thermal).
#[test]
fn fr007_tray_thermal_visual_deny_red_critical() {
    let v = linux_display::resolve_tray_gate_visual("RED", "DENY", true);
    assert_eq!(v.severity, TrayGateSeverity::Critical);
    assert_eq!(v.decision_class, "gate-deny");
    assert_eq!(v.thermal_class, "critical");
    assert_eq!(v.color_hex, "#f85149");
    assert_eq!(v.badge_label, "Critical");
    assert_eq!(v.linux_icon_name, "dialog-error");
    assert_eq!(v.swift_symbol_name, "flame.fill");
}

/// FR-007 / AC-007.57 — THROTTLE decision maps to gate-unavailable styling.
#[test]
fn fr007_tray_thermal_visual_throttle_unavailable_class() {
    let v = linux_display::resolve_tray_gate_visual("GREEN", "THROTTLE", true);
    assert_eq!(v.severity, TrayGateSeverity::Warning);
    assert_eq!(v.decision_class, "gate-unavailable");
    assert_eq!(v.badge_label, "Warning");
}

/// FR-007 / AC-007.57 — disconnected IPC → offline visual tokens.
#[test]
fn fr007_tray_thermal_visual_offline_when_disconnected() {
    let v = linux_display::resolve_tray_gate_visual("GREEN", "ADMIT", false);
    assert_eq!(v.severity, TrayGateSeverity::Offline);
    assert_eq!(v.badge_label, "Offline");
    assert_eq!(v.linux_icon_name, "network-offline");
}

/// FR-007 / AC-007.57 — Windows Rust helpers stay token-identical to Linux.
#[test]
fn fr007_tray_thermal_visual_windows_linux_parity() {
    let cases = [
        ("GREEN", "ADMIT", true),
        ("YELLOW", "ADMIT", true),
        ("RED", "DENY", true),
        ("GREEN", "THROTTLE", true),
        ("UNAVAILABLE", "UNAVAILABLE", false),
    ];
    for (thermal, decision, connected) in cases {
        let lv = linux_display::resolve_tray_gate_visual(thermal, decision, connected);
        let wv = win_display::resolve_tray_gate_visual(thermal, decision, connected);
        assert_eq!(format!("{:?}", lv.severity), format!("{:?}", wv.severity), "{thermal}/{decision}");
        assert_eq!(lv.decision_class, wv.decision_class);
        assert_eq!(lv.thermal_class, wv.thermal_class);
        assert_eq!(lv.color_hex, wv.color_hex);
        assert_eq!(lv.badge_label, wv.badge_label);
        assert_eq!(lv.linux_icon_name, wv.linux_icon_name);
        assert_eq!(lv.swift_symbol_name, wv.swift_symbol_name);
    }
}

/// FR-007 / AC-007.57 — gate snapshot helper used by tray refresh paths.
#[test]
fn fr007_tray_thermal_visual_from_gate_snapshot() {
    let gate = GateStatusSnapshot {
        thermal_pressure: "YELLOW".into(),
        detected_agents: 1,
        agent_total_rss_bytes: 512,
        agent_contention: "WARN".into(),
        gate_decision: "THROTTLE".into(),
    };
    let v = linux_display::resolve_tray_gate_visual_from_gate(&gate, true);
    assert_eq!(v.severity, TrayGateSeverity::Warning);
    assert_eq!(v.decision_class, "gate-unavailable");
    assert_eq!(v.thermal_class, "warning");
}

/// FR-007 / AC-007.57 — Linux tray wires icon + NeedsAttention from gate visual.
#[test]
fn fr007_tray_thermal_visual_linux_main_wires_icon() {
    let main_rs = include_str!("../crates/sharecli-tray-linux/src/main.rs");
    assert!(
        main_rs.contains("gate_visual.linux_icon_name"),
        "Linux tray icon MUST follow resolve_tray_gate_visual (AC-007.57)"
    );
    assert!(
        main_rs.contains("fn status(&self) -> ksni::IconStatus"),
        "Linux tray MUST expose SNI attention for warn/critical (AC-007.57)"
    );
    assert!(
        main_rs.contains("resolve_tray_gate_visual_from_gate"),
        "Linux refresh MUST derive gate visual from monitoring.report gate (AC-007.57)"
    );
}

/// FR-007 / AC-007.57 — Swift OperatorDisplay + menu bar icon wiring.
#[test]
fn fr007_tray_thermal_visual_swift_wires_operator_display() {
    let op = include_str!("../desktop/ShareCLITray/Sources/ShareCLICore/OperatorDisplay.swift");
    assert!(
        op.contains("resolveTrayGateVisual"),
        "Swift OperatorDisplay MUST expose visual resolver (AC-007.57)"
    );
    assert!(
        op.contains("gate-admit"),
        "Swift visual resolver MUST map gate decision CSS classes (AC-007.57)"
    );

    let app = include_str!("../desktop/ShareCLITray/Sources/ShareCLITray/AppEntry.swift");
    assert!(
        app.contains("visual.swiftSymbolName"),
        "macOS status item MUST update icon from gate visual (AC-007.57)"
    );

    let popover = include_str!("../desktop/ShareCLITray/Sources/ShareCLITray/TrayPopoverView.swift");
    assert!(
        popover.contains("thermalBadge"),
        "Tray popover MUST show thermal badge from gate visual (AC-007.57)"
    );

    let state = include_str!("../desktop/ShareCLITray/Sources/ShareCLICore/AppState.swift");
    assert!(
        state.contains("sharecliHealthChanged"),
        "AppState refresh MUST notify menu bar for visual updates (AC-007.57)"
    );
}

/// FR-007 / AC-007.57 — Windows WinUI thermal badge + colored gate row.
#[test]
fn fr007_tray_thermal_visual_windows_wires_operator_display() {
    let op = include_str!("../windows/ShareCLITray/OperatorDisplay.cs");
    assert!(
        op.contains("ResolveTrayGateVisual"),
        "C# OperatorDisplay MUST expose visual resolver (AC-007.57)"
    );
    assert!(
        op.contains("SeverityBrush"),
        "C# OperatorDisplay MUST map severity → brush (AC-007.57)"
    );

    let xaml = include_str!("../windows/ShareCLITray/TrayWindow.xaml");
    assert!(
        xaml.contains("ThermalBadgeText"),
        "TrayWindow XAML MUST include thermal badge row (AC-007.57)"
    );

    let code = include_str!("../windows/ShareCLITray/TrayWindow.xaml.cs");
    assert!(
        code.contains("ResolveTrayGateVisual"),
        "TrayWindow MUST bind thermal badge from gate visual (AC-007.57)"
    );
    assert!(
        code.contains("SeverityBrush"),
        "TrayWindow MUST color gate row by severity (AC-007.57)"
    );
}

/// FR-007 / AC-007.58 — Swift HealthView metric cards + thermal gate detail use gate visual.
#[test]
fn fr007_tray_thermal_visual_swift_health_view_wires_gate_visual() {
    let health = include_str!("../desktop/ShareCLITray/Sources/ShareCLITray/DashboardView.swift");
    assert!(
        health.contains("resolveTrayGateVisual"),
        "HealthView MUST resolve tray gate visual (AC-007.58)"
    );
    assert!(
        health.contains("gateVisual"),
        "HealthView MUST expose gateVisual computed property (AC-007.58)"
    );
    assert!(
        health.contains("thermalGateBadge"),
        "HealthView MUST show thermal gate badge chip (AC-007.58)"
    );
    assert!(
        health.contains("gateVisual.swiftColor"),
        "HealthView metric cards MUST use severity colors (AC-007.58)"
    );
    assert!(
        health.contains("gateVisual.badgeLabel"),
        "HealthView Thermal Gate card MUST show badge label (AC-007.58)"
    );
    assert!(
        !health.contains("h.healthy ? \"Healthy\""),
        "HealthView MUST NOT use generic healthy/warning Status card (AC-007.58)"
    );
}

/// FR-007 / AC-007.59 — Swift TrayPopoverView stats row Status cell uses gate visual.
#[test]
fn fr007_tray_thermal_visual_swift_popover_stats_row_wires_gate_visual() {
    let popover = include_str!("../desktop/ShareCLITray/Sources/ShareCLITray/TrayPopoverView.swift");
    assert!(
        popover.contains("gateVisual.swiftSymbolName"),
        "Tray popover stats row MUST use gate visual icon (AC-007.59)"
    );
    assert!(
        popover.contains("gateVisual.badgeLabel"),
        "Tray popover stats row MUST use gate visual badge label (AC-007.59)"
    );
    assert!(
        popover.contains("gateVisual.swiftColor"),
        "Tray popover stats row MUST use gate visual severity color (AC-007.59)"
    );
    assert!(
        !popover.contains("state.health?.healthy == true ? \"Healthy\""),
        "Tray popover stats row MUST NOT use generic healthy/warning Status cell (AC-007.59)"
    );
}

/// FR-007 / AC-007.60 — Windows HealthStatusText uses gate visual tokens.
#[test]
fn fr007_tray_thermal_visual_windows_health_status_wires_gate_visual() {
    let op = include_str!("../windows/ShareCLITray/OperatorDisplay.cs");
    assert!(
        op.contains("FormatHealthStatusLine"),
        "C# OperatorDisplay MUST expose health summary formatter (AC-007.60)"
    );
    assert!(
        op.contains("FormatHealthStatusOfflineLine"),
        "C# OperatorDisplay MUST expose offline health summary formatter (AC-007.60)"
    );

    let code = include_str!("../windows/ShareCLITray/TrayWindow.xaml.cs");
    assert!(
        code.contains("FormatHealthStatusLine"),
        "TrayWindow MUST bind HealthStatusText from gate visual (AC-007.60)"
    );
    assert!(
        code.contains("HealthStatusText.Foreground"),
        "TrayWindow MUST color HealthStatusText by severity (AC-007.60)"
    );
    assert!(
        !code.contains("health.Healthy ? \"✓ OK\""),
        "TrayWindow MUST NOT use generic OK/Unhealthy health summary (AC-007.60)"
    );
}
