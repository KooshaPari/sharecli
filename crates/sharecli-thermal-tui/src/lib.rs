//! `sharecli-thermal-tui` — live TUI for thermal-gate / hypervisor state.
//!
//! # Design
//!
//! All display transforms (pressure → style/label, count → gauge, decision →
//! indicator) are **pure functions** so they can be unit-tested without a
//! terminal.  The `App` struct holds only data-model state; the `render`
//! function (also pure, takes `&mut Frame`) performs the layout.
//!
//! The event loop in [`run`] polls the [`ThermalGovernor`] on a configurable
//! interval and redraws until the user presses `q` or `Ctrl-C`.

use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};
use sharecli_fleet::thermal::{ThermalGovernor, ThermalLevel};

// ---------------------------------------------------------------------------
// Pure transforms — unit-testable
// ---------------------------------------------------------------------------

/// Returns true when the key event should exit the TUI (`q` or `Ctrl-C`).
pub fn is_quit_key(key: &KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('q') => true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => true,
        _ => false,
    }
}

/// Map a [`ThermalLevel`] to a human-readable label.
pub fn level_label(level: ThermalLevel) -> &'static str {
    match level {
        ThermalLevel::Green => "GREEN",
        ThermalLevel::Yellow => "YELLOW",
        ThermalLevel::Red => "RED",
    }
}

/// Map a [`ThermalLevel`] to a foreground [`Color`].
pub fn level_color(level: ThermalLevel) -> Color {
    match level {
        ThermalLevel::Green => Color::Green,
        ThermalLevel::Yellow => Color::Yellow,
        ThermalLevel::Red => Color::Red,
    }
}

/// Map a [`ThermalLevel`] to the integer pressure value returned by sysctl.
pub fn level_pressure_raw(level: ThermalLevel) -> u8 {
    match level {
        ThermalLevel::Green => 1,
        ThermalLevel::Yellow => 2,
        ThermalLevel::Red => 4,
    }
}

/// The gate's admit/deny decision label given a [`ThermalLevel`].
///
/// Green and Yellow → ADMIT; Red → DENY.
pub fn gate_decision(level: ThermalLevel) -> &'static str {
    match level {
        ThermalLevel::Green | ThermalLevel::Yellow => "ADMIT",
        ThermalLevel::Red => "DENY",
    }
}

/// Color for the gate decision indicator.
pub fn decision_color(level: ThermalLevel) -> Color {
    match level {
        ThermalLevel::Green | ThermalLevel::Yellow => Color::Green,
        ThermalLevel::Red => Color::Red,
    }
}

/// Compute a gauge ratio for the build-slot indicator.
///
/// Returns a value in `[0.0, 1.0]`.  `active` is clamped to `[0, cap]`.
pub fn slot_ratio(active: u32, cap: u32) -> f64 {
    if cap == 0 {
        return 0.0;
    }
    let clamped = active.min(cap) as f64;
    clamped / cap as f64
}

/// Color for the slot-usage gauge: green < 50 %, yellow < 90 %, red otherwise.
pub fn slot_color(active: u32, cap: u32) -> Color {
    let ratio = slot_ratio(active, cap);
    if ratio < 0.5 {
        Color::Green
    } else if ratio < 0.9 {
        Color::Yellow
    } else {
        Color::Red
    }
}

/// Columns below this width switch the TUI to compact layout (L81.11).
pub const COMPACT_WIDTH: u16 = 80;

/// True when the terminal is narrower than [`COMPACT_WIDTH`].
///
/// Prefer `frame.area().width` (ratatui already reflects `crossterm::terminal::size`)
/// or an explicit `COLUMNS`-derived width in tests.
pub fn is_compact(width: u16) -> bool {
    width < COMPACT_WIDTH
}

/// Short pressure blurb for compact terminals; full sentence otherwise.
pub fn thermal_blurb(level: ThermalLevel, compact: bool) -> &'static str {
    if compact {
        match level {
            ThermalLevel::Green => "[GREEN] cool",
            ThermalLevel::Yellow => "[YELLOW] warm",
            ThermalLevel::Red => "[RED] hot — back-pressure",
        }
    } else {
        match level {
            ThermalLevel::Green => "[ GREEN  ] device is cool — spawns proceed",
            ThermalLevel::Yellow => "[ YELLOW ] device is warm — spawns proceed w/ warning",
            ThermalLevel::Red => "[ RED    ] device is hot — spawns BACK-PRESSURED",
        }
    }
}

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

/// Build-slot cap (max concurrent `cargo build|check|test` processes).
pub const DEFAULT_SLOT_CAP: u32 = 4;

/// Poll interval for the thermal governor.
pub const POLL_INTERVAL: Duration = Duration::from_secs(2);

/// Live application state.
pub struct App {
    /// Most-recent thermal level from the governor.
    pub thermal_level: ThermalLevel,
    /// Number of active build slots (detected via pgrep).
    pub active_slots: u32,
    /// Build-slot cap.
    pub slot_cap: u32,
    /// Timestamp of last poll.
    pub last_poll: Instant,
    /// Total number of polls performed.
    pub poll_count: u64,
}

impl App {
    /// Create with a default state (Green, 0 active slots).
    pub fn new(slot_cap: u32) -> Self {
        Self {
            thermal_level: ThermalLevel::Green,
            active_slots: 0,
            slot_cap,
            last_poll: Instant::now(),
            poll_count: 0,
        }
    }

    /// Update state from a new governor poll result.
    pub fn update(&mut self, level: ThermalLevel, active_slots: u32) {
        self.thermal_level = level;
        self.active_slots = active_slots;
        self.last_poll = Instant::now();
        self.poll_count += 1;
    }
}

// ---------------------------------------------------------------------------
// Active-slot detection
// ---------------------------------------------------------------------------

/// Count running `cargo (build|check|test)` processes via `pgrep`.
///
/// Returns 0 on any error (pgrep missing, etc.) so the TUI degrades gracefully.
pub fn count_cargo_builds() -> u32 {
    let output =
        std::process::Command::new("pgrep").args(["-f", "cargo (build|check|test)"]).output();
    match output {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout);
            s.lines().filter(|l| !l.trim().is_empty()).count() as u32
        }
        Err(_) => 0,
    }
}

// ---------------------------------------------------------------------------
// Render (pure, takes &mut Frame + &App)
// ---------------------------------------------------------------------------

/// Render the full TUI into `frame`.
///
/// Layout adapts to `frame.area().width` (backed by terminal size / `COLUMNS`).
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let compact = is_compact(area.width);
    let margin = if compact { 0 } else { 1 };
    let thermal_h = if compact { 4 } else { 5 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(margin)
        .constraints([
            Constraint::Length(3),         // title
            Constraint::Length(thermal_h), // thermal pressure block
            Constraint::Length(3),         // gate decision
            Constraint::Length(3),         // slot gauge
            Constraint::Length(3),         // footer
        ])
        .split(area);

    render_title(frame, chunks[0], compact);
    render_thermal(frame, chunks[1], app, compact);
    render_decision(frame, chunks[2], app, compact);
    render_slots(frame, chunks[3], app, compact);
    render_footer(frame, chunks[4], app, compact);
}

fn render_title(frame: &mut Frame, area: Rect, compact: bool) {
    let text = if compact { " thermal" } else { "  sharecli thermal monitor" };
    let title = Paragraph::new(Line::from(vec![Span::styled(
        text,
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::ALL).title(" sharecli "));
    frame.render_widget(title, area);
}

fn render_thermal(frame: &mut Frame, area: Rect, app: &App, compact: bool) {
    let level = app.thermal_level;
    let color = level_color(level);
    let label = level_label(level);
    let raw = level_pressure_raw(level);

    let pressure = if compact {
        format!("{label} ({raw})")
    } else {
        format!("{label}  (kern.memorystatus_vm_pressure_level = {raw})")
    };

    let mut lines = vec![Line::from(vec![
        Span::raw(if compact { " P: " } else { "  Pressure level: " }),
        Span::styled(pressure, Style::default().fg(color).add_modifier(Modifier::BOLD)),
    ])];
    if !compact {
        lines.push(Line::from(""));
    }
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(thermal_blurb(level, compact), Style::default().fg(color)),
    ]));

    let title = if compact { " Thermal " } else { " Thermal Pressure " };
    let block = Block::default().borders(Borders::ALL).title(title);
    let para = Paragraph::new(lines).block(block);
    frame.render_widget(para, area);
}

fn render_decision(frame: &mut Frame, area: Rect, app: &App, compact: bool) {
    let level = app.thermal_level;
    let decision = gate_decision(level);
    let color = decision_color(level);

    let hint = if compact {
        ""
    } else if level == ThermalLevel::Red {
        "  — hypervisor will retry up to 5x before returning Err"
    } else {
        ""
    };

    let line = Line::from(vec![
        Span::raw(if compact { " Gate: " } else { "  Gate decision: " }),
        Span::styled(
            format!("[ {decision} ]"),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(hint),
    ]);

    let title = if compact { " Gate " } else { " Gate Decision " };
    let block = Block::default().borders(Borders::ALL).title(title);
    let para = Paragraph::new(line).block(block);
    frame.render_widget(para, area);
}

fn render_slots(frame: &mut Frame, area: Rect, app: &App, compact: bool) {
    let ratio = slot_ratio(app.active_slots, app.slot_cap);
    let color = slot_color(app.active_slots, app.slot_cap);
    let label = if compact {
        format!(" {}/{} ", app.active_slots, app.slot_cap)
    } else {
        format!(" Build slots: {}/{} active ", app.active_slots, app.slot_cap)
    };

    let title = if compact { " Slots " } else { " Build Slots " };
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(title))
        .gauge_style(Style::default().fg(color).bg(Color::DarkGray))
        .ratio(ratio)
        .label(label);
    frame.render_widget(gauge, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App, compact: bool) {
    let elapsed = app.last_poll.elapsed().as_secs();
    let meta = if compact {
        format!(" polls:{} last:{}s", app.poll_count, elapsed)
    } else {
        format!(
            "  polls: {}  last: {}s ago  interval: {}s",
            app.poll_count,
            elapsed,
            POLL_INTERVAL.as_secs()
        )
    };
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" quit  "),
        Span::styled(" Ctrl-C", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" quit  "),
        Span::raw(meta),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, area);
}

// ---------------------------------------------------------------------------
// Event loop
// ---------------------------------------------------------------------------

/// Launch the TUI, polling `governor` every [`POLL_INTERVAL`].
///
/// Returns when the user presses `q` or `Ctrl-C`.
pub fn run(governor: &ThermalGovernor, slot_cap: u32) -> Result<()> {
    use std::io;

    use crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::backend::CrosstermBackend;
    use ratatui::Terminal;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(slot_cap);

    // Initial poll before first draw.
    let initial_level = governor.poll().unwrap_or(ThermalLevel::Green);
    let initial_slots = count_cargo_builds();
    app.update(initial_level, initial_slots);

    let result = event_loop(&mut terminal, &mut app, governor);

    // Always restore terminal even on error.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn event_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    governor: &ThermalGovernor,
) -> Result<()> {
    loop {
        terminal.draw(|f| render(f, app))?;

        // Poll for input with a timeout equal to the poll interval.
        // Also accept Resize so layout reflows when COLUMNS / terminal size changes.
        if event::poll(POLL_INTERVAL)? {
            match event::read()? {
                Event::Key(key) if is_quit_key(&key) => break,
                Event::Resize(_, _) => {
                    // Next draw uses the new frame.area().width (compact vs full).
                }
                _ => {}
            }
        }

        // Refresh thermal + slot state.
        let level = governor.poll().unwrap_or(ThermalLevel::Green);
        let slots = count_cargo_builds();
        app.update(level, slots);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests — pure-function coverage
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent { code, modifiers, kind: KeyEventKind::Press, state: KeyEventState::NONE }
    }

    // --- is_quit_key ---
    #[test]
    fn test_is_quit_key_q() {
        assert!(is_quit_key(&key_event(KeyCode::Char('q'), KeyModifiers::NONE)));
    }

    #[test]
    fn test_is_compact_columns() {
        assert!(is_compact(40));
        assert!(is_compact(79));
        assert!(!is_compact(80));
        assert!(!is_compact(120));
    }

    #[test]
    fn test_thermal_blurb_adapts() {
        assert_eq!(thermal_blurb(ThermalLevel::Red, true), "[RED] hot — back-pressure");
        assert!(thermal_blurb(ThermalLevel::Red, false).contains("BACK-PRESSURED"));
    }

    #[test]
    fn test_is_quit_key_ctrl_c() {
        assert!(is_quit_key(&key_event(KeyCode::Char('c'), KeyModifiers::CONTROL)));
    }

    #[test]
    fn test_is_quit_key_other_ignored() {
        assert!(!is_quit_key(&key_event(KeyCode::Char('a'), KeyModifiers::NONE)));
        assert!(!is_quit_key(&key_event(KeyCode::Char('c'), KeyModifiers::NONE)));
    }

    // --- level_label ---
    #[test]
    fn test_level_label_green() {
        assert_eq!(level_label(ThermalLevel::Green), "GREEN");
    }

    #[test]
    fn test_level_label_yellow() {
        assert_eq!(level_label(ThermalLevel::Yellow), "YELLOW");
    }

    #[test]
    fn test_level_label_red() {
        assert_eq!(level_label(ThermalLevel::Red), "RED");
    }

    // --- level_color ---
    #[test]
    fn test_level_color_green() {
        assert_eq!(level_color(ThermalLevel::Green), Color::Green);
    }

    #[test]
    fn test_level_color_yellow() {
        assert_eq!(level_color(ThermalLevel::Yellow), Color::Yellow);
    }

    #[test]
    fn test_level_color_red() {
        assert_eq!(level_color(ThermalLevel::Red), Color::Red);
    }

    // --- level_pressure_raw ---
    #[test]
    fn test_pressure_raw_green() {
        assert_eq!(level_pressure_raw(ThermalLevel::Green), 1);
    }

    #[test]
    fn test_pressure_raw_yellow() {
        assert_eq!(level_pressure_raw(ThermalLevel::Yellow), 2);
    }

    #[test]
    fn test_pressure_raw_red() {
        assert_eq!(level_pressure_raw(ThermalLevel::Red), 4);
    }

    // --- gate_decision ---
    #[test]
    fn test_decision_green_admit() {
        assert_eq!(gate_decision(ThermalLevel::Green), "ADMIT");
    }

    #[test]
    fn test_decision_yellow_admit() {
        assert_eq!(gate_decision(ThermalLevel::Yellow), "ADMIT");
    }

    #[test]
    fn test_decision_red_deny() {
        assert_eq!(gate_decision(ThermalLevel::Red), "DENY");
    }

    // --- decision_color ---
    #[test]
    fn test_decision_color_green() {
        assert_eq!(decision_color(ThermalLevel::Green), Color::Green);
    }

    #[test]
    fn test_decision_color_yellow() {
        assert_eq!(decision_color(ThermalLevel::Yellow), Color::Green);
    }

    #[test]
    fn test_decision_color_red() {
        assert_eq!(decision_color(ThermalLevel::Red), Color::Red);
    }

    // --- slot_ratio ---
    #[test]
    fn test_slot_ratio_zero_active() {
        assert!((slot_ratio(0, 4) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_slot_ratio_half() {
        assert!((slot_ratio(2, 4) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_slot_ratio_full() {
        assert!((slot_ratio(4, 4) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_slot_ratio_overflow_clamped() {
        // active > cap should clamp to 1.0
        assert!((slot_ratio(10, 4) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_slot_ratio_zero_cap() {
        assert!((slot_ratio(5, 0) - 0.0).abs() < 1e-9);
    }

    // --- slot_color ---
    #[test]
    fn test_slot_color_green_below_half() {
        assert_eq!(slot_color(1, 4), Color::Green);
    }

    #[test]
    fn test_slot_color_yellow_between_half_and_90() {
        assert_eq!(slot_color(2, 4), Color::Yellow); // 0.5 → yellow
        assert_eq!(slot_color(3, 4), Color::Yellow); // 0.75 → yellow
    }

    #[test]
    fn test_slot_color_red_at_cap() {
        assert_eq!(slot_color(4, 4), Color::Red); // 1.0 → red
    }

    #[test]
    fn test_slot_color_red_overflow_clamped() {
        assert_eq!(slot_color(10, 4), Color::Red);
    }

    // --- App::update ---
    #[test]
    fn test_app_update_increments_poll_count() {
        let mut app = App::new(4);
        assert_eq!(app.poll_count, 0);
        app.update(ThermalLevel::Yellow, 2);
        assert_eq!(app.poll_count, 1);
        app.update(ThermalLevel::Red, 3);
        assert_eq!(app.poll_count, 2);
    }

    #[test]
    fn test_app_update_stores_level() {
        let mut app = App::new(4);
        app.update(ThermalLevel::Red, 0);
        assert_eq!(app.thermal_level, ThermalLevel::Red);
    }

    #[test]
    fn test_app_update_stores_slots() {
        let mut app = App::new(4);
        app.update(ThermalLevel::Green, 3);
        assert_eq!(app.active_slots, 3);
    }

    // --- Headless render smoke test (FakeThermalGate via ThermalGovernor mock) ---
    #[test]
    fn test_render_green_headless() {
        use ratatui::{backend::TestBackend, Terminal};
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(4);
        app.update(ThermalLevel::Green, 0);
        // Should not panic.
        terminal.draw(|f| render(f, &app)).unwrap();
        let buf = terminal.backend().buffer().clone();
        // Title must appear somewhere.
        let rendered: String = buf.content.iter().map(|c| c.symbol().to_string()).collect();
        assert!(rendered.contains("GREEN"), "expected GREEN in rendered output");
        assert!(rendered.contains("ADMIT"), "expected ADMIT in rendered output");
    }

    #[test]
    fn test_render_red_headless() {
        use ratatui::{backend::TestBackend, Terminal};
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(4);
        app.update(ThermalLevel::Red, 4);
        terminal.draw(|f| render(f, &app)).unwrap();
        let buf = terminal.backend().buffer().clone();
        let rendered: String = buf.content.iter().map(|c| c.symbol().to_string()).collect();
        assert!(rendered.contains("RED"), "expected RED in rendered output");
        assert!(rendered.contains("DENY"), "expected DENY in rendered output");
    }

    #[test]
    fn test_render_yellow_headless() {
        use ratatui::{backend::TestBackend, Terminal};
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(4);
        app.update(ThermalLevel::Yellow, 2);
        terminal.draw(|f| render(f, &app)).unwrap();
        let buf = terminal.backend().buffer().clone();
        let rendered: String = buf.content.iter().map(|c| c.symbol().to_string()).collect();
        assert!(rendered.contains("YELLOW"), "expected YELLOW in rendered output");
        assert!(rendered.contains("ADMIT"), "expected ADMIT in rendered output");
    }

    // --- FakeThermalGate (via ThermalGovernor::with_mock) poll round-trip ---
    #[test]
    fn test_fake_gate_green_poll() {
        let gov = ThermalGovernor::with_mock(ThermalLevel::Green);
        let level = gov.poll().unwrap();
        assert_eq!(level, ThermalLevel::Green);
        assert_eq!(gate_decision(level), "ADMIT");
    }

    #[test]
    fn test_fake_gate_red_poll() {
        let gov = ThermalGovernor::with_mock(ThermalLevel::Red);
        let level = gov.poll().unwrap();
        assert_eq!(level, ThermalLevel::Red);
        assert_eq!(gate_decision(level), "DENY");
    }

    #[test]
    fn test_fake_gate_yellow_poll() {
        let gov = ThermalGovernor::with_mock(ThermalLevel::Yellow);
        let level = gov.poll().unwrap();
        assert_eq!(level, ThermalLevel::Yellow);
        assert_eq!(gate_decision(level), "ADMIT");
    }
}
