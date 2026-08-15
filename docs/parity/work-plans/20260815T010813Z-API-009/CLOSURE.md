# Parity work closure

- Parity row: `API-009`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `fa18d647af689334a2a15a521bc6c5738fdd8e2bf8e9ed0220706b1420ad6228`
- Active task: `task-parity-api009-command-effect-evidence-audit`

## Closure reason

The software-only plan is complete. Resumable campaigns now accumulate only
active mining segments against their 600-second lease; an operator-paused,
hardware-stopped interval does not consume the remaining active budget.

The host separates one resume request into API-visible intent confirmation and
a bounded production reactivation phase, with distinct typed failures. If a
post-start phase fails, it preserves that earliest category, sends at most one
recovery pause, and keeps the network/serial join alive until fresh API-paused
state and an authoritative post-request stopped-hardware marker agree or the
bounded recovery interval ends. Network-worker shutdown follows that closed
outcome instead of racing the evidence needed to prove safe stop.

Command-effects evidence v6 records resume intent, reactivation, and closed
recovery facts without exposing origins, ports, device/network identities,
credentials, runtime values, or traces. API-009 remains `implemented` because
this plan intentionally performed no hardware attempt and supplies no new
complete device-user quorum.

## Verification

Focused production-session, firmware-owner, command state-machine, recovery,
evidence, CLI, automation, and real-child regressions pass. The first complete
Bazel run found the new Rust recovery module missing from its explicit source
list; adding that build-graph edge made the targeted Bazel flash test pass.

The restarted ordered sequence passed formatting, strict Clippy, all-target
build, all-feature tests, Bright Builds, all 44 Bazel test targets, parity,
parity-progress, redaction, reference cleanliness, and the real ESP firmware
build. The immutable plan digest, unique task binding, sensitive-output review,
and diff checks also pass.

## Next safe action

Keep API-009 `implemented`. If it remains the highest-priority parity row,
create and push a fresh immutable exact-package attempt-024 contract for one
detector-gated Ultra 205 campaign using the repaired active-budget, resume, and
recovery protocol. Promote only on the complete command, restart, same-device,
safe-stop, cleanup, and redaction quorum.

## Non-claims

This closure does not claim live resume intent or reactivation, notification
dismissal, block-count preservation, canonical restart, same-device restart
recovery, terminal safe stop on hardware, public parity evidence, or API-009
verification. It accessed no credential, protected attempt artifact, detector,
USB, device/network, display, mining, hardware-control, direct UART, or
pin/pad/GPIO interface and exposes no sensitive runtime value or raw trace.
