// SPDX-License-Identifier: MIT OR Apache-2.0
//! JSON-RPC 2.0 envelope types used by every `phenotype-teamcomm` transport.
//!
//! The parameter and result payloads are intentionally typed as
//! [`serde_json::Value`] so this crate stays a pure data crate with no
//! dependency on the concrete method set. Method-specific schemas live in
//! the daemon and client crates.

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// JSON-RPC 2.0 request `id` field. Either a string or a non-negative integer.
///
/// We do not allow `null` IDs on the wire; use [`Option<RpcId>`] in
/// [`JsonRpcRequest`] for the "notification" case (no `id`, no response expected).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RpcId {
    /// String id.
    String(String),
    /// Unsigned integer id.
    Number(u64),
}

impl fmt::Display for RpcId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RpcId::String(s) => f.write_str(s),
            RpcId::Number(n) => write!(f, "{n}"),
        }
    }
}

impl Serialize for RpcId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            RpcId::String(s) => serializer.serialize_str(s),
            RpcId::Number(n) => serializer.serialize_u64(*n),
        }
    }
}

struct RpcIdVisitor;

impl<'de> Visitor<'de> for RpcIdVisitor {
    type Value = RpcId;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a JSON-RPC id (string or non-negative integer)")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(RpcId::String(v.to_owned()))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(RpcId::String(v))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
        Ok(RpcId::Number(v))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
        if v < 0 {
            return Err(de::Error::custom(
                "JSON-RPC id must be a non-negative integer or a string",
            ));
        }
        Ok(RpcId::Number(v as u64))
    }
}

impl<'de> Deserialize<'de> for RpcId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(RpcIdVisitor)
    }
}

/// JSON-RPC 2.0 request envelope.
///
/// `id == None` denotes a notification: the server MUST NOT reply. The
/// `params` field is intentionally a `serde_json::Value` to keep this crate
/// agnostic of any specific method's parameter shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Always the literal string `"2.0"`.
    pub jsonrpc: String,
    /// Request id. `None` for notifications.
    pub id: Option<RpcId>,
    /// Method name to invoke.
    pub method: String,
    /// Method-specific parameters.
    pub params: serde_json::Value,
}

/// JSON-RPC 2.0 success response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Always the literal string `"2.0"`.
    pub jsonrpc: String,
    /// Echoes the request id.
    pub id: RpcId,
    /// Method-specific result. May be `null` if the method has no return.
    pub result: serde_json::Value,
}

/// JSON-RPC 2.0 `error` object (the inner error of a [`JsonRpcErrorResponse`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Numeric error code. See [`crate::error::ErrorCode`] and
    /// [`crate::error::error_code`].
    pub code: i32,
    /// Short human-readable description of the error.
    pub message: String,
    /// Optional structured error data. Schema is method-specific.
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 error response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcErrorResponse {
    /// Always the literal string `"2.0"`.
    pub jsonrpc: String,
    /// Echoes the request id (or `RpcId::Number(0)` for a parse error where
    /// the original id was unreadable).
    pub id: RpcId,
    /// The error object.
    pub error: JsonRpcError,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rpc_id_string_serializes_as_string() {
        let v = serde_json::to_value(RpcId::String("abc".into())).unwrap();
        assert_eq!(v, json!("abc"));
    }

    #[test]
    fn rpc_id_number_serializes_as_number() {
        let v = serde_json::to_value(RpcId::Number(42)).unwrap();
        assert_eq!(v, json!(42));
    }

    #[test]
    fn rpc_id_deserializes_from_string() {
        let id: RpcId = serde_json::from_value(json!("hello")).unwrap();
        assert_eq!(id, RpcId::String("hello".into()));
    }

    #[test]
    fn rpc_id_deserializes_from_number() {
        let id: RpcId = serde_json::from_value(json!(7)).unwrap();
        assert_eq!(id, RpcId::Number(7));
    }

    #[test]
    fn rpc_id_rejects_negative_number() {
        let err = serde_json::from_value::<RpcId>(json!(-1)).unwrap_err();
        assert!(err.to_string().contains("non-negative"), "got: {err}");
    }

    #[test]
    fn rpc_id_rejects_bool_and_null() {
        assert!(serde_json::from_value::<RpcId>(json!(true)).is_err());
        assert!(serde_json::from_value::<RpcId>(json!(null)).is_err());
    }

    #[test]
    fn rpc_id_display() {
        assert_eq!(RpcId::String("x".into()).to_string(), "x");
        assert_eq!(RpcId::Number(99).to_string(), "99");
    }

    #[test]
    fn json_rpc_request_roundtrip_with_id() {
        let original = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(RpcId::Number(1)),
            method: "session.register".into(),
            params: json!({"agent_type": "forge", "pid": 1234}),
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: JsonRpcRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn json_rpc_request_roundtrip_notification() {
        let original = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: None,
            method: "session.heartbeat".into(),
            params: json!({}),
        };
        let s = serde_json::to_string(&original).unwrap();
        assert!(s.contains("\"id\":null"), "got: {s}");
        let back: JsonRpcRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
        assert!(back.id.is_none());
    }

    #[test]
    fn json_rpc_request_string_id_roundtrip() {
        let original = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(RpcId::String("req-uuid".into())),
            method: "ping".into(),
            params: json!(null),
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: JsonRpcRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(back.id, Some(RpcId::String("req-uuid".into())));
    }

    #[test]
    fn json_rpc_response_roundtrip() {
        let original = JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: RpcId::String("req-1".into()),
            result: json!({"ok": true}),
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: JsonRpcResponse = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn json_rpc_error_response_roundtrip() {
        let original = JsonRpcErrorResponse {
            jsonrpc: "2.0".into(),
            id: RpcId::Number(7),
            error: JsonRpcError {
                code: -32601,
                message: "Method not found".into(),
                data: Some(json!({"method": "nope"})),
            },
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: JsonRpcErrorResponse = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn json_rpc_error_without_data_serializes_data_as_null() {
        let err = JsonRpcError {
            code: -32603,
            message: "boom".into(),
            data: None,
        };
        let s = serde_json::to_string(&err).unwrap();
        assert!(s.contains("\"data\":null"), "got: {s}");
        let back: JsonRpcError = serde_json::from_str(&s).unwrap();
        assert_eq!(back, err);
    }
}
