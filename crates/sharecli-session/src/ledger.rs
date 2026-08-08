use serde::{Deserialize, Serialize};

use crate::{ResolutionConfidence, SessionStore};

/// A durable observation of an agent session discovered from a terminal
/// surface. Observations are append-only so crash recovery can distinguish
/// corroborated state from an ambiguous heuristic.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SessionObservation {
    pub id: String,
    pub session_id: String,
    pub surface_id: String,
    pub observed_at: String,
    pub confidence: ResolutionConfidence,
    pub resumable: bool,
    pub evidence: String,
}

impl SessionObservation {
    pub fn new(
        id: impl Into<String>,
        session_id: impl Into<String>,
        surface_id: impl Into<String>,
        observed_at: impl Into<String>,
        confidence: ResolutionConfidence,
        evidence: impl Into<String>,
    ) -> Self {
        let resumable =
            matches!(confidence, ResolutionConfidence::Exact | ResolutionConfidence::Corroborated);
        Self {
            id: id.into(),
            session_id: session_id.into(),
            surface_id: surface_id.into(),
            observed_at: observed_at.into(),
            confidence,
            resumable,
            evidence: evidence.into(),
        }
    }
}

impl SessionStore {
    pub fn append_observation(&self, observation: &SessionObservation) -> anyhow::Result<()> {
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO session_observations
             (id,session_id,surface_id,observed_at,confidence,resumable,evidence)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            rusqlite::params![
                observation.id,
                observation.session_id,
                observation.surface_id,
                observation.observed_at,
                serde_json::to_string(&observation.confidence)?,
                observation.resumable,
                observation.evidence,
            ],
        )?;
        Ok(())
    }

    pub fn observations(&self, session_id: &str) -> anyhow::Result<Vec<SessionObservation>> {
        let conn = self.connection()?;
        let mut statement = conn.prepare(
            "SELECT id,session_id,surface_id,observed_at,confidence,resumable,evidence
             FROM session_observations WHERE session_id=?1 ORDER BY observed_at,id",
        )?;
        let rows = statement
            .query_map([session_id], |row| {
                let confidence: String = row.get(4)?;
                Ok(SessionObservation {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    surface_id: row.get(2)?,
                    observed_at: row.get(3)?,
                    confidence: serde_json::from_str(&confidence).map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            4,
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?,
                    resumable: row.get(5)?,
                    evidence: row.get(6)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    fn connection(&self) -> anyhow::Result<std::sync::MutexGuard<'_, rusqlite::Connection>> {
        self.connection_guard()
    }
}
