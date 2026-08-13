# Parity work result

- Parity row: `THR-003`
- Final status: `verified`
- Implementation commit: `91158f51831e87bcd0fbab2a29b9f2219904b1da`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; the accepted PWR-002 hardware-regression evidence
  was reused without another device effect

## Evidence and verification

The pure safety core now implements the pinned ESP-Miner fan PID as one
explicit state transition per production 100 ms schedule slot. It retains the
pinned C `float` precision and transition order: initial `0..255` limits,
minimum-limit change detection, the task-owned temperature-input EMA with
alpha `0.2`, automatic-mode initialization, reverse-direction P-on-error gains,
100 ms integral and derivative scaling, output-sum clamping, upper/lower output
clamping with the pinned anti-windup adjustments, and retained last input and
output. The prior unscaled positive-gain/output-EMA approximation is removed.

The production fan planner still derives its cadence directly from
`PID_SAMPLE_TIME_MS`, retains the pure PID state across iterations, and converts
the raw floating-point output to the existing validated whole-percent effect.
Its campaign qualification gate, mode priority, typed single-owner safety
actuation queue, unchanged-duty suppression, bounded retry, invalid-input
closure, and absence of direct I2C/GPIO/EMC2101 ownership are unchanged.

The provenance-bound golden fixture contains five independent sequences and
12 scheduled transitions. Every step asserts the exact promoted C-float value
for filtered input, raw output, output sum, current limits, retained last input,
and automatic state, plus the Rust adapter's whole-percent duty. The sequences
cover first-sample initialization, input EMA and derivative history, reverse
P-on-error/sample-scaled gains, retained state, dynamic minimum changes, upper
and lower clamps, both anti-windup corrections, and the initial zero-minimum
`0..255` internal ceiling. A focused production regression proves that the
same retained state yields the pinned 75 then 85 percent decisions across two
100 ms iterations.

The immutable plan SHA-256 is
`3acea362f65f63ccab564b1d4af98a22f4f026dffecf258a5a5d70ca119e0348`.
The final pure PID source SHA-256 is
`825b46821cb688f0ea40b19313301371661d11616ce48fac5d9ebe9ca18d06ad`;
the golden fixture SHA-256 is
`38e9db8c613419f35f3ee04940e38f16f79bbeaf70a0f9b7626d4db3b8f54456`;
and the production planner SHA-256 is
`2f8fd12ebeb79be939de4131ce4ddfa6f5db374b90f36bee070e308616e7c871`.

The independently validated committed
`bitaxe-asic-power-initialization-evidence-v1` projection at
`docs/parity/evidence/pwr002-asic-power-initialization/power-initialization-projection.json`
has mode `0644` and SHA-256
`0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`.
It binds board 205, a typed production 100% fan command before voltage, a fresh
nonzero post-command fan RPM, safe stop, cleanup, and passed redaction. Its
immutable PWR-002 result SHA-256 is
`199509d8f95dab4287f4d3c3a7b09b381823250ff990c7ee7ad1a612ffbf6b9c`.
The projection remains independently valid and source-compatible. It is used
only as physical evidence for the unchanged typed actuator chain required by
the safety checklist guard; it is not evidence of live automatic PID response.

The following gates passed on the final implementation and evidence source:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test` (all 42 Bazel test targets passed)
- `just parity`
- `just parity-progress`
- focused safety-core, fan-controller planner, and sensor-ownership targets
- normal and rollback-probe ESP32-S3 firmware builds
- independent PWR-002 Rust evidence validation
- redaction, pinned-reference, immutable-plan, task/plan uniqueness,
  source-lineage, exact-digest, file-mode, sensitive-output, and diff checks

## Conclusion

THR-003 has a closed composed proof. The Rust functional core matches the
pinned stateful 100 ms fan PID, including C-float boundaries, and the existing
production scheduler retains and consumes that state through the already
qualified typed actuation route. The required
`unit,golden,workflow,hardware-regression` evidence is complete without
reflashing, resetting, mining, or applying another hardware effect.

## Non-claims and residual risks

This result does not prove live automatic closed-loop response, analog RPM
accuracy, thermal settling, tuning quality, arbitrary duty levels on hardware,
extended soak, physically injected fan stall or write failure, overheat/fault
behavior, another board, another thermal controller, or release readiness.
The accepted hardware run proves only the unchanged bounded actuator chain at
100% duty and a fresh nonzero RPM response. No detector, package, flash, reset,
USB or serial session, network request, credentials, mining rerun, voltage,
fan, power, I2C/GPIO, direct UART, pin, or pad effect occurred during this plan.
