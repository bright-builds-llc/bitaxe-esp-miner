# Parity work closure

- Parity row: `API-009`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `12ab02a452dfc0b4ecad41ead409998e2a98b137a2338ef6644285d9b75c800b`
- Active task: `task-parity-api009-command-effect-evidence-audit`

## Closure reason

The exact pushed source and package at
`f8c279d25f0c4a3704bf1837a0eabef58df26410` passed the complete software,
privacy, reference, integrity, package, and detector gates. The detector
admitted exactly one connected board-205 ESP32-S3 device, and the sole
authorized attempt-003 ran without a retry.

The material flash remediation succeeded. Both the exact factory image and the
generated NVS seed completed through `write-bin --no-stub` on their first
supervised attempt. Their protected diagnostics report `ready`, completed
device effects, successful termination, closed byte counts and SHA-256 values,
and no raw output. The firmware then produced a trusted runtime identity, 21
accepted campaign markers, two accepted runtime attestations, a confirmed safe
stop, and ready USB cleanup.

The campaign stopped later as public `hardware_blocked` and private
`network_correlation_failed`. Its terminal reason is
`stratum_v1_unsupported`: no active network window, HTTP success, WebSocket
frame, pool work, genuine notification, pause/resume, IDENTIFY checkpoint,
dismissal, or restart occurred. The public projection was correctly withheld,
and every protected directory and file retained mode `0700` or `0600`.

API-009 therefore remains `implemented`. The no-stub result is objective
progress, but it is not the complete five-command device-user quorum required
for promotion.

## Next safe action

Create a fresh immutable API-009 continuation that adds a closed category-only
production protocol-gate diagnostic, reproduces the gate against the exact
campaign-generated NVS image, and distinguishes partition acquisition, NVS
open, primary selector, and fallback selector decisions without exposing pool
or network data. Implement only the materially confirmed fix, cover the real
generated-image/startup seam, rerun every software and privacy gate, and define
a fresh hardware ordinal only if those results prove objective progress.

## Non-claims

This closure does not verify or promote API-009. It does not claim active pool
work, an ASIC-qualified network-target notification, pause, resume, physical
IDENTIFY rendering or clearing, notification dismissal, software restart, or a
five-command quorum. It does not infer parity from the display and authorizes
no retry under this plan.
