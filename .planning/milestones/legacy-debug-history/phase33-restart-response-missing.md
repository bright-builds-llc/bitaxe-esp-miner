---
status: resolved
trigger: "Phase 33 application-restart proof receives zero response bytes before the firmware restarts."
created: 2026-07-14T16:25:00Z
updated: 2026-07-14T16:34:00Z
---

## Current Focus

hypothesis: Confirmed and fixed — the Rust handler wrote one response chunk and immediately called `esp_restart()` before esp-idf-svc could complete the chunked response after the handler returned.
test: Complete. The inline restart is now one named, bounded-stack worker that waits the named upstream-parity 1000 ms interval after the response handler returns, then emits the existing marker and restarts.
expecting: Met by the source regression, affected Cargo/Bazel tests, canonical firmware build, and ordered Rust verification. The exact public JSON response remains unchanged.
next_action: Return the resolved debug handoff so the parent Phase 33 workflow can package, flash, and rerun its detector-gated durability proof.

## Symptoms

expected: `POST /api/system/restart` returns the existing exact 200 JSON response, completes the response, waits an explicit 1000 ms post-response interval, and performs exactly one application restart.
actual: Exact firmware commit `4d2fabb` passed detector, physical-identity, flash, setup, hostname confirmation, and passive-monitor gates, but the sole POST timed out after 15 seconds with zero response bytes.
errors: The wrapper reported `restart_response_missing`; protected capture could not qualify response-before-effect.
reproduction: The current Phase 33 hardware wrapper sends one no-body POST after passive monitoring is armed. Hardware reproduction is prohibited for this debug task.
started: Discovered during the 2026-07-14 Phase 33 same-board durability proof.

## Eliminated

- Request-body shape: both the Rust restart route and upstream C handler ignore the body. The no-body request reached the restart effect, so the upstream frontend's `{}` body is not causal.
- Access denial or JSON serialization failure: `handle_command` uses `?` after `send_json`; reaching `esp_restart()` proves access and the response-chunk write returned success.
- Wrapper retry behavior: the wrapper correctly stopped after its sole restart request, cleaned up, restored the hostname, and did not retry.

## Evidence

- timestamp: 2026-07-14T16:25:00Z
  checked: `firmware/bitaxe/src/http_api.rs` command route and esp-idf-svc 0.52.1 HTTP connection completion.
  found: `handle_command` calls `send_json` and then immediately calls `apply_command_effect`. The restart arm invokes `esp_restart()` inline. `send_json` writes a chunk, but esp-idf-svc sends the terminating zero-length chunk only from `EspHttpConnection::complete()` after the handler returns.
  implication: Restarting inside the handler prevents framework response completion even when the body-chunk write itself succeeds.
- timestamp: 2026-07-14T16:25:00Z
  checked: `reference/esp-miner/main/http_server/http_server.c::POST_restart`.
  found: Upstream sends the existing JSON response, delays 1000 ms specifically to let it leave the device, then calls `esp_restart()`.
  implication: The missing delay and inline restart are a direct behavioral-parity gap.
- timestamp: 2026-07-14T16:34:00Z
  checked: Narrow restart implementation and source-order regression.
  found: The route still writes the unchanged response first. It then starts one named worker with an explicit 8 KiB stack and returns, allowing esp-idf-svc to emit the terminating chunk. The worker waits `RESTART_POST_RESPONSE_DELAY_MS = 1_000`, emits the existing marker, and calls `esp_restart()`.
  implication: Response completion and restart are now separated by a bounded-lifetime adapter with the exact upstream delay; there is no indefinite worker or optimistic success path.
- timestamp: 2026-07-14T16:34:00Z
  checked: Request-body compatibility.
  found: Neither the Rust route nor upstream restart handler parses a body, and the failed no-body POST reached the restart effect. The frontend's `{}` body therefore cannot explain the missing terminating response chunk.
  implication: The Phase 33 wrapper remains unchanged; adding an inert body would add noise without guarding the causal boundary.
- timestamp: 2026-07-14T16:34:00Z
  checked: Focused and affected verification before the final ordered pre-commit gate.
  found: The focused restart source guard passed; all eight Phase 33 parity source guards passed; Bazel passed `//tools/parity:tests` and `//scripts:phase33_confirmed_settings_durability_test`; `just build` passed the canonical ESP32-S3 firmware target.
  implication: The changed firmware shell compiles and the causal response -> bounded schedule -> delay -> marker -> restart order is regression guarded without hardware access.

## Resolution

root_cause: `send_json` writes a response body chunk, but esp-idf-svc emits the terminating chunk only from `EspHttpConnection::complete()` after the handler returns. The command handler instead called `esp_restart()` inline, so the device reset before HTTP completion. Upstream explicitly delays 1000 ms after sending its response for this reason.
fix: Replaced the inline restart with one named, bounded-stack worker. After the unchanged response write, the handler schedules the worker and returns; the worker waits the named 1000 ms parity interval, emits the existing marker, and restarts. Thread-spawn failure propagates instead of being swallowed. The wrapper remains bodyless because body shape is not causal or parsed.
verification: Focused restart guard, all Phase 33 Cargo source guards, affected Bazel parity and wrapper tests, canonical firmware build, ordered Rust pre-commit gates, and diff/redaction review passed. No detector, board-info, flash, monitor, HTTP, restart, credential, or hardware action ran.
files_changed: [`firmware/bitaxe/src/http_api.rs`, `tools/parity/src/phase33_source_guard.rs`, `.planning/debug/phase33-restart-response-missing.md`]
