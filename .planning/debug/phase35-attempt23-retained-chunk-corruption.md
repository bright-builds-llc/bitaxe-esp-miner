---
status: verifying
trigger: "Attempt 23 passed private Boot A classification and dual finalization, then the retained-log HTTP read failed with invalid chunk framing before mutation."
created: 2026-07-22T14:43:45Z
updated: 2026-07-22T14:57:33Z
---

## Current Focus

hypothesis: A cadence task called `httpd_ws_send_frame_async` outside the HTTPD task after a WebSocket close. ESP-IDF accepted the stale numeric descriptor after it had been reused by the retained-log HTTP request, interleaving a WebSocket frame with HTTP chunk framing.
test: Require every background WebSocket frame to enter through `httpd_queue_work`, carry a per-registration lease, then revalidate both the exact lease and `HTTPD_WS_CLIENT_WEBSOCKET` state in the HTTPD callback immediately before sending. Register session-context cleanup for abrupt disconnects, compile the ESP-IDF target, and cover same-route and different-route descriptor reuse in the pure state core.
expecting: No background task can write directly to a descriptor; stale work cannot act on a newer connection even when it reuses the same descriptor and route; a descriptor reused for ordinary HTTP is rejected before any WebSocket bytes are sent.
next_action: Complete the clean software gate, commit the repair and redacted Attempt 23 checkpoint, run exact-current-HEAD preflight, then use fresh Attempt 24 under `continue_after_verified_fix`.

## Symptoms

expected: After the WebSocket helper records a clean close, the following retained-log GET returns valid HTTP chunk framing and Boot A pre-capture completes before PATCH.
actual: Boot A baseline classification and the pre-capture API/WebSocket observations passed, but the retained-log GET stopped before mutation with malformed chunk framing.
errors: Shareable signature is `boot_a_pre_capture_failed` with `flash_stage=monitor`, `flash_boundary=ready`, one closed WebSocket observation, and `chunk_hex_length_invalid=true`.
reproduction: Attempt 23 is sealed and must not be reused. Attempt 18 recorded the same private chunk-framing symptom even after the client-side close-wait repair, while source inspection shows a server-side stale-descriptor write path that the earlier repair could not prevent.
started: Attempt 23 at exact source `ead2347d32ed0dbb8be43c74a3fb3a85a32734a1` after the complete software gate and exact-head preflight.

## Eliminated

- Attempt 22's legacy trust ordering: Attempt 23 passed the private Phase 33 classifier and dual finalizer.
- Detector, probe, flash, or monitor access: every typed flash stage reached `ready`.
- Missing pre-capture data: the baseline classifier, API schema, and WebSocket observation passed.
- Client helper returning before close: the close marker was present before the retained request.
- Mutation, restoration, or cleanup effects: mutation never started and both secondary outcomes were `none`.

## Evidence

- timestamp: 2026-07-22T14:43:45Z
  checked: Attempt 23's sealed typed projection and private artifacts without printing protected identifiers or raw response material.
  found: The retained request alone reported invalid chunk-length syntax after the WebSocket close boundary; no later phase effect began.
  implication: The failure is a cross-request server output-corruption boundary, not missing baseline evidence or a client ordering omission.
- timestamp: 2026-07-22T14:43:45Z
  checked: The ESP-IDF 5.5.4 WebSocket implementation and the firmware cadence send path.
  found: The firmware called `httpd_ws_send_frame_async` from a background task. That API writes against the server's current owner of the numeric descriptor and does not establish the HTTPD work-queue ordering or verify that the descriptor is still a WebSocket.
  implication: A closed WebSocket descriptor reused for retained HTTP can receive stale WebSocket bytes and corrupt its chunk stream.
- timestamp: 2026-07-22T14:43:45Z
  checked: The repaired firmware build and Phase 35 real-process source contract.
  found: The ESP32-S3 release image compiles, and the regression proves the single direct send lives inside queued HTTPD work after route and protocol revalidation.
  implication: The targeted fix is executable on the actual firmware target and closes the identified ownership boundary.

## Resolution

root_cause: Background cadence sends bypassed the HTTPD task's work queue. A queued frame could outlive its original WebSocket session, and `httpd_ws_send_frame_async` could then write its bytes to a later ordinary HTTP connection that reused the same descriptor, corrupting retained-response chunk framing.
fix: Assign each registration a generation lease, bind it to ESP-IDF session-context cleanup, copy each background payload and lease into owned queued work, execute the only direct asynchronous WebSocket send inside the HTTPD task, verify the lease is still current, verify the descriptor is still a WebSocket, and unregister only an exact current lease.
verification: The firmware image build, five deterministic lease tests, focused Phase 35 suites, parity suite, and mandatory Rust gate pass. Code commit `fafbeec9` contains the repair. The redacted checkpoint commit and exact-head preflight remain pending before Attempt 24.
files_changed:

- firmware/bitaxe/src/http_api.rs
- firmware/bitaxe/src/websocket_api.rs
- firmware/bitaxe/BUILD.bazel
- crates/bitaxe-api/src/websocket_state.rs
- crates/bitaxe-api/src/lib.rs
- scripts/phase35-correlated-evidence-test.sh
- scripts/BUILD.bazel
- .planning/debug/phase35-attempt23-retained-chunk-corruption.md
