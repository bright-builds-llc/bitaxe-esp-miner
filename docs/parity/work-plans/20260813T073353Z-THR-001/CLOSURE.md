# Parity work closure

- Parity row: `THR-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `806a75411a98ccb242c631c7f7176fed6d94cd60c06c65163705aec3ab512f60`
- Active task: `task-parity-thr001-emc2101-live-thermal`

## Closure reason

The sole authorized `attempt-004` was consumed. The exact clean package reached
the injected-fault firmware and the protected log observed the closed
`fault_observed` marker, followed by
`thermal_fault_stimulus=aborted reason=fault_projection_missing redacted=true`.
The required baseline/fault/recovered marker quorum was incomplete, so the
public candidate and final projection were withheld.

The same transaction then restored the ordinary exact package. Its independent
system-info projection validates board 205, detector admission, stable boot,
fresh safe HTTP/WebSocket thermal truth, disabled mining and hardware control,
cleanup, protected modes, and redaction. No attempt-005 ran.

The internal primary category was `evidence_invalid`. The command's public
envelope incorrectly rendered `process_failed` and omitted the safe recovery
facts because the new typed error class was not registered in the shared CLI
failure adapter. That software-only mapping omission is fixed with regression
coverage after this attempt; it does not alter or promote the consumed hardware
result.

## Next safe action

Create a fresh immutable THR-001 continuation that first reproduces the
production owner/reducer transition which lost the required fault projection.
The regression must exercise the real `reduce_sensor_sweep` boundary rather
than mutating an observation directly, preserve real EMC2101 reads, prove the
exact five-sample/fault/recovery sequence, and retain ordinary restoration.
Only a separately bounded new ordinal may authorize another detector-gated
hardware run. Never reuse or retry attempt-004.

## Non-claims

This closure does not verify THR-001, publish thermal-fault evidence, or claim
five injected samples, recovered fault state, physical overheat, electrical
sensor open/short behavior, absolute calibration, fan/voltage/power response,
mining behavior, other-board behavior, or safety-control parity.
