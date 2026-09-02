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
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use fs2::FileExt;
use sharecli_fleet::{record_slot_acquire, record_slot_timeout, record_slot_wait};

/// Monotonic suffix so concurrent waiters in the same process get distinct tickets.
static WAITER_TICKET_SEQ: AtomicU64 = AtomicU64::new(0);

/// Queue priority levels (Feb harness `HARNESS_PRIORITY_LEVELS`).
///
/// Lower discriminant = higher priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(u8)]
pub enum QueuePriority {
    Critical = 0,
    High = 1,
    #[default]
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

/// Operator env override for Hypervisor nocache [`SlotQueue`] priority (FR-008 AC-008.15).
pub const QUEUE_PRIORITY_ENV: &str = "SHARECLI_QUEUE_PRIORITY";

/// Resolve queue priority from the operator surface.
///
/// Precedence: non-empty [`QUEUE_PRIORITY_ENV`] → optional rules.conf `priority=`
/// → [`QueuePriority::Normal`].
pub fn resolve_operator_queue_priority(rule_priority: Option<&str>) -> QueuePriority {
    if let Ok(raw) = std::env::var(QUEUE_PRIORITY_ENV) {
        if !raw.trim().is_empty() {
            return QueuePriority::parse(&raw);
        }
    }
    rule_priority.map(QueuePriority::parse).unwrap_or(QueuePriority::Normal)
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
        Self::with_options(root, max_concurrent.max(1), Self::DEFAULT_TIMEOUT, Self::DEFAULT_POLL)
    }

    /// Create a queue with explicit timeout / poll settings (tests).
    pub fn with_options(
        root: impl Into<PathBuf>,
        max_concurrent: usize,
        timeout: Duration,
        poll: Duration,
    ) -> Self {
        Self { root: root.into(), max_concurrent: max_concurrent.max(1), timeout, poll }
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
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let seq = WAITER_TICKET_SEQ.fetch_add(1, Ordering::Relaxed);
        // Priority prefix avoids read races when scanning the waiting dir (AC-008.14).
        let ticket = format!("{:02}.{}.{}.{}", priority.as_u8(), now, std::process::id(), seq);
        let path = dir.join(&ticket);
        let tmp = dir.join(format!(".{ticket}.tmp"));
        fs::write(&tmp, format!("{}\n", priority.as_u8()))
            .with_context(|| format!("write waiter ticket {}", tmp.display()))?;
        fs::rename(&tmp, &path)
            .with_context(|| format!("publish waiter ticket {}", path.display()))?;
        Ok((path, ticket))
    }

    fn ticket_priority(ticket: &str) -> u8 {
        ticket
            .split('.')
            .next()
            .and_then(|head| head.parse::<u8>().ok())
            .unwrap_or(QueuePriority::Normal.as_u8())
    }

    /// True when this ticket is next among equal-or-highest-priority waiters (FIFO by ticket name).
    fn is_my_turn(&self, lane: &str, my_priority: QueuePriority, my_ticket: &str) -> Result<bool> {
        let dir = self.waiting_dir(lane);
        let entries = match fs::read_dir(&dir) {
            Ok(rd) => rd,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(true),
            Err(e) => return Err(e).with_context(|| format!("read waiting dir {}", dir.display())),
        };

        let my_rank = my_priority.as_u8();
        let mut best_rank = u8::MAX;
        let mut tickets_at_best: Vec<String> = Vec::new();

        for entry in entries {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let ticket = entry.file_name().to_string_lossy().into_owned();
            if ticket.starts_with('.') {
                continue;
            }
            let rank = Self::ticket_priority(&ticket);

            if rank < best_rank {
                best_rank = rank;
                tickets_at_best.clear();
                tickets_at_best.push(ticket);
            } else if rank == best_rank {
                tickets_at_best.push(ticket);
            }
        }

        if best_rank == u8::MAX {
            return Ok(true);
        }
        if my_rank != best_rank {
            return Ok(false);
        }

        let winner = tickets_at_best.iter().min().map(String::as_str).unwrap_or(my_ticket);
        Ok(winner == my_ticket)
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
        let (ticket_path, ticket) = self.enqueue_waiter(lane, priority)?;
        let deadline = Instant::now() + self.timeout;

        let result = (|| {
            loop {
                if Instant::now() >= deadline {
                    record_slot_timeout();
                    anyhow::bail!(
                        "queue timeout: no free slot for lane `{lane}` within {}s \
                         (max_concurrent={})",
                        self.timeout.as_secs(),
                        self.max_concurrent
                    );
                }

                // Prefer higher-priority waiters; FIFO by ticket among ties (Feb yield-to-higher-priority).
                if !self.is_my_turn(lane, priority, &ticket)? {
                    record_slot_wait();
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
                            if !self.is_my_turn(lane, priority, &ticket)? {
                                drop(lock_file);
                                record_slot_wait();
                                thread::sleep(self.poll);
                                continue;
                            }
                            // Drop waiter before running so peers can reorder.
                            let _ = fs::remove_file(&ticket_path);
                            record_slot_acquire();
                            let value = f()?;
                            // Lock releases on drop of lock_file.
                            drop(lock_file);
                            return Ok(value);
                        }
                        Err(_) => continue,
                    }
                }

                record_slot_wait();
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
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::{Arc, Mutex};

    use tempfile::TempDir;

    use super::*;

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
        assert!(peak.load(Ordering::SeqCst) >= 2, "max_concurrent=2 MUST allow parallel slots");
        assert!(peak.load(Ordering::SeqCst) <= 2);
    }

    /// FR-008 / AC-008.14 — Critical waiters MUST acquire before Normal under contention.
    #[test]
    fn critical_dequeues_before_normal_under_contention() {
        let dir = TempDir::new().unwrap();
        let order = Arc::new(Mutex::new(Vec::<&'static str>::new()));
        let holder_ready = Arc::new(AtomicBool::new(false));
        let release_holder = Arc::new(AtomicBool::new(false));

        let holder_order = Arc::clone(&order);
        let holder_ready_flag = Arc::clone(&holder_ready);
        let holder_release = Arc::clone(&release_holder);
        let holder_root = dir.path().to_path_buf();
        let holder = thread::spawn(move || {
            let q = SlotQueue::with_options(
                holder_root,
                1,
                Duration::from_secs(5),
                Duration::from_millis(5),
            );
            q.with_slot("lane", QueuePriority::Normal, || {
                holder_order.lock().unwrap().push("holder_start");
                holder_ready_flag.store(true, Ordering::SeqCst);
                while !holder_release.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_millis(5));
                }
                holder_order.lock().unwrap().push("holder_end");
                Ok(())
            })
            .unwrap();
        });

        while !holder_ready.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(2));
        }

        let critical_order = Arc::clone(&order);
        let critical_root = dir.path().to_path_buf();
        let critical = thread::spawn(move || {
            let q = SlotQueue::with_options(
                critical_root,
                1,
                Duration::from_secs(5),
                Duration::from_millis(5),
            );
            q.with_slot("lane", QueuePriority::Critical, || {
                critical_order.lock().unwrap().push("critical");
                Ok(())
            })
            .unwrap();
        });

        let late_normal_order = Arc::clone(&order);
        let late_normal_root = dir.path().to_path_buf();
        let late_normal = thread::spawn(move || {
            let q = SlotQueue::with_options(
                late_normal_root,
                1,
                Duration::from_secs(5),
                Duration::from_millis(5),
            );
            q.with_slot("lane", QueuePriority::Normal, || {
                late_normal_order.lock().unwrap().push("normal_late");
                Ok(())
            })
            .unwrap();
        });

        let waiting = dir.path().join("lane.waiting");
        let wait_deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < wait_deadline {
            let count = fs::read_dir(&waiting).map(|rd| rd.count()).unwrap_or(0);
            if count >= 2 {
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }

        thread::sleep(Duration::from_millis(20));
        release_holder.store(true, Ordering::SeqCst);
        holder.join().unwrap();
        critical.join().unwrap();
        late_normal.join().unwrap();

        let seq = order.lock().unwrap().clone();
        assert_eq!(
            seq,
            vec!["holder_start", "holder_end", "critical", "normal_late"],
            "AC-008.14: Critical MUST dequeue before Normal"
        );
    }

    /// FR-008 / AC-008.15 — operator env overrides rules.conf priority.
    #[test]
    #[serial_test::serial]
    fn resolve_operator_queue_priority_env_overrides_rule() {
        unsafe {
            std::env::set_var(QUEUE_PRIORITY_ENV, "critical");
        }
        assert_eq!(
            resolve_operator_queue_priority(Some("low")),
            QueuePriority::Critical,
            "AC-008.15: SHARECLI_QUEUE_PRIORITY MUST win over rules.conf"
        );
        unsafe {
            std::env::remove_var(QUEUE_PRIORITY_ENV);
        }
    }

    /// FR-008 / AC-008.15 — rules.conf priority fallback when env unset.
    #[test]
    #[serial_test::serial]
    fn resolve_operator_queue_priority_rule_fallback() {
        unsafe {
            std::env::remove_var(QUEUE_PRIORITY_ENV);
        }
        assert_eq!(
            resolve_operator_queue_priority(Some("high")),
            QueuePriority::High,
            "AC-008.15: rules.conf priority MUST map to QueuePriority"
        );
        assert_eq!(resolve_operator_queue_priority(None), QueuePriority::Normal);
    }
}
