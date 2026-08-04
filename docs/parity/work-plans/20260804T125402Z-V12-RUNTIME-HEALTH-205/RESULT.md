# Parity work result

- Parity row: `V12-RUNTIME-HEALTH-205`
- Final status: `verified`
- Implementation commit: `56f4eb1ce50f81fe2a1dd80ac344e551b16771c9`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The single detector-gated `attempt-001` produced the committed closed
projection at
`docs/parity/evidence/v12-runtime-health-205/runtime-health-projection.json`.
The projection validates as `bitaxe-runtime-health-evidence-v1` and binds the
exact clean implementation package, one admitted Ultra 205, one boot session,
HTTP revision 614, later WebSocket revision 615, checkpoint sequences 3347 and
3348, and task-watchdog feed sequence 734 in both projections. Both exact
retained runtime-health tuples matched. Supervisor checkpoints were available,
healthy, and bounded-age; the task watchdog was participating with a fresh
bounded-age feed. Mining and hardware control remained disabled, cleanup was
complete, and redaction passed.

Commands run:

```text
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-targets --all-features
cargo test --all-features
bun scripts/bright-builds-check.ts all
just test
just parity
just parity-progress
just verify-redaction
just verify-reference
git diff --check
just package
just detect-ultra205
just capture-runtime-health-evidence --private-root scratch/v12-runtime-health/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/v12-runtime-health/detector-001/detector.stdout --projection docs/parity/evidence/v12-runtime-health-205/runtime-health-projection.json --capture-timeout-seconds 360
bazel run //crates/bitaxe-automation-contracts:validate_runtime_health_evidence -- "$PWD/docs/parity/evidence/v12-runtime-health-205/runtime-health-projection.json"
```

Focused regressions also proved fail-closed handling for unhealthy or stale
health, missing retained tuples, boot-session mismatch, sequence regression,
validator rejection, recovery failure precedence, sensitive-value denial, and
real child-process boundaries.

## Conclusion

The evidence closes the former `runtime_health_insufficient` gap. On the exact
implementation package, the live Ultra 205 published coherent substantive
runtime-health truth across HTTP, a later same-boot WebSocket projection, and
both corresponding retained records while preserving the passive safe state.

## Non-claims and residual risks

This result does not verify self-test execution, mining, ASIC work, voltage,
fan, thermal or power control, fault injection, restart behavior, settings,
network provisioning or reconnect behavior, OTA or recovery, other boards,
other ASIC families, release readiness, or unbounded runtime health. It proves
only the passive bounded runtime-health contract for one Ultra 205 attempt.
