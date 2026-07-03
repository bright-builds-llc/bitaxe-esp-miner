# Phase 20 Active Thermal And Fan Evidence

thermal_fan_status: pending - no fresh EMC2101 or fan RPM route observed
thermal_read_status: pending - no fresh EMC2101 thermal artifact
fan_rpm_status: pending - no fresh fan RPM artifact
fan_duty_status: pending - no production-safe bounded fan-duty route exists
pid_unit_status: covered by pure unit evidence only
overheat_fault_status: pending - no bounded overheat or fault stimulus route ran
evidence_class: hardware-smoke for read-only thermal/fan attempt; deferred for unsupported fan-duty attempt
board: 205
selected_port: /dev/cu.usbmodem1101
source_commit: c11fba2622a389af533774447956b95f254c0280
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_manifest: docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/bitaxe-ultra205-package.json
safe_baseline_serial_log: docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/flash-monitor.log

## Scope

This pack records Phase 20 thermal/fan read-only telemetry, fan RPM, PID unit,
fan duty, and overheat/fault claim boundaries for Ultra 205 board `205`. It
reuses the existing Phase 14 allow-gated thermal/fan wrapper and does not add
any fan duty actuation, overheat stimulus, fan-fault stimulus, or fault
injection route.

## Evidence Runs

| Surface | Claim tier | Evidence class | Manifest | Log | Affected rows |
| --- | --- | --- | --- | --- | --- |
| `thermal-fan` | `read-only-observation` | `hardware-smoke` | `active-thermal-fan/thermal-read/allow-thermal-fan-read.json` | `active-thermal-fan/thermal-read/thermal-fan.log` | `THR-001`, `THR-002`, `THR-003` |
| `thermal-fan` | `unsupported-pending` | `deferred` | `active-thermal-fan/fan-duty/allow-fan-duty-blocked.json` | `active-thermal-fan/fan-duty/thermal-fan.log` | `THR-001`, `THR-002`, `THR-003` |

## Observed Status

The `thermal-read` manifest passed safety-allow validation with
`claim_tier: read-only-observation`, `evidence_class: hardware-smoke`, and
`checklist_rows: THR-001,THR-002,THR-003`.

`active-thermal-fan/thermal-read/thermal-fan.log` records:

- `thermal_claim: read-only-observation`
- `fan_rpm_claim: read-only-observation`
- `phase14_thermal_fan_status: pending - no fresh EMC2101 or fan RPM route observed`
- `thermal_fan_status: pending - no fresh EMC2101 or fan RPM route observed`
- `THR-001 conclusion: read-only thermal observation remains pending without fresh artifact`
- `THR-002 conclusion: fan RPM observation remains pending without fresh artifact`
- `THR-003 conclusion: pure PID coverage remains unit evidence only`
- `fan_duty_status: pending - no production-safe bounded fan-duty route exists`

The `fan-duty` manifest passed safety-allow validation with
`claim_tier: unsupported-pending`, `evidence_class: deferred`, and
`checklist_rows: THR-001,THR-002,THR-003`.

`active-thermal-fan/fan-duty/thermal-fan.log` records the same read-only thermal
and RPM pending statuses while preserving
`fan_duty_status: pending - no production-safe bounded fan-duty route exists`.

## Claim Boundaries

`THR-001` remains below verified for fresh EMC2101 thermal readings. The
Phase 20 artifact proves only that the read-only thermal/fan surface was
allow-gated against the detector-gated safe-baseline package and that the
wrapper preserved the pending thermal-read boundary.

`THR-002` remains below verified for fresh fan RPM observations and physical fan
response. The wrapper records the fan RPM claim as read-only but pending without
a fresh RPM artifact.

`THR-003` remains supported by pure unit evidence for PID constants and decision
logic only. This pack does not promote PID output to hardware-regression fan
behavior.

Fan duty effects, overheat stimulus, thermal fault handling, fan fault behavior,
safe stop, cool/restart behavior, and API/WebSocket fault projection remain
below verified until a bounded hardware-regression route exists with recovery
and final safe-state evidence.

## Rationale

- D-05: thermal/fan reads may support only exact observed telemetry subclaims.
  Fan duty effects, physical fan response, overheat behavior, and fault
  behavior require hardware-regression evidence.
- D-06: when a production-safe fan-duty route does not exist, the correct output
  is blocked/deferred evidence instead of an unbounded active probe.
- D-13: `hardware-smoke` may support only narrow read-only thermal/fan
  observations. Active fan duty and fault-path claims require
  `hardware-regression`, so they remain below verified.

## Non-Claims

non_claims:

- No fan duty actuation ran.
- No physical fan response or fan RPM change was verified.
- No overheat stimulus, fan-fault stimulus, thermal-fault stimulus, or fault
  injection ran.
- No safe stop, cool/restart, or recovery behavior was verified.
- No API/WebSocket projection or telemetry cadence was captured by this pack.
