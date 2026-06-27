# Phase 05 AxeOS API, Logs, And Telemetry Evidence

**Date:** 2026-06-27
**Firmware commit:** `2c5ee77`
**Reference commit:** `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Scope Conclusion

Phase 05 adds AxeOS-compatible API wire contracts, settings PATCH planning, system/ASIC/statistics/scoreboard response mappers, retained log and WebSocket telemetry contracts, non-OTA command planners, firmware route shell wiring, and parity API comparison fixtures.

This evidence proves schema compatibility, captured-response fixture compatibility, static AxeOS route usage compatibility, firmware buildability, and repo test health. It does not record live Ultra 205 HTTP/WebSocket smoke. Voltage, fan, thermal, power, ASIC initialization, OTA, OTAWWW, SPIFFS, recovery update behavior, static-asset packaging, and release artifacts remain below verified unless separately covered by hardware or Phase 7 evidence.

## Evidence Type Boundary

| Evidence type | Phase 05 result | Boundary |
| --- | --- | --- |
| Schema compatibility | Passed | Route/property checks against `reference/esp-miner/main/http_server/openapi.yaml` and `bitaxe_api::phase05_routes()`. |
| Captured-response compatibility | Passed | Checked synthetic/public JSON fixtures for system info, ASIC, statistics, scoreboard, settings PATCH, logs, live telemetry, and command responses. |
| Static route compatibility | Passed | Checked existing AxeOS service route usage for API/log/WebSocket administration without adding a frontend framework or Angular build pipeline. |
| Firmware smoke | Not run | No live Ultra 205 API/WebSocket smoke was run in this plan. |

## Focused API Tests

**Command:**

```bash
bazel test //tools/parity:tests && bazel test //crates/bitaxe-api:tests
```

**Result:** Passed.

**Key output:**

```text
//tools/parity:tests PASSED
//crates/bitaxe-api:tests (cached) PASSED
```

**Covers:** API compare tests, route/static fixture guard tests, parity report validation tests, and all pure API crate tests for wire DTOs, settings, system/ASIC/mining, statistics, scoreboard, logs, telemetry, command planners, and route shell decisions.

## API Compare

**Command:**

```bash
bazel run //tools/parity:report -- api-compare
```

**Result:** Passed.

**Key output:**

```text
api_compare:
- schema | status=passed | checked=95
  note: OpenAPI route/property coverage and Rust route-shell manifest
- captured-response | status=passed | checked=47
  note: Representative checked-in JSON response fixtures
- static-route | status=passed | checked=36
  note: Existing AxeOS service route usage plus recovery/static boundaries
- firmware-smoke | status=not-run | checked=0
  note: No live Ultra 205 API/WebSocket smoke was run by Phase 05 Plan 07; hardware smoke remains separate evidence.
validation_errors: none
```

**Covers:**

- Phase 05 route fixture completeness and Rust route-shell coverage.
- OpenAPI route/property checks for system info, ASIC, statistics, scoreboard, settings PATCH, logs, command routes, OTA, and OTAWWW.
- Captured-response fixture checks for system info, ASIC, statistics, scoreboard, settings PATCH public errors, log download/raw stream data, live WebSocket frames, and command responses.
- Static AxeOS route usage for info, ASIC, statistics, scoreboard, logs, PATCH settings, pause, resume, restart, identify, block-found dismiss, `/api/ws`, and `/api/ws/live`.
- OTA and OTAWWW are present only as Phase 7-owned unsafe-success-blocked routes.
- `/recovery` and static fallback are represented separately from ordinary API route usage and do not count as Phase 05 static/release packaging success.

## Firmware Build

**Command:**

```bash
bazel build //firmware/bitaxe:firmware
```

**Result:** Passed.

**Key output:**

```text
Target //firmware/bitaxe:firmware up-to-date:
  bazel-bin/firmware/bitaxe/bitaxe-firmware.elf
INFO: Build completed successfully
```

**Covers:** ESP-IDF firmware route shell and adapter code still compile after Phase 05 API compare tooling was added. This is build evidence only, not a live device API smoke.

## Full Repo Test

**Command:**

```bash
just test
```

**Result:** Passed.

**Key output:**

```text
bazel test //...
//crates/bitaxe-api:tests (cached) PASSED
//crates/bitaxe-asic:tests (cached) PASSED
//crates/bitaxe-config:tests (cached) PASSED
//crates/bitaxe-core:tests (cached) PASSED
//crates/bitaxe-stratum:tests (cached) PASSED
//crates/bitaxe-test-support:tests (cached) PASSED
//scripts:verify_reference_clean_test (cached) PASSED
//tools/flash:tests (cached) PASSED
//tools/xtask:tests (cached) PASSED
//tools/parity:tests PASSED
Executed 1 out of 10 tests: 10 tests pass.
```

**Covers:** The repo-owned full test entrypoint remains green with Phase 05 API and parity tooling in place.

## `just parity`

**Command:**

```bash
just parity
```

**Result:** Passed after the Phase 05 checklist updates.

**Key output:**

```text
validation_errors: none
```

**Covers:** Checklist parsing, reference cleanliness guard, reference commit reporting, and invalid verified-claim checks.

## Static And Recovery Boundary

Phase 05 static compatibility evidence is limited to route usage: existing AxeOS services can still administer V1 API/log/WebSocket surfaces without a new frontend framework or Angular rewrite.

`/recovery` and static fallback are fixture-recorded as separate Phase 7-owned packaging surfaces. Phase 05 does not prove SPIFFS availability, recovery page serving, static asset packaging, release image layout, OTA update behavior, or OTAWWW asset update behavior.

## Hardware Smoke

Conclusion: not run - hardware/API smoke evidence pending.

Required before live firmware API/WebSocket behavior can move beyond implemented:

- Board: Ultra 205 BM1366.
- Command: `just flash-monitor board=205 port=<port>`.
- Firmware commit and reference commit.
- HTTP checks for representative success/error cases: info, ASIC, statistics, scoreboard, settings PATCH, logs, commands, OTA/OTAWWW fail-closed behavior.
- WebSocket checks for `/api/ws` raw log stream and `/api/ws/live` full/diff updates.
- Redacted logs with no Wi-Fi password, pool password, private key, or secret certificate content.

## Checklist Impact

- `API-001` is verified with `api-compare` evidence.
- `API-002` through `API-006`, `LOG-001`, and API-facing statistics/scoreboard rows move only to implementation-level statuses because live firmware smoke was not run.
- `API-007`, `FS-001`, `OTA-001`, and `OTA-002` remain Phase 7 pending/deferred and are not counted as Phase 05 success.
- Power, voltage, fan, thermal, and ASIC initialization rows remain below verified because Phase 05 did not add hardware-control evidence.

## Residual Risk

- The API compare command checks schema and fixtures, not a live HTTP server response.
- The static route usage fixture proves route compatibility for existing services, not browser rendering or release asset packaging.
- Firmware route buildability does not prove Wi-Fi network reachability, WebSocket client behavior on hardware, ESP-IDF HTTP runtime behavior, or retained log delivery on a device.
- Safety-critical hardware telemetry remains intentionally safe/defaulted until Phase 6 evidence exists.
