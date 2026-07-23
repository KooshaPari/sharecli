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

#[cfg(target_os = "linux")]
use fuser::SessionACL;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use fuser::{BackgroundSession, Config, MountOption};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::platform::{InterceptFs, SharedInterceptFs};
use crate::InterceptFsOptions;

/// Default [`fuser`] mount [`Config`] for sharecli-fuse sessions.
///
/// On Linux, `AutoUnmount` is paired with [`SessionACL::RootAndOwner`] because
/// fuser rejects `AutoUnmount` when `acl == Owner` (`auto_unmount requires acl !=
/// Owner`). On macOS, skip `AutoUnmount` (and the implied `allow_other` helper
/// path): macFUSE often lacks `allow_other` unless the operator enables it, and
/// [`crate::mount_smoke::force_unmount`] / session Drop already call `umount`.
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn default_fuser_config() -> Config {
    let mut config = Config::default();
    config.mount_options = vec![MountOption::FSName("sharecli-fuse".to_string())];
    #[cfg(target_os = "linux")]
    {
        config.mount_options.push(MountOption::AutoUnmount);
        config.acl = SessionACL::RootAndOwner;
    }
    config
}

/// FUSE config for privileged mount smoke / ephemeral mounts.
///
/// Omits `AutoUnmount` (and the implied `allow_other`) so smoke works without
/// `user_allow_other` in `/etc/fuse.conf`. Callers MUST unmount via
/// [`crate::mount_smoke::force_unmount`] / Drop.
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn smoke_fuser_config() -> Config {
    let mut config = Config::default();
    config.mount_options = vec![MountOption::FSName("sharecli-fuse-smoke".to_string())];
    // RootAndOwner → allow_other (needed on Colima/Lima); no AutoUnmount (Drop unmounts).
    #[cfg(target_os = "linux")]
    {
        config.acl = SessionACL::RootAndOwner;
    }
    config
}

/// Mount flags for CLI / hypervisor (`--cow`, `--cow-dir`, …).
#[derive(Debug, Clone)]
pub struct FuseMountOptions {
    /// Write-provenance session id (default: process-local).
    pub session_id: Option<String>,
    /// Enable per-agent CoW overlays.
    pub cow: bool,
    /// CoW root (default under backing when unset).
    pub cow_dir: Option<PathBuf>,
    /// Default agent id for unscoped commit/discard.
    pub agent: Option<String>,
    /// When false, skip per-path write locks (Feb `--no-serialize`).
    pub serialize: bool,
    /// Path to Feb-format `agents.conf`.
    pub agents_conf: Option<PathBuf>,
}

impl Default for FuseMountOptions {
    fn default() -> Self {
        Self {
            session_id: None,
            cow: false,
            cow_dir: None,
            agent: None,
            serialize: true,
            agents_conf: None,
        }
    }
}

impl FuseMountOptions {
    /// Convert to [`InterceptFsOptions`].
    pub fn to_intercept_options(&self) -> InterceptFsOptions {
        InterceptFsOptions {
            session_id: self.session_id.clone().unwrap_or_default(),
            cow: self.cow,
            cow_dir: self.cow_dir.clone(),
            agent: self.agent.clone(),
            serialize: self.serialize,
            agents_conf: self.agents_conf.clone(),
        }
    }
}

/// Summary of one registered FUSE mount (operator / `fuse list`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuseMountInfo {
    /// Normalized mountpoint path.
    pub mountpoint: PathBuf,
    /// Backing filesystem root mirrored by the intercept layer.
    pub backing: PathBuf,
    /// Write-provenance session id stamped on creates/writes.
    pub session_id: String,
    /// Whether per-agent CoW is enabled.
    pub cow_enabled: bool,
    /// CoW overlay root.
    pub cow_root: PathBuf,
    /// Default agent id for unscoped ops.
    pub default_agent: String,
    /// Relative paths with pending CoW staging on the default agent.
    pub pending_relpaths: Vec<PathBuf>,
    /// Pending paths grouped by agent id.
    pub pending_by_agent: Vec<(String, Vec<PathBuf>)>,
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

/// WinFsp mount entry (AC-009.25/27) — passthrough + optional CoW via [`CowMountHandle`].
#[cfg(windows)]
struct WinfspMountEntry {
    handle: std::sync::Arc<crate::CowMountHandle>,
    _session: crate::winfsp_mount::WinfspMountSession,
}

/// Process-local table of active sharecli FUSE mounts.
#[derive(Default)]
pub struct FuseSessionRegistry {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    mounts: Mutex<HashMap<PathBuf, FuseMountEntry>>,
    #[cfg(windows)]
    winfsp_mounts: Mutex<HashMap<PathBuf, WinfspMountEntry>>,
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
        let mut opts = FuseMountOptions::default();
        opts.session_id = Some(session_id.to_string());
        opts.serialize = true;
        self.mount_background_with(mountpoint, backing, &opts)
    }

    /// Background mount with full Feb-parity options.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn mount_background_with(
        &self,
        mountpoint: &Path,
        backing: &Path,
        opts: &FuseMountOptions,
    ) -> anyhow::Result<()> {
        self.mount_inner(mountpoint, backing, opts, true)
    }

    /// Foreground mount (blocks until unmounted); registers while active.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn mount_foreground(
        &self,
        mountpoint: &Path,
        backing: &Path,
        session_id: &str,
    ) -> anyhow::Result<()> {
        let mut opts = FuseMountOptions::default();
        opts.session_id = Some(session_id.to_string());
        opts.serialize = true;
        self.mount_foreground_with(mountpoint, backing, &opts)
    }

    /// Foreground mount with full Feb-parity options.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn mount_foreground_with(
        &self,
        mountpoint: &Path,
        backing: &Path,
        opts: &FuseMountOptions,
    ) -> anyhow::Result<()> {
        self.mount_inner(mountpoint, backing, opts, false)
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn mount_inner(
        &self,
        mountpoint: &Path,
        backing: &Path,
        opts: &FuseMountOptions,
        background: bool,
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

        let intercept = opts.to_intercept_options();
        let fs = Arc::new(InterceptFs::with_options(backing, intercept));
        let session_id = fs.session_id().to_string();
        let config = default_fuser_config();

        if background {
            let session =
                fuser::spawn_mount2(SharedInterceptFs(Arc::clone(&fs)), mountpoint, &config)
                    .map_err(|e| anyhow::anyhow!("fuse mount: spawn_mount2 failed: {e}"))?;
            let mut mounts = self.mounts.lock().expect("fuse registry lock");
            mounts.insert(
                key,
                FuseMountEntry {
                    fs,
                    backing: backing.to_path_buf(),
                    session_id,
                    _session: Some(session),
                },
            );
            Ok(())
        } else {
            {
                let mut mounts = self.mounts.lock().expect("fuse registry lock");
                mounts.insert(
                    key.clone(),
                    FuseMountEntry {
                        fs: Arc::clone(&fs),
                        backing: backing.to_path_buf(),
                        session_id,
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
    }

    #[cfg(windows)]
    pub fn mount_background(
        &self,
        mountpoint: &Path,
        backing: &Path,
        session_id: &str,
    ) -> anyhow::Result<()> {
        let mut opts = FuseMountOptions::default();
        opts.session_id = Some(session_id.to_string());
        self.mount_background_with(mountpoint, backing, &opts)
    }

    #[cfg(windows)]
    pub fn mount_foreground(
        &self,
        mountpoint: &Path,
        backing: &Path,
        session_id: &str,
    ) -> anyhow::Result<()> {
        let mut opts = FuseMountOptions::default();
        opts.session_id = Some(session_id.to_string());
        self.mount_foreground_with(mountpoint, backing, &opts)
    }

    #[cfg(windows)]
    pub fn mount_background_with(
        &self,
        mountpoint: &Path,
        backing: &Path,
        opts: &FuseMountOptions,
    ) -> anyhow::Result<()> {
        self.mount_winfsp(mountpoint, backing, opts, true)
    }

    #[cfg(windows)]
    pub fn mount_foreground_with(
        &self,
        mountpoint: &Path,
        backing: &Path,
        opts: &FuseMountOptions,
    ) -> anyhow::Result<()> {
        self.mount_winfsp(mountpoint, backing, opts, false)
    }

    #[cfg(windows)]
    fn mount_winfsp(
        &self,
        mountpoint: &Path,
        backing: &Path,
        opts: &FuseMountOptions,
        background: bool,
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
            let mounts = self.winfsp_mounts.lock().expect("fuse registry lock");
            if mounts.contains_key(&key) {
                anyhow::bail!("fuse mount: already registered at {}", key.display());
            }
        }
        let intercept = opts.to_intercept_options();
        let handle = std::sync::Arc::new(crate::CowMountHandle::from_options(backing, &intercept));
        let session_id = handle.session_id().to_string();

        if background {
            let session = crate::winfsp_mount::WinfspMountSession::start(
                mountpoint,
                backing,
                &session_id,
                std::sync::Arc::clone(&handle),
            )?;
            let mut mounts = self.winfsp_mounts.lock().expect("fuse registry lock");
            mounts.insert(
                key,
                WinfspMountEntry {
                    handle,
                    _session: session,
                },
            );
            Ok(())
        } else {
            crate::winfsp_mount::mount_blocking(
                mountpoint,
                backing,
                &session_id,
                handle,
            )
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    pub fn mount_background(
        &self,
        mountpoint: &Path,
        backing: &Path,
        _session_id: &str,
    ) -> anyhow::Result<()> {
        let _ = (mountpoint, backing);
        anyhow::bail!("sharecli-fuse is only supported on Linux, macOS, and Windows (WinFsp)")
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    pub fn mount_foreground(
        &self,
        mountpoint: &Path,
        backing: &Path,
        _session_id: &str,
    ) -> anyhow::Result<()> {
        let _ = (mountpoint, backing);
        anyhow::bail!("sharecli-fuse is only supported on Linux, macOS, and Windows (WinFsp)")
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    pub fn mount_background_with(
        &self,
        mountpoint: &Path,
        backing: &Path,
        _opts: &FuseMountOptions,
    ) -> anyhow::Result<()> {
        let _ = (mountpoint, backing);
        anyhow::bail!("sharecli-fuse is only supported on Linux, macOS, and Windows (WinFsp)")
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    pub fn mount_foreground_with(
        &self,
        mountpoint: &Path,
        backing: &Path,
        _opts: &FuseMountOptions,
    ) -> anyhow::Result<()> {
        let _ = (mountpoint, backing);
        anyhow::bail!("sharecli-fuse is only supported on Linux, macOS, and Windows (WinFsp)")
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
        #[cfg(windows)]
        {
            let key = Self::normalize_key(mountpoint);
            let entry = {
                let mut mounts = self.winfsp_mounts.lock().expect("fuse registry lock");
                mounts.remove(&key)
            };
            if entry.is_none() {
                anyhow::bail!("fuse unmount: no registered mount at {}", mountpoint.display());
            }
            drop(entry);
            let _ = crate::mount_smoke::force_unmount(mountpoint);
            Ok(())
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
        {
            let _ = mountpoint;
            anyhow::bail!("sharecli-fuse is only supported on Linux, macOS, and Windows (WinFsp)")
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
                    anyhow::bail!(
                        "fuse: no active FUSE mounts registered (run `fuse mount` first)"
                    );
                } else {
                    anyhow::bail!(
                        "fuse: multiple mounts registered; pass --mountpoint <path> \
                         (see `fuse list`)"
                    );
                }
            }
        }
    }

    #[cfg(windows)]
    pub fn resolve_fs(
        &self,
        mountpoint: Option<&Path>,
    ) -> anyhow::Result<std::sync::Arc<crate::CowMountHandle>> {
        let mounts = self.winfsp_mounts.lock().expect("fuse registry lock");
        match mountpoint {
            Some(mp) => {
                let key = Self::normalize_key(mp);
                mounts
                    .get(&key)
                    .map(|e| std::sync::Arc::clone(&e.handle))
                    .ok_or_else(|| anyhow::anyhow!("fuse: no active mount at {}", mp.display()))
            }
            None => {
                if mounts.len() == 1 {
                    Ok(std::sync::Arc::clone(
                        &mounts.values().next().expect("one mount").handle,
                    ))
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

    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    pub fn resolve_fs(&self, _mountpoint: Option<&Path>) -> anyhow::Result<()> {
        anyhow::bail!("sharecli-fuse is only supported on Linux, macOS, and Windows (WinFsp)")
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
                    cow_enabled: entry.fs.cow_enabled(),
                    cow_root: entry.fs.cow_root().to_path_buf(),
                    default_agent: entry.fs.default_agent().to_string(),
                    pending_relpaths: entry.fs.pending_rel_paths().unwrap_or_default(),
                    pending_by_agent: entry.fs.pending_by_agent().unwrap_or_default(),
                })
                .collect();
            out.sort_by(|a, b| a.mountpoint.cmp(&b.mountpoint));
            out
        }
        #[cfg(windows)]
        {
            let mounts = self.winfsp_mounts.lock().expect("fuse registry lock");
            let mut out: Vec<FuseMountInfo> = mounts
                .iter()
                .map(|(mp, entry)| FuseMountInfo {
                    mountpoint: mp.clone(),
                    backing: entry.handle.backing().to_path_buf(),
                    session_id: entry.handle.session_id().to_string(),
                    cow_enabled: entry.handle.cow_enabled(),
                    cow_root: entry.handle.cow_root().to_path_buf(),
                    default_agent: entry.handle.default_agent().to_string(),
                    pending_relpaths: entry.handle.pending_rel_paths().unwrap_or_default(),
                    pending_by_agent: entry.handle.pending_by_agent().unwrap_or_default(),
                })
                .collect();
            out.sort_by(|a, b| a.mountpoint.cmp(&b.mountpoint));
            out
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
        {
            Vec::new()
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos", windows))]
trait MountContext {
    fn with_context_mount(self, mountpoint: &Path) -> anyhow::Result<()>;
}

#[cfg(any(target_os = "linux", target_os = "macos", windows))]
impl MountContext for std::io::Result<()> {
    fn with_context_mount(self, mountpoint: &Path) -> anyhow::Result<()> {
        self.map_err(|e| {
            anyhow::anyhow!("fuse mount: create mountpoint {}: {e}", mountpoint.display())
        })
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[cfg(test)]
mod default_mount_options_tests {
    use super::default_fuser_config;
    use fuser::{MountOption, SessionACL};

    #[test]
    fn default_config_sets_fsname() {
        let config = default_fuser_config();
        assert!(
            config
                .mount_options
                .iter()
                .any(|o| matches!(o, MountOption::FSName(name) if name == "sharecli-fuse")),
            "MUST set FSName=sharecli-fuse"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_auto_unmount_pairs_with_non_owner_acl() {
        let config = default_fuser_config();
        assert!(
            config.mount_options.iter().any(|o| matches!(o, MountOption::AutoUnmount)),
            "Linux MUST include AutoUnmount"
        );
        assert_eq!(
            config.acl,
            SessionACL::RootAndOwner,
            "Linux MUST prefer RootAndOwner over Owner for AutoUnmount"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_skips_auto_unmount_and_keeps_owner_acl() {
        let config = default_fuser_config();
        assert!(
            !config.mount_options.iter().any(|o| matches!(o, MountOption::AutoUnmount)),
            "macOS MUST NOT use AutoUnmount (allow_other / kext friction)"
        );
        assert_eq!(config.acl, SessionACL::Owner, "macOS MUST keep Owner ACL");
    }
}
