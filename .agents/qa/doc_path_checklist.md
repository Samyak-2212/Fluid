<!-- QA-AGENT-COMPLETE: issue-1 -->
# Documentation Agent — Pre-flight Path Checklist
<!-- This file is referenced by the generate-readme skill. -->
<!-- Do not edit without Tier A review.                     -->

## Purpose

Before writing any file, the documentation agent must confirm that the
scaffold stubs exist at the expected paths. This checklist defines that
verification procedure.

## Procedure

For each crate in `knowledge/file_structure.md`:

1. Resolve the expected README path: `<crate_path>/README.md`
2. Resolve the expected USAGE path: `<crate_path>/USAGE.md`
3. Check whether each file exists on disk.
4. Check whether the first line of the file contains `[STUB]`.
   If the file exists but does NOT contain `[STUB]` on line 1, it was
   either manually edited or never scaffolded — do not overwrite it.
   File a bug under `## High` in `bug_pool/BUG_POOL.md` and skip that crate.
5. If a file does not exist at all, it was either never scaffolded or
   scaffolded at a wrong path. File a bug under `## Medium` in
   `bug_pool/BUG_POOL.md` and skip that crate.
6. Record all PASS / SKIP results in `knowledge_b/<agent_id>_<timestamp>_preflight.md`
   before proceeding to draft generation.

## Pass condition

All expected stub files exist and contain `[STUB]` on line 1.
Only proceed to draft generation after recording the preflight result.

## On partial pass

Document every skipped crate in the preflight record.
Complete documentation for all passing crates.
Do not block the entire run on a single missing stub.
