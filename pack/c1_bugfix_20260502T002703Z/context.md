# Pack: c1_bugfix_20260502T002703Z

Session ID: c1_bugfix_20260502T002703Z
Timestamp: 2026-05-02T00:27:03+05:30
Trigger: BUG-001 reactivation (critical)
Domain: core/src/ecs/

## [COMPLETED]

BUG-001 — `dyn World` object-safety — CLOSED.

### Fix applied

Split the single `World` trait into two cooperating layers:

1. `WorldAny` (new, `core/src/ecs/traits.rs`)
   - Object-safe: no generic methods.
   - Erased API: `spawn`, `despawn`, `insert_erased(TypeId, Box<dyn Any+Send+Sync>)`,
     `get_erased(TypeId) -> Option<&dyn Any+Send+Sync>`, `get_erased_mut`, `remove_erased`.
   - Bounds: `Send + Sync`.
   - `dyn WorldAny` and `Box<dyn WorldAny>` compile without error.

2. `World` (updated, `core/src/ecs/traits.rs`)
   - Supertrait of `WorldAny`: `pub trait World: WorldAny`.
   - Adds generic typed convenience methods as **default implementations**:
     `insert<C: Component>`, `get<C>`, `get_mut<C>`, `remove<C>`.
   - Blanket impl: `impl<T: WorldAny> World for T {}` — implementors write WorldAny only.
   - Not object-safe (by design — generic methods stay generic).

3. `ArchetypeWorld` (updated, `core/src/ecs/world.rs`)
   - Now implements `WorldAny` only.
   - `World` methods available automatically via blanket impl.
   - Internal storage unchanged: `HashMap<TypeId, Box<dyn Any + Send + Sync>>`.
   - `insert_erased` maps directly to the existing `map.insert(type_id, component)`.

4. `ecs/mod.rs` — re-exports `WorldAny` alongside `World`.

### Files touched (3)

- `core/src/ecs/traits.rs` — WorldAny added; World updated to supertrait + blanket; 1 new test
- `core/src/ecs/world.rs` — WorldAny impl replaces World impl; 1 new dyn boxing test
- `core/src/ecs/mod.rs` — WorldAny re-exported

### Verification

- cargo test -p core: **28 passed, 0 failed** (26 original + 2 new gate tests)
- cargo check --workspace: **0 errors**, 1 pre-existing debugger warning (not C1 domain)

## [BLOCKED_ON]

Nothing.

## [NEXT_STEPS]

No further C1 work required. Open bugs not in C1 domain:
- BUG-003 (low): builder metadata — C2 domain
- BUG-004 (low): builder UI elapsed time — C2 domain
- BUG-007 (process): QA allowlist — unassigned
- BUG-012 (medium): surface.rs formats[0] panic — C3 domain

## [OPEN_QUESTIONS]

None.
