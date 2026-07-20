//! FR-007 — thermal TUI operator watch panels (Feb harness dashboard slice)
//! FR: FR-007
//!
//! AC-007.9 FUSE read-coalesce meters in `sharecli thermal` TUI
//! AC-007.10 host resource watch in `sharecli thermal` TUI

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use sharecli_fleet::thermal::ThermalLevel;
use sharecli_fleet::ResourceWatchSample;
use sharecli_fuse::{NegDentryMeters, ReadCacheMeters};
use sharecli_thermal_tui::{fuse_coalesce_lines, fuse_neg_dentry_lines, render, resource_watch_lines, App};

const SAMPLE: ResourceWatchSample = ResourceWatchSample {
    fd_count: 24,
    net_rx_bytes: 4096,
    net_tx_bytes: 2048,
    mem_rss_bytes: 2_097_152,
    load_1m: 0.75,
};

const FUSE_METERS: ReadCacheMeters = ReadCacheMeters { hits: 5, misses: 2 };
const NEG_METERS: NegDentryMeters = NegDentryMeters { hits: 3, misses: 1 };

/// FR-007 / AC-007.10 — thermal TUI renders host resource watch lines.
#[test]
fn fr007_thermal_tui_resource_watch_lines() {
    let lines = resource_watch_lines(Some(SAMPLE), false);
    let text: String = lines.iter().map(|l| l.to_string()).collect();
    assert!(text.contains("Open FDs:") && text.contains("24"));
    assert!(text.contains("RSS:") && text.contains("2097152"));
    assert!(text.contains("Load (1m):") && text.contains("0.75"));
}

/// FR-007 / AC-007.9 — thermal TUI renders FUSE read-coalesce meters.
#[test]
fn fr007_thermal_tui_fuse_coalesce_lines() {
    let lines = fuse_coalesce_lines(FUSE_METERS, false);
    let text: String = lines.iter().map(|l| l.to_string()).collect();
    assert!(text.contains("Cache hits:") && text.contains('5'));
    assert!(text.contains("Cache misses:") && text.contains('2'));
    assert!(text.contains("Hit rate:") && text.contains("71"));
}

/// FR-009 / AC-009.9 — thermal TUI renders FUSE negative-dentry meters.
#[test]
fn fr009_thermal_tui_neg_dentry_lines() {
    let lines = fuse_neg_dentry_lines(NEG_METERS, false);
    let text: String = lines.iter().map(|l| l.to_string()).collect();
    assert!(text.contains("Neg hits:") && text.contains('3'));
    assert!(text.contains("Neg misses:") && text.contains('1'));
    assert!(text.contains("Hit rate:") && text.contains("75"));
}

/// FR-007 / AC-007.9 + AC-007.10 + FR-009 AC-009.9 — headless thermal render includes panels.
#[test]
fn fr007_thermal_tui_render_includes_operator_panels() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("terminal");
    let mut app = App::new(4)
        .with_operator_meters(Some(SAMPLE), FUSE_METERS, NEG_METERS);
    app.update(ThermalLevel::Green, 1);

    terminal.draw(|f| render(f, &app)).expect("draw");
    let rendered: String = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|c| c.symbol().to_string())
        .collect();

    assert!(
        rendered.contains("Host Resource Watch") && rendered.contains("Open FDs:"),
        "thermal TUI MUST surface host resource watch; got excerpt missing panels"
    );
    assert!(
        rendered.contains("FUSE IO Meters") && rendered.contains("Cache hits:"),
        "thermal TUI MUST surface FUSE read-coalesce meters"
    );
    assert!(
        rendered.contains("Neg hits:") && rendered.contains("Neg misses:"),
        "thermal TUI MUST surface FUSE negative-dentry meters"
    );
}
