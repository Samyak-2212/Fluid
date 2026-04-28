// Spatial-hash broadphase — Tier A post-gate implementation stub.
//
// Cell size is loaded from config/physics_core.toml key `broadphase_cell_size`
// (default: 1.0 metres). Must not be hardcoded.
//
// BVH is NOT used at Tier 0 (too expensive to construct per frame).
// Use spatial hash only at Tier 0; BVH may be offered at Tier 1+ as an option.
//
// [IMPLEMENTATION PENDING — Tier A required]
