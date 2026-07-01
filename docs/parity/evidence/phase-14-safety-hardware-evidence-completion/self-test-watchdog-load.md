# Phase 14 Self-Test Watchdog Load Evidence

## Scope

This component pack covers Ultra 205 board `205` self-test, watchdog, and load
evidence for checklist row `SELF-001` and the Phase 14 watchdog/load safety
surface. It records fresh current-commit safe-boot serial markers for the
watchdog supervisor only. It does not verify self-test hardware submodes,
bounded workload stress, reboot behavior, mining behavior, ASIC diagnostic
work, or voltage/fan/ASIC interactions.

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
| Allow manifest | `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load/allow-self-test-watchdog-load.json` |
| Wrapper command | `scripts/phase14-self-test-watchdog-load.sh --manifest docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load/allow-self-test-watchdog-load.json --out-dir docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load --serial-log docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load/current-serial/flash-monitor.log` |
| Raw wrapper artifact | `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load/self-test-watchdog-load.log` |
| Redaction review | pending in `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/redaction-review.md` |

## Observed Status

The serial capture evidence reports trusted current-firmware output:

- `firmware_commit`: `8d39c6b3379070a0b549acf9c282b5696c5c3cef`
- `observed_firmware_commit`: `8d39c6b33790`
- `observed_reference_commit`: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- `capture_status`: `timed_out_after_trusted_output`

The allow manifest passed through `tools/parity safety-allow` for surface
`self-test-watchdog-load` and claim tier `read-only-observation`.

`self-test-watchdog-load.log` records:

- `watchdog_supervisor_status: observed`
- `watchdog_supervisor_start_marker: observed - safety_supervisor=started thread=bitaxe-safety-supervisor cadence_ms=100`
- `watchdog_supervisor_yield_marker: observed - safety_supervisor_step=yield reason=yield_interval_reached`
- `load_stress_status: pending - bounded workload stimulus unavailable`
- `self_test_hardware_status: pending - no production-safe self-test hardware submode route exists`
- `phase14_self_test_watchdog_load_status: pending - self-test hardware and bounded load routes unavailable`

## Conclusion

`SELF-001` remains below verified for self-test hardware submodes. The fresh
serial evidence observes watchdog supervisor startup and yield markers only.

`watchdog_supervisor_status: observed` supports the narrow supervisor-start and
yield-marker subclaim. It does not prove bounded load responsiveness.

`load_stress_status: pending - bounded workload stimulus unavailable`

`self_test_hardware_status: pending - no production-safe self-test hardware submode route exists`

Non-claims: this evidence does not verify self-test fan checks, power checks,
diagnostic ASIC work, fake Stratum work, pass/fail/cancel behavior, restart
behavior, production-mining gate transitions beyond the safe boot marker, or any
voltage/fan/ASIC hardware-control behavior. Those claims require
`hardware-regression` evidence with recovery steps and abort conditions.
