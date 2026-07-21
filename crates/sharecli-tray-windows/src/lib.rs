//! Library surface for the Windows WinUI tray (wire types + mapping helpers).
//!
//! The WinUI binary lives under `windows/ShareCLITray/`; this crate compiles
//! cross-platform so integration tests can prove AC-007.51 mapping parity with
//! Linux/Swift tray refresh via `monitoring.report`, and AC-007.52 poll cadence.

pub mod ipc;
pub mod poll;
