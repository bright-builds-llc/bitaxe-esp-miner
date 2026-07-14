---
status: resolved
trigger: "Phase 33 hardware proof reaches the fresh same-session device origin, but GET /api/system/info times out before the original hostname can be captured."
created: 2026-07-14T15:53:20Z
updated: 2026-07-14T16:07:34Z
---

## Current Focus

hypothesis: Confirmed and fixed - `/api/system/info` alone materialized the full projection bundle, including a JSON `Value` of `SystemInfoWire`, then constructed and serialized a second full `SystemInfoWire` on the 8 KiB HTTP task. The request connection failed while the firmware and serial heartbeat remained alive.
test: Complete. The system-info path now uses a pure direct projection that overlays runtime mining state and returns one `SystemInfoWire` without constructing `ProjectedApiViews` or its duplicate telemetry JSON.
expecting: Met by source regression, pure behavior tests, firmware build, and ELF frame analysis. The public wire mapping is unchanged and the deepest named application frame chain is 1184 bytes smaller before framework and serializer frames.
next_action: Return the resolved debug handoff so the parent Phase 33 workflow can flash and rerun its own detector-gated durability proof.

## Symptoms

expected: After successful flash/setup and HTTP route start, `GET /api/system/info` returns HTTP 200 with a string `hostname` within 10 seconds, enabling Phase 33 original-hostname capture.
actual: A unique fresh same-session origin is valid and the route/network are up, but `/api/system/info` times out. The Phase 33 wrapper stops with `original_hostname_unavailable` before any PATCH or restart.
errors: Normal HTTP, HTTP/1.0, and `Connection: close` requests all time out. Setup monitor evidence contains one route-start marker and no route-unavailable, panic, Guru Meditation, ESP error, task-watchdog, or mutex-poisoned markers, but that monitor ended before the failing request.
reproduction: Issue one read-only `GET /api/system/info` against the protected fresh same-session origin after the current-package setup completes.
started: First Phase 33 proof attempt to advance beyond identity and flash setup on exact source commit `5ba51d7295ff2dffd32ace7c8861f511154e476b`.

## Eliminated

- Basic IP reachability and TCP port 80 availability: ping and TCP checks pass.
- Whole-server or listener failure: `/`, a missing route, `/api/system/statistics`, and `/api/system/scoreboard` respond; the latter two return HTTP 200.
- A setup-time firmware crash or route-registration failure: the completed setup capture contains a route-start marker and no known panic/watchdog/route-unavailable markers.
- Phase 33 state mutation: the wrapper stopped before PATCH or restart and did not change the hostname.
- Access gating, snapshot lock acquisition, projection-state cloning, and the first system-info serialization: the healthy statistics and scoreboard routes execute the same boundaries through `collect_projected_api_views_with_sample_policy`.
- Non-finite float rejection: the pinned `serde_json` writer serializes NaN and infinity as JSON `null`, matching its value serializer, so the final writer does not uniquely reject those values.
- Whole-firmware reset or panic on the diagnostic request: the passive trace continued normal boot-lifetime heartbeats with no reboot, panic, Guru Meditation, stack-canary, watchdog, allocation, heap, or HTTP error marker.

## Evidence

- timestamp: 2026-07-14T15:53:20Z
  checked: Supplied hardware-attempt summary and read-only post-attempt diagnostics.
  found: Detector, stable physical identity, current-package flash, and 360-second setup passed on exact HEAD; exactly one fresh origin was derived; only `/api/system/info` consistently timed out while neighboring API handlers returned.
  implication: The failure is isolated to the system-info request path after the listener and network boundaries.
- timestamp: 2026-07-14T16:03:50Z
  checked: Static route, projection, DTO, serialization, and exact release ELF frame analysis.
  found: Statistics and scoreboard traverse the same access, lock, projection, and initial `SystemInfoWire` serialization path. System info uniquely retains a full `ProjectedApiViews`, constructs another `SystemInfoWire`, and performs a second full serialization. The HTTP task stack is 8192 bytes; exact ELF prologues reserve 864 bytes for `handle_system_info`, 976 for `projected_system_info`, 1280 for the shared view collector, 1968 for snapshot collection, 976 for `project_api_views`, and 704 for `SystemInfoWire::from_snapshot`, before serializer and framework frames.
  implication: The route-specific duplicate materialization crosses a substantially deeper stack/temporary-allocation path than neighboring successful response handlers and is the only implementation difference that explains the isolated connection failure.
- timestamp: 2026-07-14T16:03:50Z
  checked: Sole authorized passive monitor plus exactly one read-only `/api/system/info` GET using the protected current-session detector port and fresh origin.
  found: Pre-attach physical identity and active-owner readiness passed. The GET ended with receive status 56 and no HTTP status. The 360-second passive trace captured continuing heartbeat bytes with zero reboot, panic, Guru Meditation, stack-canary, abort, watchdog, OOM, heap-error, or HTTP-error markers. Post-cleanup readiness, active-owner proof, empty holder state, and monitor exit all passed. Protected trace digest: `e39e7ab4f25847d2361542cccaa100e05fa0ffe885aa3eb91ca8044a81ed3087`.
  implication: The request kills or corrupts its connection-specific HTTP execution without resetting the firmware; this confirms request-task pressure rather than a network outage, device reboot, global lock, or serialization-domain rejection. No retry is authorized or needed for the code fix.
- timestamp: 2026-07-14T16:07:34Z
  checked: Direct-projection implementation and exact post-build Xtensa ELF frames.
  found: The new `projected_system_info` frame is 1072 bytes and the pure `project_system_info` frame is 144 bytes. The old deepest named application chain was 5312 bytes before framework and serializer frames; the new direct chain is 4128 bytes and removes the 1280-byte shared-view frame, reducing that peak by 1184 bytes.
  implication: The built firmware objectively no longer carries the duplicate full-view stack path that isolated `/api/system/info` from the neighboring successful handlers.
- timestamp: 2026-07-14T16:07:34Z
  checked: Required ordered Rust gate and affected canonical build/test surfaces.
  found: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` passed in order. `bazel test //crates/bitaxe-api:tests //tools/parity:tests` passed both targets. `just build` passed the canonical ESP32-S3 release firmware target.
  implication: The pure contract, source guard, workspace, parity tooling, and production firmware compile all accept the fix.

## Resolution

root_cause: The Phase 26 projection bundle became the common collection path for all projected routes. `/api/system/info` then reconstructs a wire DTO from that bundle and serializes it again, even though the bundle already built a system-info DTO and serialized it into `telemetry_payload`. On the 8 KiB ESP-IDF HTTP task this route-specific duplicate materialization exhausts the safe request execution margin and resets the client connection while the firmware remains alive.
fix: Added `project_system_info`, a pure direct projection that overlays the runtime mining state and constructs one `SystemInfoWire`. Firmware `/api/system/info` now calls it from a freshly collected snapshot instead of allocating unrelated statistics, scoreboard, and telemetry views. A pure unit test proves the runtime overlay and a firmware source guard prevents the duplicate full-view path from returning.
verification: The sole bounded hardware diagnostic confirmed a connection-specific failure with the firmware still alive and completed all passive ownership/cleanup gates. Ordered Rust verification, affected Bazel tests, canonical firmware build, source regression, and post-build ELF analysis passed. No second GET, detector, board-info, flash, PATCH, restart, or other hardware action ran during debugging.
files_changed: [`crates/bitaxe-api/src/runtime_projection.rs`, `crates/bitaxe-api/src/lib.rs`, `firmware/bitaxe/src/runtime_snapshot.rs`, `tools/parity/src/phase33_source_guard.rs`, `.planning/debug/phase33-system-info-timeout.md`]
