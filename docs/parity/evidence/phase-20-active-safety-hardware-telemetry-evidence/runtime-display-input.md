# Phase 20 Runtime Display And Input Evidence

## Scope

This evidence pack records the exact Phase 20 claim boundary for Ultra 205 board
`205` startup display breadcrumbs, runtime display flow, and physical input
behavior. It consumes the detector-gated safe-baseline serial log from Plan
20-02 and the existing Phase 14 display/input wrapper.

The pack does not exercise a runtime display route, physical buttons, LVGL
screens, runtime screen carousel behavior, input routing, or API/WebSocket
administration during display/input activity.

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
| Allow manifest | `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/runtime-display-input/allow-display-input.json` |
| Wrapper command | `scripts/phase14-display-input.sh --manifest docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/runtime-display-input/allow-display-input.json --out-dir docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/runtime-display-input --serial-log docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/flash-monitor.log` |
| Wrapper log | `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/runtime-display-input/display-input.log` |
| Redaction basis | Plan 20-02 committed safe-baseline evidence is redacted and marked commit-ready. |

## Observed Breadcrumbs

The allow manifest passed `tools/parity safety-allow` for surface
`display-input` with `claim_tier: read-only-observation` and
`evidence_class: hardware-smoke`.

The wrapper log records:

- `display_status=startup_text_rendered model=SSD1306 size=128x32 address=0x3c`
- `startup_display_status: observed`
- `display_input_status=runtime_gap reason=hardware_evidence_pending startup_only=true`
- `runtime_gap_marker_status: observed`
- `runtime_display_input_status: pending - no runtime display/input route or physical input observation`
- `phase14_display_input_status: pending - runtime display/input route unavailable`

## Exact Claim Boundary

Startup SSD1306 text and the runtime gap marker are supporting breadcrumbs only.
They do not verify runtime display pages, screen flow, LVGL parity, physical
input behavior, button routing, self-test display behavior, or runtime
administration behavior.

`runtime_display_input_status: pending - no runtime display/input route or physical input observation`

`display_input_status=runtime_gap reason=hardware_evidence_pending startup_only=true`

`IO-001`, `UI-001`, `UI-002`, and `UI-003` remain below verified for runtime
display/input parity unless a future plan exercises a real runtime route with
physical or log/API/WebSocket-observed behavior, abort conditions, and final
safe-state evidence.

## Non-Claims

This evidence does not verify physical input events, runtime UI navigation,
runtime display page updates, LVGL screen parity, button behavior, API/WebSocket
administration during runtime display work, self-test display flows, or any
temporary production route.

## Checklist Rows

| Row | Evidence class | Status in this pack | Claim boundary |
| --- | --- | --- | --- |
| `IO-001` | hardware-smoke breadcrumb | below verified | No physical input event was exercised. |
| `UI-001` | hardware-smoke breadcrumb | below verified | Startup SSD1306 text was observed, not runtime UI behavior. |
| `UI-002` | hardware-smoke breadcrumb | below verified | Runtime screen flow was not exercised. |
| `UI-003` | hardware-smoke breadcrumb | below verified | Display/input administration behavior was not exercised. |
