//! `sharecli fuse …` — FUSE IO intercept operator surface (FR-009).
//!
//! Exposes [`read_provenance`] so operators can inspect write provenance xattrs
//! on backing files without mounting or reaching into xattr(1) by hand.

use std::path::Path;

use anyhow::{Context, Result};
use sharecli_fuse::read_provenance;

/// Read FUSE write-provenance xattrs from a backing file path.
///
/// When `json` is true, emit a JSON object or `null` when attrs are absent.
pub fn provenance(path: &Path, json: bool) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("fuse provenance: path does not exist: {}", path.display());
    }
    if path.is_dir() {
        anyhow::bail!("fuse provenance: path must be a file, not a directory: {}", path.display());
    }

    let prov = read_provenance(path)
        .with_context(|| format!("read provenance xattrs on {}", path.display()))?;

    if json {
        match prov {
            Some(p) => {
                let body = serde_json::json!({
                    "path": path.display().to_string(),
                    "session_id": p.session_id,
                    "written_at_unix": p.written_at_unix,
                });
                println!("{}", serde_json::to_string_pretty(&body)?);
            }
            None => println!("null"),
        }
        return Ok(());
    }

    match prov {
        Some(p) => {
            println!("path:       {}", path.display());
            println!("session_id: {}", p.session_id);
            println!("written_at: {} (unix)", p.written_at_unix);
        }
        None => {
            println!("path:       {}", path.display());
            println!("provenance: (none — no sharecli write xattrs)");
        }
    }
    Ok(())
}
