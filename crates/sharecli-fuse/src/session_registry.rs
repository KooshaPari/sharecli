//! Process-local registry of live FUSE mounts (`mountpoint` → [`InterceptFs`]).
//!
//! Populated by [`FuseSessionRegistry::mount_background`] (CLI `fuse mount`) so
//! operator commands can call [`InterceptFs::commit_rel`] / [`discard_rel`] on
//! staged CoW without redesigning the hypervisor overlay.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::sync::Arc;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use fuser::{BackgroundSession, Config, MountOption};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::platform::{InterceptFs, SharedInterceptFs};

/// Summary of one registered FUSE mount (operator / `fuse list`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuseMountInfo {
    /// Normalized mountpoint path.
    pub mountpoint: PathBuf,
    /// Backing filesystem root mirrored by the intercept layer.
    pub backing: PathBuf,
    /// Write-provenance session id stamped on creates/writes.
    pub session_id: String,
    /// Relative paths with pending CoW staging on this mount.
    pub pending_relpaths: Vec<PathBuf>,
}

/// One live mount entry; dropping removes the background FUSE session.
#[cfg(any(target_os = "linux", target_os = "macos"))]
struct FuseMountEntry {
    fs: Arc<InterceptFs>,
    backing: PathBuf,
    session_id: String,
    /// Held for background mounts; `None` while a foreground `mount2` blocks.
    _session: Option<BackgroundSession>,
}

/// Process-local table of active sharecli FUSE mounts.
#[derive(Default)]
pub struct FuseSessionRegistry {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    mounts: Mutex<HashMap<PathBuf, FuseMountEntry>>,
}

impl FuseSessionRegistry {
    /// Global singleton used by the sharecli CLI.
    pub fn global() -> &'static Self {
        static REGISTRY: OnceLock<FuseSessionRegistry> = OnceLock::new();
        REGISTRY.get_or_init(FuseSessionRegistry::default)
    }

    fn normalize_key(path: &Path) -> PathBuf {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    }

    /// Mount `backing` at `mountpoint` in a background thread and register the session.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn mount_background(
        &self,
        mountpoint: &Path,
        backing: &Path,
        session_id: &str,
    ) -> anyhow::Result<()> {
        if !backing.is_dir() {
            anyhow::bail!(
                "fuse mount: backing path must be an existing directory: {}",
                backing.display()
            );
        }
        std::fs::create_dir_all(mountpoint).with_context_mount(mountpoint)?;
        let key = Self::normalize_key(mountpoint);
        {
            let mounts = self.mounts.lock().expect("fuse registry lock");
            if mounts.contains_key(&key) {
                anyhow::bail!("fuse mount: already registered at {}", key.display());
            }
        }

        let fs = Arc::new(InterceptFs::with_session(backing, session_id));
        let mut config = Config::default();
        config.mount_options =
            vec![MountOption::FSName("sharecli-fuse".to_string()), MountOption::AutoUnmount];

        let session = fuser::spawn_mount2(SharedInterceptFs(Arc::clone(&fs)), mountpoint, &config)
            .map_err(|e| anyhow::anyhow!("fuse mount: spawn_mount2 failed: {e}"))?;

        let mut mounts = self.mounts.lock().expect("fuse registry lock");
        mounts.insert(
            key,
            FuseMountEntry {
                fs,
                backing: backing.to_path_buf(),
                session_id: session_id.to_string(),
                _session: Some(session),
            },
        );
        Ok(())
    }

    /// Foreground mount (blocks until unmounted); registers while active.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn mount_foreground(
        &self,
        mountpoint: &Path,
        backing: &Path,
        session_id: &str,
    ) -> anyhow::Result<()> {
        if !backing.is_dir() {
            anyhow::bail!(
                "fuse mount: backing path must be an existing directory: {}",
                backing.display()
            );
        }
        std::fs::create_dir_all(mountpoint).with_context_mount(mountpoint)?;
        let key = Self::normalize_key(mountpoint);
        {
            let mounts = self.mounts.lock().expect("fuse registry lock");
            if mounts.contains_key(&key) {
                anyhow::bail!("fuse mount: already registered at {}", key.display());
            }
        }

        let fs = Arc::new(InterceptFs::with_session(backing, session_id));
        let mut config = Config::default();
        config.mount_options =
            vec![MountOption::FSName("sharecli-fuse".to_string()), MountOption::AutoUnmount];

        {
            let mut mounts = self.mounts.lock().expect("fuse registry lock");
            mounts.insert(
                key.clone(),
                FuseMountEntry {
                    fs: Arc::clone(&fs),
                    backing: backing.to_path_buf(),
                    session_id: session_id.to_string(),
                    _session: None,
                },
            );
        }

        let result = fuser::mount2(SharedInterceptFs(fs), mountpoint, &config);
        let mut mounts = self.mounts.lock().expect("fuse registry lock");
        mounts.remove(&key);
        result.map_err(|e| anyhow::anyhow!("fuse mount: mount2 ended with error: {e}"))?;
        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    pub fn mount_background(
        &self,
        mountpoint: &Path,
        backing: &Path,
        _session_id: &str,
    ) -> anyhow::Result<()> {
        let _ = (mountpoint, backing);
        anyhow::bail!("sharecli-fuse is only supported on Linux and macOS")
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    pub fn mount_foreground(
        &self,
        mountpoint: &Path,
        backing: &Path,
        _session_id: &str,
    ) -> anyhow::Result<()> {
        let _ = (mountpoint, backing);
        anyhow::bail!("sharecli-fuse is only supported on Linux and macOS")
    }

    /// Unmount and drop a registered session (best-effort force-unmount).
    pub fn unmount(&self, mountpoint: &Path) -> anyhow::Result<()> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let key = Self::normalize_key(mountpoint);
            let entry = {
                let mut mounts = self.mounts.lock().expect("fuse registry lock");
                mounts.remove(&key)
            };
            if entry.is_none() {
                anyhow::bail!("fuse unmount: no registered mount at {}", mountpoint.display());
            }
            let _ = crate::mount_smoke::force_unmount(mountpoint);
            Ok(())
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            let _ = mountpoint;
            anyhow::bail!("sharecli-fuse is only supported on Linux and macOS")
        }
    }

    /// Resolve `InterceptFs` for a mountpoint or the sole registered mount.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn resolve_fs(&self, mountpoint: Option<&Path>) -> anyhow::Result<Arc<InterceptFs>> {
        let mounts = self.mounts.lock().expect("fuse registry lock");
        match mountpoint {
            Some(mp) => {
                let key = Self::normalize_key(mp);
                mounts
                    .get(&key)
                    .map(|e| Arc::clone(&e.fs))
                    .ok_or_else(|| anyhow::anyhow!("fuse: no active mount at {}", mp.display()))
            }
            None => {
                if mounts.len() == 1 {
                    Ok(Arc::clone(&mounts.values().next().expect("one mount").fs))
                } else if mounts.is_empty() {
                    anyhow::bail!("fuse: no active FUSE mounts registered (run `fuse mount` first)");
                } else {
                    anyhow::bail!(
                        "fuse: multiple mounts registered; pass --mountpoint <path> \
                         (see `fuse list`)"
                    );
                }
            }
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    pub fn resolve_fs(&self, _mountpoint: Option<&Path>) -> anyhow::Result<()> {
        anyhow::bail!("sharecli-fuse is only supported on Linux and macOS")
    }

    /// Enumerate registered mounts and pending CoW paths.
    pub fn list(&self) -> Vec<FuseMountInfo> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let mounts = self.mounts.lock().expect("fuse registry lock");
            let mut out: Vec<FuseMountInfo> = mounts
                .iter()
                .map(|(mp, entry)| FuseMountInfo {
                    mountpoint: mp.clone(),
                    backing: entry.backing.clone(),
                    session_id: entry.session_id.clone(),
                    pending_relpaths: entry.fs.pending_rel_paths().unwrap_or_default(),
                })
                .collect();
            out.sort_by(|a, b| a.mountpoint.cmp(&b.mountpoint));
            out
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Vec::new()
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
trait MountContext {
    fn with_context_mount(self, mountpoint: &Path) -> anyhow::Result<()>;
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
impl MountContext for std::io::Result<()> {
    fn with_context_mount(self, mountpoint: &Path) -> anyhow::Result<()> {
        self.map_err(|e| anyhow::anyhow!("fuse mount: create mountpoint {}: {e}", mountpoint.display()))
    }
}
