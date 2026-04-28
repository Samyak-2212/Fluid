// build.rs — core crate
// Reads FLUID_TIER env var (values: 0, 1, 2, 3; default 0).
// Emits cargo:rustc-cfg=feature="tier_N" so #[cfg(feature = "tier_N")] gates work.
// Tier selection is compile-time only — no runtime switching.

fn main() {
    let tier = std::env::var("FLUID_TIER").unwrap_or_else(|_| "0".to_string());
    match tier.as_str() {
        "0" | "1" | "2" | "3" => {
            println!("cargo:rustc-cfg=feature=\"tier_{}\"", tier);
        }
        other => {
            // Unknown tier value — emit tier_0 and warn via cargo.
            println!("cargo:warning=Unknown FLUID_TIER value '{}'; defaulting to tier_0", other);
            println!("cargo:rustc-cfg=feature=\"tier_0\"");
        }
    }
    println!("cargo:rerun-if-env-changed=FLUID_TIER");
}
