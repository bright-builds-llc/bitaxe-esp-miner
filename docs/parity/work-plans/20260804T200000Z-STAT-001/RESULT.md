# STAT-001 work result

- Parity row: `STAT-001`
- Final status: `implemented`
- Implementation commit: `f5bacf322306593269b5a92d57545cb4de59391f`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none

## Evidence and verification

The pure monitor converts BM1366 instantaneous and wrapping cumulative counter
registers into GH/s, suppresses invalid instantaneous sentinels, retains a
baseline across sub-second observations, rejects regressed timestamps and
out-of-topology observations, and computes bounded 1-minute, 10-minute, and
1-hour hierarchical rolling averages. It publishes per-ASIC totals, errors,
error percentage, and four hash-domain rates without performing device I/O.

The existing sole production ASIC worker remains the only UART owner. While its
session is active and production-ready, it issues a passive register-read burst
on the one-second monitor cadence and carries each parsed register value with a
monotonic receive timestamp to the monitor adapter. Inactive sessions neither
read nor admit register samples, and an active-to-inactive transition resets
counter baselines while preserving completed rolling history. The typed
snapshot is projected through the existing Stratum runtime and `/api/system/info`
wire model, including the upstream-compatible `hashrateMonitor` shape.

The following gates passed on the implementation tree and immediately before
the implementation commit:

- seven focused pure hashrate tests;
- API projection and serialization tests;
- six production source-ownership and lifecycle regressions;
- the real ESP32-S3 firmware Bazel build;
- `cargo fmt --all`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo build --all-targets --all-features`;
- `cargo test --all-features`;
- `bun scripts/bright-builds-check.ts all` with zero findings;
- `just test` with all 31 Bazel test targets passing;
- `just parity` with no validation errors and `just parity-progress` reporting
  the pre-transition baseline of 39 of 94 active rows verified (41.5%);
- `just verify-redaction`, `just verify-reference`, and `git diff --check`.

## Conclusion

The functional counter conversion, bounded rolling monitor, typed runtime/API
projection, and imperative-shell ownership contract are implemented with unit
and workflow evidence. This supports transitioning `STAT-001` from
`not-started` to `implemented` without a hardware claim.

## Non-claims and residual evidence gap

`STAT-001` remains below `verified`. No real BM1366 register traffic, live
hashrate accuracy, active mining session, pool interaction, device API, browser
UI, voltage, frequency, fan, thermal, or power behavior was exercised. Hardware
accuracy and end-to-end operator behavior require a separate detector-gated
task and evidence contract. No origin, hostname, SSID, address, port, USB
identity, credential, pool field, worker, device identifier, or raw trace is
included in this result.
