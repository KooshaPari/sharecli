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
        // FSKit is preferred; the mount layer may reject it for incompatible
        // legacy filesystems, at which point callers can retry Kernel.
        return FuseBackend::Fskit;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_override_degrades_closed() {
        std::env::set_var("SHARECLI_FUSE_BACKEND", "invalid");
        assert_eq!(select_backend(), FuseBackend::Unavailable);
        std::env::remove_var("SHARECLI_FUSE_BACKEND");
    }
}
