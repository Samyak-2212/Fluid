// app/build.rs
// Emits `tier_N` cfg feature flag from FLUID_TIER env var.
// Same pattern as rendering/build.rs and physics_core/build.rs.
// Only tier_0 is meaningful for the app crate (in-process preview, DEC-006).
// Tier 1–3 execution is subprocess-based and does not require compile-time flags here.

fn main() {
    let tier: u32 = std::env::var("FLUID_TIER")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
        .min(3);

    println!("cargo:rustc-cfg=feature=\"tier_{tier}\"");
    println!("cargo:rerun-if-env-changed=FLUID_TIER");
}
