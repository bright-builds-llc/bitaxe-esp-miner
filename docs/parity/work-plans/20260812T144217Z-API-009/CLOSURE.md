# Parity work closure

- Parity row: `API-009`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `dcddcf397b4918a59f2778c2308798c9f5b445cd38a2463c6e3ec8ae431d6f11`
- Active task: `task-parity-api009-command-effect-evidence-audit`

## Closure reason

The single bounded `attempt-001` stopped before device admission with terminal
category `timeout`: the repository-owned local Stratum child did not publish
its private readiness document within ten seconds. The attempt root is mode
`0700`, contains only the mode-`0600` stop sentinel created during cleanup, and
has no campaign root. Consequently no USB session, flash, NVS seed, mining,
ASIC traffic, HTTP command, identify effect, software restart, or recovery
write occurred. The public API-009 projection was correctly withheld.

Host-only follow-up diagnostics proved that the exact fixture script can bind
and publish readiness on the same derived interface with both the system Node
runtime and Bazel's pinned Node runtime. This excludes the connected device,
the fixture protocol, and basic host address availability. The remaining defect
is at the automation child-lifecycle seam: readiness polling is not raced
against early child completion, and the current catch discards the child
outcome until cleanup. It therefore converts an early launch/exit defect into a
generic readiness timeout without a protected diagnostic category.

The immutable plan permits exactly one attempt and no retry. API-009 remains
`implemented` because no command-effect evidence was captured.

## Next safe action

Create a new immutable API-009 continuation plan and fresh ordinal that first:

1. races fixture readiness against child completion and classifies early exit
   as `process_failed` while preserving protected stderr only in the private
   attempt root;
2. uses a repo-owned executable/runfiles locator instead of an implicit
   `process.execPath` handoff where practical;
3. adds a real `createLocalProcessPort` integration test using the derived-host
   launch shape, including early-exit, timeout, cleanup, and redaction cases;
4. proves the fixture is ready before any detector-admitted USB operation; and
5. only then runs one fresh detector-gated command-effects attempt under a new
   explicit retry bound.

## Non-claims

This closure does not verify or promote API-009 and does not claim pause,
resume, identify, restart, block-notification dismissal, mining, ASIC, Stratum,
safe-stop, recovery, or physical-display behavior. The host-only diagnostics
do not substitute for hardware evidence and do not authorize reuse of
`attempt-001` or an ad hoc hardware retry.
