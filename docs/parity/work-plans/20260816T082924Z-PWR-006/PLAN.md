# Parity work plan

- Run ID: `20260816T082924Z-PWR-006`
- Parity row: `PWR-006`
- Initial status: `verified`
- Source commit: `413d0143288ce8d0ae8ecf8dbabee0d8853308d0`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-pwr006-legacy-wire-units`

## Selection

The clean preflight selector initially ranked `UI-003`, `SELF-001`, `BAP-002`,
and the unfinished statistics rows. Before the UI-003 plan was committed, the
user explicitly redirected the invocation to a suspected voltage-unit defect.
That investigation invalidated part of PWR-006's current verified claim:
reference `INA260_read_voltage` and `INA260_read_current` produce millivolts and
milliamps; the reference `/api/system/info` and statistics routes preserve those
milli-units; and AxeOS divides all three legacy electrical fields
(`voltage`, `current`, and `coreVoltageActual`) by 1,000 for display. The Rust
implementation correctly uses volts and amps inside the safety domain, but
serializes those SI values directly into the unqualified legacy `voltage` and
`current` wire fields. User-directed correction of a false verified claim takes
priority over the ordinary unfinished-row order. No other row is selected or
changed by this plan.

## Scope and non-scope

Keep the functional safety and sensor-acquisition core in explicit SI units:
input voltage in volts, current in amps, and power in watts. At the legacy API
boundary only, convert input voltage to millivolts and current to milliamps for
both `/api/system/info` (and its WebSocket projection) and
`/api/system/statistics`. Keep `coreVoltage` and `coreVoltageActual` in
millivolts, `power` in watts, and `nominalVoltage` in volts, matching the pinned
reference's intentionally mixed compatibility contract. Rename Rust-side wire
members with unit suffixes where doing so prevents a second accidental SI-unit
assignment. Update every repository consumer that validates the legacy wire
values, especially the mining-campaign safety observer, without weakening its
4.5–5.5 V safety requirement.

Bind the correction to the pinned reference driver, power task, system JSON,
statistics task, and AxeOS `/1000` normalization with regression tests. Update
the PWR-006 source-admission projector so the existing read-only hardware
lineage can be re-evaluated against the corrected pure conversion boundary.

This plan does not change ADC or INA260 register scaling, internal safety
ranges, DS4432U commands, core-voltage setpoints, power units, nominal-voltage
units, UI presentation code, hardware-control behavior, or any other parity
row. It authorizes repository edits, tests, and read-only reuse of already
committed evidence only. It does not authorize flash, USB/serial/network access,
credentials, mining, voltage/frequency/fan/power actuation, OTA, erase, fault
injection, direct UART, pins, or physical manipulation. Any fresh hardware run
requires a separate complete task contract.

## Implementation

- [ ] Add named volts-to-millivolts and amps-to-milliamps compatibility
      conversions and use them at both legacy wire boundaries.
- [ ] Rename ambiguous Rust wire/sample members and update the campaign safety
      validator to consume milli-units while preserving the exact volt-domain
      safety range.
- [ ] Add behavior-focused reference-bound unit tests covering system info,
      WebSocket-compatible serialization, statistics history, unavailable
      values, and campaign safety boundaries.
- [ ] Refresh PWR-006 source admission and its real-process fixtures without
      changing or inventing hardware observations.

## Verification and promotion

Focused verification must include `cargo test -p bitaxe-api`, the affected
`tools/flash` campaign tests, PWR-006 Rust evidence-contract tests, the
INA260 automation tests, generated-contract verification, reference
cleanliness, redaction, and the immutable-plan guard. Run the mandatory ordered
Rust sequence, Bright Builds checks, `just test`, `just parity`, and
`just parity-progress`, then review the complete diff.

The transition ledger intentionally rejects automatic transitions out of
`verified`, so this plan records the discovered contradiction instead of
performing an unaudited checklist edit. Retain the row's status only if the
corrected source and tests prove this exact unit matrix, the independent
PWR-006 projector accepts the unchanged read-only hardware lineage plus the new
source boundary without raw-value invention, all privacy and repository gates
pass, and the result explicitly states that no fresh hardware rerun occurred.
If any condition fails, leave this task active and report the verified overclaim
as a governance blocker requiring an authorized downgrade path. No analog
accuracy, external-meter calibration, load behavior, control effect,
other-board behavior, or release claim may be added.
