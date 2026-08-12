# Parity work closure

- Parity row: `PWR-003`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `7aff33c814262fc32ceeb082778093a055609711655ffd87d568aba37c7e2c5b`
- Active task: `task-parity-pwr003-core-voltage-control-audit`

## Closure reason

The implementation and every required software, privacy, reference, integrity,
and source-projection gate passed at exact pushed commit
`10a72b06b914f9ee376b2542e2cf66dc7bdfe2b7`. The sole software-only projection
attempt then stopped before candidate creation with typed category
`evidence_invalid`. No public PWR-003 projection was published and no hardware
command ran.

The projector admitted semantic source fragments by requiring each configured
substring to occur exactly once. The configured fragment
`CORE_VOLTAGE_STABILIZATION_MS,` occurs twice in the production
`mining_actuation_adapter.rs`: once in its import and once at the intended use
site. This made the matcher reject the real source despite the intended
stabilization route being present. The immutable plan permits only one
projection attempt and requires this row to stop after failed validation.

## Next safe action

Create a fresh PWR-003 task and immutable software-only plan. Replace the
ambiguous substring with a source-shaped fragment unique to the intended
stabilization use site, add a regression over the real production file that
fails on the current matcher and passes after the correction, and rerun the
complete software and evidence gates before one fresh projection attempt. Do
not rerun hardware; the sealed accepted PWR-002 projection remains the only
hardware evidence source.

## Non-claims

This closure does not verify or promote PWR-003. It does not prove measured
analog voltage, setpoint accuracy, rail timing or waveform, arbitrary or
dynamic voltage targets, fault recovery, INA260 correlation, another board,
or any new hardware effect. Passing implementation tests and the accepted
PWR-002 source evidence do not substitute for a valid final PWR-003 projection.
