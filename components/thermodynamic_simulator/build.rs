// build.rs for thermodynamic_simulator
// Reads the FLUID_TIER environment variable and emits the corresponding
// Cargo feature flag so that #[cfg(feature = "tier_N")] gates work throughout
// the crate without callers having to pass --features manually.

fn main() {
    println!("cargo:rerun-if-env-changed=FLUID_TIER");
    let profile = std::env::var("PROFILE").unwrap_or_default();
    let tier_str = std::env::var("FLUID_TIER").unwrap_or_else(|_| {
        if profile == "release" { "2".to_string() } else { "0".to_string() }
    });
    let tier: u8 = tier_str.trim().parse().expect("FLUID_TIER must be 0, 1, 2, or 3");

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
