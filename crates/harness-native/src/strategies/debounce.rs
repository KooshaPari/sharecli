use std::path::Path;
use std::thread;

use super::process;

pub fn run(real_cmd: &Path, debounce_ms: u64, args: &[&str]) -> Result<i32, String> {
    thread::sleep(std::time::Duration::from_millis(debounce_ms));
    process::run_status(real_cmd, args)
}
