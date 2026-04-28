QA PIPELINE REPORT
==================
Agent:     qa_agent_doc_pipeline_20260428T214714Z
Timestamp: 2026-04-28T16:17:18Z
Status:    PARTIAL — see deferrals

FILES
-----
CREATED:   .agents/qa/
CREATED:   .agents/qa/doc_path_checklist.md
CREATED:   .agents/qa/manifest_merge_checklist.md
CREATED:   .agents/qa/draft_persistence_protocol.md
CREATED:   .agents/qa/crate_exclusions.md
CREATED:   .agents/qa/model_routing_table.md
CREATED:   .agents/qa/tier_a_commit_protocol.md
CREATED:   knowledge_b/qa_agent_doc_pipeline_20260428T214714Z_issue1_resolution.md
CREATED:   knowledge_b/qa_agent_doc_pipeline_20260428T214714Z_issue2_resolution.md
CREATED:   knowledge_b/qa_agent_doc_pipeline_20260428T214714Z_issue3_resolution.md
CREATED:   knowledge_b/qa_agent_doc_pipeline_20260428T214714Z_issue4_resolution.md
CREATED:   knowledge_b/qa_agent_doc_pipeline_20260428T214714Z_issue5_resolution.md
CREATED:   knowledge_b/qa_agent_doc_pipeline_20260428T214714Z_issue7_resolution.md
CREATED:   knowledge_b/qa_agent_doc_pipeline_20260428T214714Z_issue8_resolution.md
CREATED:   knowledge_b/qa_agent_doc_pipeline_20260428T214714Z_qa_pipeline_report.md
DEFERRED:  6 — workspace has no clean baseline yet; cargo check --workspace did not exit cleanly

BUGS FILED
----------
BUG-006: Unexpected top-level directory '.cursor' found at workspace root

FAILURES
--------
None

DEFERRALS
---------
DEFERRED: Issue 6 — workspace has no clean baseline yet.
          Re-run this agent after [C1_INTERFACES_PUBLISHED] is in
          knowledge/project_manifest.md and cargo check passes.
