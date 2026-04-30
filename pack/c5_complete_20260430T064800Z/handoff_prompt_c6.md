# Handoff Prompt — C6 Debugger & Diagnostics Coordinator

Role: C6 — Debugger & Diagnostics Coordinator
Domain: debugger/
Model: Gemini 3.1 Pro
Task: Implement the embedded HTTP debugger, structured log system, event bridge,
      log archiver, bug pool reporter, and browser UI.

## Mandatory Reading (in this exact order before any action)

1. AGENTS.md
2. coordinators/debugger/PROMPT.md   — your full specification
3. knowledge/dependency_graph.md     — confirm C1+C2 gates cleared
4. knowledge/capability_tiers.md    — tier constraints
5. knowledge/config_schema.md       — config conventions
6. bug_pool/BUG_POOL.md             — open bugs; note BUG-001 (dyn World — read-only)

## Current State

All upstream coordinators are complete:
- C1 COMPLETE — core/src/event_bus.rs exists; EventBus trait ready to consume
- C2 COMPLETE — builder/ builds cleanly; build mode context available
- C3 COMPLETE — rendering HTTP preview on port 8080; use 8081 for debugger
- C4 COMPLETE — physics_core/ fully implemented
- C5 COMPLETE — all simulation components implemented (commit: d710739d)

The debugger/ crate stub currently exists with an empty lib.rs and a minimal
Cargo.toml. config/debugger.toml does NOT yet exist — create it.

## Files You Must Create

Per coordinators/debugger/PROMPT.md §Responsibilities:

| File | Purpose |
|------|---------|
| `debugger/src/lib.rs` | Public debugger API |
| `debugger/src/http_server.rs` | Embedded HTTP server, port 8081 |
| `debugger/src/log_system.rs` | Structured log writer (timestamp + sequence) |
| `debugger/src/log_archiver.rs` | Moves active logs to archive on bug close |
| `debugger/src/event_bridge.rs` | Subscribes to C1 EventBus, routes to log system |
| `debugger/src/bug_pool_reporter.rs` | Appends entries to bug_pool/BUG_POOL.md atomically |
| `debugger/src/events.rs` | PhysicsStepEvent, RenderFrameEvent, ComponentLoadEvent |
| `config/debugger.toml` | All tunables: port, log level, max_size_mb, session_id_format |
| `debugger/build.rs` | FLUID_TIER emission (match pattern from other crates' build.rs) |

## Log Entry Format

Every log line must be exactly:
```
[YYYY-MM-DDTHH:MM:SS.mmmZ] [SEQ:<N>] [LEVEL:<level>] [MODULE:<path>] <message>
```
SEQ is monotonically increasing u64 per session, starting at 0.
LEVEL values: TRACE, DEBUG, INFO, WARN, ERROR, FATAL.
MODULE: Rust module path, e.g. `physics_core::integrators::velocity_verlet`.

Log files: debugger/logs/active/<session_id>.log (gitignored).
Archive: debugger/logs/archive/<bug_id>/<session_id>.log (committed, permanent).

## HTTP Server Requirements

Port: 8081. Config key: debugger_http_port in config/debugger.toml.
Browser UI is HTML + vanilla JS only. No external CDN. All assets embedded via
include_str!(). Endpoints:

| Endpoint | Method | Description |
|----------|--------|-------------|
| /logs | GET | SSE or polling — real-time log stream |
| /stats | GET | JSON: physics_dt, render_fps, entity_count |
| /bugs | GET | Read-only HTML render of bug_pool/BUG_POOL.md |
| /report_bug | POST | Appends new entry to BUG_POOL.md via bug_pool_reporter |

## Dependency Versions (verify each on docs.rs before committing)

```toml
tiny_http = "0.12"      # [UNVERIFIED]
serde = { version = "1", features = ["derive"] }
serde_json = "1"        # [UNVERIFIED]
uuid = { version = "1", features = ["v4"] }   # [UNVERIFIED]
chrono = "0.4"          # [UNVERIFIED]
```
Remove [UNVERIFIED] tags only after checking docs.rs for version existence and API.

## Completion Gate

C6 is complete when ALL of the following are true:
1. HTTP server starts and serves log viewer at localhost:8081
2. Log entries written with timestamp and sequence number
3. Event bridge subscribes to C1 EventBus without panic
4. Log archiver moves active → archive correctly
5. Bug pool reporter appends atomically without corrupting BUG_POOL.md
6. Browser UI loads without external network requests
7. config/debugger.toml has all four tunables present
8. [C6_COMPLETE] written to knowledge/project_manifest.md

Writing [C6_COMPLETE] is a HARD RETIREMENT TRIGGER. Per AGENTS.md:
read .agents/qa/tier_a_commit_protocol.md first and execute the commit
procedure before writing the pack file. Terminate immediately after.

## Protocol Rules

- After 15 tool calls: write a pack file, then continue or hand off.
- Update knowledge/file_structure.md after touching more than 3 files.
- Tag any physics, rendering, unsafe, or CUDA/ROCm output [NEEDS_REVIEW: claude].
- Check bug_pool/BUG_POOL.md before starting — your bug may already exist.
- All config tunables go in config/debugger.toml, never hardcoded.
