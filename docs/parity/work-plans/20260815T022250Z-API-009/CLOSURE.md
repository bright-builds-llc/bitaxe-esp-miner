# Parity work closure

- Parity row: `API-009`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `468995b183a45fd3f639b3ad2004727c8e30cca06bce785768306d614b251331`
- Active task: `task-parity-api009-command-effect-evidence-audit`

## Closure reason

The software-only plan is complete. During a resumable campaign with a prior
active epoch but no current active segment, a stale safety sample after
hardware preparation now requests a resumable hardware safe stop. Confirmation
returns the campaign to armed and stopped without consuming the lease or its
accumulated active-time budget, so a later fresh observation can reprepare,
reconnect, and become active.

Initial activation has no prior active epoch, and current active mining has an
active segment. Both states therefore keep stale safety terminal and consume
the lease as before. The host command-effects, recovery join, and closed
evidence v6 contracts remain unchanged.

API-009 remains `implemented` because this plan intentionally performed no
hardware attempt and supplies no new complete device-user quorum.

## Verification

The attempt-024-shaped regression first reproduced the terminal-safe-stop
failure, then passed after the predicate repair. Negative controls prove stale
safety remains terminal before first activation and during active mining.
Focused campaign timing, recovery, production-session, and firmware-owner tests
pass.

The ordered Cargo format, strict Clippy, all-target build, and all-feature test
sequence; Bright Builds; all 44 Bazel tests; parity and progress; redaction;
reference cleanliness; and the real ESP firmware build pass. A combined parity
invocation encountered a transient host `os error 35`; an immediate exact
isolated `just parity` retry passed. The immutable plan digest, unique task
binding, sensitive-output review, and diff checks pass.

## Next safe action

Keep API-009 `implemented`. If it remains the deterministic selector's open
row, create and push a fresh immutable exact-package attempt-025 contract for
one detector-gated Ultra 205 campaign using the repaired reactivation-safety
semantics. Promote only on the complete command, restart, same-device,
safe-stop, cleanup, and redaction quorum.

## Non-claims

This closure does not claim live post-pause reactivation, notification
dismissal, block-count preservation, canonical restart, same-device restart
recovery, terminal safe stop on hardware, public parity evidence, or API-009
verification. It accessed no credential, protected attempt artifact, detector,
USB, device/network, display, mining, hardware-control, direct UART, or
pin/pad/GPIO interface and exposes no sensitive runtime value or raw trace.
