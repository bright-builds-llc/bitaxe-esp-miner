# Parity work result

- Parity row: `THR-002`
- Final status: `verified`
- Implementation commit: `ca81411093fb0a81e31cc556c07bd05a8d05343b`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; the accepted PWR-002 hardware-regression evidence
  was reused without another device effect

## Evidence and verification

The production firmware now starts one bounded fan-controller shell after the
production mining owner. Every 100 ms it reads only the confirmed typed
settings snapshot, the owner-published mining state, and fresh producer-owned
thermal truth. Its functional core applies the upstream priority order for
overheat, paused, no-pool, automatic, and manual modes; retains PID state;
validates all duty and temperature settings; suppresses unchanged duties; and
uses a 2000 ms retry delay after an unsuccessful write request.

The controller cannot authorize itself. A producer-owned atomic gate becomes
true only while the repo-owned production campaign retains its admitted lease,
the campaign state is `Active`, and both qualified safety and ASIC owners are
available. The production owner clears the gate before safe stop and on every
non-active or no-campaign publication. The shell sends only the typed
`SafetyActuationCommand::SetFanDuty` request through the existing bounded queue;
it owns no raw I2C, EMC2101 register, GPIO, voltage, ASIC, or recovery primitive.
Default boot and hardware preparation therefore remain non-authorizing.

Ten focused planner regressions prove the qualification boundary, exact 100 ms
cadence, mode priority, validated manual duty, automatic PID floor, PID-state
retention, unchanged-duty suppression and later reassertion, bounded retry,
invalid-settings rejection, and fail-closed invalid thermal truth. Production
source-ownership tests prove startup order, active-campaign qualification,
pre-safe-stop clearing, typed queue use, current settings and observation
inputs, and absence of raw actuator ownership. Both normal and rollback
ESP32-S3 firmware builds complete with the new scheduler.

The independently validated committed
`bitaxe-asic-power-initialization-evidence-v1` projection at
`docs/parity/evidence/pwr002-asic-power-initialization/power-initialization-projection.json`
has mode `0644` and SHA-256
`0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`.
It binds board 205, exact package and trusted runtime identity, fresh safety
before effects, a typed production 100% fan command before voltage, and a fresh
nonzero post-command fan RPM before successful initialization and an accepted
submit. It also binds safe stop, cleanup, no hardware rerun, and passed
redaction. Its immutable PWR-002 result has SHA-256
`199509d8f95dab4287f4d3c3a7b09b381823250ff990c7ee7ad1a612ffbf6b9c`.

The PWR-002 mining transaction, typed request queue, DS4432U, reset, and ASIC
ownership paths remain byte-identical from that result through this
implementation. The EMC2101 file changed only for the later Ultra 205
temperature offset; its fan-write trait, register addresses, three-write
sequence, and duty conversion are byte-identical. Focused EMC2101 tests still
prove tachometer decoding and the exact direct-mode full-duty register order.

The following gates passed on the implementation:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test` (all 42 Bazel test targets passed)
- `just parity`
- `just parity-progress`
- focused fan-controller, source-ownership, EMC2101, and production firmware
  build checks
- independent PWR-002 Rust evidence validation
- redaction, pinned-reference, immutable-plan, task-binding, source-lineage,
  exact-digest, file-mode, sensitive-output, and diff checks

## Conclusion

THR-002 has a closed composed proof. The current Rust production firmware owns
and schedules upstream-aligned fan-controller decisions behind the existing
campaign authorization and single I2C owner, while the sealed Ultra 205
hardware regression proves that the unchanged typed fan route applied full
duty and produced a fresh nonzero physical RPM response. The row's required
`unit,workflow,hardware-regression` evidence is complete without reflashing or
repeating a hardware effect.

## Non-claims and residual risks

This result does not independently measure analog RPM accuracy, a complete
dynamic speed curve, closed-loop thermal settling, extended soak, arbitrary
duty levels on hardware, injected fan stall or write failure, recovery from a
physically injected fault, overheat safe-stop behavior, another board, or
another thermal controller. The accepted physical run proves the bounded 100%
command and nonzero post-command response; automatic dynamic response and
fault injection remain explicit non-claims owned by their dedicated safety
rows. No detector, package, flash, reset, USB or serial session, network
request, credentials, mining rerun, fan/voltage/power effect, direct UART, or
pin interaction occurred during this plan.
