use std::path::Path;
use std::process::Command;

pub fn run(real_cmd: &Path, args: &[&str]) -> Result<i32, String> {
    match Command::new(real_cmd).args(args).status() {
        Ok(exit) => Ok(exit.code().unwrap_or(1)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err(format!("command not found: {:?}", real_cmd))
        }
        Err(e) => Err(format!("failed to spawn {:?}: {}", real_cmd, e)),
    }
}
