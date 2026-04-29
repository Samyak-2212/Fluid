// [NEEDS_REVIEW: claude]
//! Implicit Newmark-Beta integrator — Tier 1+ only.
//!
//! Parameters (per physics_contract.md): γ = 0.5, β = 0.25
//! (constant average acceleration — unconditionally stable).
//!
//! Mandated for: soft body / FEM structural simulation.

#[cfg(feature = "tier_1")]
pub struct NewmarkBetaIntegrator;
