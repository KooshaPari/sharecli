//! `sharecli fuse …` — FUSE IO intercept operator surface (FR-009).
//!
//! Exposes mount control, staged CoW commit/discard, global FUSE meters, and
//! write-provenance inspection without reaching into xattr(1) by hand.

use std::path::Path;

use anyhow::{Context, Result};
use sharecli_fuse::{
    default_session_id, global_read_cache_meters, global_write_serialize_meters,
    read_provenance, FuseSessionRegistry,
};

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

/// Mount the intercept layer over `backing` at `mountpoint`.
pub fn mount(
    backing: &Path,
    mountpoint: &Path,
    session_id: Option<&str>,
    foreground: bool,
) -> Result<()> {
    let session = session_id.map(str::to_string).unwrap_or_else(default_session_id);
    let registry = FuseSessionRegistry::global();
    if foreground {
        registry.mount_foreground(mountpoint, backing, &session)?;
    } else {
        registry.mount_background(mountpoint, backing, &session)?;
        println!(
            "fuse mount: {} over {} (session {})",
            mountpoint.display(),
            backing.display(),
            session
        );
    }
    Ok(())
}

/// Unmount a registered intercept mount.
pub fn unmount(mountpoint: &Path) -> Result<()> {
    FuseSessionRegistry::global().unmount(mountpoint)?;
    println!("fuse unmount: {}", mountpoint.display());
    Ok(())
}

/// Print global FUSE read-cache and write-serialize meter sections.
pub fn status(json: bool) -> Result<()> {
    let read = global_read_cache_meters();
    let write = global_write_serialize_meters();
    if json {
        let body = serde_json::json!({
            "read_cache": {
                "hits": read.hits,
                "misses": read.misses,
            },
            "write_serialize": {
                "passthrough_writes": write.passthrough_writes,
                "stages": write.stages,
                "commits": write.commits,
                "discards": write.discards,
            },
        });
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }
    print!("{}", read.format_status_section());
    print!("{}", write.format_status_section());
    Ok(())
}

/// Commit staged CoW for `relpath` on a registered mount.
pub fn commit(relpath: &Path, mountpoint: Option<&Path>) -> Result<()> {
    commit_or_discard(relpath, mountpoint, true)
}

/// Discard staged CoW for `relpath` on a registered mount.
pub fn discard(relpath: &Path, mountpoint: Option<&Path>) -> Result<()> {
    commit_or_discard(relpath, mountpoint, false)
}

fn commit_or_discard(relpath: &Path, mountpoint: Option<&Path>, commit: bool) -> Result<()> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use sharecli_fuse::WriteSerializeError;

        let fs = FuseSessionRegistry::global()
            .resolve_fs(mountpoint)
            .context("fuse commit/discard requires a registered mount")?;
        let result = if commit {
            fs.commit_rel(relpath)
        } else {
            fs.discard_rel(relpath)
        };
        match result {
            Ok(()) => {
                let verb = if commit { "commit" } else { "discard" };
                println!("fuse {verb}: {}", relpath.display());
                Ok(())
            }
            Err(WriteSerializeError::NoPending(p)) => {
                anyhow::bail!(
                    "fuse {}: no pending CoW staging for {}",
                    if commit { "commit" } else { "discard" },
                    p.display()
                );
            }
            Err(e) => Err(e.into()),
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (relpath, mountpoint, commit);
        anyhow::bail!("sharecli-fuse is only supported on Linux and macOS")
    }
}

/// List registered mounts and pending CoW relative paths.
pub fn list(json: bool) -> Result<()> {
    let mounts = FuseSessionRegistry::global().list();
    if json {
        let body: Vec<serde_json::Value> = mounts
            .iter()
            .map(|m| {
                serde_json::json!({
                    "mountpoint": m.mountpoint.display().to_string(),
                    "backing": m.backing.display().to_string(),
                    "session_id": m.session_id,
                    "pending_relpaths": m.pending_relpaths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }
    if mounts.is_empty() {
        println!("fuse list: (no registered mounts)");
        return Ok(());
    }
    for m in &mounts {
        println!("mountpoint: {}", m.mountpoint.display());
        println!("  backing:    {}", m.backing.display());
        println!("  session_id: {}", m.session_id);
        if m.pending_relpaths.is_empty() {
            println!("  pending:    (none)");
        } else {
            println!("  pending:");
            for p in &m.pending_relpaths {
                println!("    - {}", p.display());
            }
        }
        println!();
    }
    Ok(())
}
