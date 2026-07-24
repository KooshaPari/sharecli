//! Shell-free zmx and capability-gated Ghostty adapters.

use std::path::Path;
use std::process::Command;

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
}

pub struct GhosttyAdapter;

impl GhosttyAdapter {
    pub fn degraded_reason(caps: &GhosttyCapabilities) -> Option<&'static str> {
        (!caps.apple_events && !caps.app_intents)
            .then_some("native surface API unavailable")
            .or_else(|| (!caps.control_socket).then_some("native RPC unavailable"))
    }
}
