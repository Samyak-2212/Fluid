//! Debug HTTP server for the Fluid application.
//!
//! Runs at `127.0.0.1:8082` (configurable via `config/app.toml`, DEC-009).
//! Bound to loopback only — never exposed to the network.
//!
//! Used by C9 (Agent Debugger) to inspect and control the application.
//!
//! # Endpoints
//! - `GET /health`     — protocol handshake (version JSON)
//! - `GET /screenshot` — PNG binary (rate-limited 2/sec; headless → error JSON)
//! - `GET /state`      — JSON snapshot of AppStateSnapshot (frame-boundary update)
//! - `GET /tree`       — JSON widget registry (C8-UI widget IDs)
//! - `POST /control`   — action injection (click, keypress, set_field, menu)
//! - `GET /logs`       — log tail in C6 log format
//! - `GET /dashboard`  — embedded HTML dashboard
//!
//! # Thread safety
//! The server reads `Arc<RwLock<AppStateSnapshot>>` written by the main loop at
//! each frame boundary. The widget registry is an `Arc<RwLock<WidgetRegistry>>`
//! owned by C8-UI and shared here via clone.

#[allow(unused_imports)]
use std::io::Read;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tiny_http::{Header, Method, Response, Server, StatusCode};

use crate::ui::widget_registry::WidgetRegistry;

/// Current debug protocol version. C9 checks this in /health before any operation.
pub const PROTOCOL_VERSION: u32 = 1;

/// Rate limit for /screenshot endpoint (max 2 per second).
const SCREENSHOT_RATE_LIMIT: Duration = Duration::from_millis(500);

const DASHBOARD_HTML: &str = include_str!("../assets/dashboard.html");

// ── State snapshot ────────────────────────────────────────────────────────────

/// Frame-boundary snapshot of application state.
///
/// Written by the main loop at the end of every rendered frame.
/// The debug server reads this via `Arc<RwLock<>>` — never blocks the render loop.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppStateSnapshot {
    /// Most recent frame number.
    pub frame: u64,
    /// Simulation time in seconds.
    pub sim_time: f64,
    /// Current simulation status string.
    pub sim_status: String,
    /// Scene name.
    pub scene_name: String,
    /// Number of entities in the active scene.
    pub entity_count: u64,
    /// Whether the app is in headless mode.
    pub headless: bool,
    /// Active capability tier (0–3).
    pub tier: u32,
    /// FPS (exponential moving average).
    pub fps: f64,
}

// ── Control action ────────────────────────────────────────────────────────────

/// An action sent by C9 via `POST /control`.
#[derive(Debug, Deserialize)]
pub struct ControlAction {
    /// Action type: "click", "keypress", "set_field", "menu"
    pub action: String,
    /// Target widget ID (from the widget registry).
    pub target: Option<String>,
    /// Value for "set_field" or key name for "keypress".
    pub value: Option<String>,
    /// Menu path for "menu" (e.g. `["File", "Save"]`).
    pub path: Option<Vec<String>>,
}

// ── Pending control queue ─────────────────────────────────────────────────────

/// A queued control action with its submission timestamp (for timeout tracking).
#[derive(Debug)]
pub struct PendingControl {
    pub action: ControlAction,
    pub submitted_at: Instant,
}

// ── Debug server ──────────────────────────────────────────────────────────────

/// The C8 debug HTTP server.
///
/// Spawned in a background thread via `DebugServer::start`. The main loop
/// writes to `state_snapshot` at each frame boundary. The server reads it
/// without blocking the render loop.
pub struct DebugServer {
    port: u16,
    token: String,
    headless: bool,
    state_snapshot: Arc<RwLock<AppStateSnapshot>>,
    widget_registry: Arc<RwLock<WidgetRegistry>>,
    /// Pending control actions queued by C9, consumed by main loop.
    pub pending_controls: Arc<RwLock<Vec<PendingControl>>>,
}

impl DebugServer {
    /// Creates a new `DebugServer`.
    ///
    /// `port`: loopback port (default 8082 from `config/app.toml`).
    /// `token`: optional shared secret. If empty, no auth check is performed.
    /// `headless`: if true, /screenshot returns an error; /tree returns empty.
    pub fn new(
        port: u16,
        token: String,
        headless: bool,
        state_snapshot: Arc<RwLock<AppStateSnapshot>>,
        widget_registry: Arc<RwLock<WidgetRegistry>>,
    ) -> Self {
        Self {
            port,
            token,
            headless,
            state_snapshot,
            widget_registry,
            pending_controls: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Starts the debug server on a background thread. Non-blocking.
    /// Returns the `Arc<RwLock<Vec<PendingControl>>>` that the main loop must poll.
    pub fn start(self) -> Arc<RwLock<Vec<PendingControl>>> {
        let pending = Arc::clone(&self.pending_controls);

        thread::spawn(move || {
            let bind_addr = format!("127.0.0.1:{}", self.port);
            let server = Server::http(&bind_addr).unwrap_or_else(|e| {
                panic!("DebugServer: failed to bind {bind_addr}: {e}");
            });

            log::info!("DebugServer listening on http://{bind_addr}");

            let mut last_screenshot = Instant::now().checked_sub(SCREENSHOT_RATE_LIMIT).unwrap_or(Instant::now());

            for mut request in server.incoming_requests() {
                // Optional token auth — checked first before any routing.
                if !self.token.is_empty() {
                    let auth_ok = request
                        .headers()
                        .iter()
                        .any(|h| {
                            h.field.equiv("X-Debug-Token")
                                && h.value.as_str() == self.token
                        });
                    if !auth_ok {
                        let _ = request.respond(
                            Response::from_string(json!({"ok": false, "error": "unauthorized"}).to_string())
                                .with_status_code(StatusCode(401))
                                .with_header(json_ct()),
                        );
                        continue;
                    }
                }

                let url = request.url().to_string();
                let method = request.method().clone();

                match (method, url.as_str()) {
                    // ── GET /health ──────────────────────────────────────────
                    (Method::Get, "/health") => {
                        let body = json!({
                            "ok": true,
                            "protocol_version": PROTOCOL_VERSION
                        });
                        let _ = request.respond(json_response(body.to_string()));
                    }

                    // ── GET /state ───────────────────────────────────────────
                    (Method::Get, "/state") => {
                        let snap = self.state_snapshot
                            .read()
                            .map(|s| s.clone())
                            .unwrap_or_default();
                        let body = serde_json::to_string(&snap)
                            .unwrap_or_else(|_| json!({"error": "serialize_failed"}).to_string());
                        let _ = request.respond(json_response(body));
                    }

                    // ── GET /tree ────────────────────────────────────────────
                    (Method::Get, "/tree") => {
                        let tree = if self.headless {
                            json!({ "headless": true, "widgets": [] })
                        } else {
                            let reg = self.widget_registry
                                .read()
                                .map(|r| r.snapshot())
                                .unwrap_or_default();
                            json!({ "headless": false, "widgets": reg })
                        };
                        let _ = request.respond(json_response(tree.to_string()));
                    }

                    // ── GET /screenshot ──────────────────────────────────────
                    (Method::Get, "/screenshot") => {
                        if self.headless {
                            let body = json!({
                                "ok": false,
                                "error": "no_display",
                                "headless": true
                            });
                            let _ = request.respond(json_response(body.to_string()));
                            continue;
                        }
                        let now = Instant::now();
                        if now.duration_since(last_screenshot) < SCREENSHOT_RATE_LIMIT {
                            let body = json!({"ok": false, "error": "rate_limited"});
                            let _ = request.respond(
                                Response::from_string(body.to_string())
                                    .with_status_code(StatusCode(429))
                                    .with_header(json_ct()),
                            );
                            continue;
                        }
                        last_screenshot = now;
                        // [NEEDS_REVIEW: claude] — xcap screen capture; headless guard above.
                        match capture_screenshot() {
                            Ok(png_bytes) => {
                                let _ = request.respond(
                                    Response::from_data(png_bytes)
                                        .with_header(
                                            Header::from_bytes(
                                                &b"Content-Type"[..],
                                                &b"image/png"[..],
                                            )
                                            .unwrap(),
                                        ),
                                );
                            }
                            Err(e) => {
                                let body = json!({"ok": false, "error": e});
                                let _ = request.respond(
                                    Response::from_string(body.to_string())
                                        .with_status_code(StatusCode(500))
                                        .with_header(json_ct()),
                                );
                            }
                        }
                    }

                    // ── POST /control ────────────────────────────────────────
                    (Method::Post, "/control") => {
                        let mut body = String::new();
                        let _ = request.as_reader().read_to_string(&mut body);
                        match serde_json::from_str::<ControlAction>(&body) {
                            Ok(action) => {
                                let pending_control = PendingControl {
                                    action,
                                    submitted_at: Instant::now(),
                                };
                                self.pending_controls
                                    .write()
                                    .expect("pending_controls RwLock poisoned")
                                    .push(pending_control);
                                let _ = request.respond(json_response(
                                    json!({"ok": true}).to_string(),
                                ));
                            }
                            Err(e) => {
                                let _ = request.respond(
                                    Response::from_string(
                                        json!({"ok": false, "error": format!("invalid json: {e}")}).to_string(),
                                    )
                                    .with_status_code(StatusCode(400))
                                    .with_header(json_ct()),
                                );
                            }
                        }
                    }

                    // ── GET /logs ────────────────────────────────────────────
                    (Method::Get, "/logs") => {
                        // Reuses C6 log format — reads active log file if present.
                        // Resolve log path relative to the executable directory,
                        // not the CWD, so the server works regardless of launch directory.
                        let log_path = std::env::current_exe()
                            .ok()
                            .and_then(|p| p.parent().map(|d| d.join("../../../debugger/logs/active/app.log")))
                            .unwrap_or_else(|| std::path::PathBuf::from("debugger/logs/active/app.log"));
                        let lines: Vec<String> =
                            std::fs::read_to_string(log_path)
                                .unwrap_or_default()
                                .lines()
                                .map(|s| s.to_string())
                                .collect();
                        let body = json!({ "lines": lines });
                        let _ = request.respond(json_response(body.to_string()));
                    }

                    // ── GET /dashboard ───────────────────────────────────────
                    (Method::Get, "/dashboard") => {
                        let _ = request.respond(
                            Response::from_string(DASHBOARD_HTML).with_header(
                                Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..])
                                    .unwrap(),
                            ),
                        );
                    }

                    _ => {
                        let _ = request.respond(
                            Response::from_string(json!({"error": "not_found"}).to_string())
                                .with_status_code(StatusCode(404))
                                .with_header(json_ct()),
                        );
                    }
                }
            }
        });

        pending
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn json_ct() -> Header {
    Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()
}

fn json_response(body: String) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body).with_header(json_ct())
}

/// Captures a screenshot using xcap and returns PNG bytes.
/// [NEEDS_REVIEW: claude] — cross-platform behavior; headless guard is at call site.
fn capture_screenshot() -> Result<Vec<u8>, String> {
    // [UNVERIFIED: xcap API — see research_dump/_INDEX.md]
    // Stub implementation — replace with xcap::Monitor::all()[0].capture_image()
    // once xcap maturity is verified and the optional feature is enabled.
    Err("screenshot_not_implemented".to_string())
}
