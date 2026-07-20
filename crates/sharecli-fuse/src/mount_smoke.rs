//! Optional privileged FUSE mount smoke (operator / CI with macFUSE or libfuse).
//!
//! Default `cargo test` skips live mount verification. Set
//! `SHARECLI_FUSE_MOUNT_SMOKE=1` to run a read/write round-trip through a real
//! FUSE mountpoint (requires platform FUSE support and sufficient privileges).

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Environment variable that opts into privileged mount smoke tests.
pub const ENV_FUSE_MOUNT_SMOKE: &str = "SHARECLI_FUSE_MOUNT_SMOKE";

/// Return `true` when `SHARECLI_FUSE_MOUNT_SMOKE` is `1` or `true` (case-insensitive).
pub fn fuse_mount_smoke_enabled() -> bool {
    match std::env::var(ENV_FUSE_MOUNT_SMOKE) {
        Ok(v) => v == "1" || v.eq_ignore_ascii_case("true"),
        Err(_) => false,
    }
}

/// Best-effort force-unmount for a sharecli FUSE mountpoint.
pub fn force_unmount(mountpoint: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        let status = std::process::Command::new("fusermount")
            .arg("-uz")
            .arg(mountpoint)
            .status()?;
        if status.success() {
            return Ok(());
        }
    }
    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("umount").arg(mountpoint).status()?;
        if status.success() {
            return Ok(());
        }
    }
    Err(std::io::Error::other(format!(
        "sharecli-fuse: force_unmount failed for {}",
        mountpoint.display()
    )))
}

/// RAII session: background FUSE mount over `backing`, unmount on drop.
pub struct MountSession {
    mountpoint: PathBuf,
    _mount_dir: tempfile::TempDir,
    mount_thread: Option<JoinHandle<()>>,
}

impl MountSession {
    /// Mount `backing` at a fresh temp mountpoint; waits until the seed file is visible.
    pub fn start(backing: &Path, seed_rel: &Path) -> anyhow::Result<Self> {
        let mount_dir = tempfile::tempdir()?;
        let mountpoint = mount_dir.path().to_path_buf();
        let backing = backing.to_path_buf();
        let mp = mountpoint.clone();

        let (fail_tx, fail_rx) = mpsc::channel::<String>();
        let mount_thread = thread::spawn(move || {
            if let Err(err) = crate::mount(&mp, &backing) {
                let _ = fail_tx.send(err.to_string());
            }
        });

        let seed_on_mount = mountpoint.join(seed_rel);
        let deadline = Duration::from_secs(8);
        let poll = Duration::from_millis(100);
        let mut waited = Duration::ZERO;

        while waited < deadline {
            if seed_on_mount.is_file() {
                if std::fs::read(&seed_on_mount).is_ok() {
                    return Ok(Self {
                        mountpoint,
                        _mount_dir: mount_dir,
                        mount_thread: Some(mount_thread),
                    });
                }
            }
            if let Ok(msg) = fail_rx.try_recv() {
                anyhow::bail!("sharecli-fuse mount smoke: mount failed: {msg}");
            }
            thread::sleep(poll);
            waited += poll;
        }

        let _ = force_unmount(&mountpoint);
        anyhow::bail!(
            "sharecli-fuse mount smoke: timed out waiting for {} (is FUSE installed and permitted?)",
            seed_on_mount.display()
        )
    }

    /// Path where the FUSE layer is mounted.
    pub fn mountpoint(&self) -> &Path {
        &self.mountpoint
    }
}

impl Drop for MountSession {
    fn drop(&mut self) {
        let _ = force_unmount(&self.mountpoint);
        if let Some(handle) = self.mount_thread.take() {
            let _ = handle.join();
        }
    }
}

/// Read/write round-trip through a live FUSE mount (privileged smoke).
pub fn run_mount_smoke(backing: &Path) -> anyhow::Result<()> {
    let seed_rel = Path::new("smoke-seed.txt");
    std::fs::write(backing.join(seed_rel), b"before-fuse")?;

    let session = MountSession::start(backing, seed_rel)?;
    let mounted_seed = session.mountpoint().join(seed_rel);
    let body = std::fs::read(&mounted_seed)?;
    anyhow::ensure!(
        body == b"before-fuse",
        "mount smoke read: expected seed payload, got {} bytes",
        body.len()
    );

    std::fs::write(&mounted_seed, b"after-fuse")?;
    let backing_body = std::fs::read(backing.join(seed_rel))?;
    anyhow::ensure!(
        backing_body == b"after-fuse",
        "mount smoke write: backing file not updated through FUSE"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_env_disabled_by_default() {
        let prev = std::env::var(ENV_FUSE_MOUNT_SMOKE).ok();
        std::env::remove_var(ENV_FUSE_MOUNT_SMOKE);
        assert!(!fuse_mount_smoke_enabled());
        std::env::set_var(ENV_FUSE_MOUNT_SMOKE, "1");
        assert!(fuse_mount_smoke_enabled());
        std::env::set_var(ENV_FUSE_MOUNT_SMOKE, "true");
        assert!(fuse_mount_smoke_enabled());
        match prev {
            Some(v) => std::env::set_var(ENV_FUSE_MOUNT_SMOKE, v),
            None => std::env::remove_var(ENV_FUSE_MOUNT_SMOKE),
        }
    }
}
