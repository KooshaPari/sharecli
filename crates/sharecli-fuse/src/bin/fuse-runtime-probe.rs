//! Read-only runtime evidence for the optional FUSE interception tier.
//!
//! This probe never loads a kext, changes approval state, mounts a volume, or
//! prompts for privilege. It reports the deterministic KEXT -> FSKit ->
//! non-FUSE selection used by ShareCLI.

use serde::Serialize;
use sharecli_fuse::{
    select_backend_for_mount_with, FuseBackend, FuseBackendDiagnostic, FuseCapabilities,
};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Serialize)]
struct ProbeReport {
    platform: &'static str,
    mountpoint: PathBuf,
    kernel_loaded: bool,
    fskit_framework: bool,
    fskit_approved: bool,
    selected_backend: &'static str,
    diagnostic: Option<&'static str>,
    non_fuse_fallback: bool,
}

fn main() {
    let mountpoint = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp/sharecli-fuse-runtime-probe"));
    let kernel_loaded = kernel_backend_loaded();
    let fskit_framework = mfmount_framework_available();
    let fskit_approved = fskit_framework && fskit_approved_by_operator();
    let selection = select_backend_for_mount_with(
        FuseCapabilities { kernel_loaded, fskit_approved },
        &mountpoint,
    );
    let report = ProbeReport {
        platform: std::env::consts::OS,
        mountpoint,
        kernel_loaded,
        fskit_framework,
        fskit_approved,
        selected_backend: backend_name(selection.backend),
        diagnostic: selection.diagnostic.map(diagnostic_message),
        non_fuse_fallback: true,
    };
    println!("{}", serde_json::to_string_pretty(&report).expect("serialize probe report"));
}

fn backend_name(backend: FuseBackend) -> &'static str {
    match backend {
        FuseBackend::Kernel => "kext",
        FuseBackend::Fskit => "fskit",
        FuseBackend::Unavailable => "non-fuse",
    }
}

fn diagnostic_message(diagnostic: FuseBackendDiagnostic) -> &'static str {
    diagnostic.message()
}

fn kernel_backend_loaded() -> bool {
    Command::new("kmutil")
        .args(["showloaded"])
        .output()
        .map(|output| {
            let mut text = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
            text.push_str(&String::from_utf8_lossy(&output.stderr).to_ascii_lowercase());
            text.contains("macfuse")
        })
        .unwrap_or(false)
}

fn mfmount_framework_available() -> bool {
    Path::new("/Library/Filesystems/macfuse.fs/Contents/Frameworks/MFMount.framework").is_dir()
}

fn fskit_approved_by_operator() -> bool {
    std::env::var("SHARECLI_FUSE_FSKIT_APPROVED")
        .map(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}
