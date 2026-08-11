# Parity work closure

- Parity row: `API-010`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `26af4ac6711842bf91c8d928461adc39c580dddaa5d2448757b0e8d743de36e7`
- Active task: `task-ultra205-boot-recovery-attempt-010`

## Closure reason

The sole detector passed, proving the prior ROM-synchronization boundary had
changed. The conditional observation command then failed as
`cli_argument_rejected` before the campaign binary, evidence-root creation,
USB ownership, package flash, NVS write, credential read, or runtime
observation. The task used passthrough token `stage=observation`, while the
canonical CLI accepts `--stage observation`. The immutable attempt is consumed
and cannot be edited or silently retried.

## Next safe action

Create a fresh attempt-011 continuation only after a focused regression proves
the exact observation CLI flag shape. Use a new private root, a corrected
immutable command contract, and the already established fresh detector result
only as historical diagnosis; attempt 011 must perform its own detector.

## Non-claims

This closure does not claim a flash, installed current firmware, resolved boot
loop, NVS change, credential use, stable runtime, theme durability, mining,
ASIC or hardware-control behavior, OTA, recovery, other-board behavior, or
parity promotion.
