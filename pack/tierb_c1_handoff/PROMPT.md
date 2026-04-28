# Tier B Agent — C1 Core Systems Continuation

## Context

You are a Tier B agent. Claude (Tier A) was running C1 (Core Systems Coordinator)
and exhausted its weekly quota mid-session. You are continuing from where it stopped.

Read AGENTS.md and knowledge/model_tier_policy.md before acting.
Read knowledge_b/ if any entries exist — treat them as field notes, not ground truth.

---

## What Claude already completed — DO NOT TOUCH

These files are frozen Tier A output. Never modify them. If you accidentally edit one,
tag it [NEEDS_REVIEW: claude] at the top and file a BUG_POOL entry immediately.

- core/src/units.rs — SI newtypes, macro-generated, fully tested
- core/src/ecs/traits.rs — EntityId, Component, System, World traits
- core/src/ecs/mod.rs — re-exports
- core/src/event_bus.rs — Event + EventBus traits
- core/src/lib.rs — module declarations
- core/build.rs — FLUID_TIER env to tier_N cfg emission
- core/Cargo.toml — tier features declared, metadata.fluid present

---

## Your tasks

### Task 1 — Wire glam for math primitives

File: core/src/math/mod.rs
Current state: stub, no re-exports, glam not added to Cargo.toml

Steps:
1. Check docs.rs/glam for the latest stable version.
2. Add glam to root Cargo.toml under [workspace.dependencies]:
   glam = { version = "X.Y.Z" }
3. Add to core/Cargo.toml under [dependencies]:
   glam = { workspace = true }
4. Replace the stub body in core/src/math/mod.rs with:

   pub use glam::{Vec2, Vec3, Vec4, Mat3, Mat4, Quat, DVec3, DMat4, DQuat};

5. Remove the [UNVERIFIED: glam version] comment after confirming the version.
6. Tag core/src/math/mod.rs with [NEEDS_REVIEW: claude] at the top.
7. Run: cargo build (from workspace root). Must compile clean.

### Task 2 — Implement the fixed-timestep manager

File: core/src/time/mod.rs
Current state: stub with interface described in comments only

Implement per coordinators/core/PROMPT.md "Time-Step Manager" section:
- Struct Timestep with fields: dt (Seconds), accumulated (Seconds)
- Load dt from config/core.toml key timestep_seconds. Default: 1.0/60.0 if missing.
- Never panic on missing config. Supply the default silently.
- Methods: dt(&self) -> Seconds, accumulated(&self) -> Seconds, tick(&mut self) -> bool
- tick() adds frame_time to accumulator, subtracts dt each call, returns true if a
  fixed step should execute. Caller loops until tick() returns false.

Create config/core.toml if it does not exist:
  timestep_seconds = 0.016666666666666666

Tag core/src/time/mod.rs with [NEEDS_REVIEW: claude] at top.

### Task 3 — Implement rayon ThreadPool wrapper

File: core/src/threading/traits.rs already has the ThreadPool trait.
Create: core/src/threading/rayon_pool.rs

Steps:
1. Check docs.rs/rayon for the latest stable version.
2. Add rayon to workspace dependencies and core/Cargo.toml.
3. Implement a RayonPool struct that wraps rayon's global pool behind ThreadPool.
4. Re-export RayonPool from core/src/threading/mod.rs.
5. Tag core/src/threading/rayon_pool.rs with [NEEDS_REVIEW: claude] at top.

### Task 4 — Minimal stubs for memory and scene

Files: core/src/memory/mod.rs, core/src/scene/mod.rs
These are placeholders only. Add a comment // TODO: post-gate Tier B work and a
minimal pub struct or type alias so the module compiles and is non-empty.
Do NOT design interfaces. Do NOT tag these [NEEDS_REVIEW: claude].

### Task 5 — Build verification

Run from workspace root:
  cargo build

Must complete without errors. Warnings are acceptable.
If it fails, fix only the specific error. Do not refactor frozen files.
If a frozen file must change, file it in bug_pool/BUG_POOL.md and stop.

---

## After completing all tasks — write to knowledge_b/

You do NOT write to knowledge/ or knowledge/project_manifest.md.
Gate signals ([C1_INTERFACES_PUBLISHED], [C1_COMPLETE]) are Claude-only writes.

Write a file: knowledge_b/<your_agent_id>_<timestamp>_c1_status.md

Contents must include:
- List of every file you created or modified
- Verified crate versions (glam, rayon) with docs.rs confirmation note
- Tag list: every file you tagged [NEEDS_REVIEW: claude]
- Build result: PASS or FAIL with error if FAIL
- Any [UNRESOLVED] items you encountered and did not resolve
- Any BUG_POOL entries you filed (IDs or descriptions)
- Explicit statement: "core/src/units.rs, ecs/traits.rs, event_bus.rs — not touched"

Format every entry as a plain fact. No conclusions. No architectural recommendations.
Example: "glam version 0.29.0 confirmed on docs.rs 2026-04-27"
Example: "core/src/math/mod.rs — created, tagged [NEEDS_REVIEW: claude]"

This file is how Claude will know what happened when quota returns.

---

## Tier B hard rules (from AGENTS.md)

- Never modify: knowledge/, coordinators/*/PROMPT.md, ROOT_COORDINATOR.md
  File proposed changes in bug_pool/BUG_POOL.md under Prompt/Knowledge Changes.
- Never write gate signals. knowledge_b/ is your only output channel to the knowledge layer.
- Tag any output touching physics, rendering, unsafe, CUDA/ROCm: [NEEDS_REVIEW: claude]
- After 15 tool calls: write a pack file to pack/<agent_id>_<timestamp>/context.md, then continue.
- Check bug_pool/BUG_POOL.md before starting.
- Update knowledge/file_structure.md after touching more than 3 files.
  (file_structure.md is operational — Tier B may write it)
