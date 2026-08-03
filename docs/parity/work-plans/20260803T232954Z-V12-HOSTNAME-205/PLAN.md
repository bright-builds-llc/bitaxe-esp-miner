# Parity work plan

- Run ID: `20260803T232954Z-V12-HOSTNAME-205`
- Parity row: `V12-HOSTNAME-205`
- Initial status: `implemented`
- Source commit: pending implementation freeze
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-v12-hostname-typed-capture`

## Selection

Earlier candidates still require broader configuration, network, ASIC,
Stratum, API-effect, safety-control, logging, or release evidence, and the
terminal upstream-default soak cannot be retried or reused. `REL-09` and exact
package identity now have current typed evidence. `V12-HOSTNAME-205` is the
first actionable remaining migration correction because its pure parser and
firmware persistence path exist, while the current typed command stops at
classifying externally supplied legacy traces. A bounded semantic capture can
close that exact gap.

## Scope and architecture

Keep legacy baseline/delivery/post-restart classification behavior intact.
Add one `capture` mode to the same semantic command. The TypeScript imperative
shell will own private storage, exact-package flash/monitor, same-origin HTTP,
normal restart observation, and restoration. Pure projection validation stays
closed and testable. The public artifact records booleans and package/workflow
identity only; original and temporary hostnames remain private.

The workflow must fail closed before effects on invalid invocation, paths,
detector output, package, or credentials. After a hostname effect it must
attempt restoration exactly once before returning any failure. If the origin is
lost, one same-device exact-package recovery flash is allowed; no unchanged
workflow retry is allowed.

Attempt 001 reached normal restart but the post-restart monitor was launched
after USB re-enumeration and produced no artifact. Recovery restored and read
back the private original hostname; no public projection was emitted. Attempt
002 changes that boundary by pre-acquiring the passive monitor before the
restart request. It is the only post-fix retry.

## Implementation

- [x] Add conditional capture-mode invocation parsing and CLI dispatch without
      weakening legacy classifier modes.
- [x] Add same-origin PATCH/restart helpers, private capture orchestration,
      closed projection generation, and semantic projection tests.
- [x] Cover success, stale/ambiguous detector, unsafe boot, persistence
      mismatch, restoration, recovery, and redaction boundaries.
- [ ] Freeze a clean pushed package and run the single hardware attempt.
- [ ] Record result, transition only the selected row, synchronize progress,
      and archive the task.

## Verification and promotion

Focused TypeScript, Rust contract, parity classifier, and firmware settings
tests run before the hardware attempt. Mandatory verification is the ordered
Rust sequence, Bright Builds checks, all Bazel tests, parity/progress,
redaction, reference cleanliness, and diff checks.

Promotion requires exact-package board-205 admission, safe disabled boot,
successful hostname PATCH and immediate readback, one normal restart with a
new correlated boot, post-restart hostname match, confirmed restoration of the
private original hostname, cleanup, closed public projection, and semantic
redaction. Any missing fact leaves the row implemented.
