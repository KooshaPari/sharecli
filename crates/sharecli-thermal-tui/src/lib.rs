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

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};
use sharecli_fleet::thermal::{ThermalGovernor, ThermalLevel};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Pure transforms — unit-testable
// ---------------------------------------------------------------------------

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
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Length(5), // thermal pressure block
            Constraint::Length(3), // gate decision
            Constraint::Length(3), // slot gauge
            Constraint::Length(3), // footer
        ])
        .split(area);

    render_title(frame, chunks[0]);
    render_thermal(frame, chunks[1], app);
    render_decision(frame, chunks[2], app);
    render_slots(frame, chunks[3], app);
    render_footer(frame, chunks[4], app);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![Span::styled(
        "  sharecli thermal monitor",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::ALL).title(" sharecli "));
    frame.render_widget(title, area);
}

fn render_thermal(frame: &mut Frame, area: Rect, app: &App) {
    let level = app.thermal_level;
    let color = level_color(level);
    let label = level_label(level);
    let raw = level_pressure_raw(level);

    let lines = vec![
        Line::from(vec![
            Span::raw("  Pressure level: "),
            Span::styled(
                format!("{label}  (kern.memorystatus_vm_pressure_level = {raw})"),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                match level {
                    ThermalLevel::Green => "[ GREEN  ] device is cool — spawns proceed",
                    ThermalLevel::Yellow => "[ YELLOW ] device is warm — spawns proceed w/ warning",
                    ThermalLevel::Red => "[ RED    ] device is hot — spawns BACK-PRESSURED",
                },
                Style::default().fg(color),
            ),
        ]),
    ];

    let block = Block::default().borders(Borders::ALL).title(" Thermal Pressure ");
    let para = Paragraph::new(lines).block(block);
    frame.render_widget(para, area);
}

fn render_decision(frame: &mut Frame, area: Rect, app: &App) {
    let level = app.thermal_level;
    let decision = gate_decision(level);
    let color = decision_color(level);

    let line = Line::from(vec![
        Span::raw("  Gate decision: "),
        Span::styled(
            format!("[ {decision} ]"),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(if level == ThermalLevel::Red {
            "  — hypervisor will retry up to 5x before returning Err"
        } else {
            ""
        }),
    ]);

    let block = Block::default().borders(Borders::ALL).title(" Gate Decision ");
    let para = Paragraph::new(line).block(block);
    frame.render_widget(para, area);
}

fn render_slots(frame: &mut Frame, area: Rect, app: &App) {
    let ratio = slot_ratio(app.active_slots, app.slot_cap);
    let color = slot_color(app.active_slots, app.slot_cap);
    let label = format!(" Build slots: {}/{} active ", app.active_slots, app.slot_cap);

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Build Slots "))
        .gauge_style(Style::default().fg(color).bg(Color::DarkGray))
        .ratio(ratio)
        .label(label);
    frame.render_widget(gauge, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let elapsed = app.last_poll.elapsed().as_secs();
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" quit  "),
        Span::styled(" Ctrl-C", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" quit  "),
        Span::raw(format!(
            "  polls: {}  last: {}s ago  interval: {}s",
            app.poll_count,
            elapsed,
            POLL_INTERVAL.as_secs()
        )),
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
    use crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::backend::CrosstermBackend;
    use ratatui::Terminal;
    use std::io;

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
        if event::poll(POLL_INTERVAL)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    _ => {}
                }
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
