//! Runtime backend negotiation for macFUSE on macOS.

use std::process::Command;

/// Selected interception backend for ShareCLI's optional filesystem layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FuseBackend {
    /// macFUSE FSKit/MFMount backend.
    Fskit,
    /// macFUSE VFS/KEXT backend.
    Kernel,
    /// No verified interception backend; callers must continue without FUSE.
    Unavailable,
}

/// Host capabilities used to make a deterministic backend decision.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FuseCapabilities {
    /// macFUSE KEXT/VFS backend is loaded and usable.
    pub kernel_loaded: bool,
    /// macFUSE MFMount/FSKit backend has been approved and is usable.
    pub fskit_approved: bool,
}

/// Select KEXT first, then approved FSKit, otherwise fail open.
pub fn select_backend_with(capabilities: FuseCapabilities) -> FuseBackend {
    if capabilities.kernel_loaded {
        FuseBackend::Kernel
    } else if capabilities.fskit_approved {
        FuseBackend::Fskit
    } else {
        FuseBackend::Unavailable
    }
}

/// Select the safest available backend. `SHARECLI_FUSE_BACKEND` may force
/// `fskit` or `kernel`; unsupported forcing degrades to `Unavailable`.
pub fn select_backend() -> FuseBackend {
    if let Ok(value) = std::env::var("SHARECLI_FUSE_BACKEND") {
        return match value.to_ascii_lowercase().as_str() {
            "fskit" if fskit_backend_approved() => FuseBackend::Fskit,
            "kernel" if kernel_backend_loaded() => FuseBackend::Kernel,
            _ => FuseBackend::Unavailable,
        };
    }
    if cfg!(target_os = "macos") {
        return select_backend_with(FuseCapabilities {
            kernel_loaded: kernel_backend_loaded(),
            fskit_approved: fskit_backend_approved(),
        });
    }
    if kernel_backend_loaded() {
        FuseBackend::Kernel
    } else {
        FuseBackend::Unavailable
    }
}

fn fskit_backend_approved() -> bool {
    #[cfg(target_os = "macos")]
    {
        std::env::var("SHARECLI_FUSE_FSKIT_APPROVED")
            .map(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
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

    #[test]
    fn backend_selection_is_kext_first_then_approved_fskit() {
        assert_eq!(
            select_backend_with(FuseCapabilities { kernel_loaded: true, fskit_approved: true }),
            FuseBackend::Kernel
        );
        assert_eq!(
            select_backend_with(FuseCapabilities { kernel_loaded: false, fskit_approved: true }),
            FuseBackend::Fskit
        );
        assert_eq!(select_backend_with(FuseCapabilities::default()), FuseBackend::Unavailable);
    }
}
