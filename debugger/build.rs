fn main() {
    println!("cargo:rerun-if-env-changed=FLUID_TIER");
    
    if let Ok(tier) = std::env::var("FLUID_TIER") {
        println!("cargo:rustc-cfg=tier=\"{}\"", tier);
    } else {
        // Default to tier 0
        println!("cargo:rustc-cfg=tier=\"0\"");
    }
}
