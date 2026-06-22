use std::path::Path;

pub fn run(real_cmd: &Path, args: &[&str]) -> Result<i32, String> {
    let _ = (real_cmd, args);
    Err("batch strategy expects the caller to chunk file arguments before execution".to_string())
}
