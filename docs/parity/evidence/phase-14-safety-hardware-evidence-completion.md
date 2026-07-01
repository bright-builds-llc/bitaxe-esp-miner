# Phase 14 Safety Hardware Evidence Completion

## Scope

Phase 14 closes the safety evidence bookkeeping for Ultra 205 board `205` after
the component evidence packs were generated. It records what the fresh Phase 14
artifacts support, what remains below verified, and which artifacts have passed
redaction review.

This ledger does not promote broad active hardware parity. Active DS4432U
voltage writes, ASIC reset or power sequencing, fan duty effects, overheat and
fault injection, self-test hardware submodes, bounded load stress, runtime
display/input behavior, live HTTP values, and WebSocket frames still require
future bounded hardware-regression or live-route evidence before they can be
claimed as verified.

## Hardware Gates

| Gate | Result | Evidence |
| --- | --- | --- |
| Target board | `205` only | Every generated allow manifest records board `205`. |
| Detector | passed | `just detect-ultra205` found exactly one likely ESP32-S3 serial port, `/dev/cu.usbmodem1101`, before hardware-gated Phase 14 runs. |
| Board info | passed | Each generated component summary records `espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` as passed. |
| Reference commit | pinned | Component summaries cite `c1915b0a63bfabebdb95a515cedfee05146c1d50`. |
| Source commits | recorded | Power/thermal packs cite `ff9da3be6450127dd2cdd92c6d60452b8d475fb8`; self-test/display packs cite `8d39c6b3379070a0b549acf9c282b5696c5c3cef`; live telemetry cites `ef580c71a178c3101385a476e6964f5af80da575`. |
| Redaction | passed for generated Phase 14 artifacts | `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/redaction-review.md` records reviewed artifacts, absent API bodies/WebSocket frames, and retained bench identifiers. |

## Allow-Manifest Status

| Pack | Manifest | Surface | Claim tier | Status |
| --- | --- | --- | --- | --- |
| `power-telemetry` | `power-telemetry/allow-power-telemetry.json` | `power-telemetry` | `read-only-observation` | passed; no fresh INA260 value was captured. |
| `voltage-control` | shared through `power-telemetry/allow-power-telemetry.json` | `power-telemetry` wrapper records voltage-control blocker | `read-only-observation` | blocked for active voltage; no production-safe bounded voltage route exists. |
| `thermal-fan` | `thermal-fan/allow-thermal-fan.json` | `thermal-fan` | `read-only-observation` | passed; no fresh EMC2101/fan RPM value or fan duty effect was captured. |
| `self-test-watchdog-load` | `self-test-watchdog-load/allow-self-test-watchdog-load.json` | `self-test-watchdog-load` | `read-only-observation` | passed for watchdog startup/yield marker parsing; self-test hardware and bounded load routes remain unavailable. |
| `display-input` | `display-input/allow-display-input.json` | `display-input` | `read-only-observation` | passed for startup display/runtime-gap marker parsing; runtime display/input observation remains unavailable. |
| `live-api-websocket-telemetry` | `live-api-websocket-telemetry/allow-live-telemetry.json` | `live-api-websocket-telemetry` | `api-websocket-projection` | blocked because `DEVICE_URL` was missing and no maintained WebSocket client was available. |

## Evidence Matrix

| Pack | Primary artifacts | Phase 14 result | Checklist impact |
| --- | --- | --- | --- |
| `safe-baseline` | none generated in Phase 14 | below verified - no dedicated Phase 14 safe-baseline artifact exists | Do not cite as fresh Phase 14 proof. |
| `power-telemetry` | `power-telemetry.md`, `power-telemetry/power-voltage.log`, `power-telemetry/allow-power-telemetry.json` | below verified - no fresh serial/API INA260 telemetry route was supplied | `PWR-006` remains below verified for live current, voltage, power, freshness, and read status. |
| `voltage-control` | `voltage-control.md`, shared `power-telemetry/power-voltage.log` | below verified - no production-safe bounded voltage route exists | `PWR-003` and `PWR-005` require hardware-regression before verified. |
| `thermal-fan` | `thermal-fan.md`, `thermal-fan/thermal-fan.log`, `thermal-fan/allow-thermal-fan.json` | below verified - no fresh EMC2101 reading, fan RPM artifact, fan duty route, or fault stimulus exists | `THR-001` and `THR-002` remain below verified; `THR-003` remains unit evidence only. |
| `self-test-watchdog-load` | `self-test-watchdog-load.md`, `self-test-watchdog-load/self-test-watchdog-load.log`, `self-test-watchdog-load/current-serial/flash-monitor.log`, `self-test-watchdog-load/current-serial/flash-command-evidence.json` | verified by hardware-smoke for exact read-only/safe-unavailable subclaim - watchdog supervisor start/yield markers observed; below verified - self-test hardware and bounded load routes unavailable | `SELF-001` stays below verified for self-test hardware; watchdog/load broad parity remains below verified. |
| `display-input` | `display-input.md`, `display-input/display-input.log`, shared current serial artifacts | verified by hardware-smoke for exact read-only/safe-unavailable subclaim - startup SSD1306 and runtime-gap markers observed; below verified - no runtime display/input route or physical input observation | `IO-001`, `UI-001`, `UI-002`, and `UI-003` stay below verified for runtime parity. |
| `live-api-websocket-telemetry` | `live-api-websocket-telemetry.md`, `live-api-websocket-telemetry/live-telemetry.log`, `live-api-websocket-telemetry/allow-live-telemetry.json` | below verified - missing explicit reachable `DEVICE_URL` and maintained WebSocket client | `API-002`, `API-006`, `STAT-002`, `PWR-006`, `THR-001`, and `THR-002` live-route claims remain below verified. |

## Redaction Review

The generated Phase 14 artifact scan found no committed Wi-Fi credentials, pool
credentials, private endpoint values, API tokens, or secret NVS values. Expected
non-secret matches were retained and documented: partition labels such as `WiFi
data`, `ESP_ERR_NVS_NOT_FOUND`, `/dev/cu.usbmodem1101`, the board MAC address
inside board-info evidence, and the missing/sanitized `DEVICE_URL` marker.

No API response bodies or WebSocket frames were captured because the live
telemetry pack was blocked before any network request. The redaction review
therefore clears the generated Phase 14 artifacts for citation while preserving
the live-route claims as below verified.

## Exact Claims Supported

| Row | Phase 14 claim status |
| --- | --- |
| `PWR-001` | below verified - active reset and ASIC sequencing require bounded hardware-regression evidence. |
| `PWR-002` | below verified - active ASIC power initialization requires bounded hardware-regression evidence. |
| `PWR-003` | below verified - no production-safe bounded voltage route exists. |
| `PWR-005` | below verified - no production-safe bounded voltage route exists. |
| `PWR-006` | below verified - no fresh serial/API INA260 telemetry route was supplied. |
| `THR-001` | below verified - no fresh EMC2101 thermal reading, overheat stimulus, or thermal fault artifact exists. |
| `THR-002` | below verified - no fresh fan RPM artifact, fan duty route, fan fault stimulus, or physical fan response exists. |
| `THR-003` | below verified - pure PID remains unit evidenced; live fan hardware behavior was not exercised. |
| `IO-001` | verified by hardware-smoke for exact read-only/safe-unavailable subclaim - startup SSD1306/I2C display marker observed; below verified - shared safety peripheral I2C behavior was not read live. |
| `UI-001` | verified by hardware-smoke for exact read-only/safe-unavailable subclaim - startup display text marker observed; below verified - runtime display parity was not observed. |
| `UI-002` | verified by hardware-smoke for exact read-only/safe-unavailable subclaim - startup screen marker and runtime-gap marker observed; below verified - runtime screen flow was not observed. |
| `UI-003` | verified by hardware-smoke for exact read-only/safe-unavailable subclaim - explicit runtime input/display gap marker observed; below verified - physical input behavior was not observed. |
| `SELF-001` | verified by hardware-smoke for exact read-only/safe-unavailable subclaim - watchdog supervisor start/yield markers observed; below verified - no production-safe self-test hardware submode route exists. |
| `API-002` | below verified - `/api/system/info` was not queried because `DEVICE_URL` was missing. |
| `API-006` | below verified - no WebSocket frames were captured because `DEVICE_URL` was missing and no maintained WebSocket client was available. |
| `STAT-002` | below verified - no live statistics producer sample was captured because `DEVICE_URL` was missing. |
| `SAFE-01` | below verified - active voltage and power-control hardware effects require bounded hardware-regression evidence; Phase 14 only records fail-closed evidence governance and pending blockers. |
| `SAFE-02` | below verified - live thermal sensor, fan RPM, fan duty, and fan fault behavior require fresh hardware evidence. |
| `SAFE-03` | verified by hardware-smoke for exact read-only/safe-unavailable subclaim - Phase 14 preserved the pure PID/unit-test boundary and did not enable hardware effects. |
| `SAFE-04` | below verified - overheat, fan, power, thermal, and ASIC fault paths require documented stimulus, abort conditions, recovery, and hardware-regression evidence. |
| `SAFE-05` | below verified - self-test hardware submodes require a production-safe route and hardware-regression evidence. |
| `SAFE-06` | verified by hardware-smoke for exact read-only/safe-unavailable subclaim - startup display and runtime-gap markers were observed; below verified - runtime display/input administration behavior remains unobserved. |
| `SAFE-07` | below verified - power/current/voltage/fan/temperature telemetry freshness was not captured through a live route in Phase 14. |
| `SAFE-08` | verified by hardware-smoke for exact read-only/safe-unavailable subclaim - generated allow manifests and parity checks keep active safety-control rows from being promoted without `hardware-regression`. |
| `SAFE-09` | verified by hardware-smoke for exact read-only/safe-unavailable subclaim - watchdog supervisor start/yield markers were observed; below verified - bounded load and live telemetry responsiveness were not exercised. |
| `EVD-05` | verified by hardware-smoke for exact read-only/safe-unavailable subclaim - Phase 14 generated typed allow manifests, component evidence packs, redaction review, checklist citations, and final verification gates; below verified - missing live-route and active-control evidence remains explicitly blocked. |

## Residual Risks

- Active voltage, fan duty, fault injection, self-test hardware, and load stress
  remain intentionally unrun because no bounded Phase 14 recovery path exists.
- Live API and WebSocket telemetry remain below verified until an explicit,
  reachable `DEVICE_URL` and maintained WebSocket client are available.
- The fresh serial capture supports watchdog and display startup/gap subclaims
  only; it does not prove runtime display pages, input hardware, mining loop
  behavior, statistics cadence, or sensor freshness.
- Bench identifiers such as USB port and MAC address are retained only because
  they are required hardware evidence fields.

## Final Verification

Final command results are recorded in
`.planning/phases/14-safety-hardware-evidence-completion/14-VALIDATION.md` and
`.planning/phases/14-safety-hardware-evidence-completion/14-VERIFICATION.md`.
The final Plan 14-06 verification gate passed:

- `bash -n scripts/phase14-*.sh`
- `bazel test //scripts:phase14_power_voltage_test //scripts:phase14_thermal_fan_test //scripts:phase14_self_test_watchdog_load_test //scripts:phase14_display_input_test //scripts:phase14_live_telemetry_test`
- `cargo test -p bitaxe-safety --all-features power`
- `cargo test -p bitaxe-safety --all-features thermal`
- `cargo test -p bitaxe-safety --all-features self_test`
- `cargo test -p bitaxe-safety --all-features watchdog`
- `cargo test -p bitaxe-parity --all-features safety_allow`
- `just parity`
- `just test`
- `just verify-reference`
- `git diff -- reference/esp-miner --exit-code`
- lifecycle validation with expected id `14-2026-06-30T23-56-34` and mode
  `yolo`, which returned `valid`.
