// [REVIEWED: claude — C5 complete gate + C7 quality gate, 2026-05-02. No issues found.]
//! Runge-Kutta 4th order (RK4) integrator — Tier 1+ only.
//!
//! Mandated for: CFD (projection method stages) and thermodynamics
//! (operator splitting, consumed by C5 `fluid_simulator` CFD mode and
//! `thermodynamic_simulator`).
//!
//! # Scheme
//!
//! Classical RK4 for `dy/dt = f(t, y)`:
//!
//! ```text
//! k1 = f(t,          y)
//! k2 = f(t + dt/2,   y + dt/2 · k1)
//! k3 = f(t + dt/2,   y + dt/2 · k2)
//! k4 = f(t + dt,     y + dt   · k3)
//! y(t+dt) = y + dt/6 · (k1 + 2·k2 + 2·k3 + k4)
//! ```
//!
//! The state type must support scalar-weighted addition. This is expressed
//! via the [`Rk4State`] trait which components implement for their own
//! state vectors.
//!
//! # Algorithm source
//!
//! Kutta, W. (1901). "Beitrag zur näherungsweisen Integration totaler
//! Differentialgleichungen." Z. Math. Phys. 46, 435–453.
//!
//! Press, W.H. et al. (2007). *Numerical Recipes* (3rd ed.), §17.1.

use core::units::Seconds;

// ── Rk4State trait ────────────────────────────────────────────────────────────

/// State type that can be used with the RK4 integrator.
///
/// Implementors must support the weighted-average combination required
/// by the four-stage RK4 scheme. All arithmetic is done in `f64` weight units.
pub trait Rk4State: Clone + Send {
    /// Returns `self + weight · rhs`.
    fn scaled_add(&self, rhs: &Self, weight: f64) -> Self;
}

// ── Derivative provider ───────────────────────────────────────────────────────

/// Supplies `dy/dt` for a state `y` at time `t`.
///
/// Components implement this to provide their specific ODE right-hand side.
/// For CFD: `f = Navier-Stokes RHS`. For thermodynamics: `f = heat diffusion RHS`.
pub trait Rk4DerivativeProvider<S: Rk4State>: Send + Sync {
    /// Returns `f(t, state)` = `d(state)/dt`.
    fn derivative(&self, state: &S, t: Seconds) -> S;
}

// ── Integrator ────────────────────────────────────────────────────────────────

/// Classical RK4 integrator — Tier 1+ only.
///
/// # Type parameters
///
/// - `S` — state type implementing [`Rk4State`]
/// - `D` — derivative provider implementing [`Rk4DerivativeProvider<S>`]
///
/// The provider is stored by reference; the integrator itself is zero-cost.
///
/// # Usage
///
/// ```rust,ignore
/// use physics_core::integrators::rk4::{Rk4, Rk4State, Rk4DerivativeProvider};
/// use core::units::Seconds;
///
/// struct ThermalState { temperature: f64 }
/// impl Rk4State for ThermalState { /* ... */ }
///
/// struct ThermalDerivs { conductivity: f64 }
/// impl Rk4DerivativeProvider<ThermalState> for ThermalDerivs { /* ... */ }
///
/// let integrator = Rk4::new(ThermalDerivs { conductivity: 1.2 });
/// let next = integrator.step_rk4(&state, Seconds(0.01));
/// ```
#[cfg(feature = "tier_1")]
pub struct Rk4<S: Rk4State, D: Rk4DerivativeProvider<S>> {
    provider: D,
    _phantom: std::marker::PhantomData<S>,
}

#[cfg(feature = "tier_1")]
impl<S: Rk4State, D: Rk4DerivativeProvider<S>> Rk4<S, D> {
    /// Creates a new RK4 integrator wrapping the given derivative provider.
    pub fn new(provider: D) -> Self {
        Self { provider, _phantom: std::marker::PhantomData }
    }

    /// Advances the state by one timestep using the classical RK4 scheme.
    ///
    /// `t` is the current simulation time (used by the derivative provider
    /// for time-dependent forcing; pass `Seconds(0.0)` if time-invariant).
    pub fn step_rk4(&self, state: &S, t: Seconds, dt: Seconds) -> S {
        let dt_val = dt.value();
        let half_dt = Seconds(dt_val * 0.5);
        let t_half = Seconds(t.value() + dt_val * 0.5);
        let t_end  = Seconds(t.value() + dt_val);

        // k1 = f(t, y)
        let k1 = self.provider.derivative(state, t);
        // k2 = f(t + dt/2, y + dt/2 · k1)
        let y2 = state.scaled_add(&k1, dt_val * 0.5);
        let k2 = self.provider.derivative(&y2, t_half);
        // k3 = f(t + dt/2, y + dt/2 · k2)
        let y3 = state.scaled_add(&k2, dt_val * 0.5);
        let k3 = self.provider.derivative(&y3, t_half);
        // k4 = f(t + dt, y + dt · k3)
        let y4 = state.scaled_add(&k3, dt_val);
        let k4 = self.provider.derivative(&y4, t_end);

        // y(t+dt) = y + dt/6 · (k1 + 2·k2 + 2·k3 + k4)
        // = y + dt·k1/6 + dt·k2/3 + dt·k3/3 + dt·k4/6
        let sixth  = dt_val / 6.0;
        let third  = dt_val / 3.0;

        let result = state
            .scaled_add(&k1, sixth)
            .scaled_add(&k2, third)
            .scaled_add(&k3, third)
            .scaled_add(&k4, sixth);

        let _ = half_dt; // suppress unused warning

        result
    }
}

// ── Integrator trait impl (scalar convenience wrapper) ────────────────────────

/// Scalar RK4 state — a single `f64` value.
///
/// Used for 1D ODE tests and simple thermodynamic scalar fields.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScalarState(pub f64);

impl Rk4State for ScalarState {
    fn scaled_add(&self, rhs: &Self, weight: f64) -> Self {
        ScalarState(self.0 + weight * rhs.0)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "tier_1"))]
mod tests {
    use super::*;
    use core::units::Seconds;

    /// Simple exponential decay: dy/dt = -λ·y, y(0) = 1.
    /// Exact solution: y(t) = exp(-λ·t).
    struct ExponentialDecay {
        lambda: f64,
    }

    impl Rk4DerivativeProvider<ScalarState> for ExponentialDecay {
        fn derivative(&self, state: &ScalarState, _t: Seconds) -> ScalarState {
            ScalarState(-self.lambda * state.0)
        }
    }

    #[test]
    fn exponential_decay_fourth_order_accuracy() {
        let lambda = 2.0_f64;
        let dt = Seconds(0.01);
        let t_end = 1.0_f64;
        let n_steps = (t_end / dt.value()) as usize;

        let integrator = Rk4::new(ExponentialDecay { lambda });
        let mut state = ScalarState(1.0);
        let mut t = Seconds(0.0);

        for _ in 0..n_steps {
            state = integrator.step_rk4(&state, t, dt);
            t = Seconds(t.value() + dt.value());
        }

        let exact = (-lambda * t_end).exp();
        let err = (state.0 - exact).abs();

        // RK4 global error is O(dt⁴); for dt=0.01 expect err < 1e-7
        assert!(
            err < 1e-7,
            "RK4 exponential decay error {err:.2e} > 1e-7 (expected O(dt^4))"
        );
    }

    /// Harmonic oscillator: u'' = -ω²·u → system y = [u, v], dy/dt = [v, -ω²·u].
    #[derive(Clone)]
    struct OscState {
        pos: f64,
        vel: f64,
    }

    impl Rk4State for OscState {
        fn scaled_add(&self, rhs: &Self, w: f64) -> Self {
            OscState { pos: self.pos + w * rhs.pos, vel: self.vel + w * rhs.vel }
        }
    }

    struct OscDerivs {
        omega_sq: f64,
    }

    impl Rk4DerivativeProvider<OscState> for OscDerivs {
        fn derivative(&self, s: &OscState, _t: Seconds) -> OscState {
            OscState { pos: s.vel, vel: -self.omega_sq * s.pos }
        }
    }

    #[test]
    fn harmonic_oscillator_energy_conserved() {
        let omega_sq = 4.0_f64; // ω = 2
        let dt = Seconds(0.001);
        let n_steps = 10_000; // 10 seconds

        let integrator = Rk4::new(OscDerivs { omega_sq });
        let mut state = OscState { pos: 1.0, vel: 0.0 };
        let mut t = Seconds(0.0);
        let e0 = 0.5 * omega_sq * state.pos * state.pos + 0.5 * state.vel * state.vel;

        for _ in 0..n_steps {
            state = integrator.step_rk4(&state, t, dt);
            t = Seconds(t.value() + dt.value());
        }

        let ef = 0.5 * omega_sq * state.pos * state.pos + 0.5 * state.vel * state.vel;
        let drift = (ef - e0).abs() / e0;
        assert!(drift < 1e-5, "RK4 oscillator energy drift {drift:.2e} > 1e-5");
    }
}
