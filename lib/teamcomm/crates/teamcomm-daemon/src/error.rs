// SPDX-License-Identifier: MIT OR Apache-2.0
//! Daemon error types and JSON-RPC error envelope conversion.
//!
//! M0 keeps the surface minimal: a thin `TeamcommError` enum (distinct
//! from the protocol crate's own [`teamcomm_protocol::TeamcommError`])
//! that can be converted into a JSON-RPC 2.0 error response. Future
//! milestones can collapse the two enums once the protocol crate
//! stabilises.

use serde_json::Value;
use thiserror::Error;

use teamcomm_protocol::rpc::{JsonRpcError, JsonRpcErrorResponse, RpcId};

/// Crate-level error type for the daemon.
#[derive(Debug, Error)]
pub enum TeamcommError {
    /// Caller asked for a method we do not implement.
    #[error("method not found: {0}")]
    MethodNotFound(String),

    /// Caller sent a structurally valid but semantically wrong request
    /// (missing required field, wrong type, ...).
    #[error("invalid params: {0}")]
    InvalidParams(String),

    /// Caller referenced a session / reservation / message that does not
    /// exist.
    #[error("not found: {0}")]
    NotFound(String),

    /// Caller tried to claim a path that conflicts with an existing
    /// reservation.
    #[error("conflict: {0}")]
    Conflict(String),

    /// Catch-all for unexpected internal failures (I/O, lock poisoning,
    /// ...).
    #[error("internal error: {0}")]
    Internal(String),
}

impl TeamcommError {
    /// Map to a JSON-RPC `code` integer. Standard JSON-RPC codes are used
    /// for the spec-defined buckets; teamcomm-specific errors use the
    /// `[-32099, -32000]` "implementation-defined" band.
    pub fn rpc_code(&self) -> i32 {
        match self {
            TeamcommError::MethodNotFound(_) => -32601,
            TeamcommError::InvalidParams(_) => -32602,
            TeamcommError::NotFound(_) => -32004,
            TeamcommError::Conflict(_) => -32005,
            TeamcommError::Internal(_) => -32603,
        }
    }

    /// Build a [`JsonRpcError`] with no extra `data` payload.
    pub fn to_rpc_error(&self) -> JsonRpcError {
        JsonRpcError {
            code: self.rpc_code(),
            message: self.to_string(),
            data: None,
        }
    }

    /// Build a [`JsonRpcErrorResponse`] addressed to `id`, optionally
    /// carrying an extra structured `data` payload.
    pub fn into_response(self, id: RpcId) -> JsonRpcErrorResponse {
        JsonRpcErrorResponse {
            jsonrpc: "2.0".to_string(),
            id,
            error: self.to_rpc_error(),
        }
    }
}

/// Convenience: convert an `anyhow::Error` into a `TeamcommError::Internal`.
/// Used by the listener when a connection-level I/O failure happens
/// outside any individual RPC handler.
impl From<anyhow::Error> for TeamcommError {
    fn from(e: anyhow::Error) -> Self {
        TeamcommError::Internal(e.to_string())
    }
}

/// Convenience: serialize any `TeamcommError` as a raw `serde_json::Value`
/// map (`{"code": .., "message": ..}`) for callers that build the rest
/// of the envelope by hand.
pub fn error_to_value(e: &TeamcommError) -> Value {
    serde_json::json!({
        "code": e.rpc_code(),
        "message": e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_not_found_uses_minus_32601() {
        let e = TeamcommError::MethodNotFound("foo".into());
        assert_eq!(e.rpc_code(), -32601);
    }

    #[test]
    fn internal_uses_minus_32603() {
        let e = TeamcommError::Internal("boom".into());
        assert_eq!(e.rpc_code(), -32603);
    }

    #[test]
    fn not_found_uses_application_window() {
        let e = TeamcommError::NotFound("session xyz".into());
        assert_eq!(e.rpc_code(), -32004);
    }

    #[test]
    fn to_rpc_error_omits_data_by_default() {
        let e = TeamcommError::InvalidParams("missing pid".into());
        let rpc = e.to_rpc_error();
        assert_eq!(rpc.code, -32602);
        assert!(rpc.data.is_none());
    }

    #[test]
    fn into_response_wraps_in_envelope() {
        let resp = TeamcommError::Conflict("overlap".into()).into_response(RpcId::Number(3));
        assert_eq!(resp.jsonrpc, "2.0");
        assert_eq!(resp.id, RpcId::Number(3));
        assert_eq!(resp.error.code, -32005);
    }
}
