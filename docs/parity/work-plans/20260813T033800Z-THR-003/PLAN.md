# Parity work plan

- Run ID: `20260813T033800Z-THR-003`
- Parity row: `THR-003`
- Initial status: `implemented`
- Source commit: `1150fcd113226904df1414269a8e7929bee24ed1`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-thr003-pid-controller`

## Selection

The clean synchronized selector returned no open plan and ordered `API-009`,
`THR-001`, then `THR-003`. `API-009` is sealed at
`stop_repeated_boundary`: its exact post-fix command-effect boundary recurred,
attempt-008 is prohibited, and its next eligible action is a new software-only
diagnosis after new information. `THR-001` requires a distinct bounded
overheat/fault hardware-regression contract, but the repository currently owns
no vendor-safe stimulus command; ordinary mining cannot guarantee a bounded
75 C transition without unsafe stress, and electrical overstress, direct UART,
pins, pads, and injected signals remain prohibited. Neither row is actionable
in this invocation.

`THR-003` is the first actionable row. The pinned source defines a stateful
100 ms, reverse-direction, proportional-on-error PID with sample-scaled gains,
input EMA, initialization, output-limit updates, and output anti-windup. The
current Rust core instead applies unscaled positive gains to an output EMA and
its fixture covers fixed fan modes without sequential PID vectors. The
repository evidence ledger defines THR-003 as pure PID behavior whose math is
proved by unit/golden evidence; hardware fan behavior stays a separate claim.

This plan follows the repository task/evidence policy and the Bright Builds
architecture, code-shape, verification, testing, and Rust standards: keep the
controller pure, keep the production scheduler and I/O adapter thin, encode
state explicitly, and test each behavior at the cheapest authoritative
boundary.

## Scope and non-scope

Implement the pinned PID state machine in the pure safety core and consume it
from the existing 100 ms production fan scheduler. Preserve the existing typed
single-owner actuation queue and every THR-002 safety gate. Replace the shallow
fixture with provenance-bound sequential vectors covering first-sample
initialization, input EMA, reverse P-on-E gains, 100 ms integral/derivative
scaling, dynamic minimum limits, upper/lower clamps, anti-windup, retained
state, and the upstream zero-minimum 255-internal-limit edge. Record both the
raw PID output/state and the existing whole-percent adapter decision so the
pure math is not hidden by actuation quantization.

This is software-only. It permits source, fixture, tests, firmware builds,
committed evidence/result composition, typed checklist transition, and
repository verification. It permits no detector, package capture, flash,
reset, USB/serial, network, credential, mining, voltage, fan, power, I2C/GPIO,
fault injection, direct UART, pin, pad, or other hardware effect.

The accepted PWR-002 projection may be reused only for the already-qualified
typed fan-actuation chain. Its SHA-256 must remain
`0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`,
mode `0644`, independently valid, redacted, and source-compatible. It does not
prove live automatic PID settling. No new evidence schema, projector, or
hardware attempt is allowed.

## Implementation

- [ ] Replace the simplified PID reducer with the pinned stateful 100 ms
      reverse P-on-E algorithm, input EMA, limit updates, and anti-windup.
- [ ] Expand the provenance-bound golden fixture and focused tests across all
      state transitions and production planner retention.
- [ ] Prove the production scheduler remains at 100 ms and the typed actuator
      ownership, qualification, retry, and fail-closed boundaries are
      unchanged.
- [ ] Compose a row-specific result from the golden/workflow proof and the
      unchanged accepted PWR-002 hardware-regression boundary.

## Verification and promotion

Focused verification must include `//crates/bitaxe-safety:tests`,
`//firmware/bitaxe:fan_controller_plan_tests`,
`//firmware/bitaxe:sensor_source_ownership_tests`, normal and rollback firmware
builds, fixture provenance and exact-vector checks, and an explicit source
comparison to the pinned `PID.c`, `PID.h`, and fan-controller task. The
accepted PWR-002 Rust validator, exact projection/result digests, mode,
ancestry/source compatibility, repository redaction, reference cleanliness,
task/plan uniqueness, sensitive-output absence, and diff checks must pass.

Run the mandatory sequence in order before each commit boundary:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Promotion requires exact pure PID agreement, the production 100 ms workflow,
the unchanged typed actuation ownership boundary, a valid row-specific
`RESULT.md`, and the accepted PWR-002 hardware-regression artifact only as the
physical actuator-chain evidence required by the safety checklist guard. The
transition may change only THR-003 to `verified`, evidence to
`unit,golden,workflow,hardware-regression`, its Rust-owned targets, and its
notes. Live automatic closed-loop response, analog RPM accuracy, thermal
settling, tuning quality, arbitrary duties, fault/overheat behavior, other
boards/controllers, and release readiness remain explicit non-claims. Any
failed gate leaves THR-003 `implemented`, withholds transition/progress, and
records the earliest blocker without hardware action.
