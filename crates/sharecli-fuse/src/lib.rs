//! sharecli-fuse — IO-interception tier of the sharecli hypervisor.
//!
//! This crate implements a FUSE filesystem that sits between agent processes and
//! the real backing filesystem.  By intercepting VFS calls at the FUSE layer we
//! can:
//!
//! * Coalesce redundant reads across concurrent agent sessions.
//! * Cache hot paths (Cargo registry, node_modules, build artefacts) in RAM or
//!   on a fast local device, routing cold misses to the backing path.
//! * Remember negative dentries (ENOENT) with TTL so repeated missing-path
//!   lookups skip backing stats until create/mkdir/rename invalidates them.
//! * Meter and throttle per-process IO to prevent one agent's build from starving
//!   another's.
//! * Record provenance — every write carries a (session-id, timestamp) annotation
//!   in the extended-attribute namespace — without modifying the backing FS.
//!
//! The implementation uses [`fuser`] which provides cross-platform FUSE bindings:
//! Linux via libfuse3 and macOS via macFUSE.  The mount entry point is gated
//! behind `#[cfg(any(target_os = "linux", target_os = "macos"))]`; all other
//! targets compile to a stub that returns an unsupported-platform error.
//!
//! Ported/inspired by the harness-fuse Rust prototype; rewritten on top of
//! `fuser` (vs the earlier Linux-only libfuse ELF binary) to keep the codebase
//! portable and maintainable in pure Rust.

#![warn(missing_docs)]

mod inode_map;
mod neg_dentry;
#[cfg(any(target_os = "linux", target_os = "macos"))]
mod mount_smoke;
#[cfg(any(target_os = "linux", target_os = "macos"))]
mod provenance;
mod read_cache;
mod write_serialize;

pub use inode_map::{abs_under, join_rel, InodeMap, ROOT_INO};
pub use neg_dentry::{NegDentryMeters, NegativeDentryCache, DEFAULT_NEG_TTL};
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub use mount_smoke::{
    force_unmount, fuse_mount_smoke_enabled, run_mount_smoke, verify_mount_smoke_provenance,
    MountSession, ENV_FUSE_MOUNT_SMOKE,
};
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub use provenance::{
    annotate_write, annotate_write_at, default_session_id, read_provenance, WriteProvenance,
    ATTR_SESSION, ATTR_WRITTEN_AT,
};
pub use read_cache::{global_read_cache_meters, ReadCacheMeters, ReadContentCache};
pub use write_serialize::{WriteSerialize, WriteSerializeError};

use std::path::Path;

// ---------------------------------------------------------------------------
// Platform-gated implementation
// ---------------------------------------------------------------------------

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod platform {
    use std::{
        ffi::OsStr,
        fs::{self, OpenOptions},
        io::{Seek, SeekFrom, Write as IoWrite},
        os::unix::fs::{MetadataExt, PermissionsExt},
        path::{Path, PathBuf},
        sync::Mutex,
        time::{Duration, SystemTime},
    };

    use fuser::{
        Config, Errno, FileAttr, FileHandle, FileType, Filesystem, FopenFlags, Generation, INodeNo,
        MountOption, OpenFlags, RenameFlags, ReplyAttr, ReplyData, ReplyDirectory, ReplyEmpty,
        ReplyEntry, ReplyOpen, ReplyWrite, Request, WriteFlags,
    };
    use tracing::{debug, trace};

    use crate::inode_map::{abs_under, InodeMap, ROOT_INO};
    use crate::neg_dentry::{NegDentryMeters, NegativeDentryCache, DEFAULT_NEG_TTL};
    use crate::provenance::{annotate_write, default_session_id};
    use crate::read_cache::{ReadCacheMeters, ReadContentCache};
    use crate::write_serialize::WriteSerialize;

    const TTL: Duration = Duration::from_secs(1);

    /// Passthrough FUSE filesystem with read coalesce + write-serialize hooks.
    pub struct InterceptFs {
        /// Root of the real filesystem subtree being mirrored.
        backing: PathBuf,
        /// Session id stamped onto every write / CoW commit (`user.sharecli.session`).
        session_id: String,
        inodes: Mutex<InodeMap>,
        read_cache: Mutex<ReadContentCache>,
        neg_dentry: Mutex<NegativeDentryCache>,
        write_locks: WriteSerialize,
    }

    impl InterceptFs {
        /// Create a new [`InterceptFs`] rooted at `backing` with a default session id.
        pub fn new(backing: &Path) -> Self {
            Self::with_session(backing, default_session_id())
        }

        /// Create a new [`InterceptFs`] rooted at `backing` with an explicit session id.
        pub fn with_session(backing: &Path, session_id: impl Into<String>) -> Self {
            let staging = backing.join(".sharecli-cow-staging");
            Self {
                backing: backing.to_path_buf(),
                session_id: session_id.into(),
                inodes: Mutex::new(InodeMap::new()),
                read_cache: Mutex::new(ReadContentCache::new()),
                neg_dentry: Mutex::new(NegativeDentryCache::with_ttl(DEFAULT_NEG_TTL)),
                write_locks: WriteSerialize::with_staging_root(staging),
            }
        }

        /// Backing root path.
        pub fn backing(&self) -> &Path {
            &self.backing
        }

        /// Session id used for write provenance xattrs.
        pub fn session_id(&self) -> &str {
            &self.session_id
        }

        /// Read-coalesce meters (hits / misses) without mounting.
        pub fn cache_meters(&self) -> ReadCacheMeters {
            self.read_cache
                .lock()
                .expect("read cache lock")
                .meters()
        }

        /// Negative-dentry meters (hits / misses) without mounting.
        pub fn neg_dentry_meters(&self) -> NegDentryMeters {
            self.neg_dentry
                .lock()
                .expect("neg dentry lock")
                .meters()
        }

        /// Probe whether relative `rel` exists under the backing root.
        ///
        /// Uses the negative dentry cache: a prior ENOENT within TTL returns
        /// `Ok(false)` without re-statting. A positive result invalidates any
        /// stale negative entry for `rel`.
        pub fn exists_rel(&self, rel: &Path) -> std::io::Result<bool> {
            {
                let mut neg = self.neg_dentry.lock().expect("neg dentry lock");
                if neg.is_negative(rel) {
                    return Ok(false);
                }
            }
            let abs = abs_under(&self.backing, rel);
            match fs::metadata(&abs) {
                Ok(_) => {
                    if let Ok(mut neg) = self.neg_dentry.lock() {
                        neg.invalidate(rel);
                    }
                    Ok(true)
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    if let Ok(mut neg) = self.neg_dentry.lock() {
                        neg.remember_miss(rel.to_path_buf());
                    }
                    Ok(false)
                }
                Err(err) => Err(err),
            }
        }

        /// Drop a negative-dentry entry for `rel` (create / mkdir / rename-into).
        pub fn invalidate_neg_rel(&self, rel: &Path) {
            if let Ok(mut neg) = self.neg_dentry.lock() {
                neg.invalidate(rel);
            }
        }

        /// Read a relative path through the in-process coalesce cache (no mount).
        pub fn read_coalesced_rel(&self, rel: &Path) -> std::io::Result<Vec<u8>> {
            let abs = abs_under(&self.backing, rel);
            self.read_cache
                .lock()
                .expect("read cache lock")
                .read_coalesced(&abs)
        }

        /// Stage CoW bytes for a relative path (no mount; FR-009 helpers).
        pub fn stage_rel(
            &self,
            rel: &Path,
            contents: &[u8],
        ) -> Result<(), crate::WriteSerializeError> {
            let abs = abs_under(&self.backing, rel);
            self.write_locks.stage_bytes(&abs, contents)
        }

        /// Commit pending CoW staging for a relative path; invalidates read cache
        /// and stamps write provenance xattrs on the promoted backing file.
        pub fn commit_rel(&self, rel: &Path) -> Result<(), crate::WriteSerializeError> {
            let abs = abs_under(&self.backing, rel);
            self.write_locks.commit_pending(&abs)?;
            annotate_write(&abs, &self.session_id).map_err(crate::WriteSerializeError::Io)?;
            if let Ok(mut cache) = self.read_cache.lock() {
                cache.invalidate(&abs);
            }
            Ok(())
        }

        /// Discard pending CoW staging for a relative path (backing unchanged).
        pub fn discard_rel(&self, rel: &Path) -> Result<(), crate::WriteSerializeError> {
            let abs = abs_under(&self.backing, rel);
            self.write_locks.discard_pending(&abs)
        }

        /// Passthrough write at `offset` for relative `rel`, serialized per path.
        ///
        /// On success, stamps [`crate::ATTR_SESSION`] / [`crate::ATTR_WRITTEN_AT`]
        /// on the backing file (write provenance).
        pub fn write_rel(&self, rel: &Path, offset: u64, data: &[u8]) -> std::io::Result<u32> {
            let abs = abs_under(&self.backing, rel);
            let data = data.to_vec();
            let session = self.session_id.clone();
            self.write_locks
                .with_locked_path(&abs, || {
                    let mut file = OpenOptions::new().write(true).open(&abs)?;
                    file.seek(SeekFrom::Start(offset))?;
                    file.write_all(&data)?;
                    annotate_write(&abs, &session)?;
                    Ok::<u32, std::io::Error>(data.len() as u32)
                })
                .map_err(|e| std::io::Error::other(e.to_string()))?
                .and_then(|n| {
                    if let Ok(mut cache) = self.read_cache.lock() {
                        cache.invalidate(&abs);
                    }
                    Ok(n)
                })
        }

        fn io_errno(err: std::io::Error) -> Errno {
            Errno::from(err)
        }

        fn meta_kind(meta: &fs::Metadata) -> FileType {
            if meta.is_dir() {
                FileType::Directory
            } else if meta.is_symlink() {
                FileType::Symlink
            } else {
                FileType::RegularFile
            }
        }

        fn metadata_to_attr(ino: u64, meta: &fs::Metadata) -> FileAttr {
            let kind = Self::meta_kind(meta);
            let now = SystemTime::now();
            FileAttr {
                ino: INodeNo(ino),
                size: meta.len(),
                blocks: meta.blocks(),
                atime: meta.accessed().unwrap_or(now),
                mtime: meta.modified().unwrap_or(now),
                ctime: now,
                crtime: now,
                kind,
                perm: meta.mode() as u16,
                nlink: meta.nlink() as u32,
                uid: meta.uid(),
                gid: meta.gid(),
                rdev: meta.rdev() as u32,
                blksize: 512,
                flags: 0,
            }
        }
    }

    impl Filesystem for InterceptFs {
        fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {
            trace!(?parent, ?name, "lookup");
            let mut map = self.inodes.lock().expect("inode map");
            let Some(rel) = map.child_rel(parent.0, name) else {
                reply.error(Errno::ENOENT);
                return;
            };
            {
                let mut neg = self.neg_dentry.lock().expect("neg dentry lock");
                if neg.is_negative(&rel) {
                    reply.error(Errno::ENOENT);
                    return;
                }
            }
            let Some((ino, rel)) = map.lookup_or_alloc(parent.0, name) else {
                reply.error(Errno::ENOENT);
                return;
            };
            let path = abs_under(&self.backing, &rel);
            match fs::metadata(&path) {
                Ok(meta) => {
                    if let Ok(mut neg) = self.neg_dentry.lock() {
                        neg.invalidate(&rel);
                    }
                    let attr = Self::metadata_to_attr(ino, &meta);
                    reply.entry(&TTL, &attr, Generation(0));
                }
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::NotFound {
                        if let Ok(mut neg) = self.neg_dentry.lock() {
                            neg.remember_miss(rel.clone());
                        }
                    }
                    map.remove_rel(&rel);
                    reply.error(Self::io_errno(err));
                }
            }
        }

        fn getattr(&self, _req: &Request, ino: INodeNo, _fh: Option<FileHandle>, reply: ReplyAttr) {
            trace!(?ino, "getattr");
            let map = self.inodes.lock().expect("inode map");
            let Some(path) = map.abs_path(&self.backing, ino.0) else {
                reply.error(Errno::ENOENT);
                return;
            };
            match fs::metadata(&path) {
                Ok(meta) => reply.attr(&TTL, &Self::metadata_to_attr(ino.0, &meta)),
                Err(err) => reply.error(Self::io_errno(err)),
            }
        }

        fn open(&self, _req: &Request, ino: INodeNo, _flags: OpenFlags, reply: ReplyOpen) {
            let map = self.inodes.lock().expect("inode map");
            let Some(path) = map.abs_path(&self.backing, ino.0) else {
                reply.error(Errno::ENOENT);
                return;
            };
            match fs::metadata(&path) {
                Ok(meta) if meta.is_file() => {
                    reply.opened(FileHandle(ino.0), FopenFlags::empty());
                }
                Ok(_) => reply.error(Errno::EISDIR),
                Err(err) => reply.error(Self::io_errno(err)),
            }
        }

        fn read(
            &self,
            _req: &Request,
            ino: INodeNo,
            _fh: FileHandle,
            offset: u64,
            size: u32,
            _flags: OpenFlags,
            _lock: Option<fuser::LockOwner>,
            reply: ReplyData,
        ) {
            debug!(?ino, offset, size, "read");
            let path = {
                let map = self.inodes.lock().expect("inode map");
                match map.abs_path(&self.backing, ino.0) {
                    Some(p) => p,
                    None => {
                        reply.error(Errno::ENOENT);
                        return;
                    }
                }
            };
            match self
                .read_cache
                .lock()
                .expect("read cache")
                .read_slice(&path, offset, size)
            {
                Ok(buf) => reply.data(&buf),
                Err(err) => reply.error(Self::io_errno(err)),
            }
        }

        fn write(
            &self,
            _req: &Request,
            ino: INodeNo,
            _fh: FileHandle,
            offset: u64,
            data: &[u8],
            _write_flags: WriteFlags,
            _flags: OpenFlags,
            _lock: Option<fuser::LockOwner>,
            reply: ReplyWrite,
        ) {
            debug!(?ino, offset, len = data.len(), "write");
            let path = {
                let map = self.inodes.lock().expect("inode map");
                match map.abs_path(&self.backing, ino.0) {
                    Some(p) => p,
                    None => {
                        reply.error(Errno::ENOENT);
                        return;
                    }
                }
            };
            let payload = data.to_vec();
            let session = self.session_id.clone();
            let result = self.write_locks.with_locked_path(&path, || {
                let mut file = OpenOptions::new().write(true).open(&path)?;
                file.seek(SeekFrom::Start(offset))?;
                file.write_all(&payload)?;
                annotate_write(&path, &session)?;
                Ok::<u32, std::io::Error>(payload.len() as u32)
            });
            match result {
                Ok(Ok(n)) => {
                    if let Ok(mut cache) = self.read_cache.lock() {
                        cache.invalidate(&path);
                    }
                    reply.written(n);
                }
                Ok(Err(err)) => reply.error(Self::io_errno(err)),
                Err(err) => reply.error(Errno::from(std::io::Error::other(err.to_string()))),
            }
        }

        fn readdir(
            &self,
            _req: &Request,
            ino: INodeNo,
            _fh: FileHandle,
            offset: u64,
            mut reply: ReplyDirectory,
        ) {
            debug!(?ino, offset, "readdir");
            let entries = {
                let mut map = self.inodes.lock().expect("inode map");
                let Some(rel) = map.resolve(ino.0).map(Path::to_path_buf) else {
                    reply.error(Errno::ENOENT);
                    return;
                };
                let parent_ino = if ino.0 == ROOT_INO {
                    ROOT_INO
                } else if rel.components().count() <= 1 {
                    map.alloc_or_get(PathBuf::new())
                } else if let Some(parent_rel) = rel.parent() {
                    map.alloc_or_get(parent_rel.to_path_buf())
                } else {
                    ROOT_INO
                };
                let dir_path = abs_under(&self.backing, &rel);
                let read_dir = match fs::read_dir(&dir_path) {
                    Ok(rd) => rd,
                    Err(err) => {
                        reply.error(Self::io_errno(err));
                        return;
                    }
                };
                let mut entries: Vec<(u64, FileType, std::ffi::OsString)> = Vec::new();
                entries.push((ino.0, FileType::Directory, std::ffi::OsString::from(".")));
                entries.push((
                    parent_ino,
                    FileType::Directory,
                    std::ffi::OsString::from(".."),
                ));
                for ent in read_dir.flatten() {
                    let name = ent.file_name();
                    let child_rel = crate::inode_map::join_rel(&rel, &name);
                    let child_ino = map.alloc_or_get(child_rel);
                    let kind = ent
                        .metadata()
                        .map(|m| Self::meta_kind(&m))
                        .unwrap_or(FileType::RegularFile);
                    entries.push((child_ino, kind, name));
                }
                entries
            };
            for (i, (child_ino, kind, name)) in
                entries.into_iter().enumerate().skip(offset as usize)
            {
                let next_offset = (i + 1) as u64;
                if reply.add(INodeNo(child_ino), next_offset, kind, &name) {
                    break;
                }
            }
            reply.ok();
        }

        fn mkdir(
            &self,
            _req: &Request,
            parent: INodeNo,
            name: &OsStr,
            mode: u32,
            _umask: u32,
            reply: ReplyEntry,
        ) {
            let mut map = self.inodes.lock().expect("inode map");
            let Some(rel) = map.child_rel(parent.0, name) else {
                reply.error(Errno::ENOENT);
                return;
            };
            let path = abs_under(&self.backing, &rel);
            match fs::create_dir(&path) {
                Ok(()) => {
                    let _ = fs::set_permissions(&path, fs::Permissions::from_mode(mode));
                    if let Ok(mut neg) = self.neg_dentry.lock() {
                        neg.invalidate(&rel);
                    }
                    let ino = map.alloc_or_get(rel);
                    match fs::metadata(&path) {
                        Ok(meta) => {
                            reply.entry(&TTL, &Self::metadata_to_attr(ino, &meta), Generation(0))
                        }
                        Err(err) => reply.error(Self::io_errno(err)),
                    }
                }
                Err(err) => reply.error(Self::io_errno(err)),
            }
        }

        fn unlink(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
            let mut map = self.inodes.lock().expect("inode map");
            let Some(rel) = map.child_rel(parent.0, name) else {
                reply.error(Errno::ENOENT);
                return;
            };
            let path = abs_under(&self.backing, &rel);
            match fs::remove_file(&path) {
                Ok(()) => {
                    if let Ok(mut cache) = self.read_cache.lock() {
                        cache.invalidate(&path);
                    }
                    if let Ok(mut neg) = self.neg_dentry.lock() {
                        neg.remember_miss(rel.clone());
                    }
                    map.remove_rel(&rel);
                    reply.ok();
                }
                Err(err) => reply.error(Self::io_errno(err)),
            }
        }

        fn rmdir(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
            let mut map = self.inodes.lock().expect("inode map");
            let Some(rel) = map.child_rel(parent.0, name) else {
                reply.error(Errno::ENOENT);
                return;
            };
            let path = abs_under(&self.backing, &rel);
            match fs::remove_dir(&path) {
                Ok(()) => {
                    if let Ok(mut neg) = self.neg_dentry.lock() {
                        neg.remember_miss(rel.clone());
                    }
                    map.remove_rel(&rel);
                    reply.ok();
                }
                Err(err) => reply.error(Self::io_errno(err)),
            }
        }

        fn rename(
            &self,
            _req: &Request,
            parent: INodeNo,
            name: &OsStr,
            newparent: INodeNo,
            newname: &OsStr,
            _flags: RenameFlags,
            reply: ReplyEmpty,
        ) {
            let mut map = self.inodes.lock().expect("inode map");
            let Some(old_rel) = map.child_rel(parent.0, name) else {
                reply.error(Errno::ENOENT);
                return;
            };
            let Some(new_rel) = map.child_rel(newparent.0, newname) else {
                reply.error(Errno::ENOENT);
                return;
            };
            let old_path = abs_under(&self.backing, &old_rel);
            let new_path = abs_under(&self.backing, &new_rel);
            match fs::rename(&old_path, &new_path) {
                Ok(()) => {
                    if let Ok(mut cache) = self.read_cache.lock() {
                        cache.invalidate(&old_path);
                        cache.invalidate(&new_path);
                    }
                    if let Ok(mut neg) = self.neg_dentry.lock() {
                        neg.remember_miss(old_rel.clone());
                        neg.invalidate(&new_rel);
                    }
                    map.rename_rel(&old_rel, new_rel);
                    reply.ok();
                }
                Err(err) => reply.error(Self::io_errno(err)),
            }
        }
    }

    /// Mount the [`InterceptFs`] at `mountpoint`, mirroring `backing`.
    ///
    /// Blocks until the filesystem is unmounted.  Callers should run this on a
    /// dedicated thread or inside a `tokio::task::spawn_blocking` block.
    pub fn mount(mountpoint: &Path, backing: &Path) -> anyhow::Result<()> {
        let fs = InterceptFs::new(backing);
        let mut config = Config::default();
        // RW mount — writes are passthrough + path-serialized; CoW via WriteSerialize.
        config.mount_options = vec![
            MountOption::FSName("sharecli-fuse".to_string()),
            MountOption::AutoUnmount,
        ];
        fuser::mount2(fs, mountpoint, &config)?;
        Ok(())
    }
}

/// Passthrough FUSE filesystem; attach point for sharecli hypervisor hooks.
///
/// See the crate-level documentation for the full IO-interception design.
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub use platform::InterceptFs;

/// Mount the sharecli FUSE layer at `mountpoint` over `backing`.
///
/// On Linux and macOS this calls `fuser::mount2`; on other platforms it returns
/// an [`anyhow::Error`] indicating the platform is not supported.
pub fn mount(mountpoint: &Path, backing: &Path) -> anyhow::Result<()> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        platform::mount(mountpoint, backing)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (mountpoint, backing);
        anyhow::bail!("sharecli-fuse is only supported on Linux and macOS")
    }
}
