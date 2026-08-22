# Parity work closure

- Parity row: `STR-005`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `82b4142a0f56978e2fb586a9574e3075cfa46d4174ad575eb667edee4088a424`
- Active task: `task-parity-str005-stratum-v2`

## Closure reason

The exact attempt-002 outer command started once on clean pushed source
`c8de00ca6d593cfab460706ed9e9bbd3c6403834`, after all software, package,
detector, privacy, and effect gates passed. It stopped immediately as
`evidence_invalid` before passive monitoring, private-root creation, fixture
start, NVS construction, USB campaign ownership, flash, network connection,
mining, share submission, or hardware control. The attempt-002 root and public
projection remain absent.

Read-only closure checks reconfirmed exactly one protected local Wi-Fi input,
exactly one protected local pool input, ignored absent attempt/projection paths,
clean synchronized Git state, and exact clean source/reference package identity.
The continued plan authorizes no third attempt, so no retry or hardware parity
claim is allowed. The outer launcher currently collapses the remaining
pre-effect predicate to `evidence_invalid`; determining and correcting that
boundary requires a fresh audited plan, not inference or a weakened contract.

## Next safe action

Keep `STR-005` at `implemented`. A fresh task and immutable continuation plan
must first add a closed pre-effect checkpoint discriminator and real-launch
coverage that reproduces the exact remaining predicate without entering the
effect phase. Only a new plan with a changed, regression-backed boundary may
authorize another ordinal.

## Non-claims

This closure does not verify an Ultra 205 Noise handshake, live V2 channel,
ASIC work, target-qualified hardware nonce, encrypted share, accepted response,
hardware safe stop, settings/package restoration, external pool
interoperability, mixed-protocol live fallback, other boards, unbounded mining,
OTA, or release readiness. It does not create `RESULT.md`, `hardware-regression`
evidence, or `verified` status.
