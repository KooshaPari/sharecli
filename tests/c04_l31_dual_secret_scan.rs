//! C04 L31 — dual secret scanners (gitleaks + trufflehog) + pre-commit hook (FR-003).
//!
//! FR: FR-003

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// FR-003 / C04 L31 — `security.yml` runs gitleaks and trufflehog on push/PR.
#[test]
fn fr003_security_yml_dual_secret_scanners() {
    let security = fs::read_to_string(repo_root().join(".github/workflows/security.yml"))
        .expect("read security.yml");

    assert!(
        security.contains("gitleaks/gitleaks-action@"),
        "security.yml must run gitleaks-action"
    );
    assert!(
        security.contains("trufflesecurity/trufflehog@"),
        "security.yml must run trufflehog action"
    );
    assert!(
        security.contains("--only-verified"),
        "security.yml trufflehog must use --only-verified"
    );
}

/// FR-003 / C04 L31 — committed pre-commit config wires gitleaks + trufflehog.
#[test]
fn fr003_pre_commit_secret_hooks_present() {
    let pre_commit =
        fs::read_to_string(repo_root().join(".pre-commit-config.yaml")).expect("pre-commit config");

    assert!(
        pre_commit.contains("gitleaks"),
        "pre-commit must include gitleaks hook"
    );
    assert!(
        pre_commit.contains("trufflehog"),
        "pre-commit must include trufflehog hook"
    );
    assert!(
        pre_commit.contains("gitleaks.toml"),
        "gitleaks hook must reference gitleaks.toml"
    );
}

/// FR-003 / C04 L31 — trufflehog exclusions + local scan script exist.
#[test]
fn fr003_trufflehog_config_and_local_script() {
    let trufflehog =
        fs::read_to_string(repo_root().join(".trufflehog.yml")).expect(".trufflehog.yml");

    assert!(
        trufflehog.contains("target/**"),
        ".trufflehog.yml must exclude target/"
    );

    let script = fs::read_to_string(repo_root().join("scripts/ci/secret_scan.sh"))
        .expect("secret_scan.sh");
    assert!(script.contains("gitleaks"), "secret_scan.sh must run gitleaks");
    assert!(
        script.contains("trufflehog"),
        "secret_scan.sh must run trufflehog"
    );
}
