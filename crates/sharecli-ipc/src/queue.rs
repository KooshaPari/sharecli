//! N-slot priority queue for mutating / nocache command paths.
//!
//! Origin: Feb `core.sh` `harness::strategy::queue` — concurrency limiter with
//! priority levels. Higher priority (lower numeric rank) acquires slots first.
//!
//! Layout under `root/`:
//! ```text
//! <lane>.slot0.lock … <lane>.slot{N-1}.lock   — advisory flock sentinels
//! <lane>.waiting/<ticket>                    — waiters (priority order)
//! ```

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use fs2::FileExt;

/// Queue priority levels (Feb harness `HARNESS_PRIORITY_LEVELS`).
///
/// Lower discriminant = higher priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum QueuePriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Background = 4,
}

impl QueuePriority {
    /// Parse Feb-style priority names (`critical`, `high`, `normal`, `low`, `background`).
    pub fn parse(name: &str) -> Self {
        match name.trim().to_ascii_lowercase().as_str() {
            "critical" => Self::Critical,
            "high" => Self::High,
            "low" => Self::Low,
            "background" => Self::Background,
            _ => Self::Normal,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Filesystem-backed N-slot concurrency limiter for a named command lane.
///
/// Callers that must not coalesce (mutating / `nocache_args` paths) acquire a
/// slot via [`SlotQueue::with_slot`], run the work, then release.
pub struct SlotQueue {
    root: PathBuf,
    max_concurrent: usize,
    /// Max time to wait for a free slot before returning an error (loud fail).
    timeout: Duration,
    /// Poll interval while waiting for a slot.
    poll: Duration,
}

impl SlotQueue {
    /// Default max wait (30s) — matches Feb `HARNESS_LOCK_TIMEOUT`.
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
    /// Default poll interval (100ms) — matches Feb queue wait loop.
    pub const DEFAULT_POLL: Duration = Duration::from_millis(100);

    /// Create a queue rooted at `root` with `max_concurrent` parallel slots.
    pub fn new(root: impl Into<PathBuf>, max_concurrent: usize) -> Self {
        Self::with_options(
            root,
            max_concurrent.max(1),
            Self::DEFAULT_TIMEOUT,
            Self::DEFAULT_POLL,
        )
    }

    /// Create a queue with explicit timeout / poll settings (tests).
    pub fn with_options(
        root: impl Into<PathBuf>,
        max_concurrent: usize,
        timeout: Duration,
        poll: Duration,
    ) -> Self {
        Self {
            root: root.into(),
            max_concurrent: max_concurrent.max(1),
            timeout,
            poll,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    fn ensure_root(&self) -> Result<()> {
        fs::create_dir_all(&self.root)
            .with_context(|| format!("create queue root {}", self.root.display()))
    }

    fn slot_path(&self, lane: &str, slot: usize) -> PathBuf {
        self.root.join(format!("{lane}.slot{slot}.lock"))
    }

    fn waiting_dir(&self, lane: &str) -> PathBuf {
        self.root.join(format!("{lane}.waiting"))
    }

    /// Register a waiter ticket so higher-priority lanes can be preferred.
    fn enqueue_waiter(&self, lane: &str, priority: QueuePriority) -> Result<(PathBuf, String)> {
        let dir = self.waiting_dir(lane);
        fs::create_dir_all(&dir)
            .with_context(|| format!("create waiting dir {}", dir.display()))?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let ticket = format!("{}.{}", now, std::process::id());
        let path = dir.join(&ticket);
        fs::write(&path, format!("{}\n", priority.as_u8()))
            .with_context(|| format!("write waiter ticket {}", path.display()))?;
        Ok((path, ticket))
    }

    /// True when this ticket is among the highest-priority waiters.
    fn is_highest_priority(&self, lane: &str, my_priority: QueuePriority) -> Result<bool> {
        let dir = self.waiting_dir(lane);
        let entries = match fs::read_dir(&dir) {
            Ok(rd) => rd,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(true),
            Err(e) => return Err(e).with_context(|| format!("read waiting dir {}", dir.display())),
        };

        let mut best = my_priority.as_u8();
        for entry in entries {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let text = fs::read_to_string(entry.path()).unwrap_or_default();
            if let Ok(p) = text.trim().parse::<u8>() {
                if p < best {
                    best = p;
                }
            }
        }
        Ok(best >= my_priority.as_u8())
    }

    /// Acquire a free slot for `lane`, run `f`, then release the slot.
    ///
    /// Mutating / nocache paths MUST use this instead of [`super::CoalesceCache`].
    pub fn with_slot<T>(
        &self,
        lane: &str,
        priority: QueuePriority,
        f: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        self.ensure_root()?;
        let (ticket_path, _ticket) = self.enqueue_waiter(lane, priority)?;
        let deadline = Instant::now() + self.timeout;

        let result = (|| {
            loop {
                if Instant::now() >= deadline {
                    anyhow::bail!(
                        "queue timeout: no free slot for lane `{lane}` within {}s \
                         (max_concurrent={})",
                        self.timeout.as_secs(),
                        self.max_concurrent
                    );
                }

                // Prefer higher-priority waiters (Feb yield-to-higher-priority).
                if !self.is_highest_priority(lane, priority)? {
                    thread::sleep(self.poll);
                    continue;
                }

                for slot in 0..self.max_concurrent {
                    let lock_path = self.slot_path(lane, slot);
                    let lock_file = fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(false)
                        .open(&lock_path)
                        .with_context(|| format!("open slot lock {}", lock_path.display()))?;

                    match lock_file.try_lock_exclusive() {
                        Ok(()) => {
                            // Drop waiter before running so peers can reorder.
                            let _ = fs::remove_file(&ticket_path);
                            let value = f()?;
                            // Lock releases on drop of lock_file.
                            drop(lock_file);
                            return Ok(value);
                        }
                        Err(_) => continue,
                    }
                }

                thread::sleep(self.poll);
            }
        })();

        let _ = fs::remove_file(&ticket_path);
        result
    }
}

/// Alias documenting Feb `priority_queue` strategy (same as [`SlotQueue`]).
pub type PriorityQueue = SlotQueue;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn priority_parse() {
        assert_eq!(QueuePriority::parse("critical"), QueuePriority::Critical);
        assert_eq!(QueuePriority::parse("HIGH"), QueuePriority::High);
        assert_eq!(QueuePriority::parse("unknown"), QueuePriority::Normal);
    }

    #[test]
    fn with_slot_serializes_max_one() {
        let dir = TempDir::new().unwrap();
        let active = Arc::new(AtomicU32::new(0));
        let peak = Arc::new(AtomicU32::new(0));

        let mut handles = vec![];
        for _ in 0..4 {
            let q_root = dir.path().to_path_buf();
            let active = Arc::clone(&active);
            let peak = Arc::clone(&peak);
            handles.push(thread::spawn(move || {
                let q = SlotQueue::with_options(
                    q_root,
                    1,
                    Duration::from_secs(5),
                    Duration::from_millis(20),
                );
                q.with_slot("ruff", QueuePriority::Normal, || {
                    let n = active.fetch_add(1, Ordering::SeqCst) + 1;
                    peak.fetch_max(n, Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(40));
                    active.fetch_sub(1, Ordering::SeqCst);
                    Ok(())
                })
                .unwrap();
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(peak.load(Ordering::SeqCst), 1, "max_concurrent=1 MUST serialize");
    }

    #[test]
    fn with_slot_allows_two() {
        let dir = TempDir::new().unwrap();
        let peak = Arc::new(AtomicU32::new(0));
        let active = Arc::new(AtomicU32::new(0));
        let mut handles = vec![];
        for _ in 0..4 {
            let q_root = dir.path().to_path_buf();
            let active = Arc::clone(&active);
            let peak = Arc::clone(&peak);
            handles.push(thread::spawn(move || {
                let q = SlotQueue::with_options(
                    q_root,
                    2,
                    Duration::from_secs(5),
                    Duration::from_millis(10),
                );
                q.with_slot("pytest", QueuePriority::Low, || {
                    let n = active.fetch_add(1, Ordering::SeqCst) + 1;
                    peak.fetch_max(n, Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(50));
                    active.fetch_sub(1, Ordering::SeqCst);
                    Ok(())
                })
                .unwrap();
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert!(
            peak.load(Ordering::SeqCst) >= 2,
            "max_concurrent=2 MUST allow parallel slots"
        );
        assert!(peak.load(Ordering::SeqCst) <= 2);
    }
}
