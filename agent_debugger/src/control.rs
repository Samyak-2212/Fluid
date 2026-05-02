//! Control module — POST /control.
//!
//! Primary path for all UI automation (DEC-C9-003).
//! enigo (input.rs) is the fallback only.
//!
//! Control commands are queued async on C8 — the main loop consumes the queue.
//! Timeout: 5 seconds (DEC-C9-005 / spec §POST /control).

use serde::{Deserialize, Serialize};

use crate::config::AgentDebuggerConfig;
use crate::error::DebugError;
use crate::health::authed_post;

// ── Request / Response types ─────────────────────────────────────────────────

/// A control action sent to `POST /control`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ControlRequest {
    pub action: String,
    /// Widget ID from GET /tree (never hardcoded — DEC-C9-006).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Value string (for set_field / keypress).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Menu path (for menu action).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Vec<String>>,
}

/// Response from `POST /control`.
#[derive(Debug, Deserialize)]
pub struct ControlResponse {
    pub ok: bool,
    #[serde(default)]
    pub error: Option<String>,
}

// ── Constructors ──────────────────────────────────────────────────────────────

impl ControlRequest {
    /// Simulate a click on a widget (ID from /tree).
    pub fn click(widget_id: impl Into<String>) -> Self {
        ControlRequest {
            action: "click".to_string(),
            target: Some(widget_id.into()),
            value: None,
            path: None,
        }
    }

    /// Simulate a key press on a widget (e.g. "Enter", "Escape").
    pub fn keypress(widget_id: impl Into<String>, key: impl Into<String>) -> Self {
        ControlRequest {
            action: "keypress".to_string(),
            target: Some(widget_id.into()),
            value: Some(key.into()),
            path: None,
        }
    }

    /// Set a text input or slider value.
    pub fn set_field(widget_id: impl Into<String>, value: impl Into<String>) -> Self {
        ControlRequest {
            action: "set_field".to_string(),
            target: Some(widget_id.into()),
            value: Some(value.into()),
            path: None,
        }
    }

    /// Activate a menu path (e.g. ["File", "Save"]).
    pub fn menu(path: Vec<String>) -> Self {
        ControlRequest {
            action: "menu".to_string(),
            target: None,
            value: None,
            path: Some(path),
        }
    }
}

// ── Send ──────────────────────────────────────────────────────────────────────

/// Send a control action. Returns Ok(()) if the server accepted it.
///
/// The action is queued async on C8 — not instant. Poll /state afterward
/// to verify the effect.
pub fn send(cfg: &AgentDebuggerConfig, req: &ControlRequest) -> Result<(), DebugError> {
    let url = format!("{}/control", cfg.base_url());
    let body = serde_json::to_string(req)?;

    log::debug!("POST /control — action={} target={:?}", req.action, req.target);

    let resp = authed_post(cfg, &url, &body)?;
    let ctrl_resp: ControlResponse = resp
        .into_json()
        .map_err(|e| DebugError::Json(e.to_string()))?;

    if !ctrl_resp.ok {
        let err = ctrl_resp
            .error
            .unwrap_or_else(|| "unknown error".to_string());
        return Err(DebugError::Protocol(format!("control rejected: {err}")));
    }

    Ok(())
}
