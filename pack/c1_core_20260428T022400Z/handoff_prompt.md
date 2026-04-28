# C1 Handoff Prompt

## For: Next C1 continuation session (post-gate implementation work)

Read these files first, in order:
1. `pack/c1_core_20260428T022400Z/context.md` — this session's full state
2. `knowledge/project_manifest.md` — gate signals written so far
3. `knowledge/dependency_graph.md` — what is now unblocked
4. `bug_pool/BUG_POOL.md` — check before starting

## Gate Status

[C1_INTERFACES_PUBLISHED] has been written to `knowledge/project_manifest.md`.
C3, C4, and C6 are now unblocked.

## Remaining C1 Work (post-gate, Tier B permitted for implementation)

Priority order:

1. **`config/core.toml`** — create with `timestep_seconds` and any other tunables.
   No hardcoded values in source. Required before implementing `time/mod.rs`.

2. **`core/src/time/mod.rs`** — implement `Timestep` struct.
   Interface: `tick() -> bool`, `dt() -> Seconds`, `accumulated() -> Seconds`.
   Load `timestep_seconds` from `config/core.toml` via a typed config reader.

3. **`core/src/math/mod.rs`** — verify glam version on docs.rs, add to workspace
   `Cargo.toml` under `[workspace.dependencies]`, re-export `Vec2`, `Vec3`, `Vec4`,
   `Mat3`, `Mat4`, `Quat`, `DVec3`, `DMat4`, `DQuat` from `core::math`.
   Remove [UNVERIFIED] tag after verification.

4. **`core/src/threading/traits.rs`** — verify rayon version on docs.rs, add to
   workspace deps, implement `RayonPool: ThreadPool` wrapper. Remove [UNVERIFIED].

5. **Concrete ECS** — archetype world impl behind `World` trait.
   Tag output `[NEEDS_REVIEW: claude]` if written by Tier B.

6. **Concrete EventBus** — `RwLock<HashMap<TypeId, Vec<Box<dyn Fn>>>>` pattern.
   Tag output `[NEEDS_REVIEW: claude]` if written by Tier B.

## Files You Own

Everything under `core/`. Do not touch `physics_core/`, `rendering/`, `builder/`,
`debugger/`, `components/`, or any `knowledge/` file without incrementing its version.

## On Completion

When ALL of the above are implemented:
- Write `[C1_COMPLETE]` to `knowledge/project_manifest.md`
- Write a final pack file to `pack/c1_complete_<timestamp>/context.md`
- Present this handoff prompt as a fenced code block
- Retire immediately

## Workspace Warning (low priority, C2's problem)

`Cargo.toml` emits: `unused manifest key: workspace.edition`
File a low-severity note for C2 in `bug_pool/BUG_POOL.md` if not already present.
Do not modify `Cargo.toml` yourself — C2 owns it.
