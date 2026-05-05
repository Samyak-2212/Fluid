//! Hardware detection and tier binary selection (DEC-019).
//!
//! Reads hardware capabilities at startup to select the highest supported
//! capability tier. The user can override in Preferences (requires restart).
//!
//! Stub — full implementation is C8-SimBridge work.

use super::SimTier;

/// Detects the highest tier supported by the current hardware.
/// Falls back to Tier 0 (always available, CPU-only).
///
/// Detection heuristic (stub — expand in C8-SimBridge session):
/// - Tier 3: CUDA 12+ or ROCm 5+ GPU detected via environment probe
/// - Tier 2: Vulkan 1.3+ discrete GPU
/// - Tier 1: Any GPU with wgpu adapter support
/// - Tier 0: Fallback (always available)
pub fn detect_tier() -> SimTier {
    // TODO(C8-SimBridge): implement real GPU detection.
    // For now, default to Tier 0 (safe fallback).
    SimTier::Tier0
}

/// Returns the path to the `fluid_sim` binary for the given tier.
/// Binaries are expected at `app/bin/<tier>/fluid_sim[.exe]`.
///
/// Returns `None` if the binary does not exist at the expected path.
pub fn binary_path(tier: SimTier) -> Option<std::path::PathBuf> {
    let tier_dir = match tier {
        SimTier::Tier0 => "tier0",
        SimTier::Tier1 => "tier1",
        SimTier::Tier2 => "tier2",
        SimTier::Tier3 => "tier3",
    };
    let exe_name = if cfg!(windows) { "fluid_sim.exe" } else { "fluid_sim" };
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let path = exe_dir.join("bin").join(tier_dir).join(exe_name);
    if path.exists() { Some(path) } else { None }
}
