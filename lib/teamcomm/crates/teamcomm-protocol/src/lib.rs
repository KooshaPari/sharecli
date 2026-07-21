// SPDX-License-Identifier: MIT OR Apache-2.0
//! `teamcomm-protocol` — wire types for inter-agent coordination.
//!
//! M0 surface used by the daemon and its clients:
//!
//! - [`Session`], [`SessionRegistration`] — what an agent declares on
//!   `session.register`.
//! - [`LiveState`], [`AgentStatus`] — what an agent publishes on
//!   `state` (and incidentally in heartbeats).
//! - [`Reservation`] — placeholder for the M1 reservation catalogue.
//! - [`InboxMessage`] — placeholder for the M1 inbox store.
//! - [`rpc`] — JSON-RPC 2.0 envelope types (`JsonRpcRequest`,
//!   `JsonRpcResponse`, `JsonRpcErrorResponse`, `RpcId`).
//! - [`TeamcommError`] — crate-level error type and JSON-RPC code map.

pub mod error;
pub mod inbox;
pub mod reservation;
pub mod rpc;
pub mod session;
pub mod state;

pub use error::{error_code, ErrorCode, TeamcommError};
pub use inbox::{InboxMessage, Priority};
pub use reservation::{Reservation, ReservationMode};
pub use rpc::{JsonRpcErrorResponse, JsonRpcRequest, JsonRpcResponse, RpcId};
pub use session::{AgentType, Session, SessionRegistration, SessionSummary};
pub use state::{AgentStatus, LiveState};
