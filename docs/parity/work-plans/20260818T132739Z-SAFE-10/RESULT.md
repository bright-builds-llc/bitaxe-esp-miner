# Parity work result

- Parity row: `SAFE-10`
- Final status: `verified`
- Implementation commit: `01d5cdcb2034da4ab2a9c1e0fa373f91c78dd573`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The accepted projection is
`docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json`
(SHA-256 `4e9b91bd29629aec098b9967b9bb27b9c1358f64c11819a77f8c8da4c212a20e`).
It was exclusively created by:

`just project-safe10-evidence --attempt-root scratch/stat003-scoreboard/attempt-003 --detector-output scratch/stat003-scoreboard/wrapper-003/detector.stdout --attempt-plan docs/parity/work-plans/20260818T102038Z-STAT-003/PLAN.md --attempt-closure docs/parity/work-plans/20260818T102038Z-STAT-003/CLOSURE.md --projection docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json`

`just validate-safe10-evidence docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json`
passed independently. Projection mode is 0644; a recursive exact-key and value
denylist review found no private identities, endpoints, credentials, raw
transport, or secret-bearing fields.

The evidence binds board 205, detector-admitted protected attempt-003, exact
attempt/current/reference commits and plans, sealed campaign result/network/
observations/diagnostics digests, current 19-path evaluator/reference semantics,
and byte-identical nine-path production prerequisite semantics between attempt
source and current source.

Live prerequisite evidence records all five Ultra 205 required/fresh classes:
power, bus voltage, current, chip temperature, and fan RPM; VR temperature is
explicitly neither required nor claimed fresh. It also records safety fresh,
unblocked running-primary readiness, ready hardware, advanced observation epoch,
600,746 active milliseconds, accepted submit/work, 20/20 continuity, work
renewal, active/safety/watchdog validity, terminal transports/pool/final consumed
state, safe stop, cleanup, protected modes, and passed redaction.

Focused Rust and real-process tests plus ordered Cargo gates, Bright Builds, all
47 Bazel test targets, firmware/package, parity/progress, redaction, reference,
source inventory, file-size, and diff checks passed before projection.

## Conclusion

The evidence directly proves that the production Ultra 205 mining path admitted
work only with the exact required live prerequisite classes fresh and the
readiness transition unblocked, then sustained accepted production activity and
ended safely. The source compatibility and independent validator close the gap
identified by the checklist, so `SAFE-10` is supported at `verified` with
`unit,workflow,hardware-smoke,hardware-regression` evidence.

## Non-claims and residual risks

This result does not verify fail-closed blocker labels (`SAFE-11`), individual
active voltage/fan/thermal control policies, fault injection, self-test,
arbitrary telemetry ranges, other ASICs/boards, arbitrary profiles/pools,
unbounded mining, OTA, recovery, or release readiness. Numerical sensor values
remain private and are not claimed beyond the production safety predicate's
closed accepted result.
