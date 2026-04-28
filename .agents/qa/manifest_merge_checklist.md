<!-- QA-AGENT-COMPLETE: issue-2 -->
# Manifest Merge Checklist — Documentation Status Section
<!-- Read this before merging knowledge_b/PROPOSED_doc_status_manifest_section.md -->
<!-- into knowledge/project_manifest.md.                                           -->
<!-- Do not edit without Tier A review.                                            -->

## Pre-merge verification

Before merging, verify every row in the proposed table against
`knowledge/file_structure.md`:

1. Every crate name in the table exists in `knowledge/file_structure.md`.
   If a row names a crate not in `file_structure.md`, delete the row and
   file a `## Low` bug in `bug_pool/BUG_POOL.md`.

2. Every crate in `knowledge/file_structure.md` that requires documentation
   (i.e., is not `builder/`, `debugger/`, or `coordinators/`) has a
   corresponding row in the table.
   If a crate is missing from the table, add a row with status STUB before merging.

3. The `<scaffold timestamp>` placeholders must be replaced with the actual
   ISO 8601 timestamp from the Tier B scaffold report in
   `knowledge_b/<agent_id>_<timestamp>_scaffold_report.md`.
   If the scaffold report does not exist, use `[TIMESTAMP UNVERIFIED]` and
   file a `## Low` bug.

4. Read the current version number in `knowledge/project_manifest.md`.
   Increment it by 1 when writing the merged file.

## Merge procedure

1. Open `knowledge/project_manifest.md`.
2. Locate the end of the file (or the correct alphabetical position for
   `## Documentation Status`).
3. Append the verified table. Do not copy the HTML comment lines from the
   proposed file — those are instructions to Tier A, not content.
4. Increment the `<!-- version: N -->` header.
5. Commit with this exact message schema:
   ```
   [TIER_A_REVIEW] qa-agent(knowledge): merge doc status table into project_manifest

   gate: [DOC_STATUS_MERGED]
   agent: <agent_id>
   timestamp: <ISO 8601>
   ```
6. Run `git rev-parse HEAD` and write the SHA to `knowledge/project_manifest.md`
   as `Last clean checkpoint SHA: <sha>`. Increment the version header again.
