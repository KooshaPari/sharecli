//! Pure admission policy for agent-issued commands.

use std::path::{Path, PathBuf};
use std::time::Duration;

const DEFAULT_DEADLINE: Duration = Duration::from_secs(30);

/// The reason an agent call must wait before it can run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseCode {
    /// The command would search a host-level or otherwise unsafe root.
    HazardousRoot,
    /// The configured per-project call limit has been reached.
    ProjectLimit,
    /// The host has no thermal headroom for a new call.
    Thermal,
    /// The configured build-command slot limit has been reached.
    BuildSlot,
    /// The caller configured no time for this command to execute.
    DeadlineExceeded,
}

/// An admitted command or a structured pause instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCallDecision {
    command: String,
    pause_code: Option<PauseCode>,
    resume_condition: Option<String>,
    deadline: Duration,
}

impl AgentCallDecision {
    /// The command after policy normalization.
    pub fn command(&self) -> &str {
        &self.command
    }

    /// The reason the command is paused, when admission was refused.
    pub fn pause_code(&self) -> Option<PauseCode> {
        self.pause_code
    }

    /// A stable explanation for a structured pause, when admission was refused.
    pub fn reason(&self) -> Option<&'static str> {
        self.pause_code.map(|code| match code {
            PauseCode::HazardousRoot => "command targets a path outside the project root",
            PauseCode::ProjectLimit => "project call concurrency limit has been reached",
            PauseCode::Thermal => "host thermal headroom is unavailable",
            PauseCode::BuildSlot => "no build-command slot is currently available",
            PauseCode::DeadlineExceeded => "configured execution deadline is zero",
        })
    }

    /// A human-readable condition that permits retrying a paused command.
    pub fn resume_condition(&self) -> Option<&str> {
        self.resume_condition.as_deref()
    }

    /// An actionable suggestion for a paused command.
    pub fn suggestion(&self) -> Option<&str> {
        self.resume_condition()
    }

    /// The bounded execution deadline for this decision.
    pub fn deadline(&self) -> Duration {
        self.deadline
    }
}

/// Deterministic, local-only admission policy for agent calls.
#[derive(Debug)]
pub struct AgentCallPolicy {
    project_root: PathBuf,
    project_limit: usize,
    thermal_headroom: bool,
    build_slots: usize,
    deadline: Duration,
}

impl AgentCallPolicy {
    /// Create a policy scoped to `project_root` with unrestricted local limits.
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            project_limit: usize::MAX,
            thermal_headroom: true,
            build_slots: usize::MAX,
            deadline: DEFAULT_DEADLINE,
        }
    }

    /// Set the maximum number of admitted calls for this project.
    pub fn with_project_limit(mut self, limit: usize) -> Self {
        self.project_limit = limit;
        self
    }

    /// Set whether the host has headroom for another call.
    pub fn with_thermal_headroom(mut self, available: bool) -> Self {
        self.thermal_headroom = available;
        self
    }

    /// Set the number of available build-command slots.
    pub fn with_build_slots(mut self, slots: usize) -> Self {
        self.build_slots = slots;
        self
    }

    /// Set the execution deadline for admitted commands.
    pub fn with_deadline(mut self, deadline: Duration) -> Self {
        self.deadline = deadline;
        self
    }

    /// Normalize and admit a command, or return a pause decision.
    pub fn admit(&self, command: &str) -> AgentCallDecision {
        let words = match tokenize(command) {
            Ok(words) => words,
            Err(()) => {
                return self.paused(
                    command.to_owned(),
                    PauseCode::HazardousRoot,
                    "use a well-formed command inside the project root",
                );
            }
        };
        let normalized = self.normalize(&words, command);

        if self.targets_outside_project_root(&words) {
            return self.paused(
                normalized,
                PauseCode::HazardousRoot,
                "use a path inside the project root",
            );
        }
        if self.deadline.is_zero() {
            return self.paused(
                normalized,
                PauseCode::DeadlineExceeded,
                "set a nonzero execution deadline",
            );
        }
        if !self.thermal_headroom {
            return self.paused(normalized, PauseCode::Thermal, "wait for thermal headroom");
        }
        if self.project_limit == 0 {
            return self.paused(
                normalized,
                PauseCode::ProjectLimit,
                "wait for an active project call to finish",
            );
        }

        let build = is_build_command(&words);
        if build && self.build_slots == 0 {
            return self.paused(
                normalized,
                PauseCode::BuildSlot,
                "wait for an available build slot",
            );
        }

        AgentCallDecision {
            command: normalized,
            pause_code: None,
            resume_condition: None,
            deadline: self.deadline,
        }
    }

    fn normalize(&self, words: &[String], command: &str) -> String {
        let Some(program) = words.first() else {
            return command.to_owned();
        };

        if !matches!(program.as_str(), "grep" | "egrep") || !has_recursive_flag(&words[1..]) {
            return command.to_owned();
        }

        let mut positional = words[1..].iter().filter(|word| !word.starts_with('-'));
        let pattern = positional.next().map_or("", String::as_str);
        let target = match positional.next() {
            Some(target) if target == "." => self.project_root.as_path(),
            None => self.project_root.as_path(),
            Some(target) => Path::new(target),
        };
        format!(
            "rg --hidden --glob '!target' --glob '!node_modules' {} {}",
            shell_quote(pattern),
            shell_quote(&target.display().to_string())
        )
    }

    fn paused(
        &self,
        command: String,
        pause_code: PauseCode,
        resume_condition: &str,
    ) -> AgentCallDecision {
        AgentCallDecision {
            command,
            pause_code: Some(pause_code),
            resume_condition: Some(resume_condition.to_owned()),
            deadline: self.deadline,
        }
    }

    fn targets_outside_project_root(&self, words: &[String]) -> bool {
        let mut positional = words.iter().skip(1).filter(|word| !word.starts_with('-'));
        if is_search_program(words) {
            positional.next();
        }
        positional.any(|word| target_outside_project_root(Path::new(word)))
    }
}

fn is_search_program(words: &[String]) -> bool {
    matches!(words.first().map(String::as_str), Some("rg" | "grep" | "egrep"))
}

fn has_recursive_flag(words: &[String]) -> bool {
    words.iter().any(|word| {
        word == "--recursive"
            || word.starts_with('-') && word[1..].chars().any(|flag| matches!(flag, 'r' | 'R'))
    })
}

fn is_build_command(words: &[String]) -> bool {
    matches!(words.first().map(String::as_str), Some("cargo" | "make" | "just"))
}

fn target_outside_project_root(path: &Path) -> bool {
    path.is_absolute() || path.components().any(|component| component.as_os_str() == "..")
}

fn shell_quote(argument: &str) -> String {
    format!("'{}'", argument.replace('\'', "'\"'\"'"))
}

fn tokenize(command: &str) -> Result<Vec<String>, ()> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut escaped = false;

    for character in command.chars() {
        if escaped {
            word.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            } else {
                word.push(character);
            }
        } else if matches!(character, '\'' | '\"') {
            quote = Some(character);
        } else if character.is_whitespace() {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
        } else {
            word.push(character);
        }
    }

    if escaped || quote.is_some() {
        return Err(());
    }
    if !word.is_empty() {
        words.push(word);
    }
    Ok(words)
}
