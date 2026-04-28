// build.rs for physics_core
//
// Reads the FLUID_TIER environment variable and emits the corresponding
// Cargo feature flag so that #[cfg(feature = "tier_N")] gates work throughout
// the crate without callers having to pass --features manually.
//
// FLUID_TIER=0  → cargo:rustc-cfg=feature="tier_0"
// FLUID_TIER=1  → cargo:rustc-cfg=feature="tier_1"
// FLUID_TIER=2  → cargo:rustc-cfg=feature="tier_2"
// FLUID_TIER=3  → cargo:rustc-cfg=feature="tier_3"
//
// If FLUID_TIER is unset the debug-build default is tier_0.
// Release builds default to tier_2 per capability_tiers.md.
//
// The builder UI tier selector sets FLUID_TIER before invoking cargo.
// Tier selection is compile-time only — changing tier requires full recompile.

fn main() {
    // Re-run if FLUID_TIER changes.
    println!("cargo:rerun-if-env-changed=FLUID_TIER");

    let profile = std::env::var("PROFILE").unwrap_or_default();
    let tier_str = std::env::var("FLUID_TIER").unwrap_or_else(|_| {
        // Default: 0 for debug, 2 for release (capability_tiers.md).
        if profile == "release" {
            "2".to_string()
        } else {
            "0".to_string()
        }
    });

    let tier: u8 = tier_str
        .trim()
        .parse()
        .expect("FLUID_TIER must be 0, 1, 2, or 3");

    match tier {
        0 => println!("cargo:rustc-cfg=feature=\"tier_0\""),
        1 => {
            println!("cargo:rustc-cfg=feature=\"tier_0\"");
            println!("cargo:rustc-cfg=feature=\"tier_1\"");
        }
        2 => {
            println!("cargo:rustc-cfg=feature=\"tier_0\"");
            println!("cargo:rustc-cfg=feature=\"tier_1\"");
            println!("cargo:rustc-cfg=feature=\"tier_2\"");
        }
        3 => {
            println!("cargo:rustc-cfg=feature=\"tier_0\"");
            println!("cargo:rustc-cfg=feature=\"tier_1\"");
            println!("cargo:rustc-cfg=feature=\"tier_2\"");
            println!("cargo:rustc-cfg=feature=\"tier_3\"");
        }
        n => panic!("FLUID_TIER={n} is not a valid tier (valid: 0, 1, 2, 3)"),
    }
}
