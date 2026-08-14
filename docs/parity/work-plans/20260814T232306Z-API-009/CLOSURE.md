# Parity work closure

- Parity row: `API-009`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `68c2dc317ea5a6fcaf30ef2a8fe0bfafac360254060697defecd448a5ced18fb`
- Active task: `task-parity-api009-command-effect-evidence-audit`

## Closure reason

The software-only plan is complete. A rendered or replayed response is now a
single-use, attempt-local attestation that the operator observed the exact
IDENTIFY frame during that checkpoint's uniquely bound 30-second device effect.
The report may arrive after the physical effect ends, so chat and operator
latency no longer create a second deadline or discard an observation that
already occurred.

Each display window now uses exactly one IDENTIFY request. After a confirmed
observation, the campaign waits until the current effect is conservatively
inactive before opening the unbounded cleared checkpoint. It no longer issues a
second latency-sensitive IDENTIFY toggle to clear the frame. An explicit replay
remains available once, starts only after the prior effect is inactive, and
adds exactly one request. Closed evidence therefore requires
`identify_request_count == 1 + identify_replay_request_count`.

Pause ownership, safe stop, decline, recovery, cleanup, duplicate and
cross-checkpoint rejection, evidence withholding, and privacy remain intact.
API-009 remains `implemented` because this plan intentionally performed no
hardware attempt and supplies no new complete device-user quorum.

## Verification

Focused unit tests accept both prompt and day-late reports, enforce natural
expiry before the cleared checkpoint, and reject a second replay. The loopback
HTTP replay integration proves there is only one request in the replayed
display window and that a delayed report remains valid. The Bazel-owned
real-child supervisor proves ordered checkpoint publication with the new public
attestation semantics.

Formatting, strict Clippy, all-target build, all-feature tests, Bright Builds,
all 44 Bazel test targets, parity, parity-progress, redaction, reference
cleanliness, real ESP firmware build, immutable plan digest, unique task
binding, open-plan selection, stale-schema scan, and diff checks pass.

## Next safe action

Keep API-009 `implemented`. If it remains the highest-priority parity row,
create and push a fresh immutable exact-package attempt-023 contract for one
detector-gated Ultra 205 campaign using this latency-tolerant protocol. Promote
only on the complete command, restart, same-device, safe-stop, cleanup, and
redaction quorum.

## Non-claims

This closure does not claim a new physical IDENTIFY render or clear,
notification dismissal, pause/resume behavior on hardware, software restart,
restart survival, public parity evidence, or API-009 verification. It accessed
no credential, protected attempt artifact, detector, USB, device/network,
display, mining, hardware-control, direct UART, or pin/pad/GPIO interface and
exposes no sensitive runtime value or raw trace.
