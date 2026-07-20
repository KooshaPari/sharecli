//! FR-006 — thermal TUI DetectedAgent panel (proc scan inventory)
//! FR: FR-006
//!
//! AC-006.9 `sharecli thermal` polls `scan_host_agents` and renders a
//! DetectedAgent inventory panel each redraw.

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use sharecli_fleet::proc_scan::DetectedAgent;
use sharecli_fleet::thermal::ThermalLevel;
use sharecli_fleet::ResourceWatchSample;
use sharecli_fuse::{NegDentryMeters, ReadCacheMeters};
use sharecli_thermal_tui::{agent_lines, render, App};

const SAMPLE: ResourceWatchSample = ResourceWatchSample {
    fd_count: 12,
    net_rx_bytes: 0,
    net_tx_bytes: 0,
    mem_rss_bytes: 4096,
    load_1m: 0.5,
};

fn fixture_agents() -> [DetectedAgent; 2] {
    [
        DetectedAgent {
            pid: 100,
            family: "claude",
            comm: "claude".into(),
        },
        DetectedAgent {
            pid: 250,
            family: "cursor",
            comm: "cursor-agent".into(),
        },
    ]
}

/// FR-006 / AC-006.9 — agent_lines renders inventory rows for thermal TUI.
#[test]
fn fr006_thermal_tui_agent_lines() {
    let lines = agent_lines(&fixture_agents(), false);
    let text: String = lines.iter().map(|l| l.to_string()).collect();
    assert!(text.contains("Agents: 2"));
    assert!(text.contains("PID 100") && text.contains("claude"));
    assert!(text.contains("PID 250") && text.contains("cursor"));
}

/// FR-006 / AC-006.9 — headless thermal render includes DetectedAgent panel.
#[test]
fn fr006_thermal_tui_render_includes_agent_panel() {
    let backend = TestBackend::new(120, 34);
    let mut terminal = Terminal::new(backend).expect("terminal");
    let mut app = App::new(4)
        .with_operator_meters(
            Some(SAMPLE),
            ReadCacheMeters { hits: 1, misses: 0 },
            NegDentryMeters { hits: 0, misses: 0 },
        )
        .with_detected_agents(fixture_agents().to_vec());
    app.update(ThermalLevel::Green, 0);

    terminal.draw(|f| render(f, &app)).expect("draw");
    let rendered: String = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|c| c.symbol().to_string())
        .collect();

    assert!(
        rendered.contains("Detected Agents") && rendered.contains("PID 100"),
        "thermal TUI MUST surface proc-scan agent inventory"
    );
}
