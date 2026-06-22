use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Instant;

static FAILURE_COUNT: AtomicU32 = AtomicU32::new(0);
static WINDOW_START_MS: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u64 {
    Instant::now().elapsed().as_millis() as u64
}

pub fn run(
    real_cmd: &Path,
    breaker_threshold: u32,
    breaker_window_secs: u64,
    args: &[&str],
) -> Result<i32, String> {
    let window_ms = breaker_window_secs.saturating_mul(1000);
    let prev_start = WINDOW_START_MS.load(Ordering::SeqCst);
    let now = now_ms();

    if now.saturating_sub(prev_start) >= window_ms {
        WINDOW_START_MS.store(now, Ordering::SeqCst);
        FAILURE_COUNT.store(0, Ordering::SeqCst);
    }

    if FAILURE_COUNT.load(Ordering::SeqCst) >= breaker_threshold {
        return Err("circuit open".to_string());
    }

    match Command::new(real_cmd).args(args).status() {
        Ok(s) if s.success() => {
            FAILURE_COUNT.store(0, Ordering::SeqCst);
            Ok(s.code().unwrap_or(0))
        }
        Ok(s) => {
            FAILURE_COUNT.fetch_add(1, Ordering::SeqCst);
            Err(format!("command failed: exit {}", s.code().unwrap_or(1)))
        }
        Err(e) => {
            FAILURE_COUNT.fetch_add(1, Ordering::SeqCst);
            Err(format!("exec failed: {}", e))
        }
    }
}
