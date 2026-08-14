# Parity work closure

- Parity row: `API-009`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `2ca40f616dfadd517a7af45acb59f28180e2d8ada669a073bae6392e639a5082`
- Active task: `task-parity-api009-command-effect-evidence-audit`

## Closure reason

The sole detector-gated attempt-020 admitted one Ultra 205 and exact pushed
package `6aec8582`. Factory and NVS transfers each completed once as `ready`,
runtime attestation was `trusted`, and package admission passed.

The campaign proved the genuine notification, positive block count, one pause
and paused safe stop, the live operator-ready signal, two IDENTIFY requests,
the exact rendered frame during its 30-second authority window, and the later
live cleared observation. The unbounded ready and cleared checkpoints did not
expire. After cleared, the campaign issued its one resume request, but active
recovery was not confirmed. The serial owner closed first as `safety_stale`;
the terminal marker reported every required observation as not fresh. Dismiss
and restart did not run.

The wrapper preserved `hardware_blocked` with cleanup complete, one recovery
pause attempted, a secondary recovery failure, and terminal safe stop
unconfirmed. USB cleanup is `ready`, all private file and directory modes pass,
attempt processes are absent, the result digest is valid, and the public
projection is withheld.

## Verification

The complete mandatory Cargo, Bright Builds, Bazel, focused operator-lifetime,
real-child, parity, parity-progress, redaction, reference, real-firmware,
immutable-plan, task-uniqueness, protected-mode, process-cleanup, and diff gates
passed before the attempt. Closed categorical artifacts agree on exact-package
admission, trusted runtime identity, physical IDENTIFY success, the single
resume request, earliest safety failure, cleanup, recovery failure, and evidence
withholding.

## Next safe action

Keep API-009 `implemented`. Reproduce why resuming after a pause-held human
observation transaction emits a terminal `safety_stale` marker even though the
network-side identity and safety sample remain valid. Determine whether paused
sensor acquisition, readiness publication ordering, or terminal marker policy
owns the mismatch; implement and verify the smallest root-cause fix before any
later hardware attempt. This plan authorizes no attempt-021 or unchanged retry.

## Non-claims

This closure does not claim resume recovery, dismissal, block-count
preservation, canonical restart, restart survival, terminal safe stop, public
evidence, API-009 verification, or any broader mining/safety parity. It exposes
no credential, origin, hostname, port, USB/network identity, worker, address,
password, token, sensor value, or raw trace.

## Post-closure software repair

A production-shaped red regression reproduced the failure without hardware: a
stopped, armed command-effects resume emitted a stale readiness marker, then a
fresh observation wakeup and valid terminal marker, but the host preserved the
first marker as terminal `safety_stale`. This confirmed that firmware recovery
occurred and isolated the defect to the host's overly broad terminal policy.

The host now treats only that exact non-actuating resume-readiness state as
recoverable. Stale telemetry while command effects are active and stale
observation-stage telemetry remain terminal negative controls. Focused and
mandatory verification passes. This repair does not alter attempt-020's closed
facts, authorize attempt-021, publish evidence, or promote API-009.
