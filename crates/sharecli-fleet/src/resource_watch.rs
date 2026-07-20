//! Host FD / network resource watch sampling (FR-007).
//!
//! Used by [`sharecli_core::Hypervisor::run`] for live watch signals and
//! re-exported from `sharecli::monitoring` for ProcessStats enrichment.

use anyhow::{Context, Result};

/// Point-in-time CPU/MEM/Net/FD resource watch sample (FR-007).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ResourceWatchSample {
    pub fd_count: u64,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
    pub mem_rss_bytes: u64,
    pub load_1m: f64,
}

impl ResourceWatchSample {
    /// Capture FD, network, RSS, and host load for the current process/host.
    pub fn capture() -> Result<Self> {
        let fd_count = sample_self_fds()?;
        let (net_rx_bytes, net_tx_bytes) = sample_host_net()?;
        let mem_rss_bytes = sample_self_rss_bytes()?;
        let load_1m = sample_host_load_1m()?;
        Ok(Self {
            fd_count,
            net_rx_bytes,
            net_tx_bytes,
            mem_rss_bytes,
            load_1m,
        })
    }

    /// Operator-facing status block for `sharecli status` (FR-007 / AC-007.10).
    pub fn format_status_section(self) -> String {
        let mut out = String::from("\n=== Host Resource Watch ===\n\n");
        out.push_str(&format!(
            "Open FDs:     {}\nRSS:          {} bytes\nLoad (1m):    {:.2}\nNet RX:       {} bytes\nNet TX:       {} bytes\n",
            self.fd_count,
            self.mem_rss_bytes,
            self.load_1m,
            self.net_rx_bytes,
            self.net_tx_bytes,
        ));
        out
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

/// Resident set size (RSS) in bytes for the current process.
pub fn sample_self_rss_bytes() -> Result<u64> {
    sample_self_rss_bytes_impl()
}

/// Host 1-minute load average.
pub fn sample_host_load_1m() -> Result<f64> {
    sample_host_load_1m_impl()
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

#[cfg(target_os = "linux")]
fn sample_self_rss_bytes_impl() -> Result<u64> {
    let status =
        std::fs::read_to_string("/proc/self/status").context("failed to read /proc/self/status")?;
    for line in status.lines() {
        let Some(rest) = line.strip_prefix("VmRSS:") else {
            continue;
        };
        let kb: u64 = rest
            .trim()
            .trim_end_matches(" kB")
            .parse()
            .context("invalid VmRSS in /proc/self/status")?;
        return Ok(kb.saturating_mul(1024));
    }
    anyhow::bail!("VmRSS not found in /proc/self/status")
}

#[cfg(target_os = "macos")]
fn sample_self_rss_bytes_impl() -> Result<u64> {
    use libc::{getrusage, rusage, RUSAGE_SELF};

    let mut usage: rusage = unsafe { std::mem::zeroed() };
    let rc = unsafe { getrusage(RUSAGE_SELF, &mut usage) };
    if rc != 0 {
        anyhow::bail!("getrusage(RUSAGE_SELF) failed with status {rc}");
    }
    // macOS reports ru_maxrss in bytes.
    Ok(usage.ru_maxrss as u64)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn sample_self_rss_bytes_impl() -> Result<u64> {
    anyhow::bail!("sample_self_rss_bytes is unsupported on this OS")
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn sample_host_load_1m_impl() -> Result<f64> {
    use libc::{c_double, getloadavg};

    let mut loads = [0.0f64 as c_double; 3];
    let count = unsafe { getloadavg(loads.as_mut_ptr(), 3) };
    if count < 1 {
        anyhow::bail!("getloadavg returned no load samples");
    }
    Ok(loads[0])
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn sample_host_load_1m_impl() -> Result<f64> {
    anyhow::bail!("sample_host_load_1m is unsupported on this OS")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_watch_sample_capture() {
        let sample = ResourceWatchSample::capture().expect("resource watch sample");
        assert!(sample.fd_count >= 3, "process MUST have stdin/stdout/stderr FDs");
        assert!(sample.mem_rss_bytes > 0, "RSS MUST be non-zero for live process");
        assert!(sample.load_1m >= 0.0, "load average MUST be sampled");
    }

    #[test]
    fn test_sample_self_rss_bytes() {
        let rss = super::sample_self_rss_bytes().expect("RSS sample");
        assert!(rss > 0, "live process MUST have non-zero RSS");
    }

    #[test]
    fn test_sample_host_load_1m() {
        let load = super::sample_host_load_1m().expect("load sample");
        assert!(load >= 0.0);
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

    #[test]
    fn test_format_status_section() {
        let sample = ResourceWatchSample::capture().expect("resource watch sample");
        let section = sample.format_status_section();
        assert!(section.contains("=== Host Resource Watch ==="));
        assert!(section.contains("Open FDs:"));
        assert!(section.contains("RSS:"));
        assert!(section.contains("Load (1m):"));
        assert!(section.contains("Net RX:"));
        assert!(section.contains("Net TX:"));
    }
}
