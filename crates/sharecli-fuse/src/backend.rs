//! Runtime backend negotiation for macFUSE on macOS.

use std::process::Command;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FuseBackend {
    Fskit,
    Kernel,
    Unavailable,
}

/// Select the safest available backend. `SHARECLI_FUSE_BACKEND` may force
/// `fskit` or `kernel`; unsupported forcing degrades to `Unavailable`.
pub fn select_backend() -> FuseBackend {
    if let Ok(value) = std::env::var("SHARECLI_FUSE_BACKEND") {
        return match value.to_ascii_lowercase().as_str() {
            "fskit" => FuseBackend::Fskit,
            "kernel" if kernel_backend_loaded() => FuseBackend::Kernel,
            _ => FuseBackend::Unavailable,
        };
    }
    if cfg!(target_os = "macos") {
        // Prefer the loaded macFUSE kext for the mature, lowest-latency path.
        // FSKit remains the explicit fallback when the kext is unavailable.
        return if kernel_backend_loaded() { FuseBackend::Kernel } else { FuseBackend::Fskit };
    }
    if kernel_backend_loaded() {
        FuseBackend::Kernel
    } else {
        FuseBackend::Unavailable
    }
}

fn kernel_backend_loaded() -> bool {
    Command::new("kmutil")
        .args(["showloaded"])
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout).to_ascii_lowercase().contains("macfuse")
        })
        .unwrap_or(false)
}

/// Collect non-sensitive host state useful when a macFUSE mount negotiation fails.
///
/// This is intentionally executed only on an error path by callers. It does not
/// alter backend selection and avoids shelling out through a user-controlled shell.
pub(crate) fn runtime_diagnostics() -> String {
    #[cfg(target_os = "macos")]
    {
        let kext = Command::new("kmutil")
            .args(["showloaded"])
            .output()
            .map(|output| {
                if output.status.success() {
                    let loaded = String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .find(|line| line.to_ascii_lowercase().contains("macfuse"))
                        .map(str::trim)
                        .unwrap_or("not found")
                        .to_string();
                    format!("kext={loaded}")
                } else {
                    format!("kext=kmutil exit {}", output.status)
                }
            })
            .unwrap_or_else(|error| format!("kext=unavailable ({error})"));
        let version = std::fs::read_to_string(
            "/Library/Filesystems/macfuse.fs/Contents/version.plist",
        )
        .ok()
        .and_then(|contents| parse_bundle_version(&contents))
        .unwrap_or_else(|| "unknown".to_string());
        let fskit = Command::new("launchctl")
            .arg("list")
            .output()
            .map(|output| {
                if String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .any(|line| line.contains("com.apple.fskit.fskit_agent"))
                {
                    "running"
                } else {
                    "not-running"
                }
            })
            .unwrap_or("unavailable");
        return format!("macFUSE version-entry={version}; {kext}; fskit_agent={fskit}");
    }
    #[cfg(not(target_os = "macos"))]
    {
        "macFUSE diagnostics unavailable on this platform".to_string()
    }
}

fn parse_bundle_version(contents: &str) -> Option<String> {
    let mut lines = contents.lines();
    while let Some(line) = lines.next() {
        if line.contains("CFBundleShortVersionString") {
            return lines
                .find(|value| !value.trim().is_empty())
                .map(str::trim)
                .map(|value| value.trim_start_matches("<string>").trim_end_matches("</string>"))
                .filter(|value| !value.is_empty())
                .map(str::to_string);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_override_degrades_closed() {
        std::env::set_var("SHARECLI_FUSE_BACKEND", "invalid");
        assert_eq!(select_backend(), FuseBackend::Unavailable);
        std::env::remove_var("SHARECLI_FUSE_BACKEND");
    }

    #[test]
    fn bundle_version_probe_is_deterministic() {
        let plist = "<key>CFBundleShortVersionString</key>\n<string>5.3.3</string>";
        assert_eq!(parse_bundle_version(plist).as_deref(), Some("5.3.3"));
        assert_eq!(parse_bundle_version("<key>Other</key>\n<string>x</string>"), None);
    }
}
