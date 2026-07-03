//! Caster abstraction — sends text to a registered pane.
//!
//! A `Caster` is the runtime side of a `PaneAddress` lookup. Given a
//! resolved `PaneAddress`, it knows how to ship text to the right
//! terminal pane on the right machine.
//!
//! FR: FR-CAST-003, FR-CAST-004, FR-CAST-005, FR-CAST-007

use std::io;
use std::process::Command;

use anyhow::{anyhow, Result};

use super::address::PaneAddress;

/// Outcome of a send — distinguishes the failure modes the caller cares about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendOutcome {
    /// Text was delivered to the pane.
    Delivered,
    /// Cast is supported but the pane is not focusable (e.g. occluded).
    NeedsFocus,
    /// Cast is not supported in this environment; user must copy manually.
    Unsupported(String),
    /// Cast failed for an unexpected reason (network, race, etc.).
    Failed(String),
}

/// Pluggable transport — `wezterm`, `ghostty`, `wt`, or `clipboard`.
pub trait Caster: Send + Sync {
    /// Human-readable name (used in error messages and `--caster` flag).
    fn name(&self) -> &'static str;

    /// Resolve the pane ID for a `PaneAddress` on the current host.
    /// Returns `None` if the pane is not visible to this caster.
    fn resolve_pane_id(&self, addr: &PaneAddress) -> Result<Option<u32>>;

    /// Ship `text` to the pane.
    fn send(&self, addr: &PaneAddress, text: &str) -> SendOutcome;
}

/// Probe for an executable on `PATH`. Returns the resolved path or `None`.
pub fn which(bin: &str) -> Option<std::path::PathBuf> {
    let exts: &[&str] = if cfg!(windows) { &["", ".exe", ".cmd", ".bat"] } else { &[""] };
    let paths = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&paths) {
        for ext in exts {
            let candidate = dir.join(format!("{}{}", bin, ext));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Build a `Command` for `bin`, capturing stdout/stderr.
fn run(bin: &str, args: &[&str]) -> io::Result<std::process::Output> {
    Command::new(bin).args(args).output()
}

/// Cast through the `wezterm` CLI (`wezterm cli send-text`).
///
/// The wezterm command line is the only terminal that ships a real
/// inter-process control surface today. This caster shells out to
/// `wezterm cli list` to resolve window:pane → numeric pane id, then
/// `wezterm cli send-text --pane-id <id> <text>` to deliver.
pub struct WeztermCaster;

impl Caster for WeztermCaster {
    fn name(&self) -> &'static str {
        "wezterm"
    }

    fn resolve_pane_id(&self, addr: &PaneAddress) -> Result<Option<u32>> {
        if which("wezterm").is_none() {
            return Err(anyhow!("wezterm not found on PATH"));
        }
        // `wezterm cli list --format json` — one entry per pane with
        // {window_id, pane_id, tab_id, title, ...}.
        let output = run("wezterm", &["cli", "list", "--format", "json"])
            .map_err(|e| anyhow!("wezterm cli list failed: {}", e))?;
        if !output.status.success() {
            return Err(anyhow!(
                "wezterm cli list exited {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let body = String::from_utf8_lossy(&output.stdout);
        // Parse: we keep this minimal — JSON lines with "window_id" + "pane_id".
        // For a from-scratch caster we don't pull in a full JSON dep.
        // The full implementation lives in follow-up task; this stub
        // returns Ok(None) so callers can degrade to the next caster.
        let _ = body;
        Ok(None)
    }

    fn send(&self, addr: &PaneAddress, text: &str) -> SendOutcome {
        if which("wezterm").is_none() {
            return SendOutcome::Unsupported("wezterm not found on PATH".into());
        }
        let pane_id = match self.resolve_pane_id(addr) {
            Ok(Some(id)) => id,
            Ok(None) => {
                return SendOutcome::Failed(
                    "wezterm pane id resolution returned None (no JSON parser yet)".into(),
                );
            }
            Err(e) => return SendOutcome::Failed(e.to_string()),
        };
        match run(
            "wezterm",
            &["cli", "send-text", "--pane-id", &pane_id.to_string(), "--no-paste", text],
        ) {
            Ok(o) if o.status.success() => SendOutcome::Delivered,
            Ok(o) => SendOutcome::Failed(format!(
                "wezterm cli send-text exited {}: {}",
                o.status,
                String::from_utf8_lossy(&o.stderr)
            )),
            Err(e) => SendOutcome::Failed(e.to_string()),
        }
    }
}

/// Cast via the system clipboard (last-resort fallback). Always works,
/// but the user has to paste manually.
pub struct ClipboardCaster;

impl Caster for ClipboardCaster {
    fn name(&self) -> &'static str {
        "clipboard"
    }

    fn resolve_pane_id(&self, _addr: &PaneAddress) -> Result<Option<u32>> {
        Ok(None)
    }

    fn send(&self, _addr: &PaneAddress, text: &str) -> SendOutcome {
        let (bin, args): (&str, Vec<&str>) = if cfg!(target_os = "macos") {
            ("pbcopy", vec![])
        } else if cfg!(target_os = "windows") {
            ("clip", vec![])
        } else {
            match which("wl-copy") {
                Some(_) => ("wl-copy", vec![]),
                None => match which("xclip") {
                    Some(_) => ("xclip", vec!["-selection", "clipboard"]),
                    None => {
                        return SendOutcome::Unsupported(
                            "no clipboard binary (pbcopy/clip/wl-copy/xclip) on PATH".into(),
                        );
                    }
                },
            }
        };
        use std::io::Write;
        use std::process::Stdio;
        let mut child = match Command::new(bin)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => return SendOutcome::Failed(e.to_string()),
        };
        if let Some(mut stdin) = child.stdin.take() {
            if let Err(e) = stdin.write_all(text.as_bytes()) {
                return SendOutcome::Failed(format!("clipboard write failed: {}", e));
            }
        }
        match child.wait_with_output() {
            Ok(o) if o.status.success() => SendOutcome::Delivered,
            Ok(o) => SendOutcome::Failed(format!(
                "{} exited {}: {}",
                bin,
                o.status,
                String::from_utf8_lossy(&o.stderr)
            )),
            Err(e) => SendOutcome::Failed(e.to_string()),
        }
    }
}

/// Caster chain — try each in order; return the first non-`Unsupported` outcome.
pub fn send_with_fallback(
    addrs: &[(std::sync::Arc<dyn Caster>, String)],
    addr: &PaneAddress,
    text: &str,
) -> SendOutcome {
    let mut last_unsupported = None;
    for (caster, label) in addrs {
        let outcome = caster.send(addr, text);
        match &outcome {
            SendOutcome::Unsupported(msg) => {
                last_unsupported = Some(format!("{}: {}", label, msg));
                continue;
            }
            _ => return outcome,
        }
    }
    SendOutcome::Unsupported(last_unsupported.unwrap_or_else(|| "no casters configured".into()))
}

// ---------------------------------------------------------------------------
// unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn which_finds_common_bins() {
        // `sh` is on every unix PATH; skip on windows where it isn't.
        if !cfg!(windows) {
            assert!(which("sh").is_some(), "sh should be on PATH");
        }
    }

    #[test]
    fn which_returns_none_for_missing() {
        assert!(which("definitely-not-a-binary-12345").is_none());
    }

    #[test]
    fn send_with_fallback_returns_first_non_unsupported() {
        let a: std::sync::Arc<dyn Caster> = std::sync::Arc::new(ClipboardCaster);
        let addr = PaneAddress::parse("mbp:local:0:0").expect("addr");
        let outcome = send_with_fallback(&[(a, "clipboard".to_string())], &addr, "hello");
        // The clipboard caster either delivers (real env) or reports unsupported
        // (no clipboard binary). Either is acceptable here.
        assert!(matches!(
            outcome,
            SendOutcome::Delivered | SendOutcome::Unsupported(_)
        ));
    }

    #[test]
    fn send_with_fallback_uses_last_unsupported_when_all_unsupported() {
        struct AlwaysUnsupported;
        impl Caster for AlwaysUnsupported {
            fn name(&self) -> &'static str {
                "noop"
            }
            fn resolve_pane_id(&self, _: &PaneAddress) -> Result<Option<u32>> {
                Ok(None)
            }
            fn send(&self, _: &PaneAddress, _: &str) -> SendOutcome {
                SendOutcome::Unsupported("nope".into())
            }
        }
        let a: std::sync::Arc<dyn Caster> = std::sync::Arc::new(AlwaysUnsupported);
        let addr = PaneAddress::parse("mbp:local:0:0").expect("addr");
        let outcome = send_with_fallback(&[(a, "noop".to_string())], &addr, "x");
        assert!(matches!(outcome, SendOutcome::Unsupported(ref m) if m == "noop: nope"));
    }
}
