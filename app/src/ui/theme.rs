//! Dark professional theme for the Fluid application.
//!
//! All color tokens are defined here. No ad-hoc colors in widget code.
//! The theme is TOML-configurable — see `app/src/prefs/mod.rs` for the
//! hot-swap mechanism (DEC-017).
//!
//! # Iced integration
//! Call [`fluid_theme`] to obtain the `iced::Theme` used by the application.
//! Palette constants match the locked color spec from coordinators/app/PROMPT.md.

use serde::{Deserialize, Serialize};

// ── Color constants ────────────────────────────────────────────────────────────

/// Main background — darkest surface.  #0f0f13
pub const BG_BASE: [u8; 3] = [0x0f, 0x0f, 0x13];
/// Card/panel surface (slightly elevated).  #1a1a24
pub const BG_SURFACE: [u8; 3] = [0x1a, 0x1a, 0x24];
/// Border / separator.  #2d2d3d
pub const BORDER: [u8; 3] = [0x2d, 0x2d, 0x3d];
/// Primary accent — indigo.  #6366f1
pub const ACCENT: [u8; 3] = [0x63, 0x66, 0xf1];
/// Success / confirmation.  #22c55e
pub const SUCCESS: [u8; 3] = [0x22, 0xc5, 0x5e];
/// Warning / caution.  #f59e0b
pub const WARNING: [u8; 3] = [0xf5, 0x9e, 0x0b];
/// Error / danger.  #ef4444
pub const ERROR: [u8; 3] = [0xef, 0x44, 0x44];
/// Primary text.  #e2e8f0
pub const TEXT: [u8; 3] = [0xe2, 0xe8, 0xf0];
/// Muted / secondary text.  #8892a4
pub const TEXT_MUTED: [u8; 3] = [0x88, 0x92, 0xa4];

// ── Helper ────────────────────────────────────────────────────────────────────

fn rgb(r: u8, g: u8, b: u8) -> iced::Color {
    iced::Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

// ── Iced theme ────────────────────────────────────────────────────────────────

/// Returns the Fluid dark professional `iced::Theme`.
///
/// Background: #0f0f13  Primary: #6366f1 (indigo).
/// Use from `FluidApp::theme(&self)` — called each frame by iced.
pub fn fluid_theme() -> iced::Theme {
    let palette = iced::theme::Palette {
        background: rgb(BG_BASE[0], BG_BASE[1], BG_BASE[2]),
        text:       rgb(TEXT[0],    TEXT[1],    TEXT[2]),
        primary:    rgb(ACCENT[0],  ACCENT[1],  ACCENT[2]),
        success:    rgb(SUCCESS[0], SUCCESS[1], SUCCESS[2]),
        danger:     rgb(ERROR[0],   ERROR[1],   ERROR[2]),
    };
    iced::Theme::custom("fluid-dark".to_string(), palette)
}

// ── TOML-serialisable palette ─────────────────────────────────────────────────
// Kept for DEC-017 hot-swap: prefs.toml can override the palette at runtime.

/// Application color palette — stored in TOML for hot-swap (DEC-017).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Palette {
    /// Main background (darkest surface).
    pub bg_base:    [u8; 3],
    /// Card/panel surface (slightly elevated).
    pub bg_surface: [u8; 3],
    /// Border / separator color.
    pub border:     [u8; 3],
    /// Primary accent (interactive elements, highlights).
    pub accent:     [u8; 3],
    /// Success / confirmation color.
    pub success:    [u8; 3],
    /// Warning / caution color.
    pub warning:    [u8; 3],
    /// Error / danger color.
    pub error:      [u8; 3],
    /// Primary text color.
    pub text:       [u8; 3],
    /// Muted / secondary text color.
    pub text_muted: [u8; 3],
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            bg_base:    BG_BASE,
            bg_surface: BG_SURFACE,
            border:     BORDER,
            accent:     ACCENT,
            success:    SUCCESS,
            warning:    WARNING,
            error:      ERROR,
            text:       TEXT,
            text_muted: TEXT_MUTED,
        }
    }
}

/// Application theme state (palette + font settings) — TOML hot-swap target.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppTheme {
    pub palette: Palette,
}

impl AppTheme {
    pub fn dark() -> Self {
        Self { palette: Palette::default() }
    }

    /// Converts this TOML palette into an `iced::Theme` for runtime override.
    pub fn to_iced_theme(&self) -> iced::Theme {
        let p = &self.palette;
        let palette = iced::theme::Palette {
            background: rgb(p.bg_base[0],  p.bg_base[1],  p.bg_base[2]),
            text:       rgb(p.text[0],      p.text[1],      p.text[2]),
            primary:    rgb(p.accent[0],    p.accent[1],    p.accent[2]),
            success:    rgb(p.success[0],   p.success[1],   p.success[2]),
            danger:     rgb(p.error[0],     p.error[1],     p.error[2]),
        };
        iced::Theme::custom("fluid-dark-custom".to_string(), palette)
    }
}
