use std::path::Path;

use super::process;

pub fn run(real_cmd: &Path, args: &[&str]) -> Result<i32, String> {
    process::run_status(real_cmd, args)
}
