# Handoff Prompt

Role: C4 — Physics Core Coordinator (continuation)
Domain: physics_core/ — integrators, GJK+EPA collision, constraint solver, rigid body, soft body
Model: Claude Sonnet
Task: Complete the C4 full implementation gate ([C4_COMPLETE]) after [C4_INTERFACES_PUBLISHED] was published.

## Read First

1. AGENTS.md
2. coordinators/physics_core/PROMPT.md
3. knowledge/physics_contract.md
4. knowledge/dependency_graph.md
5. knowledge/capability_tiers.md
6. knowledge/model_tier_policy.md
7. bug_pool/BUG_POOL.md
8. pack/c4_physics_core_20260429T230621Z/context.md  ← this session's verified state
9. .agents/qa/model_routing_table.md                  ← read before writing any handoff

## Current State

- Status: [C4_INTERFACES_PUBLISHED] ✅ committed (SHA: 7aa494c26ccc3e8de729844d90464e8323d407f1)
- Gate files: all three traits.rs files exist and compile
- cargo test -p physics_core: 6 passed, 0 failed
- cargo check --workspace: EXIT:0
- All implementation stubs are PENDING — no concrete integrator or solver code exists yet

## What Is Done

- physics_core/src/integrators/traits.rs — trait surface complete
- physics_core/src/collision/traits.rs — trait surface complete
- physics_core/src/constraints/traits.rs — trait surface complete
- physics_core/src/rigid_body/mod.rs — RigidBody struct with 6 tests
- All module scaffolding and stubs in place
- physics_core/Cargo.toml — glam workspace=true, faer optional [UNVERIFIED]
- physics_core/build.rs — FLUID_TIER emission done
- config/physics_core.toml — both tunables present

## Constraints

- Euler integrator must remain `#[cfg(feature = "tier_0")]` only — never exposed at Tier 1+
- No reimplementation of core::units — consume only
- glam 0.32.1 via workspace = true — do not override
- faer version [UNVERIFIED] — verify on crates.io before using in any FEM code
- GJK algorithm source [UNVERIFIED] — verify paper citation before implementing
- Any Tier B output in integrators/, collision/, constraints/ must be tagged [NEEDS_REVIEW: claude]
- config keys loaded at runtime — no hardcoding, no panic on missing key
- After 15 tool calls: write pack, continue or hand off

## Remaining Blocker

None for C5. C4 full completion ([C4_COMPLETE]) is independent.

## Next Step

1. Verify faer version on crates.io — resolve [UNVERIFIED] in physics_core/Cargo.toml
2. Implement VelocityVerlet — test against two-body orbit energy conservation
3. Implement LeapFrog (C5 depends on this for SPH)
4. Implement GJK (verify algorithm source first) + EPA
5. Implement SequentialImpulseSolver (constraint_solver_iterations from config, not hardcoded)
6. Implement NewmarkBeta (tier_1), RK4 (tier_1)
7. All must pass at least one analytical reference test (physics_contract.md §Verification)
8. Write [C4_COMPLETE] to knowledge/project_manifest.md → hard retirement trigger

## Deliverables for [C4_COMPLETE]

- Velocity Verlet: energy conservation test passing
- Leap-Frog: impl
- GJK: intersection test passing on known pairs
- EPA: penetration depth accuracy test passing
- Sequential impulse solver: constraint satisfaction test passing
- All [UNVERIFIED] tags resolved
- config/physics_core.toml: runtime-loaded, no hardcoding
- [C4_COMPLETE] in knowledge/project_manifest.md
- Pack file and handoff prompt written and presented
