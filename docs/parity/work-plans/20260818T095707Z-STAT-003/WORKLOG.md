# Parity work log

## 2026-08-18T10:15:55Z | natural serial-closure correction

- Source commit: `9da1d2c33b3a2c7d2a200f03b19e682f476b87e4`.
- Actions: removed only the worker-close-request boolean from network acceptance
  truth; retained it in v12 evidence; made hashrate and scoreboard consumers
  require a boolean field without requiring true; added Rust and real-child
  coverage for worker-requested true, analyzer-natural false, missing,
  non-boolean, and missing-final-consumed shapes. Moved the reusable hashrate
  real-child generator into test support after Bright Builds caught the original
  test file above its limit.
- Verification: focused flash and automation targets passed. Ordered Cargo
  format, Clippy, all-target build, and all-feature tests passed. Bright Builds,
  all 47 Bazel test targets, parity, progress, firmware build/package, redaction,
  reference cleanliness, file sizes, sensitive-value review, and diff checks
  passed. The bound reducer/model/consumer source changes rotate existing source
  inventory digests without membership drift.
- Evidence: deterministic tests prove both valid closure initiators produce
  accepted network truth only after final consumed and serial-finished state;
  both evidence consumers publish false; missing/non-boolean is rejected; and
  missing final consumed state still withholds publication.
- Outcome: software correction complete, committed, and pushed. No detector,
  credentials, protected attempt evidence, device, USB, network runtime, flash,
  monitor, mining, restart, projection, recovery, or attempt-003 was used.
- Blocker or next safe action: close without parity transition. A future
  immutable hardware plan may rotate the scoreboard workflow to attempt-003,
  package this exact correction, and run one detector-gated verification. The
  same closed boundary recurring after this targeted fix must stop without
  another retry.
