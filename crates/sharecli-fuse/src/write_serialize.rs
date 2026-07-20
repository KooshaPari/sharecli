//! Per-path write serialization scaffold (FR-009).
//!
//! Concurrent writes to the same path take a per-path mutex so passthrough
//! writes do not race. Full copy-on-write commit/discard remains TODO — the
//! API stubs exist so callers can wire CoW without changing call sites later.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

/// Error from write-serialize / future CoW commit-discard operations.
#[derive(Debug, thiserror::Error)]
pub enum WriteSerializeError {
    /// CoW commit is not implemented yet (scaffold stub).
    #[error("write-serialize CoW commit not implemented yet for {0}")]
    CommitTodo(PathBuf),
    /// CoW discard is not implemented yet (scaffold stub).
    #[error("write-serialize CoW discard not implemented yet for {0}")]
    DiscardTodo(PathBuf),
    /// Internal lock poisoning.
    #[error("write-serialize lock poisoned")]
    Poisoned,
}

/// Serialize concurrent writes to the same path via a per-path lock map.
#[derive(Debug, Default)]
pub struct WriteSerialize {
    locks: Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>,
}

impl WriteSerialize {
    /// Create an empty path-lock table.
    pub fn new() -> Self {
        Self::default()
    }

    fn lock_arc(&self, path: &Path) -> Result<Arc<Mutex<()>>, WriteSerializeError> {
        let mut map = self.locks.lock().map_err(|_| WriteSerializeError::Poisoned)?;
        let entry = map
            .entry(path.to_path_buf())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();
        Ok(entry)
    }

    /// Run `f` while holding the exclusive write lock for `path`.
    pub fn with_locked_path<R, F: FnOnce() -> R>(
        &self,
        path: &Path,
        f: F,
    ) -> Result<R, WriteSerializeError> {
        let arc = self.lock_arc(path)?;
        let _guard = arc.lock().map_err(|_| WriteSerializeError::Poisoned)?;
        Ok(f())
    }

    /// TODO(hypervisor): CoW commit — promote a staging copy to the backing path.
    ///
    /// Current behaviour: returns [`WriteSerializeError::CommitTodo`] so callers
    /// fail loudly rather than silently no-op. Passthrough writes do not need this.
    pub fn commit_pending(&self, path: &Path) -> Result<(), WriteSerializeError> {
        Err(WriteSerializeError::CommitTodo(path.to_path_buf()))
    }

    /// TODO(hypervisor): CoW discard — drop a staging copy without touching backing.
    pub fn discard_pending(&self, path: &Path) -> Result<(), WriteSerializeError> {
        Err(WriteSerializeError::DiscardTodo(path.to_path_buf()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Barrier;
    use std::thread;
    use std::time::Duration;

    /// FR-009 / AC-009.5 — CoW commit/discard stubs fail loudly.
    #[test]
    fn write_serialize_commit_discard_are_stubs() {
        let ws = WriteSerialize::new();
        assert!(matches!(
            ws.commit_pending(Path::new("/tmp/x")),
            Err(WriteSerializeError::CommitTodo(_))
        ));
        assert!(matches!(
            ws.discard_pending(Path::new("/tmp/x")),
            Err(WriteSerializeError::DiscardTodo(_))
        ));
    }

    /// FR-009 / AC-009.5 — same-path writers serialize (no overlapping critical section).
    #[test]
    fn write_serialize_serializes_same_path() {
        let ws = Arc::new(WriteSerialize::new());
        let barrier = Arc::new(Barrier::new(2));
        let path = PathBuf::from("/virtual/same");
        let order = Arc::new(Mutex::new(Vec::new()));

        let ws1 = Arc::clone(&ws);
        let b1 = Arc::clone(&barrier);
        let o1 = Arc::clone(&order);
        let p1 = path.clone();
        let t1 = thread::spawn(move || {
            ws1.with_locked_path(&p1, || {
                o1.lock().expect("order").push(1);
                b1.wait();
                thread::sleep(Duration::from_millis(40));
                o1.lock().expect("order").push(2);
            })
            .expect("lock");
        });

        let ws2 = Arc::clone(&ws);
        let b2 = Arc::clone(&barrier);
        let o2 = Arc::clone(&order);
        let p2 = path;
        let t2 = thread::spawn(move || {
            b2.wait();
            ws2.with_locked_path(&p2, || {
                o2.lock().expect("order").push(3);
            })
            .expect("lock");
        });

        t1.join().expect("t1");
        t2.join().expect("t2");
        let seq = order.lock().expect("order").clone();
        assert_eq!(seq, vec![1, 2, 3]);
    }
}
