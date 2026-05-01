# C3 — Rendering Coordinator — Handoff Prompt

Role: C3 — Rendering Coordinator
Model: Gemini 3.1 Pro → Claude Sonnet (wgpu review phases)
Agent ID: c3_rendering_20260428T173700Z
Gate published: [C3_COMPLETE]
Timestamp: 2026-04-29T03:05:00+05:30
Commit SHA: d00186b1b1619c22a85f1ed347ca650a055dd019

---

## What Was Delivered

| File | Action | Notes |
|------|--------|-------|
| `rendering/Cargo.toml` | overwritten | All deps version-verified on docs.rs; `gl` → `gles` fix applied |
| `rendering/build.rs` | new | Emits `FLUID_TIER` env var as `tier_N` cargo feature flag |
| `rendering/src/lib.rs` | overwritten | Full module tree; tier-gated feature modules |
| `rendering/src/camera.rs` | new | `Camera` with `core::units::Meters` position; `Mat4` view/proj |
| `rendering/src/device.rs` | new | `GpuContext` — wgpu adapter/device/queue init; `[NEEDS_REVIEW: claude]` |
| `rendering/src/surface.rs` | new | `RenderSurface` swapchain management; `[NEEDS_REVIEW: claude]` |
| `rendering/src/scene_renderer.rs` | new | `SceneRenderer` trait + `StubRenderer` no-op implementation |
| `rendering/src/debug_overlay.rs` | new | `FrameStats` struct; display string; banner stub; font rasterization `[UNRESOLVED]` |
| `rendering/src/http_preview.rs` | new | `tiny_http` `/frame.jpg` server on port 8080 |
| `rendering/src/tier0/mod.rs` | new | `CpuFramebuffer` with JPEG encode via `image` crate |
| `rendering/src/tier0/renderer.rs` | new | `SoftbufferRenderer` with gradient test frame (headless) |
| `rendering/src/pipeline/mod.rs` | new | Pipeline directory scaffold |
| `rendering/src/pipeline/tier0_pipeline.rs` | new | `clear_pass` stub |
| `config/rendering.toml` | new | `preview_http_port`, frame, camera, and overlay tunables |
| `knowledge/project_manifest.md` | updated | version 6; `[C3_COMPLETE]`; RETIRED entry; commit SHA |
| `knowledge/file_structure.md` | updated | version 4; `rendering/` marked complete |
| `bug_pool/BUG_POOL.md` | updated | BUG-008 and BUG-009 filed for C7 review |

---

## Gate Verification

- `cargo test -p rendering`: **12 passed, 0 failed**
- `cargo check --workspace`: **EXIT:0**

---

## Dependency Versions Verified

All versions confirmed against docs.rs during this session. No `[UNVERIFIED]` tags remain.

| Crate | Version |
|-------|---------|
| wgpu | 29.0.1 |
| softbuffer | 0.4.8 |
| winit | 0.30.13 |
| tiny_http | 0.12.0 |
| image | 0.25.10 |

---

## Review Queue Items

| Bug | Component | Outcome |
|-----|-----------|---------|
| BUG-008 | `rendering/src/device.rs` | Filed `[NEEDS_REVIEW: claude]`; **CLOSED** by C7 review 2026-05-01. Adapter selection policy and `Limits::downlevel_defaults()` confirmed correct. |
| BUG-009 | `rendering/src/surface.rs` | Filed `[NEEDS_REVIEW: claude]`; **CLOSED** by C7 review 2026-05-01. sRGB preference, `PresentMode::Fifo`, and alpha fallback confirmed correct. BUG-012 filed as follow-up for `caps.formats[0]` OOB risk. |

---

## Deferred Work (next C3 session)

1. **Debug overlay font rasterization** — `debug_overlay.rs` contains `[UNRESOLVED]`; a font rasterization crate (e.g., `fontdue`) needs selection and integration.
2. **Full render pipeline** — `pipeline/` is scaffold only; render passes and bind group layouts are unimplemented; requires Tier A (Claude Sonnet).
3. **Tier 0 windowed softbuffer integration** — `SoftbufferRenderer` is currently headless only; window handle (`RawWindowHandle`) wiring to a real `winit` window is deferred.
4. **BUG-012 remediation** — `surface.rs` line 37: guard `caps.formats[0]` against empty slice; assigned to C3 (domain owner) or Tier B; open as of handoff.

---

## Open Bugs Filed This Session

| Bug | Severity | Status at handoff |
|-----|----------|-------------------|
| BUG-008 | review | CLOSED by C7 |
| BUG-009 | review | CLOSED by C7 |
| BUG-012 | medium | OPEN — `caps.formats[0]` OOB risk in `surface.rs` |

---

## Notes for Successor Rendering Agent

- **wgpu 29.x API changes**: Several APIs changed between wgpu 0.20 (what the coordinator PROMPT.md specced) and the actual 29.0.1 release. The session used compiler iteration to verify the real API surface. Treat the PROMPT.md version pins as starting points only; always verify on docs.rs.
- **`Backends::all()` policy**: Do not restrict backend selection manually. wgpu selects Vulkan/DX12/Metal/GLES at runtime. `Backends::all()` (or the default) is the correct value. Manual backend gating would violate the coordinator spec.
- **HTTP preview port**: Default is `8080` (`preview_http_port` in `config/rendering.toml`). C6 (Debugger) uses port `8081`. Ports must not be swapped. Both are configurable.
- **`[NEEDS_REVIEW: claude]` obligation**: Any successor session writing new wgpu pipeline code (render passes, bind groups, pipeline descriptors) must tag the output `[NEEDS_REVIEW: claude]` and file a review-severity bug in `BUG_POOL.md` per `knowledge/model_tier_policy.md`.
- **Model switch point**: Initial scaffolding and HTTP/softbuffer work is Tier B (Gemini 3.1 Pro). Pipeline layout and render pass structure require Tier A (Claude Sonnet). See `model_routing_table.md`.
- **Session recovery**: This session was itself a recovery from an interrupted `cargo check`. The session-recovery workflow (`/workflow-session-recovery`) was used. All files were structurally complete at recovery start; only wgpu 29.x API errors required repair.
