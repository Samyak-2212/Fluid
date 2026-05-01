# Handoff Prompt

Role: C5 — Sim Components Coordinator (Implementation Phase)
Domain: components/
Model: Claude Sonnet
Task: Implement simulation components (SPH, FEM, FFI bridges) requiring numerical solvers and memory-safe traits.

## Read First
1. AGENTS.md
2. coordinators/sim_components/PROMPT.md
3. pack/c5_scaffold_20260429T213456Z/context.md

## Current State
- Scaffold complete. `components/*/Cargo.toml` and `config/*.toml` are generated.
- Sub-coordinator prompts are located in `coordinators/sim_components/*/PROMPT.md`.

## Next Step
- Assume control of the C5 sub-coordinators.
- Begin implementing FFI traits (CUDA/ROCm) and numerical algorithms (Wendland C2, Newmark-Beta, etc.).
- Be sure to use the `[NEEDS_REVIEW: claude]` tag if delegating back to Tier B, though as Tier A you can implement directly.
- Resolve the `[UNRESOLVED]` tag for oneAPI.
