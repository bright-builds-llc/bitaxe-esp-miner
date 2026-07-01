# Phase 14 Display Input Evidence

## Scope

This component pack covers Ultra 205 board `205` display and input checklist
rows `IO-001`, `UI-001`, `UI-002`, and `UI-003`. It records fresh current-commit
startup SSD1306 and runtime-gap serial markers. It does not verify runtime
display pages, screen flow, LVGL parity, physical input behavior, or runtime
display/input administration behavior.

## Metadata

| Field | Value |
| --- | --- |
| Board | `205` |
| Selected port | `/dev/cu.usbmodem1101` |
| Detector command | `just detect-ultra205` |
| Board-info command | `espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` |
| Board-info status | passed |
| Source commit | `8d39c6b3379070a0b549acf9c282b5696c5c3cef` |
| Reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| Package manifest | `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` |
| Serial capture command | `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load/current-serial capture-timeout-seconds=25` |
| Serial artifact | `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load/current-serial/flash-monitor.log` |
| Serial command evidence | `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load/current-serial/flash-command-evidence.json` |
| Allow manifest | `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/display-input/allow-display-input.json` |
| Wrapper command | `scripts/phase14-display-input.sh --manifest docs/parity/evidence/phase-14-safety-hardware-evidence-completion/display-input/allow-display-input.json --out-dir docs/parity/evidence/phase-14-safety-hardware-evidence-completion/display-input --serial-log docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load/current-serial/flash-monitor.log` |
| Raw wrapper artifact | `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/display-input/display-input.log` |
| Redaction review | pending in `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/redaction-review.md` |

## Observed Status

The serial capture evidence reports trusted current-firmware output:

- `firmware_commit`: `8d39c6b3379070a0b549acf9c282b5696c5c3cef`
- `observed_firmware_commit`: `8d39c6b33790`
- `observed_reference_commit`: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- `capture_status`: `timed_out_after_trusted_output`

The allow manifest passed through `tools/parity safety-allow` for surface
`display-input` and claim tier `read-only-observation`.

`display-input.log` records:

- `display_status=startup_text_rendered model=SSD1306 size=128x32 address=0x3c`
- `startup_display_status: observed`
- `display_input_status=runtime_gap reason=hardware_evidence_pending startup_only=true`
- `runtime_gap_marker_status: observed`
- `runtime_display_input_status: pending - no runtime display/input route or physical input observation`
- `phase14_display_input_status: pending - runtime display/input route unavailable`

## Conclusion

`IO-001`, `UI-001`, `UI-002`, and `UI-003` remain below verified for runtime
display/input parity. The fresh serial evidence observes startup-only SSD1306
text rendering and an explicit runtime display/input gap marker.

Startup-only SSD1306 evidence cannot verify runtime display pages, screen flow,
LVGL parity, button routing, input hardware behavior, self-test display behavior,
or runtime display/input safety pages.

`runtime_display_input_status: pending - no runtime display/input route or physical input observation`

Non-claims: this evidence does not verify physical input events, runtime UI
navigation, live display page updates, API/WebSocket administration during
runtime display work, or any temporary production route.
