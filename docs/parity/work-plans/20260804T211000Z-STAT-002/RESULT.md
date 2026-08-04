# STAT-002 work result

- Parity row: `STAT-002`
- Final status: `implemented`
- Implementation commit: `35f8bb676b91bdb702dd9026cb0379f5b12e45e6`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none

## Evidence and verification

The pure statistics owner retains at most 720 chronological samples, rejects
regressed timestamps without mutation, clears and disables history when the
confirmed `statsFrequency` is zero, and applies the pinned reference task's
full-buffer removal decision. It carries the typed STAT-001 hashrate error
percentage into the existing upstream-compatible statistics response.

The firmware starts exactly one dedicated statistics producer. It uses an
absolute one-second deadline, reads `statsFrequency` from the confirmed settings
snapshot, captures the runtime projection without draining the retained marker,
and appends through the sole history owner. The HTTP route clones that history;
request timing cannot create, consume, or clear samples.

The following gates passed on the implementation tree and immediately before
the implementation commit:

- twelve focused statistics tests, including five bounded-history regressions;
- five production source-ownership and request-immutability regressions;
- the real ESP32-S3 firmware Bazel build;
- `cargo fmt --all`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo build --all-targets --all-features`;
- `cargo test --all-features`;
- `bun scripts/bright-builds-check.ts all` with zero findings;
- `just test` with all 32 Bazel test targets passing;
- `just parity` with no validation errors and `just parity-progress` reporting
  the pre-transition baseline of 39 of 94 active rows verified (41.5%);
- `just verify-redaction`, `just verify-reference`, and `git diff --check`.

## Conclusion

The bounded history, exact eviction policy, producer cadence, confirmed-setting
read, typed runtime projection, and read-only HTTP ownership contract are
implemented with unit, workflow, and API-comparison evidence. This supports
transitioning `STAT-002` from `in-progress` to `implemented` without a hardware
claim.

## Non-claims and residual evidence gap

`STAT-002` remains below `verified`. No live firmware cadence, real telemetry
accuracy, long-duration retention behavior, device API response, browser chart,
mining session, pool interaction, voltage, frequency, fan, thermal, or power
behavior was exercised. Those claims require separate detector-gated evidence.
No origin, hostname, SSID, address, port, USB identity, credential, pool field,
worker, device identifier, or raw trace is included in this result.
