# app/debug_interface_spec.md
<!-- C8 — Published debug interface contract for C9 (Agent Debugger). -->
<!-- Published at: [C8_INTERFACES_PUBLISHED] -->
<!-- Protocol version: 1 -->

## Overview

The Fluid debug server runs at `127.0.0.1:8082` (configurable via `config/app.toml`).
It is bound to the loopback interface only — never exposed to the network (DEC-009).

C9 MUST call `GET /health` first and verify `protocol_version == 1` before any other operation.

## Endpoints

### `GET /health`

Protocol handshake. C9 checks this before any operation.

**Response**: `200 OK`, `Content-Type: application/json`
```json
{ "ok": true, "protocol_version": 1 }
```

### `GET /state`

Frame-boundary snapshot of application state.

**Response**: `200 OK`, `Content-Type: application/json`
```json
{
  "frame": 1234,
  "sim_time": 0.500,
  "sim_status": "running",
  "scene_name": "Drop Test",
  "entity_count": 42,
  "headless": false,
  "tier": 0,
  "fps": 60.0
}
```

### `GET /tree`

Widget registry — C8-maintained map of `.id()`-tagged widgets.
NOT AccessKit (DEC-013). Only widgets that called `.id(iced::widget::Id::new("..."))` appear.

**Response**: `200 OK`, `Content-Type: application/json`
```json
{
  "headless": false,
  "widgets": [
    { "id": "btn_run", "kind": "Button", "label": "Run", "enabled": true, "value": null },
    { "id": "input_dt", "kind": "TextInput", "label": "Timestep", "enabled": true, "value": "0.016" }
  ]
}
```
**Headless mode**: `"headless": true, "widgets": []`

### `GET /screenshot`

Captures a PNG screenshot of the application window.

**Success**: `200 OK`, `Content-Type: image/png` — raw PNG bytes.

**Rate limited** (max 2/sec): `429 Too Many Requests`
```json
{ "ok": false, "error": "rate_limited" }
```

**Headless mode**: `200 OK` but returns JSON error:
```json
{ "ok": false, "error": "no_display", "headless": true }
```

### `POST /control`

Injects a user action into the application. C9 sends this to automate UI interactions.

**Request body**: `Content-Type: application/json`
```json
{
  "action": "click",
  "target": "btn_run",
  "value": null,
  "path": null
}
```

**Action types**:

| `action` | `target` | `value` | `path` | Effect |
|---|---|---|---|---|
| `"click"` | widget ID | — | — | Simulates a click on the target widget |
| `"keypress"` | widget ID | key name (e.g. `"Enter"`, `"Escape"`) | — | Simulates a key press |
| `"set_field"` | widget ID | new value string | — | Sets a text input or slider value |
| `"menu"` | — | — | `["File", "Save"]` | Activates a menu path |

**Timeout**: 5 seconds from submission. C9 must not queue more than one action per 5s if the previous has not been confirmed consumed.

**Response (success)**: `200 OK`
```json
{ "ok": true }
```

**Response (parse error)**: `400 Bad Request`
```json
{ "ok": false, "error": "invalid json: ..." }
```

### `GET /logs`

Log tail in C6 log format (reused).

**Response**: `200 OK`, `Content-Type: application/json`
```json
{
  "lines": [
    "[INFO  2026-05-02T09:36:42Z] Fluid starting (headless=false)",
    "[INFO  2026-05-02T09:36:43Z] Debug server started at http://127.0.0.1:8082"
  ]
}
```

### `GET /dashboard`

Embedded HTML dashboard (auto-refreshing). Served by `tiny_http` from `include_str!("assets/dashboard.html")`.

**Response**: `200 OK`, `Content-Type: text/html`

## Authentication

If `debug_server_token` is set in `config/app.toml` (non-empty), all requests must include:
```
X-Debug-Token: <token>
```
Requests without a valid token receive `401 Unauthorized`.
If the token is empty (default), no authentication is required.

## Thread Safety

The server reads `Arc<RwLock<AppStateSnapshot>>` written by the main loop at frame boundary.
The widget registry is `Arc<RwLock<WidgetRegistry>>` owned by C8-UI.
The control queue is `Arc<RwLock<Vec<PendingControl>>>` consumed by the main loop.

## Headless Mode (`--headless`)

When the app is started with `--headless`:
- No window is created.
- The debug server starts normally.
- `/state` and `/logs` work as normal.
- `/tree` returns `{ "headless": true, "widgets": [] }`.
- `/screenshot` returns `{ "ok": false, "error": "no_display", "headless": true }`.

## Implementation Notes for C9

1. Always call `/health` first — verify `protocol_version == 1`.
2. Use `/tree` to discover widget IDs before sending `/control` actions.
3. After a `/control` action, poll `/state` to verify the effect.
4. `/screenshot` is rate-limited — wait 500ms between calls.
5. If the connection is refused, the app may be starting up — retry for up to 10 seconds.
