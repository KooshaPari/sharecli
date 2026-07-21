// SPDX-License-Identifier: MIT OR Apache-2.0
//! File/path reservation types: claims, modes, and conflict reporting.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// How strongly an agent is claiming a path.
///
/// The mode ordering is read < write < exclusive; a claim only conflicts with
/// claims of *equal or stronger* mode that overlap on the same path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReservationMode {
    /// Shared read access — many `Read` claims may coexist.
    Read,
    /// Exclusive write access — conflicts with any other `Write` or
    /// `Exclusive` claim on the same path.
    Write,
    /// Hard exclusive lock — conflicts with everything on the same path
    /// including `Read`.
    Exclusive,
}

/// An active reservation held by a session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reservation {
    /// Stable, unique reservation id (UUID v4).
    pub reservation_id: String,
    /// Session that owns this reservation.
    pub session_id: String,
    /// Absolute or repo-relative path the reservation covers.
    pub path: PathBuf,
    /// Reservation strength.
    pub mode: ReservationMode,
    /// When the reservation was acquired.
    pub acquired_at: DateTime<Utc>,
    /// When the reservation auto-expires. The daemon may release early
    /// on `ReservationReleased` or session end.
    pub expires_at: DateTime<Utc>,
}

/// Request payload from an agent that wants to claim a path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimRequest {
    /// Session id of the requesting agent.
    pub session_id: String,
    /// Path the agent wants to reserve.
    pub path: PathBuf,
    /// Reservation mode being requested.
    pub mode: ReservationMode,
    /// Time-to-live in seconds from acquisition. The daemon sets
    /// `expires_at = acquired_at + ttl_sec`.
    pub ttl_sec: u64,
}

/// Result of a `claim` operation.
///
/// `conflicts` is non-empty when the claim was *not* granted (i.e. the
/// existing reservations on the path block the new one). When the claim
/// succeeds, `reservation` is populated and `conflicts` is empty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimResult {
    /// The newly acquired reservation. Present iff the claim succeeded.
    pub reservation: Reservation,
    /// Existing reservations on the same path that blocked the claim.
    /// Empty on success.
    pub conflicts: Vec<Reservation>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 12, 10, 0, 0).unwrap()
    }

    #[test]
    fn reservation_mode_serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_value(ReservationMode::Read).unwrap(),
            serde_json::json!("read")
        );
        assert_eq!(
            serde_json::to_value(ReservationMode::Write).unwrap(),
            serde_json::json!("write")
        );
        assert_eq!(
            serde_json::to_value(ReservationMode::Exclusive).unwrap(),
            serde_json::json!("exclusive")
        );
    }

    #[test]
    fn reservation_mode_roundtrip() {
        for mode in [
            ReservationMode::Read,
            ReservationMode::Write,
            ReservationMode::Exclusive,
        ] {
            let s = serde_json::to_string(&mode).unwrap();
            let back: ReservationMode = serde_json::from_str(&s).unwrap();
            assert_eq!(back, mode);
        }
    }

    #[test]
    fn reservation_roundtrip() {
        let acquired = ts();
        let expires = Utc.with_ymd_and_hms(2026, 6, 12, 10, 5, 0).unwrap();
        let original = Reservation {
            reservation_id: "res-1".into(),
            session_id: "sess-1".into(),
            path: PathBuf::from("/repo/src/lib.rs"),
            mode: ReservationMode::Write,
            acquired_at: acquired,
            expires_at: expires,
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: Reservation = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn claim_request_roundtrip() {
        let original = ClaimRequest {
            session_id: "sess-2".into(),
            path: PathBuf::from("/repo/Cargo.toml"),
            mode: ReservationMode::Exclusive,
            ttl_sec: 300,
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: ClaimRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn claim_result_with_conflicts_roundtrip() {
        let acquired = ts();
        let expires = Utc.with_ymd_and_hms(2026, 6, 12, 10, 5, 0).unwrap();
        let blocking = Reservation {
            reservation_id: "res-block".into(),
            session_id: "sess-other".into(),
            path: PathBuf::from("/repo/src/lib.rs"),
            mode: ReservationMode::Write,
            acquired_at: acquired,
            expires_at: expires,
        };
        let new_reservation = Reservation {
            reservation_id: "res-self".into(),
            session_id: "sess-1".into(),
            path: PathBuf::from("/repo/src/lib.rs"),
            mode: ReservationMode::Read,
            acquired_at: acquired,
            expires_at: expires,
        };
        let original = ClaimResult {
            reservation: new_reservation,
            conflicts: vec![blocking],
        };
        let s = serde_json::to_string(&original).unwrap();
        let back: ClaimResult = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn claim_result_with_no_conflicts_roundtrip() {
        let original = ClaimResult {
            reservation: Reservation {
                reservation_id: "r".into(),
                session_id: "s".into(),
                path: PathBuf::from("/p"),
                mode: ReservationMode::Read,
                acquired_at: ts(),
                expires_at: ts(),
            },
            conflicts: vec![],
        };
        let s = serde_json::to_string(&original).unwrap();
        assert!(s.contains("\"conflicts\":[]"), "got: {s}");
        let back: ClaimResult = serde_json::from_str(&s).unwrap();
        assert_eq!(back, original);
    }
}
