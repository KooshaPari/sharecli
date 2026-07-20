//! In-process read content cache keyed by path + mtime (FR-009 coalesce).

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::SystemTime,
};

/// Snapshot of read coalesce meter counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReadCacheMeters {
    /// Successful cache hits (served without re-reading backing bytes into a new buffer).
    pub hits: u64,
    /// Misses that loaded content from the backing path.
    pub misses: u64,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    mtime: SystemTime,
    data: Vec<u8>,
}

/// In-process whole-file content cache keyed by absolute/relative path + mtime.
///
/// A hit requires an exact mtime match; any write-side invalidation or mtime
/// change forces a miss and reload.
#[derive(Debug, Default)]
pub struct ReadContentCache {
    entries: HashMap<PathBuf, CacheEntry>,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl ReadContentCache {
    /// Empty cache with zeroed meters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Current hit/miss counters.
    pub fn meters(&self) -> ReadCacheMeters {
        ReadCacheMeters {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
        }
    }

    /// Look up cached bytes for `path` when `mtime` matches the stored entry.
    pub fn get(&mut self, path: &Path, mtime: SystemTime) -> Option<Vec<u8>> {
        match self.entries.get(path) {
            Some(entry) if entry.mtime == mtime => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Some(entry.data.clone())
            }
            Some(_) => {
                // Stale mtime — drop and count as miss on next put path.
                self.entries.remove(path);
                None
            }
            None => None,
        }
    }

    /// Store (or replace) content for `path` at `mtime` and count a miss.
    pub fn put_miss(&mut self, path: PathBuf, mtime: SystemTime, data: Vec<u8>) {
        self.misses.fetch_add(1, Ordering::Relaxed);
        self.entries.insert(path, CacheEntry { mtime, data });
    }

    /// Drop a path from the cache (e.g. after write / unlink / rename).
    pub fn invalidate(&mut self, path: &Path) {
        self.entries.remove(path);
    }

    /// Read `path` through the cache: hit returns cloned bytes; miss loads from disk.
    pub fn read_coalesced(&mut self, path: &Path) -> std::io::Result<Vec<u8>> {
        let meta = std::fs::metadata(path)?;
        let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        if let Some(data) = self.get(path, mtime) {
            return Ok(data);
        }
        let data = std::fs::read(path)?;
        self.put_miss(path.to_path_buf(), mtime, data.clone());
        Ok(data)
    }

    /// Slice a coalesced read at `offset`/`size` (FUSE read semantics).
    pub fn read_slice(&mut self, path: &Path, offset: u64, size: u32) -> std::io::Result<Vec<u8>> {
        let data = self.read_coalesced(path)?;
        let start = offset as usize;
        if start >= data.len() {
            return Ok(Vec::new());
        }
        let end = (start + size as usize).min(data.len());
        Ok(data[start..end].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// FR-009 / AC-009.4 — first read misses; second identical mtime hits.
    #[test]
    fn read_cache_miss_then_hit() {
        let mut tmp = NamedTempFile::new().expect("tmp");
        write!(tmp, "hello-coalesce").expect("write");
        tmp.flush().expect("flush");
        let path = tmp.path().to_path_buf();

        let mut cache = ReadContentCache::new();
        let a = cache.read_coalesced(&path).expect("miss read");
        assert_eq!(a, b"hello-coalesce");
        let m1 = cache.meters();
        assert_eq!(m1.misses, 1);
        assert_eq!(m1.hits, 0);

        let b = cache.read_coalesced(&path).expect("hit read");
        assert_eq!(b, a);
        let m2 = cache.meters();
        assert_eq!(m2.misses, 1);
        assert_eq!(m2.hits, 1);
    }

    /// FR-009 / AC-009.4 — invalidate forces a subsequent miss.
    #[test]
    fn read_cache_invalidate_forces_miss() {
        let mut tmp = NamedTempFile::new().expect("tmp");
        write!(tmp, "v1").expect("write");
        tmp.flush().expect("flush");
        let path = tmp.path().to_path_buf();

        let mut cache = ReadContentCache::new();
        let _ = cache.read_coalesced(&path).expect("first");
        cache.invalidate(&path);
        let _ = cache.read_coalesced(&path).expect("second");
        let m = cache.meters();
        assert_eq!(m.misses, 2);
        assert_eq!(m.hits, 0);
    }
}
