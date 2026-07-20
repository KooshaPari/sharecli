//! `sharecli-mesh` — execution-substrate mesh primitives for sharecli.
//!
//! # Boundary
//!
//! Per `thegent` / sharecli boundary audit (2026-06-21), the Maildir task queue
//! is sharecli-owned execution substrate. Control-plane orchestration stays in
//! `thegent`; this crate ports `thegent.mesh.task_queue.MaildirQueue`.
//!
//! # Maildir lifecycle
//!
//! ```text
//! tmp/  — staging (write envelope)
//! new/  — ready to claim (atomic rename from tmp)
//! cur/  — claimed / in-flight (atomic rename from new)
//! ```
//!
//! Public API: [`MaildirQueue::enqueue`], [`MaildirQueue::claim`],
//! [`MaildirQueue::ack`], [`MaildirQueue::nack`], [`MaildirQueue::list_pending`].

mod task_queue;

pub use task_queue::{MaildirQueue, TaskEnvelope};
