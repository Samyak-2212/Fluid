# Handoff Prompt

Role: C5 — Sim Components Coordinator
Domain: components/ — fluid_simulator, aerodynamic_simulator, motion_force_simulator, thermodynamic_simulator, fem_structural
Model: Gemini 3.1 Pro
Task: Begin scaffolding C5 simulation components following the publication of [C4_COMPLETE].

## Read First

1. AGENTS.md
2. coordinators/sim_components/PROMPT.md
3. knowledge/physics_contract.md
4. knowledge/dependency_graph.md
5. knowledge/capability_tiers.md
6. knowledge/model_tier_policy.md
7. bug_pool/BUG_POOL.md
8. pack/c4_physics_core_continuation_20260429T164538Z/context.md
9. .agents/qa/model_routing_table.md

## Current State

- Status: [C4_COMPLETE] ✅ committed (SHA: a4018a7aa5dd7d52baa3c0b77b8d9d1e11a6a276)
- The `physics_core` domain is fully implemented and tested (22 tests passing).
- `core/units` is available for strict dimensional typing.
- All integration schemes (VelocityVerlet, LeapFrog) are available.

## Constraints & Requirements

- You are scaffolding the sub-components. Follow `coordinators/sim_components/PROMPT.md` closely.
- Review `knowledge/capability_tiers.md` to ensure you respect the tiering requirements (e.g., CUDA/ROCm FFI requires Tier A review).
- Resolve the `[UNRESOLVED]` tag regarding Intel oneAPI support if instructed.
- All Tier B output must respect the `[NEEDS_REVIEW: claude]` policy.

## Next Step

1. Read your domain's coordinator PROMPT.md.
2. Begin scaffolding the component directories (`fluid_simulator/`, etc.) per your instructions.
3. Follow all gate signals and update `knowledge/` manifests appropriately as you progress.
