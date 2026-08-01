//! Explicit launch-time state evidence for harness session recovery.

use crate::{SessionStateProvider, SurfaceRecord};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// One deliberate surface-to-harness mapping written by a launcher or wrapper.
///
/// The surface id is mandatory. A PID, when present, prevents a stale mapping
/// from being applied after a terminal surface has been recycled.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct SidecarRecord {
    pub surface_id: String,
    pub harness: String,
    pub session_id: String,
    #[serde(default)]
    pub pid: Option<u32>,
}

/// Read-only JSONL provider for exact launch-time session mappings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidecarStateProvider {
    path: PathBuf,
}

impl SidecarStateProvider {
    /// Construct a provider. The file is read on each lookup so a long-running
    /// watcher observes wrapper registrations without a restart.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn records(&self) -> Result<Vec<SidecarRecord>> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => {
                return Err(error).with_context(|| format!("read sidecar {}", self.path.display()))
            }
        };
        text.lines()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty())
            .map(|(line_number, line)| {
                serde_json::from_str(line).with_context(|| {
                    format!("parse sidecar {} line {}", self.path.display(), line_number + 1)
                })
            })
            .collect()
    }
}

impl SessionStateProvider for SidecarStateProvider {
    fn session_id(&self, surface: &SurfaceRecord, harness: &str) -> Result<Option<String>> {
        let pid = surface.process.as_ref().and_then(|process| process.pid);
        let mut match_record = None;
        for record in self.records()? {
            if record.surface_id != surface.id || record.harness != harness {
                continue;
            }
            // JSONL is append-only: the last record for a surface/harness is
            // authoritative. A newer PID mismatch must not fall back to an
            // older mapping from a recycled process.
            match_record = Some(record);
        }
        Ok(match_record.and_then(|record| {
            if record.pid.is_some() && record.pid != pid {
                return None;
            }
            (!record.session_id.trim().is_empty()).then_some(record.session_id)
        }))
    }
}
