// SPDX-License-Identifier: MIT OR Apache-2.0
//! Error types and JSON-RPC error-code mapping for `teamcomm-protocol`.
//!
//! - [`ErrorCode`] mirrors the JSON-RPC 2.0 *standard* error codes for the
//!   pre-defined `parse`, `invalid request`, `method not found`, `invalid params`,
//!   and `internal` cases.
//! - [`TeamcommError`] is the crate-level error enum surfaced by all
//!   teamcomm operations.
//! - [`error_code`] maps a [`TeamcommError`] to the JSON-RPC `code` integer
//!   used in a [`crate::rpc::JsonRpcErrorResponse`].

use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard JSON-RPC 2.0 error codes plus teamcomm's application-level custom codes.
///
/// The variants with negative discriminants are the reserved JSON-RPC server-error
/// range (`-32768 .. -32000`). Custom application codes are still allowed in the
/// `[-32099, -32000]` window — those would be added here as the protocol grows.
///
/// On the wire these serialize/deserialize as their integer discriminant
/// (e.g. `-32700`), not as a string variant name. This matches the JSON-RPC
/// 2.0 spec which uses integer `code` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ErrorCode {
    /// Invalid JSON was received by the server (-32700).
    ParseError = -32700,
    /// The JSON sent is not a valid request object (-32600).
    InvalidRequest = -32600,
    /// The method does not exist or is not available (-32601).
    MethodNotFound = -32601,
    /// Invalid method parameters (-32602).
    InvalidParams = -32602,
    /// Internal JSON-RPC error (-32603).
    InternalError = -32603,
}

impl From<ErrorCode> for i32 {
    fn from(c: ErrorCode) -> i32 {
        c as i32
    }
}

impl TryFrom<i32> for ErrorCode {
    type Error = i32;
    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            -32700 => Ok(ErrorCode::ParseError),
            -32600 => Ok(ErrorCode::InvalidRequest),
            -32601 => Ok(ErrorCode::MethodNotFound),
            -32602 => Ok(ErrorCode::InvalidParams),
            -32603 => Ok(ErrorCode::InternalError),
            other => Err(other),
        }
    }
}

impl Serialize for ErrorCode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i32(*self as i32)
    }
}

impl<'de> Deserialize<'de> for ErrorCode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = i32::deserialize(deserializer)?;
        ErrorCode::try_from(v).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ErrorCode::ParseError => "Parse error",
            ErrorCode::InvalidRequest => "Invalid request",
            ErrorCode::MethodNotFound => "Method not found",
            ErrorCode::InvalidParams => "Invalid params",
            ErrorCode::InternalError => "Internal error",
        };
        f.write_str(s)
    }
}

/// Crate-level error type for all `phenotype-teamcomm` operations.
///
/// Wire representation uses **adjacent tagging**: a `kind` discriminator
/// string plus a `value` field for the wrapped payload. Unit variants like
/// [`TeamcommError::Unauthorized`] serialize as `{"kind":"unauthorized"}`
/// (no `value` field); newtype variants like [`TeamcommError::NotFound`]
/// serialize as `{"kind":"not_found","value":"<context>"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum TeamcommError {
    /// Requested resource (session, reservation, message) was not found.
    NotFound(String),
    /// Resource already exists (e.g. duplicate session id).
    AlreadyExists(String),
    /// Operation conflicts with the current state (e.g. reservation collision).
    Conflict(String),
    /// Caller is not authorized to perform the operation.
    Unauthorized,
    /// Request was structurally valid but semantically invalid.
    InvalidRequest(String),
    /// Catch-all for unexpected internal failures.
    Internal(String),
}

impl fmt::Display for TeamcommError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TeamcommError::NotFound(what) => write!(f, "not found: {what}"),
            TeamcommError::AlreadyExists(what) => write!(f, "already exists: {what}"),
            TeamcommError::Conflict(what) => write!(f, "conflict: {what}"),
            TeamcommError::Unauthorized => f.write_str("unauthorized"),
            TeamcommError::InvalidRequest(what) => write!(f, "invalid request: {what}"),
            TeamcommError::Internal(what) => write!(f, "internal error: {what}"),
        }
    }
}

impl std::error::Error for TeamcommError {}

/// Map a [`TeamcommError`] to the integer `code` field used in a JSON-RPC
/// error response.
///
/// Custom teamcomm-specific errors use the JSON-RPC *application* error window
/// (`-32099 .. -32000`):
///
/// | Variant           | Code     |
/// |-------------------|----------|
/// | `Unauthorized`    | `-32001` |
/// | `AlreadyExists`   | `-32003` |
/// | `NotFound`        | `-32004` |
/// | `Conflict`        | `-32005` |
/// | `InvalidRequest`  | `-32600` |
/// | `Internal`        | `-32603` |
pub fn error_code(e: &TeamcommError) -> i32 {
    match e {
        TeamcommError::Unauthorized => -32001,
        TeamcommError::AlreadyExists(_) => -32003,
        TeamcommError::NotFound(_) => -32004,
        TeamcommError::Conflict(_) => -32005,
        TeamcommError::InvalidRequest(_) => ErrorCode::InvalidRequest as i32,
        TeamcommError::Internal(_) => ErrorCode::InternalError as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_mapping_is_stable() {
        assert_eq!(error_code(&TeamcommError::Unauthorized), -32001);
        assert_eq!(
            error_code(&TeamcommError::AlreadyExists("x".into())),
            -32003
        );
        assert_eq!(error_code(&TeamcommError::NotFound("x".into())), -32004);
        assert_eq!(error_code(&TeamcommError::Conflict("x".into())), -32005);
        assert_eq!(
            error_code(&TeamcommError::InvalidRequest("x".into())),
            -32600
        );
        assert_eq!(error_code(&TeamcommError::Internal("x".into())), -32603);
    }

    #[test]
    fn teamcomm_error_display_includes_context() {
        let e = TeamcommError::NotFound("session abc".into());
        assert_eq!(e.to_string(), "not found: session abc");
        assert_eq!(TeamcommError::Unauthorized.to_string(), "unauthorized");
    }

    #[test]
    fn error_code_enum_serializes_as_integer() {
        let v = serde_json::to_value(ErrorCode::ParseError).unwrap();
        assert_eq!(v, serde_json::json!(-32700));
        let v = serde_json::to_value(ErrorCode::MethodNotFound).unwrap();
        assert_eq!(v, serde_json::json!(-32601));
    }

    #[test]
    fn error_code_enum_deserializes_from_integer() {
        let parsed: ErrorCode = serde_json::from_value(serde_json::json!(-32700)).unwrap();
        assert_eq!(parsed, ErrorCode::ParseError);
    }

    #[test]
    fn teamcomm_error_roundtrip_via_snake_case_tag() {
        let cases = vec![
            TeamcommError::NotFound("session".into()),
            TeamcommError::AlreadyExists("session".into()),
            TeamcommError::Conflict("overlap".into()),
            TeamcommError::Unauthorized,
            TeamcommError::InvalidRequest("missing field".into()),
            TeamcommError::Internal("boom".into()),
        ];
        for original in cases {
            let s = serde_json::to_string(&original).unwrap();
            let back: TeamcommError = serde_json::from_str(&s).unwrap();
            assert_eq!(back, original, "roundtrip failed for {s}");
        }
    }

    #[test]
    fn teamcomm_error_uses_snake_case_tag_in_json() {
        let s = serde_json::to_string(&TeamcommError::NotFound("x".into())).unwrap();
        assert!(s.contains("\"kind\":\"not_found\""), "got: {s}");
        let s = serde_json::to_string(&TeamcommError::AlreadyExists("x".into())).unwrap();
        assert!(s.contains("\"kind\":\"already_exists\""), "got: {s}");
        let s = serde_json::to_string(&TeamcommError::Unauthorized).unwrap();
        assert!(s.contains("\"kind\":\"unauthorized\""), "got: {s}");
    }
}
