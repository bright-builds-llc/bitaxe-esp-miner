# Parity work closure

- Parity row: `STR-007`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `4dc2f29ee6d2b7bcbcfc7e5ad6d5db3f75715fc102c9beae9381a0715f8d08cf`
- Active task: `task-parity-str007-mining-criteria-promotion`

## Closure reason

The guarded implementation at
`ad4632b0d6a91cca5b4d8a53a6bc683d1e079bdf` passed every pre-publication
gate, and the plan's single corrected projector invocation succeeded. The
immediately following independent validation command used a repository-relative
projection path with `bazel run`; Bazel executed the validator from its runfiles
working directory, so the validator returned `No such file or directory` before
opening the projection. The immutable plan stops on any failure and permits no
validator adjustment or projection retry. The unvalidated projection was
removed and no candidate remains.

## Next safe action

Start a fresh `STR-007` plan from the clean pushed closure commit. Preserve the
closed projector implementation and both wrapper regressions, add a focused
real-process check that proves the Bazel validator can resolve an absolute
projection path, rerun the complete gate, and permit exactly one new
software-only projection plus independent validation transaction.

## Non-claims

This closure does not verify or promote `STR-007`. It does not publish mining
criteria evidence, reopen the terminal default-profile soak attempt, authorize
attempt-005, inspect protected campaign inputs, interact with hardware, contact
a pool, enable mining or hardware control, or establish behavior beyond the
already committed public Phase 21 evidence and tested current criteria.
