# Phase 06 Ultra 205 Safety Hardware Smoke

## Status

Conclusion: not run - hardware verification pending

## Purpose

This template records the hardware-smoke protocol required before Phase 06 safety-critical rows can move to `verified`. It must be filled from a controlled Ultra 205 BM1366 bench run and must not include pool credentials, Wi-Fi passwords, private endpoints, or NVS secrets.

## Required Board Identity

| Field | Required Value | Observed Value |
| --- | --- | --- |
| Board | Ultra 205 | pending |
| Board version | 205 | pending |
| ASIC | BM1366 | pending |
| Reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` | pending |
| Firmware commit | record the tested commit | pending |
| Serial port | record explicit `port=` value | pending |

## Required Command

```bash
just flash-monitor board=205 port=<port>
```

Optional API/WebSocket smoke commands may be added after the firmware is reachable on a trusted local network. Redact all credentials and private network details.

## Required Observations

| Surface | Required Evidence | Observed Result |
| --- | --- | --- |
| Boot safety baseline | Boot identity, safe-state log, reference commit, firmware commit | pending |
| INA260 telemetry | Current, voltage, and power reading source, freshness, API/log/WebSocket status | pending |
| DS4432U voltage control | Requested setpoint, suppressed/armed mode, actual observed safety status | pending |
| Fan control | Duty request, RPM feedback, zero-RPM/failure behavior, API/log status | pending |
| Thermal control | Chip/board/VR readings, invalid sentinel handling, overheat status | pending |
| ASIC reset/power init | Reset/power sequencing logs and fail-closed status | pending |
| Self-test | Start source, pass/fail/cancel/restart result, factory flag behavior | pending |
| Watchdog responsiveness | API/log/WebSocket remains responsive while safety supervisor runs | pending |
| Runtime display/input | Runtime safety display or input behavior, or explicit runtime gap log | pending |

## Required Log Lines Or Equivalents

- `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`
- `mining_loop_status=blocked`
- `safety_supervisor=started thread=bitaxe-safety-supervisor cadence_ms=100`
- `safety_supervisor_step=yield reason=yield_interval_reached`
- `display_input_status=runtime_gap reason=hardware_evidence_pending startup_only=true` unless a later runtime display/input implementation replaces the gap with hardware evidence

## Acceptance Rules

- A row may use `hardware-smoke` only when this file records board identity, port, firmware commit, command, observed result, and a passing conclusion for that row's behavior.
- A safety-critical row may use `hardware-regression` only when a repeatable hardware test or scripted probe is checked in and passes.
- Unit, golden, API-compare, and workflow evidence can support `implemented` or `in-progress` rows, but not `verified` safety-critical hardware-control rows.

## Conclusion

Conclusion: not run - hardware verification pending
