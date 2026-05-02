# agent_debugger/DECISIONS.md
<!-- C9 — Architectural Decision Registry -->
<!-- All decisions locked here. Propose changes via bug_pool/BUG_POOL.md, not in-place edits. -->

## DEC-C9-001 — HTTP client: ureq (blocking)

**Decision:** Use `ureq` for all HTTP calls to the C8 debug server.
**Rationale:** Blocking I/O is sufficient for a CLI tool. Avoids tokio/async complexity.
**Alternatives rejected:** reqwest (async, heavy), curl FFI (non-portable).
**Locked:** 2026-05-02

---

## DEC-C9-002 — Screenshot capture: xcap

**Decision:** Use `xcap` crate for window screenshot capture.
**Rationale:** Cross-platform (Win32, X11, Wayland portal, ScreenCaptureKit). Single dependency.
**Fallback:** If xcap fails or is not implemented, C9 returns `{ "ok": false, "error": "screenshot_not_implemented" }` gracefully — no panic.
**Locked:** 2026-05-02

---

## DEC-C9-003 — Input injection: enigo (fallback only)

**Decision:** `enigo` is the last-resort fallback for widget control.
**Primary path:** `POST /control` HTTP endpoint (Wayland-safe, all platforms).
**Rationale:** Direct HTTP control is more reliable and does not require display access.
**Scope:** `input.rs` — tagged [NEEDS_REVIEW: claude] per AGENTS.md (OS-level input injection).
**Locked:** 2026-05-02

---

## DEC-C9-004 — Error handling: DebugError enum, no unwrap()

**Decision:** All fallible operations return `Result<_, DebugError>`. `unwrap()` is banned.
**Rationale:** CLI tools that panic on transient network errors are unusable in agent pipelines.
**Pattern:** `DebugError` is an enum covering `Http`, `Json`, `Io`, `Protocol`, `Timeout`, `NotImplemented`.
**Locked:** 2026-05-02

---

## DEC-C9-005 — Protocol version check: abort on mismatch

**Decision:** `GET /health` is called before every operation; abort if `protocol_version != 1`.
**Rationale:** Prevents silent misinterpretation of changed API shapes.
**Locked:** 2026-05-02

---

## DEC-C9-006 — Widget IDs: discovered at runtime via GET /tree

**Decision:** Widget IDs are NEVER hardcoded. All targeting uses IDs returned by `GET /tree`.
**Rationale:** C8 widget registry evolves; hardcoded IDs would silently break.
**Locked:** 2026-05-02

---

## DEC-C9-007 — Session archives: JSON committed, PNGs deleted after commit

**Decision:** Session dir `agent_debugger/sessions/<ts>/` holds both JSON (committed) and PNGs (gitignored, deleted via `cleanup --session <ts>`).
**Rationale:** JSON provides permanent audit trail; PNGs are large and transient.
**Locked:** 2026-05-02

---

## DEC-C9-008 — Config file: config/agent_debugger.toml

**Decision:** All tunables (host, port, token, timeout_secs, screenshot_min_interval_ms) read from `config/agent_debugger.toml`. No hardcoded values.
**Locked:** 2026-05-02

---

## DEC-C9-009 — Screenshot rate-limit compliance

**Decision:** C9 enforces a minimum 500ms gap between screenshot calls (matching C8's 429 rate limit).
**Implementation:** Track `last_screenshot_at: Option<Instant>`. Sleep if needed.
**Locked:** 2026-05-02

---

## DEC-C9-010 — Headless fallback: state+tree only

**Decision:** When screenshot returns `screenshot_not_implemented` or `no_display`, C9 continues with state+tree data and marks the report `incomplete: true`.
**Rationale:** Graceful degradation per PROMPT.md §Fault Tolerance.
**Locked:** 2026-05-02
