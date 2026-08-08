#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_and_lists_sessions() {
        let store = SessionStore::open_memory().unwrap();
        let session = AgentSession::new("codex", "abc", "/tmp/project");
        store.upsert(&session).unwrap();
        let rows = store.list().unwrap();
        assert_eq!(rows, vec![session]);
    }

    #[test]
    fn rpc_inspect_and_recovery_plan_are_typed() {
        let store = SessionStore::open_memory().unwrap();
        let session = AgentSession::new("forge", "def", "/tmp/work");
        store.upsert(&session).unwrap();
        let service = SessionService::new(store);
        let inspect = service.inspect(&session.id).unwrap().unwrap();
        assert_eq!(inspect.resume.harness, "forge");
        assert_eq!(service.recovery_plan().unwrap().len(), 1);
    }

    #[test]
    fn harness_recipes_and_confidence_are_explicit() {
        assert_eq!(
            AgentSession::forge("id", "/tmp").resume.argv,
            vec!["forge", "--conversation-id", "id"]
        );
        assert_eq!(AgentSession::codex("id", "/tmp").resume.argv, vec!["codex", "resume", "id"]);
        assert_eq!(
            AgentSession::opencode("id", "/tmp").resume.argv,
            vec!["opencode", "--session", "id"]
        );
        assert_eq!(AgentSession::kilo("id", "/tmp").resume.argv, vec!["kilo", "--session", "id"]);
        assert_eq!(
            AgentSession::cursor("id", "/tmp").resume.argv,
            vec!["cursor-agent", "--resume", "id"]
        );
        let session = AgentSession::codex("id", "/tmp");
        assert_eq!(session.confidence, ResolutionConfidence::Exact);
        assert_eq!(session.state, SessionState::Active);
    }
}

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub mod ledger;
pub mod rpc;
pub use ledger::SessionObservation;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResumeRecipe {
    pub harness: String,
    pub session_id: String,
    pub cwd: PathBuf,
    pub argv: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ResolutionConfidence {
    Exact,
    Corroborated,
    Heuristic,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SessionState {
    Pending,
    Active,
    Exited,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProcessEvidence {
    pub pid: Option<u32>,
    pub tty: Option<String>,
    pub cwd: PathBuf,
    pub argv: Vec<String>,
    pub started_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SurfaceRecord {
    pub id: String,
    pub terminal: String,
    pub title: Option<String>,
    pub cwd: PathBuf,
    pub process: Option<ProcessEvidence>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentSession {
    pub id: String,
    pub harness: String,
    pub session_id: String,
    pub cwd: PathBuf,
    pub resume: ResumeRecipe,
    pub confidence: ResolutionConfidence,
    pub state: SessionState,
}

impl AgentSession {
    pub fn new(
        harness: impl Into<String>,
        session_id: impl Into<String>,
        cwd: impl Into<PathBuf>,
    ) -> Self {
        let harness = harness.into();
        let session_id = session_id.into();
        let cwd = cwd.into();
        let id = format!("{harness}:{session_id}");
        let argv = vec![harness.clone(), "resume".to_string(), session_id.clone()];
        Self {
            id,
            harness: harness.clone(),
            session_id: session_id.clone(),
            cwd: cwd.clone(),
            resume: ResumeRecipe { harness, session_id, cwd, argv },
            confidence: ResolutionConfidence::Exact,
            state: SessionState::Active,
        }
    }

    pub fn forge(session_id: impl Into<String>, cwd: impl Into<PathBuf>) -> Self {
        Self::with_recipe("forge", session_id, cwd, |id| {
            vec!["forge".into(), "--conversation-id".into(), id]
        })
    }
    pub fn codex(session_id: impl Into<String>, cwd: impl Into<PathBuf>) -> Self {
        Self::with_recipe("codex", session_id, cwd, |id| vec!["codex".into(), "resume".into(), id])
    }
    pub fn opencode(session_id: impl Into<String>, cwd: impl Into<PathBuf>) -> Self {
        Self::with_recipe("opencode", session_id, cwd, |id| {
            vec!["opencode".into(), "--session".into(), id]
        })
    }
    pub fn kilo(session_id: impl Into<String>, cwd: impl Into<PathBuf>) -> Self {
        Self::with_recipe("kilo", session_id, cwd, |id| vec!["kilo".into(), "--session".into(), id])
    }
    pub fn cursor(session_id: impl Into<String>, cwd: impl Into<PathBuf>) -> Self {
        Self::with_recipe("cursor-agent", session_id, cwd, |id| {
            vec!["cursor-agent".into(), "--resume".into(), id]
        })
    }
    fn with_recipe<F>(
        harness: &str,
        session_id: impl Into<String>,
        cwd: impl Into<PathBuf>,
        build: F,
    ) -> Self
    where
        F: FnOnce(String) -> Vec<String>,
    {
        let session_id = session_id.into();
        let argv = build(session_id.clone());
        let cwd = cwd.into();
        Self {
            id: format!("{harness}:{session_id}"),
            harness: harness.to_string(),
            session_id: session_id.clone(),
            cwd: cwd.clone(),
            resume: ResumeRecipe { harness: harness.to_string(), session_id, cwd, argv },
            confidence: ResolutionConfidence::Exact,
            state: SessionState::Active,
        }
    }
}

pub struct SessionStore {
    conn: Mutex<Connection>,
}

impl SessionStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path).context("open session database")?;
        Self::init(conn)
    }
    pub fn open_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }
    fn init(conn: Connection) -> Result<Self> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (id TEXT PRIMARY KEY, harness TEXT NOT NULL, session_id TEXT NOT NULL, cwd TEXT NOT NULL, resume_json TEXT NOT NULL, confidence TEXT NOT NULL, state TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS session_observations (
                 id TEXT PRIMARY KEY,
                 session_id TEXT NOT NULL,
                 surface_id TEXT NOT NULL,
                 observed_at TEXT NOT NULL,
                 confidence TEXT NOT NULL,
                 resumable INTEGER NOT NULL,
                 evidence TEXT NOT NULL
             );",
        )?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub(crate) fn connection_guard(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn.lock().map_err(|_| anyhow::anyhow!("session store poisoned"))
    }
    pub fn upsert(&self, session: &AgentSession) -> Result<()> {
        self.conn.lock().map_err(|_| anyhow::anyhow!("session store poisoned"))?.execute("INSERT INTO sessions (id,harness,session_id,cwd,resume_json,confidence,state) VALUES (?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(id) DO UPDATE SET harness=excluded.harness, session_id=excluded.session_id, cwd=excluded.cwd, resume_json=excluded.resume_json, confidence=excluded.confidence, state=excluded.state", params![session.id, session.harness, session.session_id, session.cwd.to_string_lossy(), serde_json::to_string(&session.resume)?, serde_json::to_string(&session.confidence)?, serde_json::to_string(&session.state)?])?;
        Ok(())
    }
    pub fn list(&self) -> Result<Vec<AgentSession>> {
        let conn = self.conn.lock().map_err(|_| anyhow::anyhow!("session store poisoned"))?;
        let mut stmt =
            conn.prepare("SELECT id,harness,session_id,cwd,resume_json,confidence,state FROM sessions ORDER BY id")?;
        let rows = stmt.query_map([], Self::row)?.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }
    pub fn get(&self, id: &str) -> Result<Option<AgentSession>> {
        let conn = self.conn.lock().map_err(|_| anyhow::anyhow!("session store poisoned"))?;
        let mut stmt =
            conn.prepare("SELECT id,harness,session_id,cwd,resume_json,confidence,state FROM sessions WHERE id=?1")?;
        let mut rows = stmt.query([id])?;
        rows.next()?.map(Self::row).transpose().map_err(Into::into)
    }
    fn row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentSession> {
        let cwd: String = row.get(3)?;
        let resume: String = row.get(4)?;
        let confidence: String = row.get(5)?;
        let state: String = row.get(6)?;
        Ok(AgentSession {
            id: row.get(0)?,
            harness: row.get(1)?,
            session_id: row.get(2)?,
            cwd: PathBuf::from(cwd),
            resume: serde_json::from_str(&resume).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    4,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?,
            confidence: serde_json::from_str(&confidence).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    5,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?,
            state: serde_json::from_str(&state).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    6,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?,
        })
    }
}

pub struct SessionService {
    store: SessionStore,
}
impl SessionService {
    pub fn new(store: SessionStore) -> Self {
        Self { store }
    }
    pub fn list(&self) -> Result<Vec<AgentSession>> {
        self.store.list()
    }
    pub fn inspect(&self, id: &str) -> Result<Option<AgentSession>> {
        self.store.get(id)
    }
    pub fn recovery_plan(&self) -> Result<Vec<AgentSession>> {
        self.store.list()
    }
}
