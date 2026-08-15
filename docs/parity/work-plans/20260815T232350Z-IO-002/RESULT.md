# Parity work result

- Parity row: `IO-002`
- Final status: `verified`
- Implementation commit: `166d1e9f3c4065946e6e3bb60398671bcdceab62`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

Committed evidence:

- `docs/parity/evidence/io002-adc/adc-observation-projection.json`
- `docs/parity/work-plans/20260815T232350Z-IO-002/PLAN.md`
- `docs/parity/work-plans/20260815T232350Z-IO-002/WORKLOG.md`

The software boundary was proved red before correction: after registering
`adc-observation-evidence.test.ts` in the deployed `all.test.ts` entrypoint, an
uncached `bazel test //tools/automation:automation_test` run failed exactly at
the checked-in ADC source-semantic test with `ADC source semantic fragment is
not unique`. Replacing the broad bit-width token with the unique production
channel-config initializer context made that real runfiles test pass. The same
registered suite rejects missing, duplicated, and drifted context and reports
337 passing tests.

Before device access, the following passed on the exact implementation:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just package`
- `just test` (45 Bazel test targets)
- `just parity`
- `just parity-progress` (`69/94`, `73.4%` before promotion)
- `just verify-redaction`
- `just verify-reference`

The implementation was committed and pushed as
`166d1e9f3c4065946e6e3bb60398671bcdceab62`, local `main` matched
`origin/main`, and `just package` rebuilt the exact clean package. The plan's
exact detector command admitted one Ultra 205. Its sole one-shot
`just capture-adc-observation-evidence` attempt used protected attempt-004
paths, the exact package manifest, local credential input, detector handoff,
the named public projection, and the bounded 360-second capture setting.

The public projection independently validates:

- schema `bitaxe-adc-observation-evidence-v1`, board 205, and attempt 4;
- exact source, pinned reference, package, plan, and workflow identity;
- detector admission, stable exact-package boot, protected modes, current
  production source, and unique source semantics across seven Rust paths plus
  the pinned upstream ADC source;
- ADC unit 1, channel 1, GPIO 2, 12 dB attenuation, default resolution, curve
  calibration, and the 500 ms producer cadence;
- read-only acquisition and fresh finite nonnegative integer-millivolt HTTP and
  WebSocket observations bound to disabled mining and hardware control;
- same boot session, non-regressing sequence and acquisition time, exact public
  numeric/status correlation, and exact package identity; and
- complete cleanup, no recovery flash, atomic publication, and passed
  redaction.

After publication, both of these passed:

- `just validate-adc-observation-evidence docs/parity/evidence/io002-adc/adc-observation-projection.json`
- `just verify-redaction` (19 accepted public evidence documents)

## Conclusion

The evidence supports verified IO-002 parity for the scoped Ultra 205 behavior.
The Rust firmware uses the same ESP-IDF calibrated millivolt domain evidenced by
the pinned upstream ADC implementation, acquires it through the declared board
205 ADC configuration, and projects fresh coherent values and acquisition state
through HTTP and WebSocket while the rail, mining, and hardware controls remain
disabled. The exact clean package, one physical device, current source
semantics, evidence workflow, cleanup, and privacy contract are all bound and
independently validated.

## Non-claims and residual risks

This result does not prove energized-rail voltage values or accuracy, external
meter calibration, induced ADC read failures, voltage actuation, behavior under
mining or thermal load, long-duration drift, electrical tolerances, other
boards, or release readiness. It verifies passive disabled-state acquisition
and public projection only. Those excluded surfaces retain their own safety and
hardware-evidence requirements.
