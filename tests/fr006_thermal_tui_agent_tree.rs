//! FR-006 — thermal TUI DetectedAgent tree panel (`build_agent_forests`)
//! FR: FR-006
//!
//! AC-006.22 full-layout thermal TUI renders parent-child agent forests.

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use sharecli_fleet::proc_scan::DetectedAgent;
use sharecli_fleet::thermal::ThermalLevel;
use sharecli_fleet::CoalesceMeters;
use sharecli_fleet::SlotQueueMeters;
use sharecli_fleet::{AgentResourceSample, AgentTreeNode, DetectedAgentWatch, ResourceWatchSample};
use sharecli_fuse::{NegDentryMeters, ReadCacheMeters, WriteSerializeMeters};
use sharecli_thermal_tui::{agent_forest_lines, render, App};

const SAMPLE: ResourceWatchSample = ResourceWatchSample {
    fd_count: 12,
    net_rx_bytes: 0,
    net_tx_bytes: 0,
    mem_rss_bytes: 4096,
    load_1m: 0.5,
};

fn fixture_watched() -> Vec<DetectedAgentWatch> {
    vec![
        DetectedAgentWatch {
            agent: DetectedAgent { pid: 100, family: "claude", comm: "claude".into() },
            resource: AgentResourceSample { mem_rss_bytes: 52_428_800, fd_count: Some(24) },
        },
        DetectedAgentWatch {
            agent: DetectedAgent { pid: 101, family: "claude", comm: "node".into() },
            resource: AgentResourceSample { mem_rss_bytes: 2_097_152, fd_count: None },
        },
    ]
}

fn fixture_forests() -> Vec<AgentTreeNode> {
    vec![AgentTreeNode {
        pid: 100,
        ppid: 1,
        comm: "claude".into(),
        family: Some("claude"),
        children: vec![AgentTreeNode {
            pid: 101,
            ppid: 100,
            comm: "node".into(),
            family: None,
            children: vec![],
        }],
    }]
}

/// FR-006 / AC-006.22 — agent_forest_lines renders proc-scan tree connectors + RSS.
#[test]
fn fr006_thermal_tui_agent_forest_lines() {
    let lines = agent_forest_lines(&fixture_forests(), &fixture_watched(), false);
    let text: String = lines.iter().map(|l| l.to_string()).collect();
    assert!(text.contains("Forests: 1"), "full layout MUST show forest count; got: {text}");
    assert!(
        text.contains("[100]") && text.contains("claude") && text.contains("RSS 50M"),
        "root MUST show family + RSS; got: {text}"
    );
    assert!(
        text.contains("└──") && text.contains("[101]") && text.contains("node"),
        "child subtree MUST use tree connectors; got: {text}"
    );
}

/// FR-006 / AC-006.22 — compact layout keeps flat agent summary (no tree connectors).
#[test]
fn fr006_thermal_tui_agent_forest_lines_compact_flat() {
    let lines = agent_forest_lines(&fixture_forests(), &fixture_watched(), true);
    let text: String = lines.iter().map(|l| l.to_string()).collect();
    assert!(!text.contains("└──"), "compact MUST NOT render tree connectors; got: {text}");
    assert!(text.contains("claude:100@"), "compact MUST keep flat summary; got: {text}");
}

/// FR-006 / AC-006.22 — headless thermal render includes agent process tree.
#[test]
fn fr006_thermal_tui_render_includes_agent_tree() {
    let backend = TestBackend::new(120, 34);
    let mut terminal = Terminal::new(backend).expect("terminal");
    let mut app = App::new(4)
        .with_operator_meters(
            Some(SAMPLE),
            ReadCacheMeters { hits: 1, misses: 0 },
            NegDentryMeters { hits: 0, misses: 0 },
            CoalesceMeters::default(),
            SlotQueueMeters::default(),
            WriteSerializeMeters::default(),
        )
        .with_detected_agents(fixture_watched())
        .with_agent_forests(fixture_forests());
    app.update(ThermalLevel::Green, 0);

    terminal.draw(|f| render(f, &app)).expect("draw");
    let rendered: String =
        terminal.backend().buffer().content.iter().map(|c| c.symbol().to_string()).collect();

    assert!(
        rendered.contains("Detected Agents") && rendered.contains("[100]") && rendered.contains("└──"),
        "thermal TUI MUST surface build_agent_forests tree; got excerpt: {}",
        &rendered.chars().take(400).collect::<String>()
    );
}
