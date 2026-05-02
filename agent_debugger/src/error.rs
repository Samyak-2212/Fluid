//! Shared error type for agent_debugger.
//!
//! All fallible operations return `Result<_, DebugError>`.
//! `unwrap()` is banned across this crate (DEC-C9-004).

use std::fmt;

/// Structured error type for all agent_debugger operations.
#[derive(Debug)]
#[allow(dead_code)]
pub enum DebugError {
    /// HTTP transport error (connection refused, timeout, etc.).
    Http(String),
    /// JSON parse or serialise error.
    Json(String),
    /// Filesystem I/O error.
    Io(String),
    /// Protocol version mismatch or unexpected response shape.
    Protocol(String),
    /// Operation timed out.
    Timeout(String),
    /// Feature not implemented in C8 (e.g. screenshot stub).
    NotImplemented(String),
    /// Rate-limited by server (HTTP 429).
    RateLimited,
    /// Authentication failed (HTTP 401).
    Unauthorized,
    /// Config file missing or malformed.
    Config(String),
}

impl fmt::Display for DebugError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DebugError::Http(s) => write!(f, "HTTP error: {s}"),
            DebugError::Json(s) => write!(f, "JSON error: {s}"),
            DebugError::Io(s) => write!(f, "I/O error: {s}"),
            DebugError::Protocol(s) => write!(f, "Protocol error: {s}"),
            DebugError::Timeout(s) => write!(f, "Timeout: {s}"),
            DebugError::NotImplemented(s) => write!(f, "Not implemented: {s}"),
            DebugError::RateLimited => write!(f, "Rate limited (429) — wait 500ms"),
            DebugError::Unauthorized => write!(f, "Unauthorized (401) — check debug_server_token"),
            DebugError::Config(s) => write!(f, "Config error: {s}"),
        }
    }
}

impl From<ureq::Error> for DebugError {
    fn from(e: ureq::Error) -> Self {
        match &e {
            ureq::Error::Status(429, _) => DebugError::RateLimited,
            ureq::Error::Status(401, _) => DebugError::Unauthorized,
            ureq::Error::Status(code, resp) => {
                DebugError::Http(format!("HTTP {code}: {}", resp.status_text()))
            }
            ureq::Error::Transport(t) => DebugError::Http(t.to_string()),
        }
    }
}

impl From<serde_json::Error> for DebugError {
    fn from(e: serde_json::Error) -> Self {
        DebugError::Json(e.to_string())
    }
}

impl From<std::io::Error> for DebugError {
    fn from(e: std::io::Error) -> Self {
        DebugError::Io(e.to_string())
    }
}
