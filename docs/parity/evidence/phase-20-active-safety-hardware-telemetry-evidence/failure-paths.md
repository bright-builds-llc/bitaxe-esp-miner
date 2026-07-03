# Phase 20 Failure-Path Evidence

## Scope

This evidence pack records the exact Phase 20 claim boundary for Ultra 205 board
`205` safety failure paths. It uses the Phase 20 `failure-paths` safety allow
surface and the phase-owned wrapper, but it does not run live fault injection or
any active fault stimulus.

The purpose of this pack is to make `SAFE-04` and related power, thermal, fan,
ASIC, and self-test fault rows independently trackable without overclaiming
fault behavior.

## Metadata

| Field | Value |
| --- | --- |
| Board | `205` |
| Selected port | `/dev/cu.usbmodem1101` |
| Detector command | `just detect-ultra205` |
| Board-info command | `espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` |
| Board-info status | passed |
| Source commit | `c11fba2622a389af533774447956b95f254c0280` |
| Reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| Package manifest | `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/bitaxe-ultra205-package.json` |
| Safe-baseline serial log | `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/flash-monitor.log` |
| Allow manifest | `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths/allow-failure-paths.json` |
| Wrapper command | `scripts/phase20-failure-paths.sh --manifest docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths/allow-failure-paths.json --out-dir docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths` |
| Wrapper log | `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths/failure-paths.log` |
| Redaction basis | Plan 20-02 committed safe-baseline evidence is redacted and marked commit-ready. |

## Observed Status

The allow manifest passed `tools/parity safety-allow` for surface
`failure-paths` with `claim_tier: unsupported-pending` and
`evidence_class: deferred`.

The wrapper log records:

- `failure_paths_status: blocked - no production-safe fault stimulus route`
- `fault_stimulus_status: not_run`
- `expected_fault_status: not_observed`
- `api_projection_status: not_run`
- `websocket_projection_status: not_run`
- `final_safe_state_status: required-before-promotion`
- `active_rows_status: below_verified`
- `checklist_rows: PWR-001,PWR-002,THR-001,THR-002,SELF-001,SAFE-04`
- `non_claims: overheat stimulus, fan fault stimulus, power fault stimulus, thermal fault stimulus, ASIC fault stimulus`

## Missing Prerequisites

| Failure area | Missing stimulus | Missing expected_fault proof | Missing abort condition | Missing restore path | Missing projection | Missing final_safe_state marker |
| --- | --- | --- | --- | --- | --- | --- |
| Overheat safe stop | No bounded overheat stimulus route. | No observed `overheat_safe_stop` or thermal fault status. | No temperature ceiling, timeout, or immediate stop condition was exercised. | No post-stimulus cooldown or restore package path was exercised. | No API/log/WebSocket projection was captured. | No final safe-state marker after overheat stimulus was observed. |
| Fan fault | No bounded fan zero-RPM or fan-set-failed stimulus route. | No observed `fan_zero_rpm` or `fan_set_failed` status. | No fan-duty threshold, RPM sample window, timeout, or stop condition was exercised. | No fan restore path was exercised. | No API/log/WebSocket projection was captured. | No final safe-state marker after fan fault stimulus was observed. |
| Power fault | No bounded power-fault stimulus route. | No observed `power_fault` status. | No voltage/current/power threshold or stop condition was exercised. | No power restore path was exercised. | No API/log/WebSocket projection was captured. | No final safe-state marker after power fault stimulus was observed. |
| Thermal sensor fault | No bounded thermal sensor unavailable/invalid stimulus route. | No observed `thermal_sensor_fault` status. | No sensor-fault timeout or stop condition was exercised. | No sensor restore path was exercised. | No API/log/WebSocket projection was captured. | No final safe-state marker after thermal fault stimulus was observed. |
| ASIC fault | No bounded ASIC fault stimulus route. | No observed ASIC safe-blocked status. | No ASIC reset/work timeout or stop condition was exercised. | No ASIC restore path was exercised. | No API/log/WebSocket projection was captured. | No final safe-state marker after ASIC fault stimulus was observed. |
| Self-test failure | No bounded self-test hardware failure stimulus route. | No observed self-test fail/cancel/restart result from hardware. | No self-test abort condition was exercised. | No self-test recovery path was exercised. | No API/log/WebSocket projection was captured. | No final safe-state marker after self-test failure stimulus was observed. |

## Exact Claim Boundary

`failure_paths_status: blocked - no production-safe fault stimulus route`

`fault_stimulus_status: not_run`

`expected_fault_status: not_observed`

`final_safe_state_status: required-before-promotion`

`SAFE-04` remains below verified. The pure fault module classifies overheat,
fan, power, thermal sensor, and ASIC faults into fail-closed plans, but this pack
does not provide hardware-regression proof that real fault stimuli enter those
states or expose compatible user-visible status.

Future promotion requires a plan-approved route that names the stimulus, expected
fault, abort condition, restore path, log/API/WebSocket projection, and final
safe-state marker before any fault-path row can move above blocked/deferred
evidence.

## Non-Claims

`non_claims: overheat stimulus, fan fault stimulus, power fault stimulus, thermal fault stimulus, ASIC fault stimulus`

This evidence does not verify fault injection, overheat stop/cool/restart
behavior, fan zero-RPM behavior, fan set failure behavior, power fault behavior,
thermal sensor fault behavior, ASIC fault behavior, self-test hardware
fail/cancel/restart behavior, API projection, WebSocket projection, or recovery
after a fault stimulus.

## Checklist Rows

| Row | Evidence class | Status in this pack | Claim boundary |
| --- | --- | --- | --- |
| `PWR-001` | deferred | below verified | Power fault behavior was not stimulated. |
| `PWR-002` | deferred | below verified | Power safe-state behavior was not observed after a fault. |
| `THR-001` | deferred | below verified | Thermal fault behavior was not stimulated. |
| `THR-002` | deferred | below verified | Fan fault behavior was not stimulated. |
| `SELF-001` | deferred | below verified | Self-test failure-path behavior was not stimulated. |
| `SAFE-04` | deferred | below verified | No production-safe bounded fault-stimulus route exists in this plan. |
