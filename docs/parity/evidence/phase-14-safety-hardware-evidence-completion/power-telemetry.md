# Phase 14 Power Telemetry Evidence

## Scope

This component pack covers Ultra 205 board `205` power/current telemetry for
checklist row `PWR-006`. It records the detector and allow-manifest gate for a
read-only observation attempt. It does not verify DS4432U voltage writes, ASIC
power sequencing, or stale cached values.

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

The allow manifest passed through `tools/parity safety-allow` for surface
`power-telemetry` and claim tier `read-only-observation`.

`power-voltage.log` records:

- `phase14_power_voltage_status: pending - serial log unavailable`
- `phase14_power_telemetry_status: pending - serial log unavailable`
- `power_telemetry_status: pending - hardware_evidence_pending`
- `power_telemetry_claim: read-only-observation`
- `voltage_control_status: pending - no production-safe bounded voltage route exists`

No fresh INA260 current, bus-voltage, power, freshness, or read-status artifact
was captured in this task.

## Conclusion

`PWR-006` conclusion: below verified for fresh read-only INA260 telemetry. The
Phase 14 wrapper and allow manifest are present, but the actual telemetry value
remains `power_telemetry_status: pending - hardware_evidence_pending` because no
fresh serial/API route was supplied for this run.

Non-claims: this evidence does not verify DS4432U voltage writes, ASIC power
sequencing, active power initialization, or stale cached values. Active power
behavior still requires `hardware-regression` evidence.
