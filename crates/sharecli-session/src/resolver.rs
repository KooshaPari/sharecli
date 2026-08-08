//! Harness/session evidence resolution without shell evaluation.

use crate::{AgentSession, ResolutionConfidence};
use std::path::{Path, PathBuf};

/// Evidence source used to resolve a harness session identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceSource {
    Adapter,
    StateFile,
    Argv,
    Unavailable,
}

/// Resolver output, including confidence and the safe resume recipe.
#[derive(Clone, Debug)]
pub struct Resolution {
    pub session: Option<AgentSession>,
    pub confidence: ResolutionConfidence,
    pub source: EvidenceSource,
}

/// Resolve a known harness using explicit state, then corroborated argv evidence.
pub fn resolve(
    harness: &str,
    cwd: impl Into<PathBuf>,
    argv: &[String],
    state_session_id: Option<&str>,
    adapter_session_id: Option<&str>,
) -> Resolution {
    let cwd = cwd.into();
    if let Some(id) = adapter_session_id.filter(|id| !id.is_empty()) {
        return exact_recipe(harness, id, cwd, EvidenceSource::Adapter);
    }
    if let Some(id) = state_session_id.filter(|id| !id.is_empty()) {
        let confidence = if argv_mentions_id(argv, id) {
            ResolutionConfidence::Corroborated
        } else {
            ResolutionConfidence::Exact
        };
        return recipe(harness, id, cwd, confidence, EvidenceSource::StateFile);
    }
    if let Some(id) = session_id_from_argv(harness, argv) {
        return recipe(harness, &id, cwd, ResolutionConfidence::Exact, EvidenceSource::Argv);
    }
    Resolution {
        session: None,
        confidence: ResolutionConfidence::Unavailable,
        source: EvidenceSource::Unavailable,
    }
}

fn exact_recipe(harness: &str, id: &str, cwd: PathBuf, source: EvidenceSource) -> Resolution {
    recipe(harness, id, cwd, ResolutionConfidence::Exact, source)
}

fn recipe(
    harness: &str,
    id: &str,
    cwd: PathBuf,
    confidence: ResolutionConfidence,
    source: EvidenceSource,
) -> Resolution {
    let session = match harness {
        "forge" => AgentSession::forge(id, cwd),
        "codex" => AgentSession::codex(id, cwd),
        "opencode" => AgentSession::opencode(id, cwd),
        "kilo" => AgentSession::kilo(id, cwd),
        "cursor" | "cursor-agent" => AgentSession::cursor(id, cwd),
        _ => {
            return Resolution {
                session: None,
                confidence: ResolutionConfidence::Unavailable,
                source: EvidenceSource::Unavailable,
            }
        }
    };
    let mut session = session;
    session.confidence = confidence;
    Resolution { session: Some(session), confidence, source }
}

fn argv_mentions_id(argv: &[String], id: &str) -> bool {
    argv.iter().any(|value| value == id)
}

fn session_id_from_argv(harness: &str, argv: &[String]) -> Option<String> {
    let names = match harness {
        "forge" => ["--conversation-id"].as_slice(),
        "codex" => ["resume"].as_slice(),
        "opencode" | "kilo" => ["--session"].as_slice(),
        "cursor" | "cursor-agent" => ["--resume"].as_slice(),
        _ => return None,
    };
    for (index, value) in argv.iter().enumerate() {
        if names.contains(&value.as_str()) {
            return argv.get(index + 1).filter(|id| !id.is_empty()).cloned();
        }
    }
    None
}

#[allow(dead_code)]
fn _cwd_is_valid(cwd: &Path) -> bool {
    cwd.is_absolute() && cwd.exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_argv_resolves_exact_recipe() {
        let result =
            resolve("codex", "/tmp", &["codex".into(), "resume".into(), "id-1".into()], None, None);
        assert_eq!(result.confidence, ResolutionConfidence::Exact);
        assert_eq!(result.session.unwrap().resume.argv, vec!["codex", "resume", "id-1"]);
    }

    #[test]
    fn state_and_argv_are_corroborated() {
        let argv = vec!["codex".into(), "resume".into(), "id-2".into()];
        let result = resolve("codex", "/tmp", &argv, Some("id-2"), None);
        assert_eq!(result.confidence, ResolutionConfidence::Corroborated);
    }

    #[test]
    fn ambiguous_process_never_yields_recipe() {
        let result = resolve("codex", "/tmp", &["codex".into()], None, None);
        assert!(result.session.is_none());
        assert_eq!(result.confidence, ResolutionConfidence::Unavailable);
    }
}
