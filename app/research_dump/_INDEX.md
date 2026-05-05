# app/research_dump/_INDEX.md
<!-- C8 — Fluid GUI Application: Research Dump Index -->
<!-- All entries carry an expiry date. Expired = re-verify before use. -->
<!-- Format: | Topic | Date | Expiry | Source | Summary | -->

## Active Research

| Topic | Date | Expiry | Source | Summary |
|---|---|---|---|---|
| iced 0.13 pane_grid API | 2026-05-02 | 2026-08-02 | docs.rs/iced | `pane_grid::State`, `pane_grid::Pane`, resize/drag events confirmed available in 0.13. `PaneGrid::new` takes a view closure. |
| iced::widget::shader status | 2026-05-02 | 2026-08-02 | iced changelog | `iced::widget::shader` stabilized in 0.13. Provides `wgpu::Device` + `wgpu::Queue` to shader program; compatible with external wgpu pipelines. |
| tiny_http 0.12 loopback binding | 2026-05-02 | 2026-08-02 | docs.rs/tiny_http | `Server::http("127.0.0.1:8082")` binds loopback only. Response body can be static `&[u8]` or `Vec<u8>`. Confirmed used by C6 at port 8081. |
| fbxcel-dom maturity | 2026-05-02 | 2026-06-02 | crates.io/crates/fbxcel-dom | [UNVERIFIED] Last release 0.9.0 (2023-01). Check for activity before C8-Import starts work. DEC-003 flagged as [UNVERIFIED]. |
| directories crate OS paths | 2026-05-02 | 2026-11-02 | docs.rs/directories | `ProjectDirs::from("", "", "Fluid")` → `config_dir()` returns `%APPDATA%\Fluid` (Win), `~/.config/fluid` (Linux), `~/Library/Application Support/Fluid` (macOS). Matches DEC-016. |
| notify + arc-swap file watcher | 2026-05-02 | 2026-08-02 | docs.rs/notify, docs.rs/arc-swap | `notify::RecommendedWatcher` sends `DebouncedEvent` over a channel. For iced integration use `Subscription::run` + `stream::channel`. Never use bare threads (DEC-017). |
| xcap screenshot crate | 2026-05-02 | 2026-06-02 | crates.io/crates/xcap | [UNVERIFIED] xcap provides cross-platform screen capture. Verify headless behavior (should return Err, not panic). Rate-limit to 2/sec per PROMPT.md spec. |
| MessagePack map-based serde | 2026-05-02 | 2026-11-02 | docs.rs/rmp-serde | `rmp_serde::encode::write_named` serializes structs as maps (field names preserved). DEC-011 mandates this. Use `#[serde(rename_all = "snake_case")]` for stability. |

## Expired Research

<!-- Move entries here when past their Expiry date. Re-verify before promoting back to Active. -->

## Research Protocol

1. Add an entry here before writing implementation code that depends on external API behavior.
2. Set expiry to 90 days for stable APIs, 30 days for crates with active churn.
3. Tag `[UNVERIFIED]` if not personally confirmed against docs.rs or crate source.
4. Expired entries MUST be re-verified before use — do not trust stale research.
