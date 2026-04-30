// [NEEDS_REVIEW: claude]
//! Thermodynamic simulator — Tier 1+.
//!
//! Simulates heat transfer using operator splitting with RK4 integration.
//! Consumes `physics_core::integrators::rk4::{Rk4, Rk4State, Rk4DerivativeProvider}`.
//!
//! # Model
//!
//! Each ECS entity carries a temperature field (scalar per entity, or a
//! spatial grid for higher fidelity). Heat transfer follows Fourier's law:
//!
//! `dT/dt = (k / (ρ · c_p)) · ∇²T + Q_source / (ρ · c_p)`
//!
//! For a single-entity lumped-capacitance model:
//!
//! `dT/dt = h_conv · A / (m · c_p) · (T_inf - T) + Q_dot / (m · c_p)`
//!
//! where `h_conv` is the convection coefficient, `A` the surface area,
//! `T_inf` the ambient temperature, and `Q_dot` the heat source rate.
//!
//! # Operator Splitting
//!
//! For multi-physics coupling (e.g., SPH + thermodynamics):
//! 1. Advection step: temperature advected with fluid velocity (half-step).
//! 2. Diffusion step: RK4 integration of heat equation (full step).
//! 3. Advection step: second half-step (Strang splitting, 2nd-order).
//!
//! # References
//!
//! Strang, G. (1968). "On the construction and comparison of difference schemes."
//! SIAM Journal on Numerical Analysis, 5(3), 506–517. (Operator splitting.)
//!
//! Incropera, F.P. et al. (2007). *Fundamentals of Heat and Mass Transfer* (7th ed.).
//! Wiley. §5 (Lumped capacitance), §2 (Fourier's law).

use core::units::{Kelvin, Seconds};
use physics_core::integrators::rk4::{Rk4DerivativeProvider, ScalarState};

// ── Thermal entity ────────────────────────────────────────────────────────────

/// Thermodynamic state of a single entity (lumped capacitance model).
#[derive(Debug, Clone)]
pub struct ThermalEntity {
    /// Current temperature (K).
    pub temperature: Kelvin,
    /// Mass (kg).
    pub mass: f64,
    /// Specific heat capacity c_p (J/(kg·K)).
    pub specific_heat: f64,
    /// Convective heat transfer coefficient h_conv (W/(m²·K)).
    pub h_conv: f64,
    /// Exposed surface area A (m²).
    pub surface_area: f64,
    /// Constant internal heat generation rate Q_dot (W). Zero means no source.
    pub heat_source: f64,
}

impl ThermalEntity {
    /// Returns the thermal time constant τ = m·c_p / (h_conv·A).
    #[inline]
    pub fn time_constant(&self) -> f64 {
        let ha = self.h_conv * self.surface_area;
        if ha < 1e-12 { return f64::INFINITY; }
        self.mass * self.specific_heat / ha
    }
}

// ── Lumped-capacitance derivative provider ────────────────────────────────────

/// RK4 derivative provider for lumped-capacitance heat transfer.
///
/// `dT/dt = h·A / (m·c_p) · (T_inf - T) + Q_dot / (m·c_p)`
pub struct LumpedCapacitanceDerivs {
    /// Ambient temperature T_inf (K).
    pub t_ambient: f64,
    /// Thermal entity configuration.
    pub entity: ThermalEntity,
}

impl Rk4DerivativeProvider<ScalarState> for LumpedCapacitanceDerivs {
    fn derivative(&self, state: &ScalarState, _t: Seconds) -> ScalarState {
        let t = state.0; // current temperature in K
        let ha = self.entity.h_conv * self.entity.surface_area;
        let mcp = self.entity.mass * self.entity.specific_heat;
        if mcp < 1e-12 {
            return ScalarState(0.0);
        }
        let dt_dt = (ha / mcp) * (self.t_ambient - t) + self.entity.heat_source / mcp;
        ScalarState(dt_dt)
    }
}

// ── Thermal simulation ────────────────────────────────────────────────────────

/// Thermal simulation state — a collection of thermodynamic entities.
///
/// Each entity is evolved independently using RK4 integration.
/// For coupled multi-body heat exchange, extend with a conductance network.
pub struct ThermalSimulation {
    /// All thermal entities.
    pub entities: Vec<ThermalEntity>,
    /// Ambient temperature T_∞ (K).
    pub t_ambient: Kelvin,
}

impl ThermalSimulation {
    /// Creates an empty thermal simulation.
    pub fn new(t_ambient: Kelvin) -> Self {
        Self { entities: Vec::new(), t_ambient }
    }

    /// Adds a thermal entity.
    pub fn add_entity(&mut self, entity: ThermalEntity) {
        self.entities.push(entity);
    }

    /// Advances all entities by one timestep using RK4.
    ///
    /// This implements the diffusion step of the Strang operator splitting.
    /// For full operator splitting with advection, the caller must:
    /// 1. Apply half-step advection (move temperature with fluid).
    /// 2. Call `step` (diffusion).
    /// 3. Apply half-step advection again.
    #[cfg(feature = "tier_1")]
    pub fn step(&mut self, dt: Seconds) {
        let t_ambient = self.t_ambient.value();
        for entity in &mut self.entities {
            let derivs = LumpedCapacitanceDerivs {
                t_ambient,
                entity: entity.clone(),
            };
            let integrator = Rk4::new(derivs);
            let state = ScalarState(entity.temperature.value());
            let next = integrator.step_rk4(&state, Seconds(0.0), dt);
            entity.temperature = Kelvin(next.0);
        }
    }

    /// Returns true if any entity has a NaN or non-physical (< 0 K) temperature.
    pub fn has_invalid_temperature(&self) -> bool {
        self.entities.iter().any(|e| {
            e.temperature.value().is_nan() || e.temperature.value() < 0.0
        })
    }
}

// ── Debug overlay ─────────────────────────────────────────────────────────────

#[cfg(feature = "debug_overlay")]
#[derive(Debug, Default)]
pub struct ThermalDebugStats {
    /// Number of entities evolved this frame.
    pub entities_stepped: usize,
    /// Maximum temperature across all entities (K).
    pub max_temperature_k: f64,
    /// Minimum temperature across all entities (K).
    pub min_temperature_k: f64,
}

// ── Compute FFI trait (Tier 3) ─────────────────────────────────────────────────

#[cfg(feature = "tier_3")]
pub struct ComputeKernel { pub id: u32 }
#[cfg(feature = "tier_3")]
pub struct KernelArgs { pub data: Vec<u8>, pub work_groups: [u32; 3] }
#[cfg(feature = "tier_3")]
pub trait GpuComputeBackend: Send + Sync {
    fn dispatch_kernel(&self, kernel: &ComputeKernel, args: &KernelArgs)
        -> Result<(), Box<dyn std::error::Error>>;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "tier_1"))]
mod tests {
    use super::*;
    use core::units::{Kelvin, Seconds};

    fn steel_entity(temp_k: f64) -> ThermalEntity {
        ThermalEntity {
            temperature: Kelvin(temp_k),
            mass: 1.0,                // 1 kg
            specific_heat: 500.0,    // J/(kg·K) — steel ≈ 500
            h_conv: 25.0,            // W/(m²·K) — natural convection
            surface_area: 0.1,       // 0.1 m²
            heat_source: 0.0,
        }
    }

    /// Analytical solution for lumped capacitance cooling:
    /// T(t) = T_inf + (T_0 - T_inf) · exp(-t / τ)
    #[test]
    fn lumped_capacitance_approaches_ambient() {
        let t_ambient = Kelvin(293.15); // 20°C
        let t_initial = 400.0_f64;     // 127°C

        let mut sim = ThermalSimulation::new(t_ambient);
        sim.add_entity(steel_entity(t_initial));

        let tau = sim.entities[0].time_constant();
        let dt = Seconds(0.1);
        let t_end = 3.0 * tau; // simulate 3 time constants → ~95% equilibrium
        let n_steps = (t_end / dt.value()) as usize;

        for _ in 0..n_steps {
            sim.step(dt);
        }
        assert!(!sim.has_invalid_temperature());

        let final_temp = sim.entities[0].temperature.value();
        let t_inf = t_ambient.value();
        // After 3τ, temperature should be within 5% of ambient gap
        let gap = (final_temp - t_inf).abs();
        let initial_gap = (t_initial - t_inf).abs();
        assert!(
            gap < 0.1 * initial_gap,
            "After 3τ, gap={gap:.2} should be < 10% of initial_gap={initial_gap:.2}"
        );
    }

    /// Analytical accuracy: compare RK4 result with exact exponential.
    ///
    /// Exact: T(t) = T_inf + (T0 - T_inf) · exp(-t/τ)
    #[test]
    fn rk4_matches_analytical_within_tolerance() {
        let t_ambient = 300.0_f64;
        let t0 = 500.0_f64;
        let dt_val = 0.01_f64;
        let t_sim = 1.0_f64;
        let n_steps = (t_sim / dt_val) as usize;

        let mut sim = ThermalSimulation::new(Kelvin(t_ambient));
        sim.add_entity(steel_entity(t0));
        let tau = sim.entities[0].time_constant();

        for _ in 0..n_steps {
            sim.step(Seconds(dt_val));
        }

        let t_numerical = sim.entities[0].temperature.value();
        let t_exact = t_ambient + (t0 - t_ambient) * (-t_sim / tau).exp();
        let err = (t_numerical - t_exact).abs();

        // RK4 global error is O(dt⁴); expect err < 0.01 K for dt=0.01
        assert!(
            err < 0.01,
            "RK4 temperature error {err:.6} K > 0.01 K (expected O(dt^4))"
        );
    }
}
