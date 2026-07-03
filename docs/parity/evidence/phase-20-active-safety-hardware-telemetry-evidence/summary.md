# Phase 20 Active Safety Hardware Telemetry Evidence Summary

## Status

phase20_status: complete
redaction_status: passed
checklist_status: ready-for-conservative-citation
phase: 20-active-safety-hardware-telemetry-evidence
source: .planning/phases/20-active-safety-hardware-telemetry-evidence/20-06-PLAN.md
evidence_root: docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence

## Purpose

This ledger closes Phase 20 with exact claims only. It records what the
detector-gated Ultra 205 evidence supports, what remains below verified, and
which artifacts may be cited after redaction review.

Phase 20 proves package/release-gate identity and a detector-gated safe-baseline
hardware smoke run on board `205`. It also records allow-gated evidence
boundaries for active power/voltage, thermal/fan, self-test/watchdog/load,
runtime display/input, failure paths, and live API/WebSocket telemetry. No
active safety-control, fault-stimulus, runtime display/input, self-test hardware
submode, bounded load, or live telemetry freshness claim is promoted beyond its
actual evidence class.

## Final Evidence Matrix

| Pack | status | evidence_class | artifact | checklist_rows | supported_subclaims | below_verified_subclaims | non_claims |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `safe-baseline` | passed | hardware-smoke | `safe-baseline.md`; `safe-baseline/detect-ultra205.log`; `safe-baseline/flash-command-evidence.json`; `safe-baseline/flash-monitor.log`; `target-lock.json` | `SAFE-01`, `SAFE-02`, `SAFE-07`, `SAFE-08`, `SAFE-09`, `EVD-05` | Detector found exactly one likely ESP32-S3 USB port; board-info passed; board `205` package flashed and monitored; package and reference commits matched; committed serial output shows `safe_state: mining=disabled`, `asic_work_submission=disabled`, and `hardware_control=disabled`. | Target origin remains blocked because `target-lock.json` has `network_scan: disabled` and no trusted raw origin-only target artifact. Live API/WebSocket projection is not proven by serial output. | No voltage, fan, thermal, self-test, load, runtime input/display, failure-path, mining, erase, rollback, OTA, or fault-injection command ran. |
| `active-power-voltage` | complete-boundary | hardware-smoke for read-only wrapper attempt; deferred for unsupported voltage-control | `active-power-voltage.md`; `active-power-voltage/power-telemetry/allow-power-telemetry.json`; `active-power-voltage/power-telemetry/power-voltage.log`; `active-power-voltage/voltage-control/allow-voltage-control.json`; `active-power-voltage/voltage-control/power-voltage.log` | `PWR-003`, `PWR-005`, `PWR-006`, `SAFE-01`, `SAFE-07` | The read-only power telemetry surface was allow-gated against the Phase 20 detector/package chain and preserved the pending telemetry boundary for `PWR-006`. | Fresh INA260 current, bus-voltage, power, freshness, read-status telemetry, DS4432U actuation, voltage setpoint effects, active power sequencing, and unsafe-voltage recovery remain below verified. | No DS4432U active voltage actuation, raw I2C, voltage setpoint, register poke, load, ASIC power sequencing, mining, or API/WebSocket correlation ran. |
| `active-thermal-fan` | complete-boundary | hardware-smoke for read-only wrapper attempt; deferred for unsupported fan-duty/fault behavior | `active-thermal-fan.md`; `active-thermal-fan/thermal-read/allow-thermal-fan-read.json`; `active-thermal-fan/thermal-read/thermal-fan.log`; `active-thermal-fan/fan-duty/allow-fan-duty-blocked.json`; `active-thermal-fan/fan-duty/thermal-fan.log` | `THR-001`, `THR-002`, `THR-003`, `SAFE-02`, `SAFE-03`, `SAFE-07` | The thermal/fan surface was allow-gated against the Phase 20 detector/package chain; pure PID coverage remains unit evidence. | Fresh EMC2101 thermal readings, fan RPM observations, physical fan response, fan duty effects, overheat stimulus, thermal fault handling, fan fault behavior, safe stop, cool/restart behavior, and API/WebSocket fault projection remain below verified. | No fan duty actuation, physical fan response proof, overheat stimulus, fan-fault stimulus, thermal-fault stimulus, fault injection, or telemetry projection ran. |
| `self-test-watchdog-load` | complete-boundary | deferred with startup watchdog breadcrumb | `self-test-watchdog-load.md`; `self-test-watchdog-load/allow-self-test-watchdog-load.json`; `self-test-watchdog-load/self-test-watchdog-load.log` | `SELF-001`, `SAFE-05`, `SAFE-09` | The safe-baseline serial log records watchdog supervisor startup and yield breadcrumbs. | Self-test hardware submodes, pass/fail/cancel behavior, restart behavior, factory-flag behavior, production-mining gate behavior, bounded load, blocked-task behavior, watchdog intervention recovery, API/WebSocket responsiveness under load, and post-action safe state remain below verified. | No self-test hardware submode, mining work, ASIC diagnostic work, reboot test, voltage/fan/ASIC control, watchdog reset, bounded load, or active fault path ran. |
| `runtime-display-input` | complete-boundary | hardware-smoke breadcrumb for startup display only | `runtime-display-input.md`; `runtime-display-input/allow-display-input.json`; `runtime-display-input/display-input.log` | `IO-001`, `UI-001`, `UI-002`, `UI-003`, `SAFE-06` | Startup SSD1306 text marker and runtime gap marker were observed from detector-gated safe-baseline serial evidence. | Runtime display pages, screen flow, LVGL parity, physical input events, button routing, self-test display flow, and API/WebSocket administration during display/input activity remain below verified. | No runtime display route, physical button/input action, LVGL screen carousel, runtime UI navigation, or temporary production route ran. |
| `failure-paths` | complete-boundary | deferred | `failure-paths.md`; `failure-paths/allow-failure-paths.json`; `failure-paths/failure-paths.log` | `PWR-001`, `PWR-002`, `THR-001`, `THR-002`, `SELF-001`, `SAFE-04` | The Phase 20 `failure-paths` allow surface and wrapper record a blocked, no-stimulus evidence boundary with missing stimulus, expected fault, abort, restore, projection, and final safe-state fields named explicitly. | Overheat safe stop, fan fault, power fault, thermal sensor fault, ASIC fault, self-test failure, API/log/WebSocket projection, recovery paths, and final safe-state markers after fault stimulus remain below verified. | No fault injection, overheat stimulus, fan zero-RPM/fan-set-failed stimulus, power fault stimulus, thermal sensor fault stimulus, ASIC fault stimulus, self-test failure stimulus, projection, or recovery ran. |
| `live-api-websocket-telemetry` | blocked-boundary | deferred | `live-api-websocket-telemetry.md`; `live-api-websocket-telemetry/allow-live-telemetry.json`; `live-api-websocket-telemetry/live-telemetry.log`; `live-api-websocket-telemetry/websocket/api-ws-live.txt` | `API-002`, `API-006`, `STAT-002`, `PWR-006`, `THR-001`, `THR-002`, `SAFE-07` | The allow-gated live telemetry helpers recorded explicit missing-target evidence with `network_scan: disabled`, `/api/ws/live`, `duration_ms: 10000`, and `max_frames: 5`. | `/api/system/info` body, safety telemetry fields, WebSocket frames, freshness, cadence, correlation with live hardware readings, and post-capture safe-state evidence remain below verified. | No `curl`, socket connection, network scan, ARP/mDNS discovery, inferred target, production mining, active control behavior, fault recovery, or soak evidence ran. |
| `parity-redaction` | passed | workflow | `redaction-review.md` | `SAFE-08`, `EVD-05` | Final redaction review passed for committed Phase 20 serial logs, JSON manifests, API/WebSocket artifacts, detector/board-info output, package logs, command output, and manual-observation notes. `raw_artifacts_committed: no`. | Raw origin-only target evidence remains unavailable; live-device target information remains blocked and uncited. | No raw `DEVICE_URL`, IP address, MAC address, SSID, Wi-Fi credential, pool credential, worker secret, API token, NVS secret value, or local terminal secret is committed. |

## Requirement Closure

| Requirement | Phase 20 outcome |
| --- | --- |
| `SAFE-01` | Package/safe-baseline evidence and active power/voltage boundaries are complete. Active voltage and unsafe-power recovery remain below verified without hardware-regression evidence. |
| `SAFE-02` | Thermal/fan boundaries are complete. Fresh thermal/RPM and active fan/fault behavior remain below verified without hardware-regression evidence. |
| `SAFE-03` | PID/thermal-control remains supported by pure unit evidence only; no active fan behavior was promoted. |
| `SAFE-04` | Failure-path evidence is closed as blocked/deferred with all missing fault-stimulus fields named; no fault-path row is verified. |
| `SAFE-05` | Self-test lifecycle hardware submodes remain below verified; watchdog startup/yield breadcrumbs are recorded only as breadcrumbs. |
| `SAFE-06` | Startup display breadcrumbs are recorded; runtime display/input remains below verified. |
| `SAFE-07` | Safe-baseline hardware markers exist, but fresh live power/fan/temperature API/WebSocket telemetry remains blocked by missing target evidence. |
| `SAFE-08` | Checklist-ready evidence preserves the safety-critical no-overclaim rule and final redaction review passed. |
| `SAFE-09` | Watchdog supervisor startup/yield breadcrumbs are recorded; bounded load and recovery remain below verified. |
| `EVD-05` | Phase 20 includes unit/workflow tests, detector-gated hardware smoke, blocked evidence where prerequisites are absent, redaction review, parity validation, lifecycle validation, and reference cleanliness. |

## Deferred Boundaries

These remain explicit non-claims after Phase 20:

- Live production mining, accepted/rejected shares, live pool behavior, and soak evidence.
- Firmware OTA, OTAWWW, rollback, failed-update recovery, large erase, interrupted update, and recovery-regression behavior.
- Non-205 boards and any inheritance of Ultra 205 evidence by other board targets.
- Stratum v2, BAP, all-board release images, and performance tuning.
- Full LVGL runtime display parity, display carousel/configuration, and broad button-routing parity.
- Active voltage control, fan duty effects, overheat/fault stimulus, self-test hardware submodes, bounded load, watchdog recovery, and live telemetry freshness until a future plan supplies the required evidence class.

## Citation Rules

- Cite this summary only after `redaction-review.md` reports `redaction_status: passed`.
- Promote a row to `verified` only when the exact row has matching
  `hardware-smoke` or `hardware-regression` evidence.
- Active safety-control and failure-path rows require `hardware-regression`.
- Blocked, deferred, safe-unavailable, startup-only, and read-only wrapper
  evidence must stay below verified unless a later artifact proves the exact
  missing subclaim.
