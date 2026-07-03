//! Thermal governor — reads system thermal/memory pressure before scheduling work.

/// System thermal pressure level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalLevel {
    Green,
    Yellow,
    Red,
}

/// A thermal / memory-pressure governor.
///
/// `poll()` returns the current system thermal state.  The `with_mock()`
/// constructor is available (in all builds) for downstream tests that need
/// deterministic control over the thermal level.
#[derive(Debug, Clone)]
pub struct ThermalGovernor {
    /// When `Some`, `poll()` returns this level instead of querying the real
    /// system.  Intended for downstream tests.
    mock_level: Option<ThermalLevel>,
    /// Prevents direct construction outside the crate.
    _private: (),
}

impl ThermalGovernor {
    /// Create a new `ThermalGovernor` that polls the real system.
    pub fn new() -> Self {
        Self { mock_level: None, _private: () }
    }

    /// Create a `ThermalGovernor` that always returns the given level.
    ///
    /// Intended for downstream tests that need deterministic thermal states
    /// (e.g. verifying that `Red` correctly blocks spawns).
    pub fn with_mock(level: ThermalLevel) -> Self {
        Self { mock_level: Some(level), _private: () }
    }

    /// Poll the current thermal / memory-pressure level.
    ///
    /// Returns the mock level if one was set via [`Self::with_mock`];
    /// otherwise queries the real system.
    pub fn poll(&self) -> anyhow::Result<ThermalLevel> {
        if let Some(level) = self.mock_level {
            return Ok(level);
        }
        self.poll_impl()
    }

    #[cfg(target_os = "macos")]
    fn poll_impl(&self) -> anyhow::Result<ThermalLevel> {
        let output = std::process::Command::new("sysctl")
            .arg("-n")
            .arg("kern.memorystatus_vm_pressure_level")
            .output()?;
        let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
        match raw.as_str() {
            "1" => Ok(ThermalLevel::Green),
            "2" => Ok(ThermalLevel::Yellow),
            "4" => Ok(ThermalLevel::Red),
            other => anyhow::bail!("unexpected pressure level: {other}"),
        }
    }

    #[cfg(target_os = "linux")]
    fn poll_impl(&self) -> anyhow::Result<ThermalLevel> {
        let contents = std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")?;
        let millidegrees: u64 = contents.trim().parse()?;
        Ok(match millidegrees {
            t if t < 70_000 => ThermalLevel::Green,
            t if t < 85_000 => ThermalLevel::Yellow,
            _ => ThermalLevel::Red,
        })
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    fn poll_impl(&self) -> anyhow::Result<ThermalLevel> {
        Ok(ThermalLevel::Green)
    }
}
