// [NEEDS_REVIEW: claude]
//! Aerodynamic simulator — Tier 1+.
//!
//! Provides aerodynamic force vectors (lift, drag, thrust) for bodies in the ECS.
//! Consumes `physics_core::rigid_body::RigidBody` via the C4 API.
//!
//! # Model
//!
//! Thin-aerofoil / modified Newtonian impact theory with a quasi-steady model:
//!
//! - **Lift**: `L = 0.5 · ρ · v² · S_ref · C_L(α)`
//! - **Drag**: `D = 0.5 · ρ · v² · S_ref · C_D(α, Re)`
//! - **Thrust**: Applied separately as a body-fixed force (engine model).
//!
//! Configuration: air density, viscosity, reference area from
//! `config/aerodynamic_simulator.toml`.
//!
//! # References
//!
//! Anderson, J.D. (2017). *Introduction to Flight* (8th ed.). McGraw-Hill.
//! §§ 4.1–4.4 (lift and drag).
//!
//! Prandtl, L. (1918). "Tragflügeltheorie I." Nachr. Ges. Wiss. Göttingen.
//! (Lifting-line theory, basis of C_L = 2π·α for thin aerofoil.)

use core::units::KilogramsPerCubicMeter;
use glam::Vec3;
use physics_core::rigid_body::RigidBody;

// ── Air properties ────────────────────────────────────────────────────────────

/// Standard atmospheric properties at sea level (ISA conditions).
///
/// Source: ICAO Standard Atmosphere (ISO 2533:1975).
pub mod isa {
    /// Sea-level air density (kg/m³). ISA: 1.225 kg/m³.
    pub const RHO_SEA_LEVEL: f64 = 1.225;
    /// Sea-level dynamic viscosity (Pa·s). ISA: 1.789e-5 Pa·s.
    pub const MU_SEA_LEVEL: f64 = 1.789e-5;
    /// Sea-level speed of sound (m/s). ISA: 340.29 m/s.
    pub const C_SOUND: f64 = 340.29;
}

// ── Aerodynamic configuration ─────────────────────────────────────────────────

/// Aerodynamic surface configuration for one body.
///
/// Loaded from `config/aerodynamic_simulator.toml` at startup.
#[derive(Debug, Clone)]
pub struct AeroConfig {
    /// Air density ρ (kg/m³).
    pub air_density: KilogramsPerCubicMeter,
    /// Reference area S_ref (m²) — typically wing planform area.
    pub reference_area: f64,
    /// Lift slope d(C_L)/dα (per radian). Thin aerofoil theory: 2π ≈ 6.283.
    ///
    /// For a finite wing: `a = a₀ / (1 + a₀/(π·AR))` (Prandtl lifting-line).
    pub lift_slope: f64,
    /// Zero-lift angle of attack α₀ (radians). Typically ≈ 0 for symmetric foil.
    pub alpha_zero_lift: f64,
    /// Drag at zero lift C_D0 (profile drag coefficient, dimensionless).
    pub cd_zero: f64,
    /// Oswald efficiency factor e (dimensionless) — typically 0.7–0.9.
    pub oswald_efficiency: f64,
    /// Aspect ratio AR = b²/S_ref (dimensionless).
    pub aspect_ratio: f64,
}

impl Default for AeroConfig {
    fn default() -> Self {
        Self {
            air_density: KilogramsPerCubicMeter(isa::RHO_SEA_LEVEL),
            reference_area: 1.0,
            lift_slope: 2.0 * std::f64::consts::PI,
            alpha_zero_lift: 0.0,
            cd_zero: 0.02,
            oswald_efficiency: 0.85,
            aspect_ratio: 8.0,
        }
    }
}

// ── Aerodynamic forces ────────────────────────────────────────────────────────

/// Computed aerodynamic force output for one body.
#[derive(Debug, Clone, Copy)]
pub struct AeroForces {
    /// Lift force vector (N) — perpendicular to velocity, in the lift plane.
    pub lift: Vec3,
    /// Drag force vector (N) — opposing velocity.
    pub drag: Vec3,
    /// Total aerodynamic force = lift + drag (N).
    pub total: Vec3,
    /// Angle of attack α (radians) — for diagnostics.
    pub alpha: f64,
}

/// Computes aerodynamic lift and drag forces for a rigid body.
///
/// `body_up` — the body's local up-axis in world space (unit vector).
///             Used to define the lift plane.
/// `config`  — aerodynamic configuration.
///
/// # Algorithm
///
/// 1. Compute `α` from the angle between velocity and body chord (zero-lift line).
/// 2. `C_L = lift_slope · (α - α₀)`
/// 3. `C_D = C_D0 + C_L² / (π · e · AR)` (drag polar)
/// 4. `q = 0.5 · ρ · |v|²` (dynamic pressure)
/// 5. `L = q · S · C_L · lift_direction`
/// 6. `D = q · S · C_D · (-v̂)` (opposing velocity)
pub fn compute_aero_forces(
    body: &RigidBody,
    body_up: Vec3,
    config: &AeroConfig,
) -> AeroForces {
    let v = body.velocity;
    let speed_sq = v.length_squared() as f64;

    if speed_sq < 1e-6 {
        return AeroForces { lift: Vec3::ZERO, drag: Vec3::ZERO, total: Vec3::ZERO, alpha: 0.0 };
    }

    let speed = speed_sq.sqrt();
    let v_hat = v / speed as f32;

    // Angle of attack: angle between velocity and chord (body_up ⊥ chord assumed here
    // as a simplification; full aerodynamic frame uses body_forward vs velocity).
    // α = arcsin(v_hat · body_up) — component of velocity perpendicular to chord.
    let alpha = (v_hat.dot(body_up) as f64).clamp(-1.0, 1.0).asin();

    // Aerodynamic coefficients
    let cl = config.lift_slope * (alpha - config.alpha_zero_lift);
    let cd = config.cd_zero
        + (cl * cl) / (std::f64::consts::PI * config.oswald_efficiency * config.aspect_ratio);

    // Dynamic pressure q = 0.5 · ρ · v²
    let q = 0.5 * config.air_density.value() * speed_sq;
    let s = config.reference_area;

    // Lift direction: perpendicular to velocity, in the plane of (velocity, body_up)
    // lift_dir = normalize(body_up - (body_up · v̂) · v̂)
    let v_hat_f64 = v_hat.as_dvec3();
    let body_up_f64 = body_up.as_dvec3();
    let lift_dir_raw = body_up_f64 - body_up_f64.dot(v_hat_f64) * v_hat_f64;
    let lift_dir_len = lift_dir_raw.length();

    let lift = if lift_dir_len > 1e-6 {
        let lift_dir = (lift_dir_raw / lift_dir_len).as_vec3();
        lift_dir * (q * s * cl) as f32
    } else {
        Vec3::ZERO
    };

    // Drag opposes velocity
    let drag = -v_hat * (q * s * cd) as f32;

    AeroForces { lift, drag, total: lift + drag, alpha }
}

/// Applies aerodynamic forces to a body's force accumulator.
pub fn apply_aero_forces(body: &mut RigidBody, body_up: Vec3, config: &AeroConfig) {
    let forces = compute_aero_forces(body, body_up, config);
    body.apply_force(forces.total);
}

// ── Compute FFI trait (Tier 3) ─────────────────────────────────────────────────

#[cfg(feature = "tier_3")]
pub use fluid_simulator_compute_trait::*;

#[cfg(feature = "tier_3")]
mod fluid_simulator_compute_trait {
    pub struct ComputeKernel { pub id: u32 }
    pub struct KernelArgs { pub data: Vec<u8>, pub work_groups: [u32; 3] }
    pub trait GpuComputeBackend: Send + Sync {
        fn dispatch_kernel(&self, kernel: &ComputeKernel, args: &KernelArgs)
            -> Result<(), Box<dyn std::error::Error>>;
    }
}

// ── Debug overlay ─────────────────────────────────────────────────────────────

/// Diagnostic data for the C6 debugger overlay.
#[cfg(feature = "debug_overlay")]
#[derive(Debug, Default)]
pub struct AeroDebugStats {
    /// Number of bodies evaluated this frame.
    pub bodies_evaluated: usize,
    /// Mean angle of attack across all bodies (radians).
    pub mean_alpha_rad: f64,
    /// Maximum lift force magnitude this frame (N).
    pub max_lift_n: f64,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use core::units::Kilograms;
    use glam::Mat3;

    fn flying_body(velocity: Vec3) -> RigidBody {
        let mut b = RigidBody::new(Kilograms(100.0), Mat3::IDENTITY);
        b.velocity = velocity;
        b
    }

    #[test]
    fn zero_velocity_gives_zero_forces() {
        let body = flying_body(Vec3::ZERO);
        let config = AeroConfig::default();
        let forces = compute_aero_forces(&body, Vec3::Y, &config);
        assert_eq!(forces.total, Vec3::ZERO);
    }

    #[test]
    fn drag_opposes_velocity_direction() {
        let body = flying_body(Vec3::new(50.0, 0.0, 0.0));
        let config = AeroConfig::default();
        let forces = compute_aero_forces(&body, Vec3::Y, &config);
        // Drag must have a negative x component (opposing +x velocity)
        assert!(
            forces.drag.x < 0.0,
            "Drag must oppose velocity direction: drag={:?}", forces.drag
        );
    }

    #[test]
    fn lift_nonzero_at_angle_of_attack() {
        // Body moving horizontally + slightly upward → nonzero angle of attack
        let body = flying_body(Vec3::new(50.0, 5.0, 0.0));
        let config = AeroConfig::default();
        let forces = compute_aero_forces(&body, Vec3::Y, &config);
        // At positive AoA, lift should be positive (away from velocity, toward up)
        let lift_mag = forces.lift.length();
        assert!(
            lift_mag > 0.01,
            "Lift should be nonzero at positive AoA: {lift_mag:.4}"
        );
    }

    #[test]
    fn drag_increases_with_speed() {
        let config = AeroConfig::default();
        let b_slow = flying_body(Vec3::new(10.0, 0.0, 0.0));
        let b_fast = flying_body(Vec3::new(100.0, 0.0, 0.0));
        let d_slow = compute_aero_forces(&b_slow, Vec3::Y, &config).drag.length();
        let d_fast = compute_aero_forces(&b_fast, Vec3::Y, &config).drag.length();
        assert!(d_fast > d_slow, "Drag at 100 m/s ({d_fast:.2}) must exceed drag at 10 m/s ({d_slow:.2})");
    }
}
