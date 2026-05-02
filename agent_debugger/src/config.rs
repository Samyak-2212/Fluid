//! Config loader for agent_debugger.
//!
//! Reads `config/agent_debugger.toml` relative to the workspace root.
//! All tunables come from this file — no hardcoded values (DEC-C9-008).

use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::error::DebugError;

/// Runtime configuration for agent_debugger.
#[derive(Debug, Deserialize, Clone)]
pub struct AgentDebuggerConfig {
    /// Debug server host (default: 127.0.0.1).
    #[serde(default = "default_host")]
    pub host: String,
    /// Debug server port (default: 8082).
    #[serde(default = "default_port")]
    pub port: u16,
    /// Optional auth token — empty string means no auth.
    #[serde(default)]
    pub token: String,
    /// HTTP request timeout in seconds (default: 10).
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    /// Minimum gap between screenshot calls in milliseconds (default: 500).
    #[serde(default = "default_screenshot_min_interval_ms")]
    pub screenshot_min_interval_ms: u64,
    /// Protocol version this tool expects (default: 1).
    #[serde(default = "default_protocol_version")]
    pub expected_protocol_version: u32,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}
fn default_port() -> u16 {
    8082
}
fn default_timeout_secs() -> u64 {
    10
}
fn default_screenshot_min_interval_ms() -> u64 {
    500
}
fn default_protocol_version() -> u32 {
    1
}

impl Default for AgentDebuggerConfig {
    fn default() -> Self {
        AgentDebuggerConfig {
            host: default_host(),
            port: default_port(),
            token: String::new(),
            timeout_secs: default_timeout_secs(),
            screenshot_min_interval_ms: default_screenshot_min_interval_ms(),
            expected_protocol_version: default_protocol_version(),
        }
    }
}

impl AgentDebuggerConfig {
    /// Base URL of the debug server, e.g. `http://127.0.0.1:8082`.
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    /// Load from `config/agent_debugger.toml`, walking up from `start_dir` to find workspace root.
    pub fn load(start_dir: &Path) -> Result<Self, DebugError> {
        let config_path = find_config(start_dir)?;
        let text = std::fs::read_to_string(&config_path)
            .map_err(|e| DebugError::Config(format!("cannot read {}: {e}", config_path.display())))?;
        let cfg: AgentDebuggerConfig = toml::from_str(&text)
            .map_err(|e| DebugError::Config(format!("TOML parse error in {}: {e}", config_path.display())))?;
        Ok(cfg)
    }

    /// Load config or fall back to defaults silently.
    pub fn load_or_default(start_dir: &Path) -> Self {
        Self::load(start_dir).unwrap_or_default()
    }
}

/// Walk parent dirs until we find a directory that contains `config/agent_debugger.toml`.
fn find_config(start: &Path) -> Result<PathBuf, DebugError> {
    let mut dir = start.to_path_buf();
    loop {
        let candidate = dir.join("config").join("agent_debugger.toml");
        if candidate.exists() {
            return Ok(candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    Err(DebugError::Config(
        "config/agent_debugger.toml not found (walked to filesystem root)".to_string(),
    ))
}
