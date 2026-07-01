# Phase 14 Voltage Control Evidence

## Scope

This component pack covers Ultra 205 board `205` voltage-control rows `PWR-003`
and `PWR-005`. It records that the Phase 14 power/voltage wrapper explicitly
keeps active DS4432U voltage behavior below verified because no
production-safe bounded voltage route exists.

## Metadata

| Field | Value |
| --- | --- |
| Board | `205` |
| Selected port | `/dev/cu.usbmodem1101` |
| Detector command | `just detect-ultra205` |
| Board-info command | `espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` |
| Board-info status | passed |
| Source commit | `ff9da3be6450127dd2cdd92c6d60452b8d475fb8` |
| Reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| Package manifest | `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` |
| Allow manifest | `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/power-telemetry/allow-power-telemetry.json` |
| Exact command | `scripts/phase14-power-voltage.sh --manifest docs/parity/evidence/phase-14-safety-hardware-evidence-completion/power-telemetry/allow-power-telemetry.json --surface power-telemetry --out-dir docs/parity/evidence/phase-14-safety-hardware-evidence-completion/power-telemetry` |
| Raw artifact | `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/power-telemetry/power-voltage.log` |
| Redaction review | pending in `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/redaction-review.md` |

## Observed Status

`power-voltage.log` records:

- `voltage_control_status: pending - no production-safe bounded voltage route exists`
- `PWR-003 conclusion: below verified until hardware-regression exists`
- `PWR-005 conclusion: below verified until hardware-regression exists`

No voltage actuation command, DS4432U write, raw I2C operation, or bounded
hardware-regression voltage procedure ran in this task.

## Conclusion

`PWR-003` and `PWR-005` remain below verified.

`voltage_control_status: pending - no production-safe bounded voltage route exists`

Non-claims: this evidence does not verify active voltage writes, DS4432U
register effects, power sequencing, recovery from unsafe voltage, or ASIC work.
Those claims require a future Phase 14 allow manifest that proves bounded
inputs, abort conditions, recovery steps, post-action safe-state markers, and
`hardware-regression` artifacts.
