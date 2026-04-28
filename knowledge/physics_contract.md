<!-- version: 1 -->
# Physics Accuracy Contract

Every physics component coordinator must read this file before writing their PROMPT.

## Units

SI exclusively. No unit conversion at runtime.
Enforce dimensional correctness via newtype wrappers (e.g. `Meters(f64)`).
A `units` module in `core` is mandatory before any physics work begins.

**Ownership:** C1 owns the `units` module exclusively. C4 consumes it as a dependency.
There is no negotiation — C4 must not re-implement or fork this module.

## Time Integration

Each physics domain must explicitly select its integrator:

| Domain | Integrator | Rationale |
|--------|-----------|-----------|
| Rigid body dynamics | Velocity Verlet (symplectic) | Preserves energy |
| Soft body / FEM | Implicit Newmark-beta or HHT-alpha | Unconditional stability for stiff systems |
| Fluid (SPH) | Leap-frog symplectic | Symplectic, time-reversible |
| Fluid (grid/CFD) | RK4 or implicit Crank-Nicolson | Accuracy + stability trade-off |
| Thermodynamics | Operator splitting with RK4 | Decouples fast/slow dynamics |

**Euler integration** is only permitted in Tier 0 simplified mode and must be gated with
the `tier_0` Cargo feature flag: `#[cfg(feature = "tier_0")]`.

No Euler integration in any scientific accuracy path (Tiers 1–3).

## Numerical Methods per Domain

### Structural FEM
- Linear and nonlinear (Newton-Raphson) solvers
- Sparse solvers: `faer` or `nalgebra-sparse`
- Minimum tier: 1 (basic FEM), 2 (full nonlinear)

### CFD
- Incompressible Navier-Stokes: projection method
- Compressible: Euler equations
- Minimum tier: 1 (incompressible), 2 (compressible)

### SPH
- Kernel: Wendland C2
- Correction: XSPH
- Density: summation (not continuity)
- Minimum tier: 0 (low-res), 1 (medium), 2 (high-res)

### Rigid Body
- Collision detection: GJK + EPA
- Constraint solver: Sequential impulse
- Minimum tier: 0

## Dimensional Correctness

All quantities crossing module boundaries must use SI newtype wrappers from `core::units`.

Examples:
```rust
pub struct Meters(pub f64);
pub struct Kilograms(pub f64);
pub struct Seconds(pub f64);
pub struct Newtons(pub f64);
pub struct Pascals(pub f64);
pub struct KilogramsPerCubicMeter(pub f64);
```

Mixing raw `f64` for physical quantities is a bug. Tag it `[NEEDS_REVIEW: claude]` if
a Tier B model writes raw `f64` for any quantity that has a physical unit.

## Verification Requirement

C7 (Quality Gate) must validate physics output against known analytical solutions.
Examples:
- Rigid body: two-body gravitational orbit conserves energy and angular momentum
- SPH: Sod shock tube reference solution
- FEM: cantilever beam deflection formula

No physics component may be marked `[CX_COMPLETE]` without passing at least one
analytical reference test.

## Hallucination Checkpoint

After producing any mathematical formula, crate API call, or version number, the agent
must verify it against source (docs.rs, crates.io, the paper). Mark unverified items
`[UNVERIFIED]`. Do not use `[UNVERIFIED]` items in physics, rendering, or unsafe code.
