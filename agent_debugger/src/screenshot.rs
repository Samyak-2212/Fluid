//! Screenshot module — GET /screenshot.
//!
//! C8 GET /screenshot returns:
//!  - 200 image/png  → success (raw PNG bytes)
//!  - 200 application/json { "ok": false, "error": "screenshot_not_implemented" }
//!    → xcap stub in C8, graceful fallback (DEC-C9-010)
//!  - 200 application/json { "ok": false, "error": "no_display", "headless": true }
//!    → headless mode
//!  - 429 → rate limited (DEC-C9-009)
//!
//! Rate limiting: minimum 500ms between calls (DEC-C9-009).
//! The xcap-based local capture path is also available for direct window capture.

use std::path::Path;
use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::config::AgentDebuggerConfig;
use crate::error::DebugError;

/// Result of a screenshot attempt.
#[derive(Debug)]
pub enum ScreenshotResult {
    /// PNG bytes fetched from server.
    Png(Vec<u8>),
    /// Server screenshot not implemented (xcap stub) — fallback mode.
    NotImplemented,
    /// Headless mode — no display.
    Headless,
}

/// JSON error response from /screenshot when not returning PNG.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ScreenshotError {
    ok: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    headless: bool,
}

/// Rate-limit tracker — call `check_rate_limit` before requesting.
pub struct ScreenshotRateLimit {
    last: Option<Instant>,
    min_interval: Duration,
}

impl ScreenshotRateLimit {
    pub fn new(min_interval_ms: u64) -> Self {
        ScreenshotRateLimit {
            last: None,
            min_interval: Duration::from_millis(min_interval_ms),
        }
    }

    /// Sleep if needed to respect the rate limit.
    pub fn wait(&mut self) {
        if let Some(last) = self.last {
            let elapsed = last.elapsed();
            if elapsed < self.min_interval {
                std::thread::sleep(self.min_interval - elapsed);
            }
        }
        self.last = Some(Instant::now());
    }
}

/// Fetch a screenshot from the server.
///
/// Returns `ScreenshotResult::NotImplemented` gracefully when C8's xcap is stubbed.
/// Callers should handle all three variants without unwrap.
pub fn fetch(
    cfg: &AgentDebuggerConfig,
    rate_limit: &mut ScreenshotRateLimit,
) -> Result<ScreenshotResult, DebugError> {
    rate_limit.wait();

    let url = format!("{}/screenshot", cfg.base_url());

    // Build a plain ureq agent with timeout.
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(cfg.timeout_secs))
        .build();

    let req = agent.get(&url);
    let req = if cfg.token.is_empty() {
        req
    } else {
        req.set("X-Debug-Token", &cfg.token)
    };

    let resp = match req.call() {
        Ok(r) => r,
        Err(ureq::Error::Status(429, _)) => return Err(DebugError::RateLimited),
        Err(ureq::Error::Status(401, _)) => return Err(DebugError::Unauthorized),
        Err(e) => return Err(DebugError::from(e)),
    };

    let content_type = resp
        .header("Content-Type")
        .unwrap_or("")
        .to_string();

    if content_type.contains("image/png") {
        // Raw PNG bytes
        let mut bytes = Vec::new();
        use std::io::Read;
        resp.into_reader()
            .read_to_end(&mut bytes)
            .map_err(|e| DebugError::Io(e.to_string()))?;
        log::debug!("Screenshot received: {} bytes PNG", bytes.len());
        return Ok(ScreenshotResult::Png(bytes));
    }

    // JSON error response
    let err_resp: ScreenshotError = resp
        .into_json()
        .map_err(|e| DebugError::Json(e.to_string()))?;

    if err_resp.headless {
        log::debug!("Screenshot: headless mode — no display");
        return Ok(ScreenshotResult::Headless);
    }

    let error_str = err_resp.error.as_deref().unwrap_or("unknown");
    if error_str == "screenshot_not_implemented" {
        log::debug!("Screenshot: xcap not implemented in C8 yet — fallback mode");
        return Ok(ScreenshotResult::NotImplemented);
    }

    Err(DebugError::Protocol(format!(
        "screenshot error: {}",
        error_str
    )))
}

/// Save PNG bytes to `path`. Creates parent dirs if needed.
pub fn save_png(bytes: &[u8], path: &Path) -> Result<(), DebugError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| DebugError::Io(format!("create_dir_all: {e}")))?;
    }
    std::fs::write(path, bytes)
        .map_err(|e| DebugError::Io(format!("write PNG to {}: {e}", path.display())))?;
    log::debug!("Screenshot saved: {}", path.display());
    Ok(())
}
