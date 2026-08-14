//! End-to-end coverage for shell-free `agent-call` execution.
//! FR: FR-004

#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, Instant};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sharecli"))
}

fn combined_output(out: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))
}

fn write_probe(project: &Path) -> PathBuf {
    let probe = project.join("argv-probe.sh");
    fs::write(
        &probe,
        r#"#!/bin/sh
{
  printf 'cwd=%s\n' "$PWD"
  printf 'argc=%s\n' "$#"
  for arg in "$@"; do
    printf 'argv=%s\n' "$arg"
  done
} > "$SHARECLI_AGENT_CALL_PROBE_LOG"
"#,
    )
    .expect("write argv probe");
    fs::set_permissions(&probe, fs::Permissions::from_mode(0o755))
        .expect("make argv probe executable");
    probe
}

fn read_probe_log(log: &Path) -> String {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match fs::read_to_string(log) {
            Ok(contents) => return contents,
            Err(error)
                if error.kind() == std::io::ErrorKind::NotFound && Instant::now() < deadline =>
            {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("read probe log {}: {error}", log.display()),
        }
    }
}

/// `agent-call` executes direct argv in the requested project without a shell.
#[test]
fn agent_call_passes_literal_argv_and_project_cwd_to_probe() {
    let fixture = tempfile::tempdir().expect("temp fixture");
    let project = fixture.path().join("project");
    fs::create_dir(&project).expect("create project");
    let probe = write_probe(&project);
    let log = fixture.path().join("allowed-probe.log");

    let out = bin()
        .arg("agent-call")
        .arg("--project")
        .arg(&project)
        .arg("--")
        .arg(&probe)
        .arg("one arg; echo never")
        .arg("two words")
        .env("SHARECLI_AGENT_CALL_PROBE_LOG", &log)
        .output()
        .expect("spawn allowed agent-call");

    assert!(
        out.status.success(),
        "allowed agent-call MUST exit successfully; combined output: {}",
        combined_output(&out)
    );
    assert_eq!(
        read_probe_log(&log),
        format!(
            "cwd={}\nargc=2\nargv=one arg; echo never\nargv=two words\n",
            project.canonicalize().expect("canonical project path").display()
        ),
        "agent-call MUST pass each supplied argument literally and set cwd to --project"
    );
}

/// Admission failure is structured and happens before the requested probe starts.
#[test]
fn agent_call_denial_reports_structured_fields_without_running_probe() {
    let fixture = tempfile::tempdir().expect("temp fixture");
    let project = fixture.path().join("project");
    fs::create_dir(&project).expect("create project");
    let probe = write_probe(&project);
    let log = fixture.path().join("denied-probe.log");
    let outside_project = fixture.path().join("outside-project");

    let out = bin()
        .arg("agent-call")
        .arg("--project")
        .arg(&project)
        .arg("--")
        .arg(&probe)
        .arg("needle")
        .arg(&outside_project)
        .env("SHARECLI_AGENT_CALL_PROBE_LOG", &log)
        .output()
        .expect("spawn denied agent-call");

    assert!(
        !out.status.success(),
        "out-of-project target MUST be denied; combined output: {}",
        combined_output(&out)
    );
    let combined = combined_output(&out);
    for field in ["code=", "reason=", "resume=", "suggestion="] {
        assert!(
            combined.contains(field),
            "denial MUST include structured {field} field; combined output: {combined}"
        );
    }
    assert!(
        !log.exists(),
        "denied agent-call MUST not start the probe; probe log exists at {}",
        log.display()
    );
}
