---
name: generate-readme
description: >
  Generates README.md and USAGE.md documentation for all Fluid framework
  crates. Activate only by typing "generate-readme" as your first message
  in a new session. Do not activate during implementation, debugging, or
  any coordinator task.
---

# generate-readme Skill

## Activation

This skill is activated **only** by opening a fresh session and typing `generate-readme`
as the very first message. Do not invoke this skill mid-session, during coordinator
tasks, or during implementation work.

## Purpose

Produce human-readable `README.md` and `USAGE.md` for every Fluid framework crate.
These files are the primary onboarding surface for human contributors and must be
accurate, concise, and consistent across crates.

## Target Files

For each crate listed below, this skill produces exactly two files:

| Crate path | README.md | USAGE.md |
|---|---|---|
| `.` (workspace root) | `README.md` | `USAGE.md` |
| `core/` | `core/README.md` | `core/USAGE.md` |
| `components/fluid_simulator/` | `components/fluid_simulator/README.md` | `components/fluid_simulator/USAGE.md` |
| `components/aerodynamic_simulator/` | `components/aerodynamic_simulator/README.md` | `components/aerodynamic_simulator/USAGE.md` |
| `components/motion_force_simulator/` | `components/motion_force_simulator/README.md` | `components/motion_force_simulator/USAGE.md` |
| `components/thermodynamic_simulator/` | `components/thermodynamic_simulator/README.md` | `components/thermodynamic_simulator/USAGE.md` |
| `components/fem_structural/` | `components/fem_structural/README.md` | `components/fem_structural/USAGE.md` |
| `rendering/` | `rendering/README.md` | `rendering/USAGE.md` |

Do **not** produce documentation for `builder/`, `debugger/`, or `coordinators/`.

## Pre-Execution Checklist

Before writing any file, perform these checks in order:

1. Read `knowledge/project_manifest.md` — confirm all relevant coordinator gate signals
   are present. If a crate's coordinator has not yet published `[CX_COMPLETE]` or
   `[CX_INTERFACES_PUBLISHED]`, the crate's source is incomplete. Document what is
   available but mark affected sections `[UNVERIFIED — coordinator gate not yet published]`.

2. Read `knowledge/capability_tiers.md` — required to fill the **Capability Tier**
   section of each `README.md` accurately.

3. Read `knowledge/physics_contract.md` — required for the **Numerical Details** section
   of `README.md` and `USAGE.md` for `core/`, all `components/`, and `rendering/`.

4. Read `knowledge/config_schema.md` — required for the **Configuration** section of
   each `USAGE.md`.

5. Read the crate's `Cargo.toml` — extract: crate name, version, description, feature
   flags, and dependencies.

6. Read the crate's `src/lib.rs` (or `src/main.rs`) — extract: public API surface,
   re-exports, and top-level doc comments.

7. Read `knowledge_b/PROPOSED_doc_status_manifest_section.md` if it exists — use its
   table to understand current documentation status and avoid re-writing WRITTEN files
   unless instructed.

8. Check `bug_pool/BUG_POOL.md` for any open bugs tagged to the crate being documented.
   Note them in the **Known Limitations** section of `README.md`.

## README.md Content Specification

Write each `README.md` with this structure, in this order. Do not add or remove
top-level headings.

```
# <Crate Name>

<One-sentence tagline. Must be accurate. No marketing fluff.>

## What It Does

<2–4 paragraphs. Describe what the crate does, what problem it solves, and how
it fits into the Fluid framework. Reference coordinator domains where relevant.
Mention dependencies on other crates by name.>

## Capability Tier

<State the tier(s) from knowledge/capability_tiers.md that this crate targets.
List which features require which tier. Use a table if more than two tiers apply.>

## Quick Start

<Minimal working example: Cargo.toml dependency line, a code snippet that
compiles, and the command to run it. Use fenced code blocks with language tags.
If the crate has no binary entry point, show library usage instead.>

## Build Instructions

<How to build this crate specifically. Reference the AGENTS.md build commands.
List any feature flags that affect this crate. State which tier requires which
flags.>

## Known Limitations

<Bullet list of known limitations. Pull from bug_pool/BUG_POOL.md for open bugs.
If there are none, write "None known at this time." Do not omit this section.>
```

## USAGE.md Content Specification

Write each `USAGE.md` with this structure, in this order. Do not add or remove
top-level headings. Omit **Numerical Details** only if the crate is not a physics
or rendering crate — state the omission explicitly with a comment.

```
# <Crate Name> — Usage Reference

## Architecture Overview

<Describe internal module structure. Name the key modules/files and their roles.
Include a simple ASCII diagram if the module graph has more than 4 nodes.>

## Public API

<Document every pub function, struct, trait, and enum that a downstream crate
would use. Use Rust doc-comment style. Include type signatures.>

## Configuration

<Describe the corresponding config/<crate>.toml file from knowledge/config_schema.md.
List every key, its type, its default value, and its effect. If no config file
exists yet, state that and mark [UNVERIFIED].>

## Integration with Other Crates

<Describe how this crate is consumed by other Fluid crates. Show example integration
code. Reference the dependency_graph from knowledge/dependency_graph.md.>

## Numerical Details

<For physics and rendering crates only: document precision guarantees, integration
schemes, solver tolerances, and any accuracy contracts from knowledge/physics_contract.md.
For non-physics crates, write: <!-- This section intentionally omitted: not a physics or rendering crate. -->

## Examples

<At least two worked examples. Prefer examples that exercise the most common
use case and one edge case. Use fenced Rust code blocks.>

## Troubleshooting

<Common failure modes and their fixes. Include compiler errors if the API has
known sharp edges. If none are known, write "No known issues."
```

## Writing Rules

- Do not copy stub placeholder text (`[PENDING]`, `[STUB]`, `[TITLE PENDING]`, etc.)
  into output files. Replace every placeholder with real content.
- Do not invent API surface. If a function is not visible in source, do not document it.
  Mark uncertain items `[UNVERIFIED]`.
- Keep sentences declarative. No marketing language ("best", "powerful", "seamlessly").
- All code examples must use feature flags and dependency lines consistent with the
  actual `Cargo.toml` at the time of writing.
- After completing all files for a crate, update `knowledge_b/PROPOSED_doc_status_manifest_section.md`
  to change that crate's status from `STUB` to `DRAFT`.
- Do **not** write to `knowledge/project_manifest.md` directly — propose changes via
  `knowledge_b/`.

## Output Order

Process crates in this order. Within each crate, write `README.md` before `USAGE.md`.

1. Workspace root
2. `core/`
3. `components/fluid_simulator/`
4. `components/aerodynamic_simulator/`
5. `components/motion_force_simulator/`
6. `components/thermodynamic_simulator/`
7. `components/fem_structural/`
8. `rendering/`

## Completion

After all 16 files are written:

1. Update `knowledge_b/PROPOSED_doc_status_manifest_section.md` — mark all completed
   crates as `DRAFT` with the ISO 8601 timestamp of this session.
2. Output a summary table to the session (not to any file) listing each file written,
   its byte count, and any `[UNVERIFIED]` items that require coordinator review.
3. Do not publish any gate signal. This skill does not retire a coordinator.
