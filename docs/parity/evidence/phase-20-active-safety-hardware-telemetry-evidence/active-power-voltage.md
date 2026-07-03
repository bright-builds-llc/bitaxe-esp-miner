# Phase 20 Active Power And Voltage Evidence

power_telemetry_status: pending - hardware_evidence_pending
voltage_control_status: pending - no production-safe bounded voltage route exists
evidence_class: hardware-smoke for read-only power telemetry attempt; deferred for unsupported voltage-control attempt
board: 205
selected_port: /dev/cu.usbmodem1101
source_commit: c11fba2622a389af533774447956b95f254c0280
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_manifest: docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/bitaxe-ultra205-package.json
safe_baseline_serial_log: docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/flash-monitor.log

## Scope

This pack records Phase 20 power/current/voltage telemetry and voltage-control
claim boundaries for Ultra 205 board `205`. It reuses the existing Phase 14
allow-gated power/voltage wrapper and does not introduce any DS4432U actuation,
raw I2C command, voltage setpoint change, ASIC power sequencing, load, or
hardware-burn route.

## Evidence Runs

| Surface | Claim tier | Evidence class | Manifest | Log | Affected rows |
| --- | --- | --- | --- | --- | --- |
| `power-telemetry` | `read-only-observation` | `hardware-smoke` | `active-power-voltage/power-telemetry/allow-power-telemetry.json` | `active-power-voltage/power-telemetry/power-voltage.log` | `PWR-006` |
| `voltage-control` | `unsupported-pending` | `deferred` | `active-power-voltage/voltage-control/allow-voltage-control.json` | `active-power-voltage/voltage-control/power-voltage.log` | `PWR-003`, `PWR-005` |

## Observed Status

The `power-telemetry` manifest passed safety-allow validation with
`claim_tier: read-only-observation`, `evidence_class: hardware-smoke`, and
`checklist_rows: PWR-006`.

`active-power-voltage/power-telemetry/power-voltage.log` records:

- `phase14_power_voltage_status: pending - read-only route unavailable`
- `power_telemetry_status: pending - hardware_evidence_pending`
- `power_telemetry_claim: read-only-observation`
- `PWR-006 conclusion: read-only observation remains pending without fresh INA260 artifact`

The `voltage-control` manifest passed safety-allow validation with
`claim_tier: unsupported-pending`, `evidence_class: deferred`, and
`checklist_rows: PWR-003,PWR-005`.

`active-power-voltage/voltage-control/power-voltage.log` records:

- `phase14_power_voltage_status: pending - bounded voltage route unavailable`
- `voltage_control_claim: unsupported-pending`
- `voltage_control_status: pending - no production-safe bounded voltage route exists`
- `PWR-003 conclusion: below verified until hardware-regression exists`
- `PWR-005 conclusion: below verified until hardware-regression exists`

## Claim Boundaries

`PWR-006` remains below verified for fresh INA260 current, bus-voltage, power,
freshness, and read-status telemetry. The Phase 20 artifact proves only that the
read-only surface was allow-gated against the detector-gated safe-baseline
package and that the wrapper preserved the pending telemetry boundary.

`PWR-003` and `PWR-005` remain below verified for DS4432U voltage actuation,
setpoint effects, active power sequencing, and unsafe-voltage recovery. The
Phase 20 artifact is blocked/deferred evidence because no production-safe
bounded DS4432U route exists.

## Rationale

- D-05: read-only observations may support only exact observed telemetry
  subclaims. DS4432U actuation, voltage setpoint changes, ASIC power sequencing,
  and stale cached values require hardware-regression evidence.
- D-06: when a production-safe active voltage route does not exist, the correct
  output is observe-only or blocked evidence rather than an unsafe workaround.
- D-13: `hardware-smoke` may support only narrow read-only evidence. Active
  voltage-control rows require `hardware-regression`, so they remain below
  verified.

## Non-Claims

non_claims:

- No DS4432U active voltage actuation ran.
- No raw I2C, voltage setpoint, or register-poke command ran.
- No flash-erase, hardware-burn, load, ASIC power sequencing, or mining command ran.
- No fresh INA260 telemetry value, freshness marker, or API/WebSocket
  correlation was captured by this pack.
- No active voltage-control, over-voltage, under-voltage, recovery, or
  closed-loop power behavior is verified by this pack.
