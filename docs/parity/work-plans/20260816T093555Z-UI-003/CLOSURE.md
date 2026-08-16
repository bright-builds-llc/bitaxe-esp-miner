# Parity work closure

- Parity row: `UI-003`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `6c907fc08bfe2031b20f414b15b6483fd77d0048088ca365b39a132a750409fd`
- Active task: `task-parity-ui003-boot-button`

## Closure reason

The sole authorized `attempt-001` admitted exactly one Ultra 205 and completed
the exact-package flash, but the first live receive boundary ended with
`runtime_attestation_invalid` before the physical-input checkpoint. No BOOT
press occurred and no public projection was written. Code review against the
transcript-free observer found that arbitrary USB chunks were parsed as whole
lines; a runtime-attestation marker split between chunks was therefore treated
as malformed. The reducer now retains a bounded partial line and focused tests
prove split-marker recovery, but the immutable plan authorized no retry.

## Next safe action

Create a fresh immutable UI-003 continuation plan with a new attempt ordinal,
rebuild an exact clean package containing the incremental-line fix, rerun the
detector, and perform one brief BOOT press only after the trusted-runtime
checkpoint appears.

## Non-claims

This closure does not verify physical BOOT-button input, device-side debounce
timing, short-click screen advance on hardware, long-press behavior, self-test,
configuration-AP changes, mining, controls, updates, or any other board.
