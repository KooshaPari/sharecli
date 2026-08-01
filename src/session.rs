//! Shell-free zmx and capability-gated Ghostty adapters.

use serde_json::{json, Value};
use sharecli_session::{SurfaceAdapter, SurfaceCapabilities, SurfaceRecord};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZmxCommand {
    pub program: String,
    pub args: Vec<String>,
}

impl ZmxCommand {
    pub fn new<I, S>(program: impl Into<String>, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self { program: program.into(), args: args.into_iter().map(Into::into).collect() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZmxCapabilities {
    pub available: bool,
    pub durable_pty: bool,
    pub unix_socket: bool,
    pub history: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZmxSessionAdapter {
    binary: String,
}

impl ZmxSessionAdapter {
    pub fn new(binary: impl Into<String>) -> Self {
        Self { binary: binary.into() }
    }
    pub fn attach(&self, name: &str, command: &[&str]) -> ZmxCommand {
        let mut args = vec!["attach".to_owned(), name.to_owned()];
        args.extend(command.iter().map(|arg| (*arg).to_owned()));
        ZmxCommand::new(self.binary.clone(), args)
    }
    pub fn send(&self, name: &str, input: &str) -> ZmxCommand {
        ZmxCommand::new(self.binary.clone(), ["send", name, input])
    }
    pub fn tail(&self, name: &str, lines: Option<u32>) -> ZmxCommand {
        let mut args = vec!["tail".to_owned()];
        if let Some(lines) = lines {
            args.extend(["--lines".to_owned(), lines.to_string()]);
        }
        args.push(name.to_owned());
        ZmxCommand::new(self.binary.clone(), args)
    }
    pub fn history(&self, name: &str, vt: bool) -> ZmxCommand {
        let mut args = vec!["history".to_owned()];
        if vt {
            args.push("--vt".to_owned());
        }
        args.push(name.to_owned());
        ZmxCommand::new(self.binary.clone(), args)
    }
    pub fn capabilities(&self, probe_socket: bool) -> ZmxCapabilities {
        let available = command_available(&self.binary);
        ZmxCapabilities {
            available,
            durable_pty: available,
            unix_socket: available && probe_socket,
            history: available,
        }
    }
}

fn command_available(binary: &str) -> bool {
    Path::new(binary).is_file()
        || std::env::var_os("PATH")
            .is_some_and(|path| std::env::split_paths(&path).any(|dir| dir.join(binary).is_file()))
}

pub fn execute(command: &ZmxCommand) -> std::io::Result<std::process::Output> {
    Command::new(&command.program).args(&command.args).output()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GhosttyCapabilities {
    pub apple_events: bool,
    pub app_intents: bool,
    pub accessibility_readback: bool,
    pub control_socket: bool,
}

impl GhosttyCapabilities {
    pub fn from_probe(apple_events: bool, app_intents: bool, accessibility_readback: bool) -> Self {
        Self { apple_events, app_intents, accessibility_readback, control_socket: false }
    }

    pub fn with_control_socket(mut self, available: bool) -> Self {
        self.control_socket = available;
        self
    }
}

/// Minimal JSON-RPC client for a ShareCLI-enabled Ghostty control socket.
///
/// A stock Ghostty install reports an unavailable socket; callers must then
/// use a degraded caster or zmx rather than pretending to have pane readback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GhosttyControlClient {
    socket: PathBuf,
    token: Option<String>,
}

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

impl GhosttyControlClient {
    pub fn new(socket: impl Into<PathBuf>, token: Option<String>) -> Self {
        Self { socket: socket.into(), token }
    }

    pub fn socket(&self) -> &Path {
        &self.socket
    }

    pub fn request(&self, method: &str, params: Value) -> anyhow::Result<Value> {
        #[cfg(unix)]
        {
            let mut stream = std::os::unix::net::UnixStream::connect(&self.socket)?;
            let id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
            let mut request = json!({"id": id, "method": method, "params": params});
            if let Some(token) = &self.token {
                request["token"] = Value::String(token.clone());
            }
            serde_json::to_writer(&mut stream, &request)?;
            stream.write_all(b"\n")?;
            stream.flush()?;
            let mut response = String::new();
            BufReader::new(stream).read_line(&mut response)?;
            let response: Value = serde_json::from_str(&response)?;
            if let Some(error) = response.get("error") {
                anyhow::bail!("Ghostty RPC {method} failed: {error}");
            }
            Ok(response.get("result").cloned().unwrap_or(Value::Null))
        }
        #[cfg(not(unix))]
        {
            let _ = (method, params);
            anyhow::bail!("Ghostty control sockets require a Unix platform")
        }
    }

    pub fn send_text(&self, surface_id: &str, text: &str) -> anyhow::Result<()> {
        self.request("surface.io.send", json!({"surface_id": surface_id, "text": text}))?;
        Ok(())
    }

    pub fn list_surfaces(&self) -> anyhow::Result<Vec<SurfaceRecord>> {
        Ok(serde_json::from_value(self.request("surface.list", json!({}))?)?)
    }

    pub fn surface_capabilities(&self, surface_id: &str) -> anyhow::Result<SurfaceCapabilities> {
        Ok(serde_json::from_value(
            self.request("surface.io.capabilities", json!({"surface_id": surface_id}))?,
        )?)
    }

    pub fn read_surface(&self, surface_id: &str, max_bytes: usize) -> anyhow::Result<Value> {
        self.request("surface.io.read", json!({"surface_id": surface_id, "max_bytes": max_bytes}))
    }

    pub fn resize(&self, surface_id: &str, rows: u16, cols: u16) -> anyhow::Result<()> {
        self.request(
            "surface.io.resize",
            json!({"surface_id": surface_id, "rows": rows, "cols": cols}),
        )?;
        Ok(())
    }
}

impl SurfaceAdapter for GhosttyControlClient {
    fn capabilities(&self, surface: &SurfaceRecord) -> anyhow::Result<SurfaceCapabilities> {
        self.surface_capabilities(&surface.id)
    }

    fn discover(&self) -> anyhow::Result<Vec<SurfaceRecord>> {
        self.list_surfaces()
    }
}

pub struct GhosttyAdapter;

impl GhosttyAdapter {
    pub fn degraded_reason(caps: &GhosttyCapabilities) -> Option<&'static str> {
        (!caps.apple_events && !caps.app_intents && !caps.control_socket)
            .then_some("native surface API unavailable")
            .or_else(|| (!caps.control_socket).then_some("native RPC unavailable"))
    }
}
