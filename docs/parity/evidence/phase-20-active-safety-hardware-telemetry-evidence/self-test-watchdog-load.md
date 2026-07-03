# Phase 20 Self-Test, Watchdog, And Load Evidence

## Scope

This evidence pack records the exact Phase 20 claim boundary for Ultra 205 board
`205` self-test hardware submodes, watchdog supervisor behavior, and bounded
load behavior. It consumes the detector-gated safe-baseline serial log from
Plan 20-02 and the existing Phase 14 self-test/watchdog/load wrapper.

The pack does not run self-test hardware submodes, mining work, ASIC diagnostic
work, voltage/fan control, reboot tests, fault stimuli, or bounded load work.

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
| Allow manifest | `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/self-test-watchdog-load/allow-self-test-watchdog-load.json` |
| Wrapper command | `scripts/phase14-self-test-watchdog-load.sh --manifest docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/self-test-watchdog-load/allow-self-test-watchdog-load.json --out-dir docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/self-test-watchdog-load --serial-log docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/flash-monitor.log` |
| Wrapper log | `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/self-test-watchdog-load/self-test-watchdog-load.log` |
| Redaction basis | Plan 20-02 committed safe-baseline evidence is redacted and marked commit-ready. |

## Observed Breadcrumbs

The allow manifest passed `tools/parity safety-allow` for surface
`self-test-watchdog-load` with `claim_tier: unsupported-pending` and
`evidence_class: deferred`.

The wrapper log records:

- `watchdog_supervisor_status: observed`
- `watchdog_supervisor_start_marker: observed - safety_supervisor=started thread=bitaxe-safety-supervisor cadence_ms=100`
- `watchdog_supervisor_yield_marker: observed - safety_supervisor_step=yield reason=yield_interval_reached`
- `load_stress_status: pending - bounded workload stimulus unavailable`
- `self_test_hardware_status: pending - no production-safe self-test hardware submode route exists`
- `phase14_self_test_watchdog_load_status: pending - self-test hardware and bounded load routes unavailable`

## Exact Claim Boundary

`watchdog_supervisor_status: observed` supports only the startup/yield shell
breadcrumb for the supervisor loop. It does not prove blocked-task behavior,
bounded workload responsiveness, recovery after watchdog intervention, or API
and WebSocket responsiveness under load.

`load_stress_status: pending - bounded workload stimulus unavailable`

`self_test_hardware_status: pending - no production-safe self-test hardware submode route exists`

`SELF-001` remains below verified for self-test hardware submodes, exact
pass/fail/cancel behavior, restart behavior, factory-flag behavior, production
mining gate behavior, recovery path, and post-action safe state.

`SAFE-09` remains below verified for bounded load and watchdog recovery. The
serial supervisor markers are breadcrumbs only and cannot establish bounded load
behavior without a safe route, pass/fail criteria, abort conditions, and final
safe-state evidence.

## Non-Claims

This evidence does not verify self-test fan checks, power checks, diagnostic ASIC
work, fake Stratum work, mining work, reboot behavior, watchdog reset behavior,
bounded load behavior, voltage/fan/ASIC control, or any active fault path.

## Checklist Rows

| Row | Evidence class | Status in this pack | Claim boundary |
| --- | --- | --- | --- |
| `SELF-001` | deferred | below verified | Hardware self-test submodes and lifecycle behavior were not exercised. |
| `SAFE-09` | deferred | below verified | Watchdog supervisor startup/yield markers were observed, but bounded load and recovery were not exercised. |
