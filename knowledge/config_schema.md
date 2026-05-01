<!-- version: 3 -->
# Config Schema

All tunables live in `config/`. Format: TOML.
Runtime panics on missing config keys are forbidden — use typed defaults.

## builder_flags.toml — Entry Schema

File: `config/builder_flags.toml`
This is the source of truth for all builder flags, environment variables, and build options.
The builder UI reads this file at startup and generates its panels dynamically.
Adding a new flag requires only a new entry in this file — no UI code changes.

### Entry Format

```toml
[[flag]]
name        = "FLUID_TIER"          # env var or flag name
kind        = "env"                 # "env" | "cargo_flag" | "feature"
label       = "Capability Tier"     # display label in UI
description = "Hardware tier 0-3"  # tooltip text
type        = "select"              # "select" | "bool" | "string"
options     = ["0", "1", "2", "3"] # for type = "select" only
default     = "0"                   # default value
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Environment variable name or cargo flag name |
| `kind` | enum | yes | `"env"` sets env var before cargo; `"cargo_flag"` appends `--flags`; `"feature"` appends `--features` |
| `label` | string | yes | Human-readable label shown in builder UI |
| `description` | string | yes | Tooltip text shown in builder UI on hover |
| `type` | enum | yes | `"select"` renders a dropdown; `"bool"` renders a checkbox; `"string"` renders a text field |
| `options` | string[] | if type=select | Ordered list of allowed values for dropdown |
| `default` | string | yes | Default value used if not set by user |

### Kind Semantics

- `"env"`: Builder sets `std::env::set_var(name, value)` before invoking cargo subprocess.
  Example: `FLUID_TIER=2 cargo build ...`
- `"cargo_flag"`: Builder appends `--<name>` to the cargo invocation.
  Example: `cargo build --release`
- `"feature"`: Builder appends `--features <name>` to the cargo invocation.
  Example: `cargo build --features fluid_simulator`

## Per-Component Config Files

Each component may define its own runtime config file under `config/<component_name>.toml`.
Schema for component config files is defined by the owning coordinator (C1–C7).
All component config files follow the same TOML format rule:
- No hardcoded values in source — all tunables in config
- Typed defaults in code — no runtime panics on missing keys
- Schema must be documented in this file (config_schema.md) when first created

## Feature Flag Naming Convention

All feature flags use snake_case matching the component folder name exactly.

Examples:
- `fluid_simulator`
- `aerodynamic_simulator`
- `fem_structural`
- `motion_force_simulator`
- `thermodynamic_simulator`

No abbreviations. No hyphens. Defined in root `Cargo.toml` workspace.
Every component's `Cargo.toml` declares its own feature as `default = []` and is opt-in only.

## Tier Feature Flags

Every crate must declare these features in its `Cargo.toml`:

```toml
[features]
default = []
tier_0 = []
tier_1 = []
tier_2 = []
tier_3 = []
```

`build.rs` reads `FLUID_TIER` and emits the correct `cargo:rustc-cfg=feature="tier_N"`.

## Coordinator Registration

When a coordinator introduces a new flag, it must add it to `config/builder_flags.toml`
as part of their implementation — not as a separate step.
Document the new flag's schema here immediately after it is added.

| Config File | Owner | Status |
|-------------|-------|--------|
| `config/builder_flags.toml` | C2 | Created by C2 at initialization |
| `config/physics_core.toml` | C4 | Created by C4 at interfaces-published gate |
| `config/fluid_simulator.toml` | C5 | Scaffolded by C5 |
| `config/aerodynamic_simulator.toml` | C5 | Scaffolded by C5 |
| `config/thermodynamic_simulator.toml` | C5 | Scaffolded by C5 |
| `config/fem_structural.toml` | C5 | Scaffolded by C5 |
| `config/motion_force_simulator.toml` | C5 | Scaffolded by C5 |

## physics_core.toml — Key Schema

File: `config/physics_core.toml`
All values are loaded at runtime. Typed defaults in code — no runtime panics on missing keys.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `constraint_solver_iterations` | integer | `10` | Maximum sequential impulse solver iterations per frame. Higher = more accurate, more CPU. Typical range: 5–20. |
| `broadphase_cell_size` | float | `1.0` | Spatial hash cell size in metres. Set to ≈ 2× largest object bounding radius. Smaller cells reduce false positives but increase memory. |

## fluid_simulator.toml — Key Schema

File: `config/fluid_simulator.toml`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `sph_particle_cap` | integer | `10000` | Maximum number of particles in SPH mode for Tier 0 |
| `cfd_grid_resolution` | array | `[64, 64, 64]` | Base grid resolution for CFD mode (Tier 1+) |
| `sph_smoothing_multiplier` | float | `1.2` | Smoothing length multiplier for SPH kernel |

## aerodynamic_simulator.toml — Key Schema

File: `config/aerodynamic_simulator.toml`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `air_density` | float | `1.225` | Default air density in kg/m^3 (sea level standard) |
| `air_viscosity` | float | `1.81e-5` | Dynamic viscosity of air in Pa*s (N*s/m^2) |
| `default_reference_area` | float | `1.0` | Default reference area for aerodynamic bodies in m^2 |

## thermodynamic_simulator.toml — Key Schema

File: `config/thermodynamic_simulator.toml`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `default_heat_capacity` | float | `1000.0` | Default specific heat capacity in J/(kg*K) |
| `default_thermal_conductivity` | float | `0.026` | Default thermal conductivity in W/(m*K) |
| `ambient_temperature` | float | `293.15` | Ambient temperature in Kelvin |

## fem_structural.toml — Key Schema

File: `config/fem_structural.toml`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `default_youngs_modulus` | float | `1e6` | Default Young's modulus in Pascals |
| `default_poissons_ratio` | float | `0.3` | Default Poisson's ratio |
| `max_newton_iterations` | integer | `20` | Maximum Newton-Raphson iterations for nonlinear solver (Tier 2) |
| `solver_tolerance` | float | `1e-6` | Solver error tolerance for convergence |

## motion_force_simulator.toml — Key Schema

File: `config/motion_force_simulator.toml`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `gravity_vector` | array | `[0.0, -9.81, 0.0]` | Global gravity vector (m/s^2) |
| `default_actuator_max_force` | float | `1000.0` | Default max force for generic actuators in Newtons |
| `default_joint_spring_constant` | float | `5000.0` | Default spring constant for joint constraints in N/m |
