//! agent_debugger — C9 CLI entry point.
//!
//! Wires all subcommands:
//!   run            — full agent loop (health → screenshot → state+tree → control → screenshot → report)
//!   health         — check server health
//!   state          — print current state
//!   tree           — print widget tree
//!   logs           — print log tail
//!   screenshot     — capture and save screenshot
//!   control        — send a control action
//!   cleanup        — delete session PNGs
//!
//! All errors are structured; no unwrap() anywhere (DEC-C9-004).

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

mod cleanup;
mod config;
mod control;
mod error;
mod health;
mod input;
mod report;
mod screenshot;
mod state;

use config::AgentDebuggerConfig;
use error::DebugError;

// ── CLI ───────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "agent_debugger",
    version,
    about = "Fluid Agent Debugger (C9) — automated test harness for the Fluid GUI application"
)]
struct Cli {
    /// Path to workspace root (auto-detected if not specified).
    #[arg(long)]
    workspace: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Full agent loop: health → screenshot → state+tree → [optional control] → screenshot → report.
    Run {
        /// Control action to perform: click, keypress, set_field, menu (optional).
        #[arg(long)]
        action: Option<String>,
        /// Target widget ID (from /tree — never hardcoded).
        #[arg(long)]
        target: Option<String>,
        /// Value for keypress or set_field.
        #[arg(long)]
        value: Option<String>,
        /// Menu path (comma-separated, e.g. "File,Save").
        #[arg(long)]
        menu_path: Option<String>,
        /// Session ID override (default: timestamp).
        #[arg(long)]
        session_id: Option<String>,
    },
    /// Check server health and protocol version.
    Health,
    /// Print current application state JSON.
    State,
    /// Print widget tree JSON.
    Tree,
    /// Print log tail JSON.
    Logs,
    /// Capture a screenshot and save to path.
    Screenshot {
        /// Output path (default: /tmp/fluid_screenshot.png).
        #[arg(long, default_value = "fluid_screenshot.png")]
        out: PathBuf,
    },
    /// Send a control action to the app.
    Control {
        /// Action type: click, keypress, set_field, menu.
        action: String,
        /// Target widget ID (from /tree).
        #[arg(long)]
        target: Option<String>,
        /// Value (for keypress or set_field).
        #[arg(long)]
        value: Option<String>,
        /// Menu path (comma-separated).
        #[arg(long)]
        menu_path: Option<String>,
    },
    /// Delete PNG files from session archives.
    Cleanup {
        /// Delete PNGs from a specific session: --session <timestamp>.
        #[arg(long, conflicts_with = "committed")]
        session: Option<String>,
        /// Delete PNGs from all sessions (after git commit).
        #[arg(long)]
        committed: bool,
    },
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    let workspace = cli
        .workspace
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let cfg = AgentDebuggerConfig::load_or_default(&workspace);

    let result = dispatch(&cli.command, &cfg, &workspace);

    if let Err(e) = result {
        eprintln!("ERROR: {e}");
        std::process::exit(1);
    }
}

fn dispatch(cmd: &Commands, cfg: &AgentDebuggerConfig, workspace: &Path) -> Result<(), DebugError> {
    match cmd {
        Commands::Health => cmd_health(cfg),
        Commands::State => cmd_state(cfg),
        Commands::Tree => cmd_tree(cfg),
        Commands::Logs => cmd_logs(cfg),
        Commands::Screenshot { out } => cmd_screenshot(cfg, out),
        Commands::Control {
            action,
            target,
            value,
            menu_path,
        } => cmd_control(cfg, action, target.as_deref(), value.as_deref(), menu_path.as_deref()),
        Commands::Cleanup { session, committed } => {
            cmd_cleanup(cfg, workspace, session.as_deref(), *committed)
        }
        Commands::Run {
            action,
            target,
            value,
            menu_path,
            session_id,
        } => cmd_run(
            cfg,
            workspace,
            action.as_deref(),
            target.as_deref(),
            value.as_deref(),
            menu_path.as_deref(),
            session_id.as_deref(),
        ),
    }
}

// ── Subcommand implementations ────────────────────────────────────────────────

fn cmd_health(cfg: &AgentDebuggerConfig) -> Result<(), DebugError> {
    let h = health::check(cfg)?;
    println!("ok={} protocol_version={}", h.ok, h.protocol_version);
    Ok(())
}

fn cmd_state(cfg: &AgentDebuggerConfig) -> Result<(), DebugError> {
    health::check(cfg)?;
    let s = state::fetch_state(cfg)?;
    let json = serde_json::to_string_pretty(&s)?;
    println!("{json}");
    Ok(())
}

fn cmd_tree(cfg: &AgentDebuggerConfig) -> Result<(), DebugError> {
    health::check(cfg)?;
    let t = state::fetch_tree(cfg)?;
    let json = serde_json::to_string_pretty(&t)?;
    println!("{json}");
    Ok(())
}

fn cmd_logs(cfg: &AgentDebuggerConfig) -> Result<(), DebugError> {
    health::check(cfg)?;
    let l = state::fetch_logs(cfg)?;
    for line in &l.lines {
        println!("{line}");
    }
    Ok(())
}

fn cmd_screenshot(cfg: &AgentDebuggerConfig, out: &Path) -> Result<(), DebugError> {
    health::check(cfg)?;
    let mut rl = screenshot::ScreenshotRateLimit::new(cfg.screenshot_min_interval_ms);
    match screenshot::fetch(cfg, &mut rl)? {
        screenshot::ScreenshotResult::Png(bytes) => {
            screenshot::save_png(&bytes, out)?;
            println!("Screenshot saved: {}", out.display());
        }
        screenshot::ScreenshotResult::NotImplemented => {
            println!(
                r#"{{"ok":false,"error":"screenshot_not_implemented"}}"#
            );
        }
        screenshot::ScreenshotResult::Headless => {
            println!(r#"{{"ok":false,"error":"no_display","headless":true}}"#);
        }
    }
    Ok(())
}

fn cmd_control(
    cfg: &AgentDebuggerConfig,
    action: &str,
    target: Option<&str>,
    value: Option<&str>,
    menu_path: Option<&str>,
) -> Result<(), DebugError> {
    health::check(cfg)?;
    let req = build_control_request(action, target, value, menu_path)?;
    control::send(cfg, &req)?;
    println!(r#"{{"ok":true}}"#);
    Ok(())
}

fn cmd_cleanup(
    _cfg: &AgentDebuggerConfig,
    workspace: &Path,
    session: Option<&str>,
    committed: bool,
) -> Result<(), DebugError> {
    let sessions_dir = workspace.join("agent_debugger").join("sessions");

    if let Some(sid) = session {
        let deleted = cleanup::cleanup_session(&sessions_dir, sid)?;
        println!("Deleted {} PNG file(s) from session {sid}:", deleted.len());
        for p in &deleted {
            println!("  {}", p.display());
        }
    } else if committed {
        let deleted = cleanup::cleanup_all_committed(&sessions_dir)?;
        println!(
            "Deleted {} PNG file(s) from all committed sessions.",
            deleted.len()
        );
        for p in &deleted {
            println!("  {}", p.display());
        }
    } else {
        eprintln!("cleanup: specify --session <id> or --committed");
        std::process::exit(1);
    }
    Ok(())
}

fn cmd_run(
    cfg: &AgentDebuggerConfig,
    workspace: &Path,
    action: Option<&str>,
    target: Option<&str>,
    value: Option<&str>,
    menu_path: Option<&str>,
    session_id_override: Option<&str>,
) -> Result<(), DebugError> {
    // 1. Health check
    health::check(cfg)?;
    log::info!("Health OK");

    // Session ID = timestamp or override
    let session_id = session_id_override
        .map(str::to_string)
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| format!("{}", d.as_secs()))
                .unwrap_or_else(|_| "session".to_string())
        });

    let session_dir = workspace
        .join("agent_debugger")
        .join("sessions")
        .join(&session_id);

    std::fs::create_dir_all(&session_dir)
        .map_err(|e| DebugError::Io(format!("create session dir: {e}")))?;

    let mut rl = screenshot::ScreenshotRateLimit::new(cfg.screenshot_min_interval_ms);
    let mut incomplete = false;
    let mut notes = Vec::new();

    // 2. Screenshot before
    let screenshot_before = screenshot::fetch(cfg, &mut rl)?;
    match &screenshot_before {
        screenshot::ScreenshotResult::Png(bytes) => {
            screenshot::save_png(bytes, &session_dir.join("screen_before.png"))?;
        }
        screenshot::ScreenshotResult::NotImplemented => {
            incomplete = true;
            notes.push("screenshot_not_implemented: C8 xcap stub active".to_string());
            log::warn!("Screenshot not implemented — continuing without visual");
        }
        screenshot::ScreenshotResult::Headless => {
            incomplete = true;
            notes.push("headless: no display".to_string());
        }
    }

    // 3. State + tree before
    let state_before = state::fetch_state(cfg)?;
    let tree_before = state::fetch_tree(cfg)?;

    // Save before snapshots
    let state_before_json = serde_json::to_string_pretty(&state_before)?;
    std::fs::write(session_dir.join("state_before.json"), &state_before_json)
        .map_err(|e| DebugError::Io(e.to_string()))?;

    let tree_before_json = serde_json::to_string_pretty(&tree_before)?;
    std::fs::write(session_dir.join("tree.json"), &tree_before_json)
        .map_err(|e| DebugError::Io(e.to_string()))?;

    log::info!(
        "State before: frame={} status={}",
        state_before.frame,
        state_before.sim_status
    );

    // 4. Agent action (optional)
    let action_str = action.unwrap_or("none");
    if let Some(act) = action {
        let req = build_control_request(act, target, value, menu_path)?;
        control::send(cfg, &req)?;
        log::info!("Control sent: action={act}");
        // Small settle time for C8 main loop to consume the queued action.
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    // 5. Screenshot after
    let screenshot_after = screenshot::fetch(cfg, &mut rl)?;
    match &screenshot_after {
        screenshot::ScreenshotResult::Png(bytes) => {
            screenshot::save_png(bytes, &session_dir.join("screen_after.png"))?;
        }
        screenshot::ScreenshotResult::NotImplemented | screenshot::ScreenshotResult::Headless => {
            // Already flagged above
        }
    }

    // 6. State after
    let state_after = state::fetch_state(cfg)?;
    let tree_after = state::fetch_tree(cfg)?;

    let state_after_json = serde_json::to_string_pretty(&state_after)?;
    std::fs::write(session_dir.join("state_after.json"), &state_after_json)
        .map_err(|e| DebugError::Io(e.to_string()))?;

    log::info!(
        "State after: frame={} status={}",
        state_after.frame,
        state_after.sim_status
    );

    // 7. Report
    let timestamp = chrono_like_timestamp();
    let rpt = report::SessionReport::build(
        &session_id,
        &timestamp,
        action_str,
        incomplete,
        &state_before,
        &state_after,
        &tree_before.widgets,
        &tree_after.widgets,
        notes,
    );
    rpt.save(&session_dir.join("report.json"))?;

    println!("Session complete: {}", session_dir.display());
    println!("incomplete={incomplete}");
    let report_json = serde_json::to_string_pretty(&rpt)?;
    println!("{report_json}");

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn build_control_request(
    action: &str,
    target: Option<&str>,
    value: Option<&str>,
    menu_path: Option<&str>,
) -> Result<control::ControlRequest, DebugError> {
    match action {
        "click" => {
            let t = target.ok_or_else(|| {
                DebugError::Protocol("click action requires --target <widget_id>".to_string())
            })?;
            Ok(control::ControlRequest::click(t))
        }
        "keypress" => {
            let t = target.ok_or_else(|| {
                DebugError::Protocol("keypress action requires --target <widget_id>".to_string())
            })?;
            let v = value.ok_or_else(|| {
                DebugError::Protocol("keypress action requires --value <key_name>".to_string())
            })?;
            Ok(control::ControlRequest::keypress(t, v))
        }
        "set_field" => {
            let t = target.ok_or_else(|| {
                DebugError::Protocol("set_field action requires --target <widget_id>".to_string())
            })?;
            let v = value.ok_or_else(|| {
                DebugError::Protocol("set_field action requires --value <value>".to_string())
            })?;
            Ok(control::ControlRequest::set_field(t, v))
        }
        "menu" => {
            let path_str = menu_path.ok_or_else(|| {
                DebugError::Protocol(
                    "menu action requires --menu-path <File,Save>".to_string(),
                )
            })?;
            let path: Vec<String> = path_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            Ok(control::ControlRequest::menu(path))
        }
        other => Err(DebugError::Protocol(format!(
            "unknown action: {other} (valid: click, keypress, set_field, menu)"
        ))),
    }
}

/// Minimal ISO-8601 timestamp without chrono dependency.
fn chrono_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Simple epoch seconds — full RFC 3339 requires chrono (not in deps).
    format!("{secs}")
}
