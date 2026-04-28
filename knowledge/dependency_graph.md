<!-- version: 1 -->
# Dependency Graph

## Coordinator Dependencies

```
C2 (Build System) ──────────────────────────────► unblocks all coordinators for testable output
C1 (Core Systems) ──────────────────────────────► C3, C4, C6
C4 (Physics Core) [interfaces published] ───────► C5 begins
C4 (Physics Core) [fully implemented] ──────────► C5 full implementation
C3 + C4 ────────────────────────────────────────► C7 review begins
C1 + C2 ────────────────────────────────────────► C6 begins

Parallel from day 0:    C1 and C2
First unblock event:    C1 publishes core trait interfaces → C4 begins
                        Signal: [C1_INTERFACES_PUBLISHED] in knowledge/project_manifest.md
Second unblock event:   C4 publishes physics traits → C5 begins
                        Signal: [C4_INTERFACES_PUBLISHED] in knowledge/project_manifest.md
```

## Completion Gate Signals

All signals are written to `knowledge/project_manifest.md`.
Writing a signal is a hard retirement trigger — see AGENTS.md and sustainability rule 11.

| Signal | Written by | Meaning |
|--------|-----------|---------|
| `[C1_INTERFACES_PUBLISHED]` | C1 | Core traits exist — C4 may begin |
| `[C4_INTERFACES_PUBLISHED]` | C4 | Physics traits exist — C5 may begin |
| `[C1_COMPLETE]` | C1 | All C1 work done — session retires |
| `[C2_COMPLETE]` | C2 | All C2 work done — session retires |
| `[C3_COMPLETE]` | C3 | All C3 work done — session retires |
| `[C4_COMPLETE]` | C4 | All C4 work done — session retires |
| `[C5_COMPLETE]` | C5 | All C5 work done — session retires |
| `[C6_COMPLETE]` | C6 | All C6 work done — session retires |
| `[C7_COMPLETE]` | C7 | All C7 work done — session retires |
| `[ROOT_COMPLETE]` | Root | Root coordinator work done — session retires |

## C1 Completion Gate (Interfaces Published)

C1 is considered "interfaces published" when ALL of the following files exist and are non-empty:

- `core/src/units.rs` — SI newtype wrappers
- `core/src/ecs/traits.rs` — ECS component and system traits
- `core/src/event_bus.rs` — event bus trait
- An entry `[C1_INTERFACES_PUBLISHED]` in `knowledge/project_manifest.md`

## C4 Interface Publication Gate

C4 is considered "interfaces published" when ALL of the following files exist and are non-empty:

- `physics_core/src/integrators/traits.rs`
- `physics_core/src/collision/traits.rs`
- `physics_core/src/constraints/traits.rs`
- An entry `[C4_INTERFACES_PUBLISHED]` in `knowledge/project_manifest.md`

## Build Order Summary

```
Wave 0 (parallel):   C1, C2
Wave 1 (after C1):   C3, C4, C6
Wave 2 (after C4):   C5
Wave 3 (after C3+C4+C5): C7 full review
```

C7 setup begins with C1+C2. C7 review work begins only after C3/C4/C5 interfaces are published.
