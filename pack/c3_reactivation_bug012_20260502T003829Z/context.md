# C3 Rendering — BUG-012 Reactivation Context

Session ID: c3_reactivation_bug012_20260502T003829Z
Reactivated: 2026-05-02T00:38:29+05:30
Retired: 2026-05-02T00:40:00+05:30
Prior session: c3_rendering_20260428T173700Z
Bug fixed: BUG-012 (medium)

## Task

Guard `caps.formats[0]` in `RenderSurface::new` (rendering/src/surface.rs, line 37)
against empty format list.

## Files Modified

| File | Action | Notes |
|------|--------|-------|
| `rendering/src/surface.rs` | patched | Line 37: bare index replaced with `.or_else(|| caps.formats.get(0).copied()).unwrap_or(TextureFormat::Bgra8UnormSrgb)` |
| `bug_pool/BUG_POOL.md` | updated | BUG-012 → CLOSED |
| `knowledge/project_manifest.md` | updated | [RETIRED] entry added; BUG-012 noted CLOSED in coordinator status |

## Fix

Before:
```rust
let format = caps
    .formats
    .iter()
    .copied()
    .find(|f| f.is_srgb())
    .unwrap_or(caps.formats[0]);   // panics if formats is empty
```

After:
```rust
let format = caps
    .formats
    .iter()
    .copied()
    .find(|f| f.is_srgb())
    .or_else(|| caps.formats.get(0).copied())
    .unwrap_or(TextureFormat::Bgra8UnormSrgb);   // safe on empty caps
```

## Verification

`cargo test -p rendering`: 12 passed, 0 failed, EXIT:0

## Open Items

None. All C3 domain bugs are now CLOSED.
