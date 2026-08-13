# Parity work closure

- Parity row: `API-009`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `eae200d52f71f0c32ffc53054f076424f1c0b25b2c0fd6cd84f56057b5cc950d`
- Active task: `task-parity-api009-command-effect-evidence-audit`

## Closure reason

The host checkpoint transaction now publishes a private mode-0600 `ready`
checkpoint before any IDENTIFY request. Only a valid request-once readiness
confirmation advances the campaign to its single enable request. The later
`rendered` and `cleared` confirmations preserve the ordered single-enable,
single-disable transaction, typed failure precedence, safe stop, cleanup, and
evidence withholding.

The private checkpoint schema is `bitaxe-identify-checkpoint-v2`; the public
signal is `bitaxe-operator-checkpoint-v2`. Every public signal names its closed
confirmation condition, carries the exact blank / `BITAXE IDENTIFY` / `Hello!`
/ blank frame, and states the pinned 30-second duration without exposing a
device origin, hostname, port, USB or network identity, credential, address,
path, sensor value, or raw trace. The confirmation CLI now uses
`--checkpoint ready|rendered|cleared`.

The attempt-010 public record is corrected to `stop_authority_boundary`. Its
late normal-screen report is neither positive nor negative IDENTIFY evidence;
all sealed package, block, share, pause, resume, safe-stop, cleanup, redaction,
withholding, and attempt-consumption facts remain unchanged.

Focused Rust campaign/CLI tests, TypeScript checkpoint and orchestration tests,
the real-child supervisor boundary, strict Clippy, all-target build,
all-feature tests, all 42 Bazel tests, Bright Builds checks, parity validation
and progress, redaction, reference cleanliness, and the real firmware build
pass. The immutable plan digest is unchanged.

API-009 remains `implemented`: this software-only plan produced no new
physical IDENTIFY observation and therefore cannot promote the row.

## Next safe action

This plan does not authorize attempt-011. A future clean selector may create a
separate bounded hardware contract using the pre-armed transaction. That
contract must retain the existing exact-package, detector, privacy, physical
confirmation, safe-stop, cleanup, recovery, one-attempt, and promotion gates.

## Non-claims

This closure does not claim IDENTIFY rendered or cleared on hardware, dismiss
the block notification, restart the device, publish parity evidence, promote
API-009, or access credentials, USB, the device, its network origin, direct
UART, or pins/pads/GPIO.
