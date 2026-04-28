# Tier B Handoff — C2 Build System

## Who you are
You are a Tier B agent continuing C2 (Build System) work on the Fluid framework.
You are picking up from a Claude (Tier A) session that exhausted its weekly quota.

Read AGENTS.md before anything else. You are a Tier B model — the restrictions apply.

## What Claude already completed (do not touch)

The following files are complete Tier A output. Do NOT modify them.

- `builder/src/main.rs` — full egui app, command builder, subprocess lifecycle. FROZEN.
- `builder/src/config.rs` — FlagEntry, BuilderConfig, FlagState, TOML loader. FROZEN.
- `builder/src/state.rs` — ComponentStatus, BuildSessionState, BuildState. FROZEN.
- `builder/src/subprocess.rs` — BuildProcess with spawn/poll/kill/is_running. FROZEN.
- `builder/src/ui/mod.rs` — module declarations. FROZEN.
- `builder/src/ui/component_list.rs` — ComponentEntry, render_component_list. FROZEN.
- `builder/src/ui/flag_panel.rs` — render_flag_panel with bool/select/string widgets. FROZEN.
- `builder/src/ui/output_panel.rs` — render_output_panel with auto-scroll. FROZEN.
- `builder/Cargo.toml` — all crate versions verified on crates.io 2026-04-27. FROZEN.
- `config/builder_flags.toml` — all initial flags populated. FROZEN.

## Dependency versions already verified (do not re-check unless a build error occurs)

From `builder/Cargo.toml`:
- egui 0.34.1
- eframe 0.34.1 (wgpu backend, default_fonts)
- toml 1.1.2
- serde 1 (derive)
- crossbeam-channel 0.5.15

## Your tasks (in priority order)

### Task 1 — Verify the builder compiles [HIGHEST PRIORITY]

Run from the workspace root:
```
cargo build -p builder
```

Expected: compiles without errors. Warnings are acceptable.

If it fails:
- Read the error carefully.
- Fix only the specific compilation error. Do not refactor.
- If the fix requires changing a FROZEN file, do NOT change it.
  Instead, file the issue in `bug_pool/BUG_POOL.md` with severity `high` and
  write a description of the error. Then stop — this requires Claude review.
- Tag any file you change to fix a build error with `[NEEDS_REVIEW: claude]`.

### Task 2 — Verify the builder runs

After a clean build, run:
```
cargo run -p builder --bin fluid_builder
```

Verify:
- The egui window opens with title "Fluid Builder".
- The flags panel loads entries from `config/builder_flags.toml` (FLUID_TIER, release, component features).
- The components panel shows the five simulators as checkboxes.
- The Build button is present and clicking it invokes cargo (output should stream in).
- The Cancel button kills the subprocess.

If the window opens but config does not load (shows "Config error" in title bar):
- Check that `config/builder_flags.toml` exists and is valid TOML.
- The locate_config() function in main.rs walks up from cwd — run from the workspace root.

Document the result in your pack file.

### Task 3 — Pull the C2 complete gate

Only after Tasks 1 and 2 confirm the builder compiles and runs:

1. Read `knowledge/project_manifest.md`, note the current `<!-- version: N -->`.
2. Append `[C2_COMPLETE]` under `## Completion Gate Signals`.
3. Update the C2 row in Coordinator Status: Status → `COMPLETE`, Gate Signal → `[C2_COMPLETE] ✅`.
4. Increment the version counter.
5. Write a final pack file to `pack/tierb_c2_<timestamp>/context.md`.
6. Write `pack/tierb_c2_<timestamp>/handoff_for_claude.md` — see format below.
7. Terminate the session. Do not continue after writing `[C2_COMPLETE]`.

### Task 4 — Update knowledge/file_structure.md

If you touched or verified more than 3 files (you will), update `knowledge/file_structure.md`
to reflect the current builder/ and config/ file layout. Increment its version counter.

This is an operational write — Tier B is permitted to update file_structure.md.

## handoff_for_claude.md format

```
# C2 Handoff — Claude Review Optional

## Gate status
[C2_COMPLETE]: written by Tier B on <date>

## Build verification result
- cargo build -p builder: PASS / FAIL (describe if fail)
- cargo run -p builder: PASS / FAIL (describe if fail)

## Files Tier B touched
<list any files modified, if any>

## Files requiring Claude review
<list only if you modified a FROZEN file or tagged [NEEDS_REVIEW: claude]>
None if the build passed without modifications.

## Notes for Claude
- builder/Cargo.toml crate versions were verified by Claude (Tier A) on 2026-04-27.
- config/builder_flags.toml contains all initial flags. Other coordinators (C3–C5)
  must add their own flags here as they come online.
- The component dependency metadata (requires = [...] in each Cargo.toml) is not yet
  read dynamically by the builder — it uses a hardcoded list in main.rs default_components().
  This is a known limitation. File as severity: low in BUG_POOL if not already filed.
```

## What C2 does NOT yet do (known gaps — file in BUG_POOL if not already there)

1. Component dependency metadata is hardcoded in `main.rs::default_components()`.
   The PROMPT.md specifies reading `[package.metadata.fluid]` from each Cargo.toml dynamically.
   This is a known gap. File as `severity: low` bug if not already in BUG_POOL.

2. Per-component elapsed build time tracking is defined in `state.rs::ComponentStatus`
   but is not wired to the UI — the UI shows session-level elapsed time only.
   File as `severity: low` bug if not already in BUG_POOL.

Do NOT implement these gaps. File them and move on. They are post-gate work.

## Tier B restrictions (from AGENTS.md and model_tier_policy.md)

- Do NOT modify: `knowledge/`, `coordinators/*/PROMPT.md`, `ROOT_COORDINATOR.md`
  Exception: you ARE permitted to write gate signals to `knowledge/project_manifest.md`
  and update `knowledge/file_structure.md`.
- Do NOT modify any FROZEN file listed above.
- After 15 tool calls: write a pack file, then continue or hand off.
- Check `bug_pool/BUG_POOL.md` before starting.
- Update `knowledge/file_structure.md` after touching more than 3 files.

## Build commands reference

```
# From workspace root
cargo build                          # debug, all workspace members
cargo build -p builder               # builder only
cargo run -p builder --bin fluid_builder   # run the builder UI
```
