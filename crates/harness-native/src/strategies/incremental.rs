use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

pub fn run(real_cmd: &Path, args: &[&str]) -> Result<i32, String> {
    let mut cmd = Command::new(real_cmd);
    cmd.args(args);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    match cmd.spawn() {
        Ok(mut child) => {
            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            let stdout_handle = stdout.map(|stdout| {
                thread::spawn(move || {
                    for line in BufReader::new(stdout).lines() {
                        match line {
                            Ok(line) => println!("{}", line),
                            Err(e) => {
                                eprintln!("stdout read error: {}", e);
                                break;
                            }
                        }
                    }
                })
            });

            let stderr_handle = stderr.map(|stderr| {
                thread::spawn(move || {
                    for line in BufReader::new(stderr).lines() {
                        match line {
                            Ok(line) => eprintln!("{}", line),
                            Err(e) => {
                                eprintln!("stderr read error: {}", e);
                                break;
                            }
                        }
                    }
                })
            });

            let exit = match child.wait() {
                Ok(exit) => Ok(exit.code().unwrap_or(1)),
                Err(e) => Err(format!("wait failed: {}", e)),
            };

            if let Some(handle) = stdout_handle {
                if handle.join().is_err() {
                    return Err("stdout drain thread panicked".to_string());
                }
            }
            if let Some(handle) = stderr_handle {
                if handle.join().is_err() {
                    return Err("stderr drain thread panicked".to_string());
                }
            }

            exit
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err(format!("command not found: {:?}", real_cmd))
        }
        Err(e) => Err(format!("failed to spawn {:?}: {}", real_cmd, e)),
    }
}
