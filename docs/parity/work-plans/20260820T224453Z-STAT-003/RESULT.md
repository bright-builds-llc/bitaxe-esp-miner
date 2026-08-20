# Parity work result

- Parity row: `STAT-003`
- Final status: `verified`
- Capture source: `a31af2873e6b2d41fe47aa18a57626f33aaf099b`
- Evaluation source: `cbc5fa7f`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The immutable plan is
`docs/parity/work-plans/20260820T224453Z-STAT-003/PLAN.md` with SHA-256
`80188c102499798c1907c9f757861c3ee28bc51206c225653908e0462836b692`.
The independently validated public evidence is
`docs/parity/evidence/stat003-scoreboard/scoreboard-projection.json` with
SHA-256 `e8054e9176154f154a82b4c9f5301f9d87f64ca558e2ad117be7c37fc4efe920`.

The sole v2 protected command frozen in the plan exited zero. It performed no
hardware, device, USB, detector, network, credential, mining, share, restart,
or recovery action. It read only the immutable attempt-005 artifacts and
published only after the Rust validator accepted the v2 candidate.

Pre-effect verification passed ordered Cargo format/clippy/build/test and
doctests, Bright Builds, focused v1/v2 contract and recheck tests, all 48 Bazel
tests, firmware build/package, redaction, reference, parity/progress, selector,
file-size, and diff gates. Post-effect validation passed the independent Rust
validator and the v2-aware semantic redaction scanner with `checked=1`.

The projection binds the capture/evaluator sources, pinned reference, old/new
plans, old closure/terminal boundary, retained package identity, protected
input and source-inventory commitments, accepted campaign seals, 20/20 windows,
qualified nonce and submit outcome, trusted runtime, safe stop, cleanup, modes,
and no hardware rerun.

It proves a stable 20-entry live scoreboard, exact wire/bounds/order, live SPA,
one exact restart, changed session, ordinal +1, software-CPU reset, disabled boot
mining, same package identity, stable post-restart repeat, exact non-difficulty
fields/order/count, and the pinned one-decimal durable difficulty projection.

## Conclusion

Attempt-005 plus the corrected v2 evaluator supplies the missing live mining,
API/SPA, restart, durability, safety, cleanup, privacy, and independent
validation evidence. This supports transitioning only `STAT-003` to `verified`
with `unit,workflow,api-compare,static-route,hardware-smoke,hardware-regression`.

## Non-claims and residual risks

This result does not recover or claim original manifest bytes. It does not
verify profitability, laboratory difficulty calibration, arbitrary pools or
profiles, other ASICs/boards, unbounded mining, updates/recovery, release
readiness, UART/BAP, pins, or electrical behavior. It publishes no protected
values or raw attempt artifacts.
