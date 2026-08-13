# Parity work closure

- Parity row: `API-009`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `4e54992c4466ddf01ef079066d97f4183da7e59d27e502908afbeca044419096`
- Active task: `task-parity-api009-command-effect-evidence-audit`

## Closure reason

The attempt-009 blocker was a production contract mismatch, not a need for a
longer host deadline. Operator-resumable pause had no typed stop purpose and
therefore synchronously used the terminal eight-step hardware shutdown. Its
120-second fresh-temperature cooling proof blocked the production owner inside
the host's 130-second pause join, preventing the owner from consuming sensor
wakeups and publishing its same-lease stopped-hardware confirmation.

The production session now emits a closed `ResumablePause` or `Terminal`
purpose. Resumable pause executes the six immediate fail-closed actions through
ASIC/core disable and full fan duty, without waiting for the terminal cooling
proof or lowering the fan. Terminal, fault, shutdown, lease-consumption, and
failed-preparation rollback retain the complete eight-step cooling plan.
Earliest typed failure precedence and independent later cleanup attempts are
unchanged.

The composed regression drives the actual production session from resumable
lease admission through operator pause, executes the selected actuation plan,
feeds the same-lease hardware confirmation back, and proves `Armed + Stopped`
without either cooling-only step. Focused session, actuation, campaign marker,
sensor/wakeup ownership, and host pause-join tests pass. The firmware build and
all mandatory Cargo, Bazel, Bright Builds, parity, privacy, and reference gates
pass.

API-009 remains `implemented`. This software-only invocation contacted no
device, read no protected attempt content or credentials, emitted no parity
evidence, and made no checklist transition.

## Next safe action

A future clean selector run may bind API-009 to a new immutable hardware
contract for exactly one attempt-010. That contract must independently define
the detector, package, command effects, live physical-observation checkpoints,
privacy/evidence policy, cleanup, recovery, timeout, and stop outcomes. This
closure establishes software eligibility only and does not itself authorize
attempt-010.

## Non-claims

This closure does not verify or promote API-009 and does not claim live pause,
resume, IDENTIFY rendering or clearing, notification dismissal, restart, or a
complete five-command hardware quorum. It does not infer physical observations,
weaken safety freshness, extend the host deadline, reuse attempt-009 artifacts,
or expose device, network, credential, sensor, process, path, or trace values.
