<!-- version: 1 -->
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
