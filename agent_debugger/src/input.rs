//! Input module — enigo fallback for OS-level input injection.
//!
//! [NEEDS_REVIEW: claude] — OS-level input injection; Tier A review required.
//!
//! This module is the LAST RESORT fallback.
//! Primary path: `POST /control` HTTP endpoint (control.rs).
//! Use enigo only when the HTTP control endpoint is unavailable.
//!
//! Platform support: Windows + X11 Linux only.
//! Wayland: NOT SUPPORTED — use POST /control instead.

// Enigo on Wayland will fail to initialise. The caller must check the platform
// and prefer /control at all times (DEC-C9-003).

#[allow(dead_code)]
use crate::error::DebugError;

/// Move the mouse to `(x, y)` in screen coordinates and click the left button.
///
/// Returns Err on Wayland or unsupported platforms.
#[allow(dead_code)]
pub fn click_at(x: i32, y: i32) -> Result<(), DebugError> {
    use enigo::{Enigo, Mouse, Settings};

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| DebugError::NotImplemented(format!("enigo init failed: {e}")))?;

    enigo
        .move_mouse(x, y, enigo::Coordinate::Abs)
        .map_err(|e| DebugError::NotImplemented(format!("move_mouse: {e}")))?;

    enigo
        .button(enigo::Button::Left, enigo::Direction::Click)
        .map_err(|e| DebugError::NotImplemented(format!("button click: {e}")))?;

    log::warn!("input.rs: used enigo fallback — prefer POST /control (DEC-C9-003)");
    Ok(())
}

/// Type a string at the current focus position.
///
/// Returns Err on Wayland or unsupported platforms.
#[allow(dead_code)]
pub fn type_text(text: &str) -> Result<(), DebugError> {
    use enigo::{Enigo, Keyboard, Settings};

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| DebugError::NotImplemented(format!("enigo init failed: {e}")))?;

    enigo
        .text(text)
        .map_err(|e| DebugError::NotImplemented(format!("type_text: {e}")))?;

    log::warn!("input.rs: used enigo fallback — prefer POST /control (DEC-C9-003)");
    Ok(())
}

/// Press and release a key by name (e.g. "Return", "Escape").
///
/// Returns Err on Wayland or unsupported platforms.
#[allow(dead_code)]
pub fn press_key(key_name: &str) -> Result<(), DebugError> {
    use enigo::{Enigo, Keyboard, Settings};

    let key = parse_key(key_name)?;

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| DebugError::NotImplemented(format!("enigo init failed: {e}")))?;

    enigo
        .key(key, enigo::Direction::Click)
        .map_err(|e| DebugError::NotImplemented(format!("press_key {key_name}: {e}")))?;

    log::warn!("input.rs: used enigo fallback — prefer POST /control (DEC-C9-003)");
    Ok(())
}

/// Parse a key name string into an enigo Key.
fn parse_key(name: &str) -> Result<enigo::Key, DebugError> {
    use enigo::Key;
    match name {
        "Return" | "Enter" => Ok(Key::Return),
        "Escape" => Ok(Key::Escape),
        "Tab" => Ok(Key::Tab),
        "Space" => Ok(Key::Space),
        "Backspace" => Ok(Key::Backspace),
        "Delete" => Ok(Key::Delete),
        "Up" => Ok(Key::UpArrow),
        "Down" => Ok(Key::DownArrow),
        "Left" => Ok(Key::LeftArrow),
        "Right" => Ok(Key::RightArrow),
        "Home" => Ok(Key::Home),
        "End" => Ok(Key::End),
        "PageUp" => Ok(Key::PageUp),
        "PageDown" => Ok(Key::PageDown),
        "F1" => Ok(Key::F1),
        "F2" => Ok(Key::F2),
        "F3" => Ok(Key::F3),
        "F4" => Ok(Key::F4),
        "F5" => Ok(Key::F5),
        "F6" => Ok(Key::F6),
        "F7" => Ok(Key::F7),
        "F8" => Ok(Key::F8),
        "F9" => Ok(Key::F9),
        "F10" => Ok(Key::F10),
        "F11" => Ok(Key::F11),
        "F12" => Ok(Key::F12),
        other => Err(DebugError::NotImplemented(format!(
            "unknown key name: {other} — add to input.rs::parse_key()"
        ))),
    }
}
