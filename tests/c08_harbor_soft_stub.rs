//! C08 L76 soft gate - Harbor soft stub (T-720 Wave16)
//! FR: FR-003
//! Scope: doc stub only, no 7d log, EXTRACTED lane noted.

use std::fs;
use std::path::Path;

fn has_harbor_log(dir: &Path) -> bool {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                let name = p.file_name().unwrap_or_default().to_string_lossy();
                if name.contains("harbor") && name.contains("7d") && name.ends_with(".log") {
                    return true;
                }
            }
            if p.is_dir() {
                let s = p.to_string_lossy();
                if s.contains(".git") || s.contains("target") || s.contains(".cargo") {
                    continue;
                }
                if has_harbor_log(&p) {
                    return true;
                }
            }
        }
    }
    false
}

#[test]
fn c08_harbor_soft_stub_no_7d_log() {
    // Soft gate, no 7d log required. Detect any harbor-7d.log anywhere.
    assert!(!has_harbor_log(Path::new(".")), "no harbor-7d.log should exist anywhere in repo");
    assert!(
        Path::new("docs/eval/harbor-soft-stub.md").exists(),
        "harbor-soft-stub.md must exist for T-720"
    );
}

#[test]
fn c08_harbor_soft_stub_extracted_notion() {
    // Ensures stub acknowledges EXTRACTED status by reading the doc.
    let content =
        fs::read_to_string("docs/eval/harbor-soft-stub.md").expect("harbor-soft-stub.md exists");
    assert!(content.contains("EXTRACTED"), "Harbor 7d log is EXTRACTED, not in sharecli main");
    assert!(content.contains("benchora/harbor-soft"), "doc must reference EXTRACTED lane");
}

#[test]
fn c08_harbor_soft_stub_no_live_infra_doc_scope() {
    // Doc-scope only - verifies stub doc scopes correctly, no live infra.
    let content = fs::read_to_string("docs/eval/harbor-soft-stub.md").unwrap();
    assert!(
        content.contains("doc stub only"),
        "scope must remain doc stub only"
    );
    // Ensure no Harbor workflow or live infra script is committed in sharecli main.
    assert!(
        !Path::new(".github/workflows/harbor.yml").exists(),
        "no Harbor live workflow should exist in sharecli main"
    );
}
