//! FR-006 — thermal TUI DetectedAgent panel (proc scan + per-PID watch)
//! FR: FR-006, FR-007
//!
//! AC-006.9 `sharecli thermal` polls agent inventory each redraw.
//! AC-006.10 rows include per-PID RSS (and FD on Linux).

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use sharecli_fleet::proc_scan::DetectedAgent;
use sharecli_fleet::thermal::ThermalLevel;
use sharecli_fleet::{AgentResourceSample, DetectedAgentWatch, ResourceWatchSample};
use sharecli_fuse::{NegDentryMeters, ReadCacheMeters, WriteSerializeMeters};
use sharecli_fleet::CoalesceMeters;
use sharecli_thermal_tui::{agent_lines, render, App};

const SAMPLE: ResourceWatchSample = ResourceWatchSample {
    fd_count: 12,
    net_rx_bytes: 0,
    net_tx_bytes: 0,
    mem_rss_bytes: 4096,
    load_1m: 0.5,
};

fn fixture_agents() -> Vec<DetectedAgentWatch> {
    vec![
        DetectedAgentWatch {
            agent: DetectedAgent {
                pid: 100,
                family: "claude",
                comm: "claude".into(),
            },
            resource: AgentResourceSample {
                mem_rss_bytes: 52_428_800,
                fd_count: Some(24),
            },
        },
        DetectedAgentWatch {
            agent: DetectedAgent {
                pid: 250,
                family: "cursor",
                comm: "cursor-agent".into(),
            },
            resource: AgentResourceSample {
                mem_rss_bytes: 104_857_600,
                fd_count: None,
            },
        },
    ]
}

/// FR-006 / AC-006.9 + AC-006.10 — agent_lines renders inventory + RSS rows.
#[test]
fn fr006_thermal_tui_agent_lines() {
    let lines = agent_lines(&fixture_agents(), false);
    let text: String = lines.iter().map(|l| l.to_string()).collect();
    assert!(text.contains("Agents: 2"));
    assert!(text.contains("PID 100") && text.contains("claude"));
    assert!(text.contains("RSS 50M"));
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
            CoalesceMeters::default(),
            WriteSerializeMeters::default(),
        )
        .with_detected_agents(fixture_agents());
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
    assert!(
        rendered.contains("RSS"),
        "thermal TUI MUST surface per-agent RSS watch"
    );
}
