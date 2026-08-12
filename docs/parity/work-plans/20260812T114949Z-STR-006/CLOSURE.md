# Parity work closure

- Parity row: `STR-006`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `e1143b95b1aef4d41ec36ec9a106716787cba2b55670397cb1a4fe2a475e0b63`
- Active task: `task-parity-str006-protocol-coordinator-promotion`

## Closure reason

The clean pushed implementation at
`5d62a7ceb15b9e59934b7ae6d08ae1b1da49f324` passed every pre-publication
gate. Its one permitted projection attempt then failed closed as
`evidence_invalid` before publication because the projector required the
fragment `AsicWorkerCommand::Dispatch {` to be unique while the admitted ASIC
worker module contains two legitimate occurrences. The immutable plan stops
on any failed projection and permits neither a retry nor hardware fallback.
No candidate or public projection remains.

## Next safe action

Start a fresh `STR-006` plan from the clean pushed closure commit. Replace the
over-broad ASIC-worker uniqueness assertion with a source guard that binds the
two legitimate dispatch spans or a uniquely identifying enclosing span, add a
production-shaped regression using the real admitted module, rerun the complete
gate, and allow exactly one new software-only projection attempt.

## Non-claims

This closure does not verify or promote `STR-006`. It does not publish protocol
coordinator evidence, reopen protected campaign inputs, interact with hardware,
contact a pool, enable mining or hardware control, or establish any behavior
beyond the already accepted prerequisite projections and tested implementation.
