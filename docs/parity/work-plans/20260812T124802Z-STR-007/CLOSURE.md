# Parity work closure

- Parity row: `STR-007`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `08d4753fbb77304b0edde8552d4220efbe3354c1b2125631e33b33b250d6a7bf`
- Active task: `task-parity-str007-mining-criteria-promotion`

## Closure reason

The clean pushed implementation at
`3978d828b55de61aa97d276528510cd1d66b6e3e` passed every pre-publication
gate. Its one permitted publication invocation then failed as
`invalid_invocation` before the projector ran because the Bazel target already
injects `project-mining-criteria-evidence` and the host command supplied that
token a second time. The immutable plan stops on any failed attempt and permits
neither a retry nor hardware fallback. No candidate or public projection was
created.

## Next safe action

Start a fresh `STR-007` plan from the clean pushed closure commit. Preserve the
unchanged implementation and evidence boundary, add a regression or explicit
command-shape check for the Bazel wrapper invocation, rerun the complete gate,
and allow exactly one new software-only projection attempt that passes only
flags after `--`.

## Non-claims

This closure does not verify or promote `STR-007`. It does not publish mining
criteria evidence, reopen the terminal default-profile soak attempt, authorize
attempt-005, inspect protected campaign inputs, interact with hardware, contact
a pool, enable mining or hardware control, or establish behavior beyond the
already committed public Phase 21 evidence and tested current criteria.
