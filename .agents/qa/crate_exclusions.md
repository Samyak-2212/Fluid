<!-- QA-AGENT-COMPLETE: issue-4 -->
# Crate Exclusions — Authoritative List
<!-- version: 1 -->
<!-- Tier A owned. Increment version on every write.                    -->
<!-- Any agent generating documentation or scaffolding files MUST read  -->
<!-- this file before producing any crate-level output.                 -->

## Excluded from documentation and scaffolding

The following directory names exist in the repository but are NOT source
crates requiring README.md or USAGE.md. Do not create documentation files
for any entry on this list under any circumstances.

| Directory | Reason | Source crate location |
|---|---|---|
| `coordinators/physics_core/` | coordinator prompt only — not a source crate | `core/` owns physics primitives; `components/` owns simulation domains |
| `builder/` | developer tooling — not a framework deliverable | N/A |
| `debugger/` | internal diagnostics — not a public API | N/A |
| `coordinators/` (all subdirs) | agent instructions — never shipped | N/A |
| `physics_core/` (root level) | temporary workspace stub created by C1 — owned by C4, not a documentation target; will be resolved on [C4_INTERFACES_PUBLISHED] | N/A |

## If a path not on this list appears

If you encounter a directory that is not in `knowledge/file_structure.md`
and not on this exclusion list, do not document it. Write a fact to
`knowledge_b/<agent_id>_<timestamp>_unknown_directory.md` and continue.
Do not act on undocumented directories.
