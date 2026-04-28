# C1 Core Systems — Session Context Pack

Session ID: c1_core_20260428T022400Z
Created: 2026-04-28T02:24:00+05:30
Status: RETIRED (hard retirement on [C1_INTERFACES_PUBLISHED])

---

## Gate Status

[C1_INTERFACES_PUBLISHED] — COMPLETE

All three gate files exist, are non-empty, and pass `cargo test -p core` (13/13, EXIT:0).

---

## Files Created

### core/ (owned by C1)

| File | Purpose |
|------|---------|
| `core/Cargo.toml` | Tier features (tier_0..tier_3), metadata.fluid |
| `core/build.rs` | Reads FLUID_TIER, emits rustc-cfg feature="tier_N" |
| `core/src/lib.rs` | Public crate API; declares all submodules |
| `core/src/units.rs` | **GATE FILE** — 13 SI newtype wrappers via `si_unit!` macro |
| `core/src/ecs/mod.rs` | Re-exports EntityId, Component, World, System |
| `core/src/ecs/traits.rs` | **GATE FILE** — EntityId(u64), Component (blanket), World, System<W: World> |
| `core/src/event_bus.rs` | **GATE FILE** — Event (blanket), EventBus traits |
| `core/src/math/mod.rs` | Stub — glam re-export pending version verification |
| `core/src/time/mod.rs` | Stub — Timestep interface documented, impl deferred |
| `core/src/threading/mod.rs` | Stub |
| `core/src/threading/traits.rs` | ThreadPool trait — rayon version [UNVERIFIED] |
| `core/src/memory/mod.rs` | Stub |
| `core/src/scene/mod.rs` | Stub |

### Placeholder stubs created for workspace resolution (owned by respective coordinators)

- `physics_core/Cargo.toml`, `physics_core/src/lib.rs`
- `rendering/Cargo.toml`, `rendering/src/lib.rs`
- `debugger/Cargo.toml`, `debugger/src/lib.rs`
- `components/fluid_simulator/src/lib.rs`
- `components/aerodynamic_simulator/src/lib.rs`
- `components/motion_force_simulator/src/lib.rs`

---

## Architectural Decisions

### System<W: World> instead of System with &mut dyn World

`World` has generic methods (`insert<C>`, `get<C>`, `get_mut<C>`, `remove<C>`) which
make it dyn-incompatible in Rust's stable object model. `System<W: World>` preserves
the full generic interface. `Box<dyn System<ConcreteWorld>>` is usable for a known W.
If type-erased world dispatch is needed in the future, a separate `AnyWorld` trait
with `Any` downcasting should be introduced under Tier A review.

### No concrete ECS implementation

The concrete storage layer (archetype table, sparse sets, etc.) is Tier B work.
No concrete impl was written; only the trait surface.

### No concrete EventBus implementation

Deferred. Likely a `RwLock<HashMap<TypeId, Vec<Box<dyn Fn>>>>` pattern.

### glam and rayon versions

Both are [UNVERIFIED] — must be confirmed against docs.rs before adding to
`Cargo.toml`. The workspace currently has no math or threading dependencies;
the math and threading modules are stubs.

---

## Known Issues / Items for Next C1 Session

1. **`config/core.toml`** — not yet created. Timestep value and other core tunables
   must be loaded from this file (not hardcoded). Create before implementing `time/mod.rs`.

2. **glam version** — must be verified on docs.rs before wiring into `core/src/math/mod.rs`.

3. **rayon version** — must be verified on docs.rs before wiring into `core/src/threading/`.

4. **Concrete ECS implementation** — archetype layout, world storage. Tier B work.
   Assign to a Tier B session after C7 setup.

5. **Concrete EventBus implementation** — Tier B work, post-gate.

6. **Concrete Timestep implementation** — reads from `config/core.toml`. Tier B work.

7. **workspace.edition warning** — `Cargo.toml` has `workspace.edition` which is unused.
   C2 owns `Cargo.toml`; file as a low-severity note for C2.

---

## knowledge/ files modified

| File | Version after | Change |
|------|--------------|--------|
| `knowledge/file_structure.md` | 2 | Added all C1-created files |
| `knowledge/project_manifest.md` | 2 | [C1_INTERFACES_PUBLISHED] written, C1 RETIRED |

---

## What Unblocks Now

- **C3 (Rendering)** — may begin. Depends on `core::ecs`, `core::units`.
- **C4 (Physics Core)** — may begin. Depends on `core::units` exclusively.
- **C6 (Debugger)** — may begin. Depends on C1 + C2 both in-progress; C2 parallel.
