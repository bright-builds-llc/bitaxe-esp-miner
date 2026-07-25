---
status: resolved
trigger: "Phase 35 attempt 16 proves a complete original-settings GET transmission, but receives no HTTP response status before the bounded deadline."
created: 2026-07-21T23:38:12Z
updated: 2026-07-21T23:47:16Z
---

## Current Focus

hypothesis: Confirmed and fixed - Phase 34 ordered snapshot publication increased the `/api/system/info` request path beyond the safe margin of the unchanged 8 KiB ESP-IDF HTTP server task.
test: Complete. The HTTP server task now uses an explicit 16 KiB stack, a source regression locks that minimum to the ESP-IDF server configuration, and the canonical firmware plus affected Phase 33/35 tests pass.
expecting: Met by the source guard and rebuilt exact firmware. The larger bounded task preserves the Phase 34 publication-order contract rather than moving serialization back outside its authority.
next_action: Complete the full repository gate, commit the repair, run exact-head preflight, and invoke fresh attempt 17 once under the standing progress-gated authority.

## Symptoms

expected: The instrumented original-settings GET receives a status line, headers, and a bounded JSON body before mutation begins.
actual: Attempt 16 connected and completed the full request write plus transport flush, then reached the 10-second deadline without a response status, headers, or body.
errors: The typed boundary projection reports `response_status_missing` plus `response_timeout`; no raw origin, host, address, headers, body, or identifier is recorded here.
reproduction: A local silent-response peer deterministically reproduces the classifier category. The exact release ELF deterministically reproduces the unsafe stack-capacity relationship without another device request.
started: Phase 35 attempt 16 at exact source `823309599209cde451435c85bb882fe8a456f80d`.

## Eliminated

- Request construction or transmission failure: the schema-v2 probe recorded a complete 99-byte write and transport flush.
- TCP reachability: the typed projection records a completed TCP connection.
- Setup-time Wi-Fi or route-shell failure: the protected capture contains one connected marker and one route-start marker, with no route-unavailable marker.
- Whole-firmware panic or reboot during setup: the protected capture contains continued heartbeat evidence and no panic, abort, or stack-overflow marker.
- Mutation, restoration, or cleanup interference: the original read failed before mutation; the root sealed non-promotion and non-reusable.
- The pre-Phase-34 duplicate full-view path: commit `638a72e1` removed that path, and the current handler still uses the direct system-info projector.

## Evidence

- timestamp: 2026-07-21T23:38:12Z
  checked: Attempt-16 typed seal and schema-v2 projection only.
  found: TCP connected, request transmission completed, and no response status, headers, or body arrived before the bounded deadline. The root is non-promotable and non-reusable.
  implication: Diagnosis advances beyond connection and request transmission to response-side firmware execution.
- timestamp: 2026-07-21T23:45:00Z
  checked: Protected boot capture through category-only counts and first-occurrence positions.
  found: Wi-Fi connected once, the route shell started once, heartbeats continued, and no route-unavailable, panic, abort, stack-overflow, or HTTP-error marker was present.
  implication: The listener was established and the firmware remained generally alive; the failure is isolated to request handling.
- timestamp: 2026-07-21T23:46:00Z
  checked: Exact current release ELF disassembly and the Phase 33/34 source history.
  found: The system-info `OperatorSnapshotPublisher::publish` instantiation reserves 6,080 bytes and candidate collection reserves 1,456 bytes, before the handler, framework, retained-record, and JSON writer frames. The ESP-IDF HTTP server task remains 8,192 bytes. Phase 34 deliberately moved completion, retention, serialization, and external issuance into the ordered publisher after the earlier Phase 33 stack reduction.
  implication: The current ordered request path has no safe margin on the configured task. Increasing the bounded task stack is narrower than weakening the required publication-order contract.
- timestamp: 2026-07-21T23:47:16Z
  checked: Targeted source regression, affected Bazel suites, and canonical ESP32-S3 firmware build after the stack correction.
  found: The new 16 KiB HTTP server constant is consumed by `Configuration::stack_size`; the source guard rejects the prior literal 8 KiB value. Phase 33 durability, Phase 35 HTTP/supervisor/promotion, Phase 30 non-promotion, hardware-attempt policy, parity, and canonical firmware targets pass.
  implication: The fix is regression-backed across the affected software boundaries and preserves the existing safety, no-promotion, and publication-order contracts.
- timestamp: 2026-07-21T23:49:28Z
  checked: Full pre-commit verification after all repair and planning changes.
  found: The ordered Rust format, warnings-denied Clippy, all-target/all-feature build, and all-feature tests passed. Redaction, reference, parity, lifecycle, and diff checks also passed.
  implication: The repair is ready for an atomic commit; exact-current-HEAD preflight remains the only software gate before attempt 17.

## Resolution

root_cause: The exact ELF proves that the Phase 34 ordered system-info publication path consumes nearly the entire unchanged 8 KiB HTTP task before framework and serialization overhead. That route-specific task exhaustion matches the connection timeout while the listener and firmware remain alive.
fix: Configure the ESP-IDF HTTP server with an explicit 16 KiB task stack and enforce that constant through the Phase 33 source guard. Keep the Phase 34 completion, retention, serialization, and issuance ordering intact.
verification: The targeted Rust guard, affected uncached Bazel suites, canonical firmware build, mandatory ordered Rust sequence, redaction, reference, parity, lifecycle, and diff checks pass. Exact-head preflight remains required after the repair commit and before attempt 17; hardware is the final runtime regression for task-stack sufficiency.
files_changed: [`firmware/bitaxe/src/http_api.rs`, `tools/parity/src/phase33_source_guard.rs`, `.planning/debug/phase35-attempt16-response-timeout.md`]
