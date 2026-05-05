//! Fluid — native simulation application entry point.
//!
//! Initialises the Iced window, loads configuration, detects hardware tier,
//! starts the debug HTTP server, and enters the main event loop.
//!
//! # Headless mode
//! Pass `--headless` to run without a window. The debug server still starts.
//! `/state` and `/logs` work; `/screenshot` and `/tree` return appropriate
//! error/empty responses.

use std::sync::{Arc, RwLock};

mod app;
mod debug_server;
mod file;
mod import;
mod plugin;
mod prefs;
mod scene;
mod sim_bridge;
mod ui;
mod viewport;

use app::FluidApp;
use debug_server::{AppStateSnapshot, DebugServer};
use ui::widget_registry::WidgetRegistry;

/// Application-level configuration loaded from `config/app.toml`.
#[derive(Debug, Clone, serde::Deserialize)]
struct AppConfig {
    #[serde(default = "default_port")]
    debug_server_port: u16,
    #[serde(default)]
    debug_server_token: String,
    #[serde(default = "default_log_level")]
    log_level: String,
}

fn default_port() -> u16 { 8082 }
fn default_log_level() -> String { "INFO".to_string() }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            debug_server_port: 8082,
            debug_server_token: String::new(),
            log_level: "INFO".to_string(),
        }
    }
}

/// Loads `config/app.toml` relative to the executable.
///
/// Returns `AppConfig::default()` on any error — no panic.
fn load_app_config() -> AppConfig {
    // Resolve relative to executable directory (DEC-018 — no hardcoded paths).
    let config_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .map(|d| d.join("config").join("app.toml"))
        .unwrap_or_else(|| std::path::PathBuf::from("config/app.toml"));

    match std::fs::read_to_string(&config_path) {
        Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
            log::warn!("app.toml parse error: {e}; using defaults");
            AppConfig::default()
        }),
        Err(_) => {
            log::info!("config/app.toml not found; using defaults");
            AppConfig::default()
        }
    }
}

fn main() {
    // ── Parse CLI ──────────────────────────────────────────────────────────
    let args: Vec<String> = std::env::args().collect();
    let headless = args.contains(&"--headless".to_string());

    // ── Load app config ────────────────────────────────────────────────────
    let app_config = load_app_config();

    // ── Init logging ───────────────────────────────────────────────────────
    let log_filter = app_config.log_level.as_str();
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(log_filter),
    )
    .init();

    log::info!("Fluid starting (headless={headless})");

    // ── Detect hardware tier ────────────────────────────────────────────────
    let detected_tier = sim_bridge::tier_select::detect_tier();
    log::info!("Detected sim tier: {:?}", detected_tier);

    // ── Load user preferences ───────────────────────────────────────────────
    let shared_prefs = prefs::shared_prefs();
    log::debug!("Loaded user prefs: {:?}", shared_prefs.load());

    // ── Shared state for debug server ───────────────────────────────────────
    let state_snapshot: Arc<RwLock<AppStateSnapshot>> = Arc::new(RwLock::new({
        let mut snap = AppStateSnapshot::default();
        snap.headless = headless;
        snap.tier = detected_tier as u32;
        snap
    }));

    let widget_registry: Arc<RwLock<WidgetRegistry>> =
        Arc::new(RwLock::new(WidgetRegistry::new()));

    // ── Start debug server ──────────────────────────────────────────────────
    let debug_server = DebugServer::new(
        app_config.debug_server_port,
        app_config.debug_server_token.clone(),
        headless,
        Arc::clone(&state_snapshot),
        Arc::clone(&widget_registry),
    );
    let _pending_controls = debug_server.start();
    log::info!(
        "Debug server started at http://127.0.0.1:{}",
        app_config.debug_server_port
    );

    // ── Headless mode — spin without a window ──────────────────────────────
    if headless {
        log::info!("Running in headless mode. Debug server active. Press Ctrl-C to exit.");
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    // ── Windowed mode — launch Iced application ────────────────────────────
    // Clone Arcs for move into the run_with closure.
    let snap_arc    = Arc::clone(&state_snapshot);
    let registry_arc = Arc::clone(&widget_registry);

    iced::application("Fluid", FluidApp::update, FluidApp::view)
        .theme(FluidApp::theme)
        .subscription(FluidApp::subscription)
        .run_with(move || FluidApp::new(snap_arc, registry_arc))
        .expect("Iced run failed");
}
