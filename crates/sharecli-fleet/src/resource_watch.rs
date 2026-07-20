//! Host FD / network resource watch sampling (FR-007).
//!
//! Used by [`sharecli_core::Hypervisor::run`] for live watch signals and
//! re-exported from `sharecli::monitoring` for ProcessStats enrichment.

use anyhow::{Context, Result};

/// Point-in-time FD and host network byte counters for resource watch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResourceWatchSample {
    pub fd_count: u64,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
}

impl ResourceWatchSample {
    /// Capture FD and host network byte counters for the current process/host.
    pub fn capture() -> Result<Self> {
        let fd_count = sample_self_fds()?;
        let (net_rx_bytes, net_tx_bytes) = sample_host_net()?;
        Ok(Self {
            fd_count,
            net_rx_bytes,
            net_tx_bytes,
        })
    }
}

/// Count open file descriptors for the current process.
pub fn sample_self_fds() -> Result<u64> {
    sample_self_fds_impl()
}

/// Sum host-wide network RX/TX byte counters (non-loopback where applicable).
pub fn sample_host_net() -> Result<(u64, u64)> {
    sample_host_net_impl()
}

#[cfg(target_os = "linux")]
fn sample_self_fds_impl() -> Result<u64> {
    let entries = std::fs::read_dir("/proc/self/fd")
        .context("failed to read /proc/self/fd for FD watch")?;
    Ok(entries.count() as u64)
}

#[cfg(target_os = "macos")]
fn sample_self_fds_impl() -> Result<u64> {
    let entries =
        std::fs::read_dir("/dev/fd").context("failed to read /dev/fd for FD watch on macOS")?;
    Ok(entries.count() as u64)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn sample_self_fds_impl() -> Result<u64> {
    anyhow::bail!("sample_self_fds is unsupported on this OS")
}

#[cfg(target_os = "linux")]
fn sample_host_net_impl() -> Result<(u64, u64)> {
    let contents =
        std::fs::read_to_string("/proc/net/dev").context("failed to read /proc/net/dev")?;
    let mut rx_total = 0u64;
    let mut tx_total = 0u64;

    for line in contents.lines().skip(2) {
        let Some((iface, rest)) = line.split_once(':') else {
            continue;
        };
        let iface = iface.trim();
        if iface == "lo" {
            continue;
        }
        let fields: Vec<&str> = rest.split_whitespace().collect();
        if fields.len() < 9 {
            continue;
        }
        let rx: u64 = fields[0]
            .parse()
            .with_context(|| format!("invalid RX bytes for interface {iface}"))?;
        let tx: u64 = fields[8]
            .parse()
            .with_context(|| format!("invalid TX bytes for interface {iface}"))?;
        rx_total = rx_total.saturating_add(rx);
        tx_total = tx_total.saturating_add(tx);
    }

    Ok((rx_total, tx_total))
}

#[cfg(target_os = "macos")]
fn sample_host_net_impl() -> Result<(u64, u64)> {
    use std::process::Command;

    let output = Command::new("netstat")
        .args(["-ib"])
        .output()
        .context("failed to spawn netstat -ib for network watch")?;
    if !output.status.success() {
        anyhow::bail!(
            "netstat -ib exited with status {}",
            output.status.code().unwrap_or(-1)
        );
    }

    let stdout = String::from_utf8(output.stdout)
        .context("netstat -ib output was not valid UTF-8")?;
    let mut rx_total = 0u64;
    let mut tx_total = 0u64;

    for line in stdout.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 10 {
            continue;
        }
        let name = cols[0];
        if name == "Name" || name.starts_with("lo") {
            continue;
        }
        let rx: u64 = cols[6]
            .parse()
            .with_context(|| format!("invalid Ibytes for interface {name}"))?;
        let tx: u64 = cols[9]
            .parse()
            .with_context(|| format!("invalid Obytes for interface {name}"))?;
        rx_total = rx_total.saturating_add(rx);
        tx_total = tx_total.saturating_add(tx);
    }

    Ok((rx_total, tx_total))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn sample_host_net_impl() -> Result<(u64, u64)> {
    anyhow::bail!("sample_host_net is unsupported on this OS")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_watch_sample_capture() {
        let sample = ResourceWatchSample::capture().expect("resource watch sample");
        assert!(sample.fd_count >= 3, "process MUST have stdin/stdout/stderr FDs");
    }

    #[test]
    fn test_sample_self_fds() {
        let fds = super::sample_self_fds().expect("FD sample");
        assert!(fds >= 3, "open FD count MUST include stdio");
    }

    #[test]
    fn test_sample_host_net() {
        let (rx, tx) = super::sample_host_net().expect("network sample");
        let _ = (rx, tx);
    }
}
