// build.rs — rendering crate
// Reads FLUID_TIER env var and emits the matching cargo:rustc-cfg feature flag.
// Tier is compile-time only — no runtime switching.
// Default: tier_0 for debug builds, tier_2 for release builds.

fn main() {
    let tier = std::env::var("FLUID_TIER").unwrap_or_else(|_| {
        // Debug default: tier_0 (CPU-only path, no GPU required)
        // Release default: tier_2 (discrete GPU, Vulkan/DX12/Metal)
        if cfg!(debug_assertions) {
            "0".to_string()
        } else {
            "2".to_string()
        }
    });

    match tier.trim() {
        "0" => println!("cargo:rustc-cfg=feature=\"tier_0\""),
        "1" => println!("cargo:rustc-cfg=feature=\"tier_1\""),
        "2" => println!("cargo:rustc-cfg=feature=\"tier_2\""),
        "3" => println!("cargo:rustc-cfg=feature=\"tier_3\""),
        other => {
            eprintln!(
                "rendering/build.rs: unknown FLUID_TIER={:?}; defaulting to tier_0",
                other
            );
            println!("cargo:rustc-cfg=feature=\"tier_0\"");
        }
    }

    // Re-run this script if the env var changes.
    println!("cargo:rerun-if-env-changed=FLUID_TIER");
}
