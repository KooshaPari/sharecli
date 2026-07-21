//! Library surface for sharecli-tray-linux (IPC client + wire types).
//!
//! The binary target uses this module; integration tests import mapping helpers
//! for AC-007.48 tray refresh via `monitoring.report`.

pub mod ipc;
pub mod poll;
