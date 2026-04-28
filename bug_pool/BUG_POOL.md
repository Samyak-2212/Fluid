# BUG_POOL.md

Central bug tracking for the Fluid framework project.
All agents must check this file before starting work.
Closed entries are never deleted — they stay in `## Closed` permanently.

## Entry Schema

```
### BUG-<id>
- Severity: <critical | high | medium | low | review | process>
- Component: <crate/module>
- Reported by: <agent_id>
- Description: <one precise sentence>
- Reproduction: <minimal steps or N/A>
- Assigned to: <agent_id or UNASSIGNED>
- Status: <OPEN | IN_PROGRESS | PENDING_REVIEW | CLOSED>
- Resolution: <fill on close, leave blank otherwise>
```

---

## Critical

### BUG-006
- Severity: critical
- Component: workspace root
- Reported by: qa-agent-doc-pipeline
- Description: Unexpected top-level directory '.cursor' found. This violates the master coordinator folder structure spec. Likely created by an agent that misread coordinator names as source crate names.
- Reproduction: ls at project root.
- Assigned to: UNASSIGNED
- Status: CLOSED
- Resolution: False positive. `.cursor/` is already documented in `knowledge/file_structure.md` as a root-owned directory containing Cursor project rules; the QA prompt allowlist is outdated.

### BUG-001
- Severity: critical
- Component: core/ecs
- Reported by: tier_b_agent
- Description: trait `World` cannot be made into an object (`dyn World`) because methods `insert`, `get`, `get_mut`, `remove` have generic type parameters `C: Component`.
- Reproduction: Run `cargo build` from workspace root.
- Assigned to: UNASSIGNED
- Status: OPEN
- Resolution: 

## High

### BUG-002
- Severity: high
- Component: builder/src/main.rs
- Reported by: tier_b_agent
- Description: `builder/src/main.rs` compilation fails due to `eframe::App` in `eframe 0.34.1` changing trait signature to require `ui` method instead of `update`.
- Reproduction: Run `cargo build -p builder`.
- Assigned to: claude
- Status: CLOSED
- Resolution: Rewrote `impl eframe::App` to use required `fn ui(&mut self, ui: &mut Ui, frame: &mut Frame)` and provided `fn logic(&mut self, ctx, frame)` for subprocess polling. Updated all deprecated panel APIs (TopBottomPanel→Panel::top/bottom, SidePanel→Panel::left/right, .show→.show_inside, .min_width→.min_size). Verified 0 errors, 0 warnings.

## Medium

### BUG-008
- Severity: medium
- Component: rendering/Cargo.toml
- Reported by: c2_build_system_20260429T173700Z
- Description: `rendering/Cargo.toml` uses `wgpu` feature `"gl"` which was renamed to `"gles"` in wgpu 29, causing workspace resolution failure and blocking all `-p crate` builds.
- Reproduction: `cargo build -p builder` — fails with feature selection error.
- Assigned to: C3
- Status: CLOSED
- Resolution: C2 fixed the typo (`gl` → `gles`) in `rendering/Cargo.toml` as an emergency workspace unblock. C3 should verify the gles backend is the intended target and confirm against their PROMPT.md.

## Low

### BUG-003
- Severity: low
- Component: builder/src/main.rs
- Reported by: tier_b_agent
- Description: Component dependency metadata is hardcoded in `main.rs::default_components()`. Should read `[package.metadata.fluid]` dynamically from Cargo.toml.
- Reproduction: N/A (Deferred post-gate work)
- Assigned to: UNASSIGNED
- Status: OPEN
- Resolution: 

### BUG-004
- Severity: low
- Component: builder/src/ui
- Reported by: tier_b_agent
- Description: Per-component elapsed build time is tracked in `state.rs::ComponentStatus` but not displayed in the UI.
- Reproduction: N/A (Deferred post-gate work)
- Assigned to: UNASSIGNED
- Status: OPEN
- Resolution:

### BUG-005
- Severity: low
- Component: Cargo.toml (workspace)
- Reported by: c1_continuation_20260428T080000Z
- Description: `Cargo.toml` emits `warning: unused manifest key: workspace.edition`; the edition key belongs under `[workspace.package]`, not `[workspace]` directly.
- Reproduction: `cargo build` from workspace root — warning appears in stderr.
- Assigned to: C2
- Status: CLOSED
- Resolution: Removed `edition = "2021"` from the `[workspace]` table. The key was duplicated — `[workspace.package]` already had it correctly.


## Pending Claude Review

<!-- No entries -->

## Prompt/Knowledge Changes

### BUG-007
- Severity: process
- Component: QA agent prompt root allowlist
- Reported by: qa-agent-doc-pipeline
- Description: The QA prompt's permitted top-level directory allowlist omits `.cursor/`, but `knowledge/file_structure.md` already documents `.cursor/` as a valid root directory.
- Reproduction: Compare the QA prompt root anomaly allowlist with `knowledge/file_structure.md`.
- Assigned to: UNASSIGNED
- Status: OPEN
- Resolution:

## Process Violations

<!-- No entries -->

## Closed

### BUG-006
- Severity: critical
- Component: workspace root
- Reported by: qa-agent-doc-pipeline
- Description: Unexpected top-level directory '.cursor' found. This violates the master coordinator folder structure spec. Likely created by an agent that misread coordinator names as source crate names.
- Reproduction: ls at project root.
- Assigned to: UNASSIGNED
- Status: CLOSED
- Resolution: False positive. `.cursor/` is already documented in `knowledge/file_structure.md` as a root-owned directory containing Cursor project rules; the QA prompt allowlist is outdated.
