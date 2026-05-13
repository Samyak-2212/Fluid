# Graph Report - Fluid  (2026-05-02)

## Corpus Check
- 76 files · ~86,229 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 499 nodes · 734 edges · 39 communities detected
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 60 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Community 14|Community 14]]
- [[_COMMUNITY_Community 15|Community 15]]
- [[_COMMUNITY_Community 16|Community 16]]
- [[_COMMUNITY_Community 17|Community 17]]
- [[_COMMUNITY_Community 19|Community 19]]
- [[_COMMUNITY_Community 20|Community 20]]
- [[_COMMUNITY_Community 21|Community 21]]
- [[_COMMUNITY_Community 22|Community 22]]
- [[_COMMUNITY_Community 23|Community 23]]
- [[_COMMUNITY_Community 24|Community 24]]
- [[_COMMUNITY_Community 25|Community 25]]
- [[_COMMUNITY_Community 26|Community 26]]
- [[_COMMUNITY_Community 27|Community 27]]
- [[_COMMUNITY_Community 28|Community 28]]
- [[_COMMUNITY_Community 29|Community 29]]
- [[_COMMUNITY_Community 30|Community 30]]
- [[_COMMUNITY_Community 31|Community 31]]
- [[_COMMUNITY_Community 38|Community 38]]
- [[_COMMUNITY_Community 39|Community 39]]
- [[_COMMUNITY_Community 40|Community 40]]
- [[_COMMUNITY_Community 44|Community 44]]
- [[_COMMUNITY_Community 45|Community 45]]
- [[_COMMUNITY_Community 47|Community 47]]
- [[_COMMUNITY_Community 50|Community 50]]
- [[_COMMUNITY_Community 53|Community 53]]

## God Nodes (most connected - your core abstractions)
1. `SphSimulation` - 10 edges
2. `ArchetypeWorld` - 10 edges
3. `FluidBuilderApp` - 9 edges
4. `BeamAssembly` - 9 edges
5. `epa_penetration()` - 8 edges
6. `RigidBody` - 8 edges
7. `Simplex` - 7 edges
8. `do_triangle()` - 7 edges
9. `do_tetrahedron()` - 7 edges
10. `gjk_intersect()` - 7 edges

## Surprising Connections (you probably didn't know these)
- `epa_penetration()` --calls--> `gjk_intersect()`  [INFERRED]
  physics_core\src\collision\epa.rs → physics_core\src\collision\gjk.rs

## Communities

### Community 0 - "Community 0"
Cohesion: 0.09
Nodes (27): free_fall_position_exact(), two_body_orbit_energy_conservation(), VelocityVerlet, VerletState, zero_acceleration_no_drift(), clear_pass(), clear_pass_fills_buffer(), GpuContext (+19 more)

### Community 1 - "Community 1"
Cohesion: 0.11
Nodes (17): exponential_decay_fourth_order_accuracy(), ExponentialDecay, harmonic_oscillator_energy_conserved(), OscDerivs, OscState, Rk4, Rk4<S, D>, Rk4DerivativeProvider (+9 more)

### Community 2 - "Community 2"
Cohesion: 0.11
Nodes (15): CargoMetadata, CargoPackage, CargoToml, FluidBuilderApp, FluidMetadata, hardcoded_components(), label_from_name(), load_components() (+7 more)

### Community 3 - "Community 3"
Cohesion: 0.14
Nodes (16): apply_gravity(), body(), electric_motor_torque_direction(), ElectricMotor, force_system_applies_gravity_to_all_dynamic_bodies(), ForceSystem, gravity_applies_correct_force(), HydraulicActuator (+8 more)

### Community 4 - "Community 4"
Cohesion: 0.16
Nodes (16): AaBox, do_line(), do_simplex(), do_tetrahedron(), do_triangle(), gjk_intersect(), intersects(), minkowski_support() (+8 more)

### Community 5 - "Community 5"
Cohesion: 0.14
Nodes (11): Particle, particle_cap_enforced(), sph_1000_steps_no_nan(), SphSimulation, TaitEos, wendland_c2_grad_zero_at_origin(), wendland_c2_kernel_normalization_approximate(), wendland_c2_positive_inside_support() (+3 more)

### Community 6 - "Community 6"
Cohesion: 0.09
Nodes (9): BugEntry, report_bug(), Severity, BugReportPayload, DebuggerHttpServer, Stats, BuildProcess, OutputLine (+1 more)

### Community 7 - "Community 7"
Cohesion: 0.09
Nodes (8): BuilderConfig, DebuggerConfig, FlagEntry, FlagState, wire(), Debugger, Level, LogSystem

### Community 8 - "Community 8"
Cohesion: 0.17
Nodes (10): BeamAssembly, BeamElement, cantilever_beam_1pct_accuracy(), ComputeKernel, FemDebugStats, GpuComputeBackend, KernelArgs, mass_matrix_is_symmetric() (+2 more)

### Community 9 - "Community 9"
Cohesion: 0.2
Nodes (12): archetype_world_as_dyn_world_any(), ArchetypeWorld, despawn_removes_entity(), get_mut_modifies_component(), insert_and_get(), insert_on_despawned_entity_is_silent(), multiple_component_types_on_one_entity(), overwrite_component() (+4 more)

### Community 10 - "Community 10"
Cohesion: 0.23
Nodes (8): apply_force_accumulates(), clear_accumulators_zeroes_force_and_torque(), inv_mass_dynamic(), inv_mass_static_is_zero(), new_body_at_rest(), RigidBody, static_body_ignores_force(), unit_body()

### Community 11 - "Community 11"
Cohesion: 0.23
Nodes (7): epa_penetration(), EpaResult, non_intersecting_returns_none(), Polytope, Sphere, sphere_contact_normal_direction(), sphere_penetration_depth_accurate()

### Community 12 - "Community 12"
Cohesion: 0.28
Nodes (7): handlers_are_type_isolated(), LocalEventBus, multiple_handlers_all_called(), Ping, Pong, publish_with_no_handlers_is_silent(), single_handler_receives_event()

### Community 13 - "Community 13"
Cohesion: 0.18
Nodes (8): Component, entity_id_copy(), entity_id_raw(), EntityId, System, TestComp, World, WorldAny

### Community 14 - "Community 14"
Cohesion: 0.24
Nodes (6): debug_overlay_update_stores_stats(), DebugOverlay, display_string_non_empty(), frame_stats_fps_derived_correctly(), FrameStats, main()

### Community 15 - "Community 15"
Cohesion: 0.22
Nodes (3): BuildSessionState, BuildState, ComponentStatus

### Community 16 - "Community 16"
Cohesion: 0.4
Nodes (5): divergence_zero_for_zero_velocity(), mac_grid_creates_zero_fields(), MacGrid, project(), projection_reduces_divergence()

### Community 17 - "Community 17"
Cohesion: 0.24
Nodes (4): Camera, SceneRenderer, stub_renderer_increments_frame_count(), StubRenderer

### Community 19 - "Community 19"
Cohesion: 0.44
Nodes (4): CpuFramebuffer, fill_solid_color(), jpeg_encode_does_not_panic(), set_pixel_bounds_check()

### Community 20 - "Community 20"
Cohesion: 0.25
Nodes (5): ComputeKernel, CudaBackend, GpuComputeBackend, KernelArgs, RocmBackend

### Community 21 - "Community 21"
Cohesion: 0.29
Nodes (1): Timestep

### Community 22 - "Community 22"
Cohesion: 0.43
Nodes (3): sequential_impulse_satisfies_constraint(), SequentialImpulseSolver, StopXConstraint

### Community 23 - "Community 23"
Cohesion: 0.38
Nodes (4): free_particle_linear_motion(), harmonic_oscillator_energy_bounded(), LeapFrog, LeapFrogState

### Community 24 - "Community 24"
Cohesion: 0.33
Nodes (3): Event, EventBus, TestEvent

### Community 25 - "Community 25"
Cohesion: 0.33
Nodes (5): Broadphase, CollisionDetector, ContactManifold, ConvexShape, ShapeRef

### Community 26 - "Community 26"
Cohesion: 0.47
Nodes (4): harmonic_oscillator_bounded_energy(), NewmarkBeta, NewmarkBetaState, static_displacement_converges()

### Community 27 - "Community 27"
Cohesion: 0.5
Nodes (3): ComponentLoadEvent, PhysicsStepEvent, RenderFrameEvent

### Community 28 - "Community 28"
Cohesion: 0.5
Nodes (1): RenderSurface<'window>

### Community 29 - "Community 29"
Cohesion: 0.5
Nodes (2): RenderSurface, SurfaceError

### Community 30 - "Community 30"
Cohesion: 0.67
Nodes (2): Constraint, ConstraintSolver

### Community 31 - "Community 31"
Cohesion: 0.67
Nodes (2): DerivativeProvider, Integrator

### Community 38 - "Community 38"
Cohesion: 1.0
Nodes (1): AllocatorStub

### Community 39 - "Community 39"
Cohesion: 1.0
Nodes (1): SceneGraphStub

### Community 40 - "Community 40"
Cohesion: 1.0
Nodes (1): ThreadPool

### Community 44 - "Community 44"
Cohesion: 1.0
Nodes (1): EulerIntegrator

### Community 45 - "Community 45"
Cohesion: 1.0
Nodes (1): SoftBody

### Community 47 - "Community 47"
Cohesion: 1.0
Nodes (1): PipelineDescriptor

### Community 50 - "Community 50"
Cohesion: 1.0
Nodes (1): T

### Community 53 - "Community 53"
Cohesion: 1.0
Nodes (1): T

## Knowledge Gaps
- **63 isolated node(s):** `FlagEntry`, `CargoPackage`, `CargoMetadata`, `FluidMetadata`, `CargoToml` (+58 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 21`** (7 nodes): `mod.rs`, `Timestep`, `.accumulated()`, `.add_frame_time()`, `.dt()`, `.new()`, `.tick()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 28`** (4 nodes): `RenderSurface<'window>`, `.format()`, `.new()`, `.resize()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 29`** (4 nodes): `surface.rs`, `RenderSurface`, `SurfaceError`, `.fmt()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 30`** (3 nodes): `Constraint`, `ConstraintSolver`, `traits.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 31`** (3 nodes): `DerivativeProvider`, `Integrator`, `traits.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 38`** (2 nodes): `mod.rs`, `AllocatorStub`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 39`** (2 nodes): `mod.rs`, `SceneGraphStub`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 40`** (2 nodes): `traits.rs`, `ThreadPool`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 44`** (2 nodes): `EulerIntegrator`, `euler.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 45`** (2 nodes): `mod.rs`, `SoftBody`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 47`** (2 nodes): `PipelineDescriptor`, `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 50`** (1 nodes): `T`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 53`** (1 nodes): `T`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `epa_penetration()` connect `Community 11` to `Community 4`?**
  _High betweenness centrality (0.074) - this node is a cross-community bridge._
- **Why does `gjk_intersect()` connect `Community 4` to `Community 11`?**
  _High betweenness centrality (0.065) - this node is a cross-community bridge._
- **What connects `FlagEntry`, `CargoPackage`, `CargoMetadata` to the rest of the system?**
  _63 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.09 - nodes in this community are weakly interconnected._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.11 - nodes in this community are weakly interconnected._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.11 - nodes in this community are weakly interconnected._
- **Should `Community 3` be split into smaller, more focused modules?**
  _Cohesion score 0.14 - nodes in this community are weakly interconnected._