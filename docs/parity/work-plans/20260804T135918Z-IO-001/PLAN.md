# Parity work plan

- Run ID: `20260804T135918Z-IO-001`
- Parity row: `IO-001`
- Initial status: `in-progress`
- Source commit: `61170d8f61d2702366e6b7a88f18f1e0fbde556e`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-io001-i2c-retry-contract`

## Selection

The deterministic selector reported no open plan. Earlier `implemented`
candidates retain documented live hardware, network, mining, safety, release,
other-board, or browser evidence gaps. `IO-001` is the first bounded
`in-progress` software gap: the firmware already owns I2C0, pins 47/48, 400 kHz
configuration, display traffic, sensor reads, and safety-controlled writes, but
its transfers use one 50 ms attempt. The pinned reference uses a 500 ms timeout,
three attempts, and a 10 ms delay after each failed attempt.

## Scope and non-scope

Add one host-testable retry contract with the exact pinned timeout, attempt
count, and delay. Route display reads/writes/transactions, INA260 and EMC2101
reads, and EMC2101 and DS4432U writes through that single helper while retaining
the existing closed address capabilities and one runtime I2C owner. Make the
internal pull-up configuration explicit and keep I2C0, GPIO 47/48, and 400 kHz
as compile-time contract facts.

Add deterministic tests for first-attempt success, eventual success, terminal
failure, delay count, final-error preservation, and exact constants. Extend the
source-ownership guard so no direct firmware transfer bypass remains. Build the
real ESP-IDF firmware target to prove the adapter integration.

Do not enable mining, change voltage or fan policy, add a device probe, invent
device-map state, change sensor cadence, run hardware, or claim live retry/fault
behavior. Compile-time address capabilities replace the reference's runtime
device map without weakening ownership. Existing hardware-smoke breadcrumbs
remain historical startup evidence only.

## Implementation

- [ ] Add the pure retry policy and focused host tests with exact reference
      constants and terminal behavior.
- [ ] Apply the helper to every display, sensor, and actuation I2C transfer and
      retain contextual final errors.
- [ ] Extend source-ownership tests to reject direct transfer bypasses and build
      the real ESP-IDF firmware target.
- [ ] Record `RESULT.md` only after focused and repository-wide gates pass.

## Verification and promotion

Run the focused retry and ownership Bazel targets plus the real firmware build,
then the mandatory ordered Rust checks, Bright Builds, `just test`,
parity/progress, redaction, reference cleanliness, and diff checks. Transition
only `IO-001` to `implemented`, preserving `unit,workflow,hardware-smoke`
evidence while stating that the hardware breadcrumb predates the retry change.
Do not mark it `verified` without detector-gated live transient-fault and shared
bus evidence. No hardware or recovery contract exists in this plan.
