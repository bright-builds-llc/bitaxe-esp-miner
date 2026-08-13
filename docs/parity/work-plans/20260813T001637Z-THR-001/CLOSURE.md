# Parity work closure

- Parity row: `THR-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `1bfc569edbfe9e307128fad2c0eb2a54c8d8925f6b412e30ed201ca6052b2860`
- Active task: `task-parity-thr001-emc2101-live-thermal`

## Closure reason

The sole authorized `attempt-001` was consumed and ended with the earliest
typed category `evidence_invalid`. The public projection and its candidate
were both withheld. Protected post-run checks established that the detector,
exact package identity, stable boot, disabled mining and hardware control,
private modes, fresh finite below-throttle HTTP and WebSocket samples, and
their value, state, acquisition-stamp, boot-session, and package correlation
all passed.

The failing member was host-owned source admission. The orchestration expected
the exact fragment
`let adjusted = temperature_celsius + ULTRA205_EMC2101_TEMP_OFFSET_C;`, but the
clean pushed implementation's final simplification directly passes that
expression to `validate_temperature`. The runtime behavior is equivalent and
covered by the reducer tests, but the immutable attempt contract required the
missing textual fragment. Attempt-001 therefore cannot publish accepted
evidence, and its terminal outcome is `stop_impossible_contract`.

## Next safe action

Create a fresh THR-001 plan that replaces the stale textual fragment with the
actual production semantic boundary and adds a regression that validates the
checked-in source rather than only synthetic fixtures. Commit and push that
software fix, build an exact clean package, then authorize at most one fresh
detector-gated attempt with a new ordinal. Do not reuse or retry attempt-001.

## Non-claims

This closure does not verify THR-001, does not publish a thermal projection,
and does not claim absolute sensor calibration, other-board behavior,
configurable offsets, thermal or fan actuation, fault or overheat response,
fan RPM, long-duration stability, mining behavior, or any safety-critical
hardware-control parity.
