//! Health check module — GET /health.
//!
//! Every agent_debugger operation calls this first (DEC-C9-005).
//! Aborts if `protocol_version != expected` (default: 1).

use serde::Deserialize;

use crate::config::AgentDebuggerConfig;
use crate::error::DebugError;

/// Response shape for `GET /health`.
#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub protocol_version: u32,
}

/// Call `GET /health` and verify protocol_version matches `cfg.expected_protocol_version`.
///
/// Returns `Ok(HealthResponse)` on success.
/// Returns `Err(DebugError::Protocol)` if the version does not match.
pub fn check(cfg: &AgentDebuggerConfig) -> Result<HealthResponse, DebugError> {
    let url = format!("{}/health", cfg.base_url());
    let agent = build_agent(cfg);

    let resp = agent
        .get(&url)
        .call()
        .map_err(DebugError::from)?;

    let health: HealthResponse = resp
        .into_json()
        .map_err(|e| DebugError::Json(e.to_string()))?;

    if health.protocol_version != cfg.expected_protocol_version {
        return Err(DebugError::Protocol(format!(
            "protocol_version mismatch: server={}, expected={}",
            health.protocol_version, cfg.expected_protocol_version
        )));
    }

    log::debug!("Health OK — protocol_version={}", health.protocol_version);
    Ok(health)
}

/// Build a ureq agent with the configured timeout and optional auth header.
pub(crate) fn build_agent(cfg: &AgentDebuggerConfig) -> ureq::Agent {
    let mut builder = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(cfg.timeout_secs));

    if !cfg.token.is_empty() {
        // Token is injected per-request; ureq AgentBuilder cannot set default headers,
        // so we return a plain agent and let call sites add the header.
        // The token is attached by `authed_get`/`authed_post` helpers.
        let _ = builder; // builder consumed below
        builder = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(cfg.timeout_secs));
    }

    builder.build()
}

/// GET request with optional X-Debug-Token header.
pub(crate) fn authed_get(
    cfg: &AgentDebuggerConfig,
    url: &str,
) -> Result<ureq::Response, DebugError> {
    let agent = build_agent(cfg);
    let req = agent.get(url);
    let req = if cfg.token.is_empty() {
        req
    } else {
        req.set("X-Debug-Token", &cfg.token)
    };
    req.call().map_err(DebugError::from)
}

/// POST request with optional X-Debug-Token header.
pub(crate) fn authed_post(
    cfg: &AgentDebuggerConfig,
    url: &str,
    body: &str,
) -> Result<ureq::Response, DebugError> {
    let agent = build_agent(cfg);
    let req = agent.post(url).set("Content-Type", "application/json");
    let req = if cfg.token.is_empty() {
        req
    } else {
        req.set("X-Debug-Token", &cfg.token)
    };
    req.send_string(body).map_err(DebugError::from)
}
