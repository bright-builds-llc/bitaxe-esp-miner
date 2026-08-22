# Parity work closure

- Parity row: `STR-005`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `ec61e1332d2203a671572c3dcde6c01e6fe75e4461fa3fc2762a62a1f12d7fbc`
- Active task: `task-parity-str005-stratum-v2`

## Closure reason

Clean pushed source and exact package `39aefd23` passed every mandatory
software gate and the read-only preflight returned schema
`bitaxe-stratum-v2-campaign-preflight-v1`, status `ready`, checkpoint
`pre_effect_ready`, `effect_started=false`, and `private_root_created=false`.
The attempt root and public projection were absent, and a fresh detector
admitted exactly one Ultra 205.

The sole attempt-003 campaign then stopped after about 20 seconds with earliest
category `hardware_blocked`, checkpoint `unclassified`, and no public
projection. The attempt root remained absent, proving that settings backup,
fixture start, temporary SV2 credentials, flash, pool/network traffic, mining,
share submission, and hardware control did not begin. No fixture or campaign
process remained, recovery was unnecessary, and a post-attempt detector
confirmed the same USB session ready.

Execution order and bounded duration place the failure after the software-only
preflight and before private-root creation, within passive runtime monitor,
origin extraction, same-origin settings reads, restoration-input comparison, or
prior-package selection. The consumed public result cannot distinguish those
sub-boundaries, so inference cannot authorize an unchanged retry.

## Next safe action

Keep `STR-005` at `implemented`. Add closed checkpoints for every remaining
pre-root boundary and a task-gated read-only runtime-admission command that runs
the exact monitor/settings/restoration checks without creating the attempt root,
fixture, settings, flash, pool session, mining lease, or hardware effects. A
fresh ordinal is eligible only if that command returns a closed ready result on
clean pushed source and the new real-boundary regressions pass.

## Non-claims

This closure does not verify a Noise handshake, V2 channel, ASIC work, target-
qualified nonce, encrypted share, accepted response, terminal safe stop,
settings/package restoration, external-pool interoperability, mixed-protocol
fallback, other boards, unbounded mining, OTA, or release readiness. It does
not create `RESULT.md`, hardware-regression evidence, or `verified` status.
