//! Safe, bounded session recovery execution.

use std::process::Command;

use anyhow::Result;
use serde::Serialize;

use crate::{AgentSession, ResumeRecipe, SessionState};

/// Per-session recovery result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum RecoveryOutcome {
    Resumed,
    DryRun,
    SkippedAmbiguous,
    UnsupportedSurface,
    LaunchFailed(String),
}

/// Recovery item paired with its durable session identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RecoveryResult {
    pub session_id: String,
    pub outcome: RecoveryOutcome,
}

/// Execute verified recipes in bounded batches. No shell is involved.
///
/// A successful result means the harness process was spawned. The executor does
/// not wait for an agent UI to exit, which keeps recovery non-blocking for long
/// lived sessions.
pub struct RecoveryExecutor {
    pub max_parallel: usize,
}

impl RecoveryExecutor {
    pub fn new(max_parallel: usize) -> Self {
        Self { max_parallel: max_parallel.max(1) }
    }

    pub fn dry_run(&self, sessions: &[AgentSession]) -> Vec<RecoveryResult> {
        sessions
            .iter()
            .map(|session| RecoveryResult {
                session_id: session.id.clone(),
                outcome: if session.auto_resumable() {
                    RecoveryOutcome::DryRun
                } else {
                    RecoveryOutcome::SkippedAmbiguous
                },
            })
            .collect()
    }

    pub fn execute(&self, sessions: &[AgentSession]) -> Vec<RecoveryResult> {
        let mut results = Vec::with_capacity(sessions.len());
        for batch in sessions.chunks(self.max_parallel) {
            std::thread::scope(|scope| {
                let handles = batch
                    .iter()
                    .map(|session| scope.spawn(|| (session.id.clone(), launch(session))))
                    .collect::<Vec<_>>();
                for handle in handles {
                    let (session_id, outcome) = handle.join().unwrap_or_else(|_| {
                        (
                            "unknown".to_string(),
                            RecoveryOutcome::LaunchFailed("worker panicked".to_string()),
                        )
                    });
                    results.push(RecoveryResult { session_id, outcome });
                }
            });
        }
        results
    }
}

fn launch(session: &AgentSession) -> RecoveryOutcome {
    if !session.auto_resumable() {
        return RecoveryOutcome::SkippedAmbiguous;
    }
    if !matches!(session.state, SessionState::Active | SessionState::Exited | SessionState::Pending)
    {
        return RecoveryOutcome::UnsupportedSurface;
    }
    if let Err(error) = validate_recipe(&session.resume) {
        return RecoveryOutcome::LaunchFailed(error.to_string());
    }
    match Command::new(&session.resume.argv[0])
        .args(&session.resume.argv[1..])
        .current_dir(&session.resume.cwd)
        .spawn()
    {
        Ok(_) => RecoveryOutcome::Resumed,
        Err(error) => RecoveryOutcome::LaunchFailed(error.to_string()),
    }
}

/// Validate that a recipe is argv-based and has a usable cwd before launch.
pub fn validate_recipe(recipe: &ResumeRecipe) -> Result<()> {
    if recipe.argv.is_empty() || recipe.argv.iter().any(|arg| arg.contains('\0')) {
        anyhow::bail!("resume recipe has no safe argv")
    }
    if !recipe.cwd.is_absolute() {
        anyhow::bail!("resume recipe cwd must be absolute")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResolutionConfidence;

    #[test]
    fn dry_run_never_launches_ambiguous_sessions() {
        let mut ambiguous = AgentSession::codex("id", "/tmp");
        ambiguous.confidence = ResolutionConfidence::Heuristic;
        let results = RecoveryExecutor::new(2).dry_run(&[ambiguous]);
        assert_eq!(results[0].outcome, RecoveryOutcome::SkippedAmbiguous);
    }

    #[test]
    fn recipe_validation_rejects_relative_cwd() {
        let session = AgentSession::codex("id", "relative");
        assert!(validate_recipe(&session.resume).is_err());
    }

    #[test]
    fn executor_clamps_parallelism_to_one() {
        assert_eq!(RecoveryExecutor::new(0).max_parallel, 1);
    }
}
