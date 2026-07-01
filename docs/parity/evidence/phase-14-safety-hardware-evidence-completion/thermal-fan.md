# Phase 14 Thermal And Fan Evidence

## Scope

This component pack covers Ultra 205 board `205` thermal and fan rows
`THR-001`, `THR-002`, and pure PID row `THR-003`. It records the detector and
allow-manifest gate for a read-only thermal/fan observation attempt. It does not
verify fan duty effects, overheat/fault behavior, or physical fan response.

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
| Allow manifest | `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/thermal-fan/allow-thermal-fan.json` |
| Exact command | `scripts/phase14-thermal-fan.sh --manifest docs/parity/evidence/phase-14-safety-hardware-evidence-completion/thermal-fan/allow-thermal-fan.json --surface thermal-fan --out-dir docs/parity/evidence/phase-14-safety-hardware-evidence-completion/thermal-fan` |
| Raw artifact | `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/thermal-fan/thermal-fan.log` |
| Redaction review | pending in `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/redaction-review.md` |

## Observed Status

The allow manifest passed through `tools/parity safety-allow` for surface
`thermal-fan` and claim tier `read-only-observation`.

`thermal-fan.log` records:

- `phase14_thermal_fan_status: pending - serial log unavailable`
- `thermal_fan_status: pending - serial log unavailable`
- `thermal_claim: read-only-observation`
- `fan_rpm_claim: read-only-observation`
- `fan_duty_status: pending - no production-safe bounded fan-duty route exists`

No fresh EMC2101 thermal reading, fan RPM artifact, overheat stimulus, fan
fault stimulus, or fan duty hardware-regression route was captured in this task.

## Conclusion

`THR-001` and `THR-002` remain below verified for fresh hardware observations.
`THR-003` remains unit evidence only for pure PID logic.

`fan_duty_status: pending - no production-safe bounded fan-duty route exists`

Non-claims: this evidence does not verify fan duty effects, overheat/fault
behavior, fan-fault handling, physical fan response, or stale cached thermal
values. Active fan and failure-path claims require `hardware-regression`
artifacts from a bounded procedure with recovery and safe-state markers.
