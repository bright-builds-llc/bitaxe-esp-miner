# Parity work closure

- Parity row: `API-009`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `78bbac00fd79966db1f23fc5b6013b13f57f74f385b63bf032da65f85f429daf`
- Active task: `task-parity-api009-command-effect-evidence-audit`

## Closure reason

The software-only plan is complete. The typed checkpoint protocol now accepts
one explicit `replay` outcome from the original rendered checkpoint even after
its observation window has elapsed. It keeps the campaign paused and
safe-stopped, queues the replay until the prior physical effect is
conservatively inactive, issues one additional IDENTIFY request, and emits a
distinct replayed checkpoint with one exact 30-second confirmation window.

Confirmation and safe-replay timing use separate boundaries. Request latency
may conservatively shorten an admissible observation window, but it cannot
extend that window or permit an early second toggle. A replayed checkpoint
cannot request another replay. Pending, decline, cleanup, recovery, and the
earliest typed failure remain compatible with the existing transaction.

The closed command-effects evidence schema records a replay count of zero or
one and requires the corresponding total IDENTIFY request count. The host
supervisor records the ordered ready/rendered/optional-replayed/cleared
sequence and rejects any mismatch before public evidence publication.

API-009 remains `implemented` because this software plan intentionally ran no
hardware attempt and supplies no physical display observation, restart, or
complete five-command device-user quorum.

## Verification

The initial production-seam parser regression failed deterministically before
the replay outcome existed and passed after the repair. A loopback HTTP test
drives the real command transport through a missed first window, proves no
request before the safe replay boundary, observes exactly one replay request,
accepts a timely replayed confirmation, and observes the final clear request.
Negative controls cover late confirmation, duplicate replay, unbounded human
wait, explicit decline, malformed or mismatched checkpoint/evidence sequences,
and the legacy no-replay path.

Formatting, strict Clippy, all-target build, all-feature tests, Bright Builds,
all 44 Bazel test targets, parity, parity-progress, redaction, reference
cleanliness, real ESP firmware build, immutable plan digest, unique task
binding, open-plan selection, and diff checks pass. The canonical automation
suite includes the real child-process checkpoint supervisor. The Bazel graph
explicitly owns the extracted Rust checkpoint module.

## Next safe action

Keep API-009 `implemented`. If hardware verification remains the highest
priority candidate, create a separate immutable exact-package attempt-022
contract using this replay protocol. Run detector admission and at most the
single hardware attempt authorized by that future contract; promote only on
the complete claim-specific quorum.

## Non-claims

This closure does not claim a physical IDENTIFY render or clear, notification
dismissal, pause/resume behavior on hardware, software restart, restart
survival, API-009 verification, or any hardware effect. It creates no parity
evidence and exposes no credential, origin, hostname, port, USB/network
identity, address, worker, password, token, sensor value, timing origin, or raw
trace.
