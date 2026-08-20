//! C08 L76 soft gate - Harbor soft stub (T-720 Wave16)
//! FR: FR-003
//! Scope: doc stub only, no 7d log, EXTRACTED lane noted.

use std::fs;
use std::path::Path;

#[test]
fn c08_harbor_soft_stub_no_7d_log() {
    // Soft gate, no 7d log required. Verify no in-repo 7d log and stub doc exists.
    assert!(!Path::new("docs/eval/harbor-7d.log").exists());
    assert!(!Path::new("harbor-7d.log").exists());
    assert!(
        Path::new("docs/eval/harbor-soft-stub.md").exists(),
        "harbor-soft-stub.md must exist for T-720"
    );
}

#[test]
fn c08_harbor_soft_stub_extracted_notion() {
    // Ensures stub acknowledges EXTRACTED status by reading the doc.
    let content = fs::read_to_string("docs/eval/harbor-soft-stub.md")
        .expect("harbor-soft-stub.md exists");
    assert!(
        content.contains("EXTRACTED"),
        "Harbor 7d log is EXTRACTED, not in sharecli main"
    );
    assert!(
        content.contains("benchora/harbor-soft"),
        "doc must reference EXTRACTED lane"
    );
}

#[test]
fn c08_harbor_soft_stub_no_live_infra() {
    // No live infra, no 7d soak - verify doc scopes correctly, soft gate only.
    let content = fs::read_to_string("docs/eval/harbor-soft-stub.md").unwrap();
    assert!(
        content.contains("doc stub only"),
        "scope must remain doc stub only"
    );
}
