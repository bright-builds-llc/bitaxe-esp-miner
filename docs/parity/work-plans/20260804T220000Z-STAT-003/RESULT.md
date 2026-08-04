# STAT-003 work result

- Parity row: `STAT-003`
- Final status: `implemented`
- Implementation commit: `0f3d46a77f5b2492880921cf524bc052d2283bc4`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none

## Evidence and verification

The pure scoreboard owner validates upstream-compatible fields, keeps at most
20 entries in stable descending difficulty order, applies the exact indexed-NVS
record shape, and exposes mutation boundaries for transactional persistence.
Failed persistence never publishes a candidate, while successful persistence is
reloaded and compared against the durable one-decimal projection before the
in-memory snapshot advances.

Current-generation valid nonce correlation computes difficulty once and carries
a redacted scoreboard candidate independently of pool submit policy. The
production session emits the typed effect for qualifying, below-target, and
duplicate valid candidates, while stale, uncorrelated, and invalid observations
remain excluded. The firmware owns boot loading and indexed NVS writes and the
HTTP projection is a read-only clone of the confirmed owner state.

The following gates passed on the implementation tree and immediately before
the implementation commit:

- ten focused API scoreboard tests;
- three production-session scoreboard tests;
- five firmware source-ownership tests;
- the real ESP32-S3 firmware Bazel build;
- `cargo fmt --all`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo build --all-targets --all-features`;
- `cargo test --all-features`;
- `bun scripts/bright-builds-check.ts all` with zero findings;
- `just test` with all 33 Bazel test targets passing;
- `just parity` with no validation errors and `just parity-progress` reporting
  the pre-transition baseline of 39 of 94 active rows verified (41.5%);
- `just verify-redaction`, `just verify-reference`, and `git diff --check`.

## Conclusion

The stable bounded owner, valid-nonce receipt, transactional firmware
persistence, boot load, and read-only API projection are implemented with unit,
workflow, and API-comparison evidence. This supports transitioning `STAT-003`
from `in-progress` to `implemented` without a hardware or mining claim.

## Non-claims and residual evidence gap

`STAT-003` remains below `verified`. No live ASIC nonce, real difficulty,
device persistence, device API response, browser scoreboard, mining session,
accepted or rejected share, pool interaction, voltage, frequency, fan, thermal,
or power behavior was exercised. Those claims require separate detector-gated
evidence. No origin, hostname, SSID, address, port, USB identity, credential,
pool field, worker, device identifier, or raw trace is included in this result.
