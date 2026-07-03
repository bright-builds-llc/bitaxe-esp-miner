---
phase: 20
plan: "06"
slug: active-safety-hardware-telemetry-evidence
status: passed
verified_at: 2026-07-03T23:33:15Z
redaction_status: passed
reference_clean: passed
key_links: valid
lifecycle_validation: valid
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 20-2026-07-03T20-48-00
generated_at: 2026-07-03T23:33:15Z
lifecycle_validated: true
hardware_commands_used: recorded
network_commands_used: recorded
---

# Phase 20 Final Verification

Phase 20 is closed as an evidence-governance pass. The final evidence pack is redacted, checklist citations are conservative, requirements traceability includes the final evidence note, and no active safety behavior is promoted above its evidence class.

## Required Command Results

| Command | Result | Evidence |
|---------|--------|----------|
| `bazel test //scripts:phase14_power_voltage_test //scripts:phase14_thermal_fan_test //scripts:phase14_self_test_watchdog_load_test //scripts:phase14_display_input_test //scripts:phase14_live_telemetry_test //scripts:phase20_failure_paths_test` | passed | Six scoped script tests passed. |
| `cargo test -p bitaxe-parity --all-features safety_allow` | passed | 13 safety-allow tests passed. |
| `cargo test -p bitaxe-safety --all-features` | passed | 32 unit tests and doc tests passed. |
| `node scripts/phase17-websocket-capture.mjs --help` | passed | CLI usage printed successfully. |
| `just test` | passed | `bazel test //...` passed with 30 tests and firmware package build. |
| `just parity` | passed | Parity report ended with `validation_errors: none`. |
| `just verify-reference` | passed | Reference clean at `c1915b0a63bfabebdb95a515cedfee05146c1d50`. |
| `git diff -- reference/esp-miner --exit-code` | passed | No reference tree diff. |
| `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 20 --expect-id 20-2026-07-03T20-48-00 --expect-mode yolo --raw` | passed | Returned `valid`. |
| `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify key-links .planning/phases/20-active-safety-hardware-telemetry-evidence/20-06-PLAN.md --raw` | passed | Returned `valid`. |

## Redaction

`redaction_status: passed`

The final broad redaction scan found only allowlisted policy text, redacted placeholders, package/tool version strings, and documented blocked-target language. The stricter private IP and MAC value scan returned no matches across the Phase 20 evidence pack, checklist, requirements note, and validation file. No raw `DEVICE_URL`, IP address, MAC address, SSID, Wi-Fi credential, pool credential, worker secret, API token, NVS secret value, or local terminal secret is committed.

## hardware_commands_used

These hardware-facing commands were used by the Phase 20 evidence pack:

| Command | Status | Artifact |
|---------|--------|----------|
| `just detect-ultra205` | passed | `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/detect-ultra205.log` |
| `espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` | passed through detector gate | `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/detect-ultra205.log` |
| `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline capture-timeout-seconds=45 redact-evidence=true` | passed with credential file supplied by path only | `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/flash-command-evidence.json` and `safe-baseline/flash-monitor.log` |
| `scripts/phase14-power-voltage.sh` wrapper paths | passed or blocked according to allow manifest | `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/active-power-voltage.md` |
| `scripts/phase14-thermal-fan.sh` wrapper paths | passed or blocked according to allow manifest | `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/active-thermal-fan.md` |
| `scripts/phase14-self-test-watchdog-load.sh` wrapper path | blocked boundary recorded | `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/self-test-watchdog-load.md` |
| `scripts/phase14-display-input.sh` wrapper path | startup-only boundary recorded | `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/runtime-display-input.md` |
| `scripts/phase20-failure-paths.sh --manifest docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths/allow-failure-paths.json --out-dir docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths` | blocked boundary recorded | `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths.md` |

Safe-baseline hardware evidence passed for board `205`: detector success, board-info success, package flash/monitor success, package/reference identity, and serial safe-state markers for `mining=disabled`, `asic_work_submission=disabled`, and `hardware_control=disabled`.

## network_commands_used

No live network probe, `curl`, socket connection, network scan, ARP discovery, mDNS discovery, or inferred target command was used for Phase 20 closure.

| Command | Status | Artifact |
|---------|--------|----------|
| `scripts/phase14-live-telemetry.sh` without explicit target | blocked boundary recorded | `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/live-api-websocket-telemetry.md` |
| `node scripts/phase17-websocket-capture.mjs --help` | passed CLI contract only | Task 3 terminal verification |

Live API/WebSocket telemetry is blocked, not passed. `target-lock.json` records `network_scan: disabled`, and no trusted raw origin-only target artifact exists. The evidence pack does not infer a device origin from redacted serial logs.

## residual_below_verified

These remain below verified and are not Phase 20 claims:

- Active DS4432U voltage actuation, voltage setpoint effects, unsafe-voltage recovery, and active power sequencing.
- Fresh INA260 current, bus voltage, power, freshness, and API/WebSocket correlation.
- Fresh EMC2101 thermal readings, fan RPM observations, physical fan response, fan duty effects, overheat behavior, thermal fault behavior, and fan fault behavior.
- Self-test hardware submodes, pass/fail/cancel/restart behavior, factory-flag behavior, production-mining gate behavior, bounded load, blocked-task behavior, and watchdog intervention recovery.
- Runtime display pages, LVGL parity, screen carousel behavior, physical input events, and self-test display flow.
- Fault stimulus, API/log/WebSocket projection of faults, recovery paths, and final safe-state markers after fault stimulus.
- Live `/api/system/info` safety telemetry, `/api/ws/live` frames, freshness, cadence, and correlation with hardware readings.
- Production mining, live pool behavior, accepted/rejected shares, live soak, OTA/OTAWWW recovery, rollback, interrupted update recovery, non-205 boards, Stratum v2, BAP, and performance tuning.

## Conclusion

Phase 20 verification passed for exact evidence closure only. The phase is ready for wrapper-level closure because final citations, traceability, lifecycle validation, key-link validation, parity validation, reference cleanliness, and redaction review passed. Active safety controls and live telemetry remain below verified until a future plan supplies the required `hardware-smoke` or `hardware-regression` evidence.
