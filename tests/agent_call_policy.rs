use std::path::PathBuf;
use std::time::Duration;

use sharecli::agent_call_policy::{AgentCallPolicy, PauseCode};

fn policy() -> AgentCallPolicy {
    AgentCallPolicy::new(PathBuf::from("/workspace/project"))
}

#[test]
fn rewrites_recursive_grep_to_a_bounded_ripgrep_command() {
    let decision = policy().admit("grep -R TODO .");

    assert_eq!(
        decision.command(),
        "rg --hidden --glob '!target' --glob '!node_modules' 'TODO' '/workspace/project'"
    );
}

#[test]
fn quotes_user_derived_recursive_grep_pattern_and_target() {
    let decision = policy().admit(r#"grep -R "a; b's" "src dir""#);

    assert_eq!(
        decision.command(),
        "rg --hidden --glob '!target' --glob '!node_modules' 'a; b'\"'\"'s' 'src dir'"
    );
}

#[test]
fn leaves_nonrecursive_grep_unchanged() {
    let decision = policy().admit("grep TODO README.md");

    assert_eq!(decision.command(), "grep TODO README.md");
}

#[test]
fn pauses_searches_targeting_a_hazardous_root() {
    let decision = policy().admit("rg TODO /System");

    assert_eq!(decision.pause_code(), Some(PauseCode::HazardousRoot));
    assert_eq!(decision.suggestion(), Some("use a path inside the project root"));
}

#[test]
fn pauses_any_absolute_target() {
    let decision = policy().admit("rg TODO /tmp/other-project");

    assert_eq!(decision.pause_code(), Some(PauseCode::HazardousRoot));
}

#[test]
fn pauses_quoted_absolute_targets() {
    let decision = policy().admit("rg TODO '/tmp/other project'");

    assert_eq!(decision.pause_code(), Some(PauseCode::HazardousRoot));
}

#[test]
fn admits_absolute_regex_patterns_when_the_target_is_inside_the_project() {
    let decision = policy().admit("grep -R /tmp/needle src");

    assert_eq!(decision.pause_code(), None);
    assert_eq!(
        decision.command(),
        "rg --hidden --glob '!target' --glob '!node_modules' '/tmp/needle' 'src'"
    );
}

#[test]
fn pauses_commands_with_unmatched_quotes() {
    let decision = policy().admit("rg TODO 'src");

    assert_eq!(decision.pause_code(), Some(PauseCode::HazardousRoot));
}

#[test]
fn pauses_relative_traversal_outside_the_project_root() {
    let decision = policy().admit("grep -R TODO ../other-project");

    assert_eq!(decision.pause_code(), Some(PauseCode::HazardousRoot));
}

#[test]
fn pauses_when_project_limit_is_zero() {
    let decision = policy().with_project_limit(0).admit("rg TODO src");

    assert_eq!(decision.pause_code(), Some(PauseCode::ProjectLimit));
}

#[test]
fn pauses_when_thermal_headroom_is_unavailable() {
    let decision = policy().with_thermal_headroom(false).admit("rg TODO src");

    assert_eq!(decision.pause_code(), Some(PauseCode::Thermal));
}

#[test]
fn pauses_builds_when_no_build_slots_are_configured() {
    let decision = policy().with_build_slots(0).admit("cargo build");

    assert_eq!(decision.pause_code(), Some(PauseCode::BuildSlot));
}

#[test]
fn attaches_a_nonzero_deadline_to_admitted_calls() {
    let decision = policy().admit("rg TODO src");

    assert!(decision.deadline() > Duration::ZERO);
}

#[test]
fn pauses_when_the_configured_deadline_is_zero() {
    let decision = policy().with_deadline(Duration::ZERO).admit("rg TODO src");

    assert_eq!(decision.pause_code(), Some(PauseCode::DeadlineExceeded));
    assert_eq!(decision.suggestion(), Some("set a nonzero execution deadline"));
}

#[test]
fn uses_the_callers_configured_deadline() {
    let deadline = Duration::from_secs(5);
    let decision = policy().with_deadline(deadline).admit("rg TODO src");

    assert_eq!(decision.deadline(), deadline);
}

#[test]
fn all_pauses_expose_nonempty_suggestions() {
    let decisions = [
        policy().admit("rg TODO /tmp/other-project"),
        policy().with_project_limit(0).admit("rg TODO src"),
        policy().with_thermal_headroom(false).admit("rg TODO src"),
        policy().with_build_slots(0).admit("cargo build"),
        policy().with_deadline(Duration::ZERO).admit("rg TODO src"),
    ];

    for decision in decisions {
        assert!(decision.pause_code().is_some());
        assert!(decision.suggestion().is_some_and(|suggestion| !suggestion.is_empty()));
    }
}

#[test]
fn admission_is_deterministic_for_identical_inputs() {
    let policy = policy().with_project_limit(1).with_build_slots(1);

    assert_eq!(policy.admit("cargo test"), policy.admit("cargo test"));
}
