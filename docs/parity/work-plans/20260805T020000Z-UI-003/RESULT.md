# UI-003 work result

- Parity row: `UI-003`
- Final status: `implemented`
- Implementation commit: `4c31650d8810d7749891b31c05ffc68e5c35dce4`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none

## Evidence and verification

The pure button owner samples an active-low input, requires 30 ms of stable
level before admitting an edge, emits a short click on stable release, and
emits one long press at the exact 2,000 ms held boundary. Release after a long
press emits no short click. Regressed clocks preserve state and all deadline
arithmetic is checked.

The firmware retains one pull-up GPIO0 owner in a named thread at the bounded
10 ms cadence. A normal short click atomically cancels active identify state or
queues a bounded screen advance; the display owner consumes those requests at
its existing 500 ms boundary and records activity through the existing power
policy. A normal long press uses the sole retained Wi-Fi owner to toggle the
configuration AP, retains validated AP and optional station configuration,
admits captive DNS at most once, and publishes runtime state only after the
ESP-IDF configuration is accepted. Classifier, runtime-state, display, Wi-Fi,
snapshot, AP-address, DNS, configuration, and self-test-owner gaps fail through
closed categories without creating parallel effect owners.

The following gates passed on the implementation tree immediately before the
implementation commit:

- six focused pure input tests and fourteen focused screen-flow tests;
- firmware display and Wi-Fi source-ownership tests;
- the real ESP32-S3 firmware Bazel build;
- `cargo fmt --all`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo build --all-targets --all-features`;
- `cargo test --all-features`;
- `bun scripts/bright-builds-check.ts all` with zero findings;
- `just test` with all 34 Bazel test targets passing;
- `just parity` with no validation errors and `just parity-progress` reporting
  the pre-transition baseline of 39 of 94 active rows verified (41.5%);
- `just verify-redaction`, `just verify-reference`, immutable-plan, sensitive-
  log, reference-pin/cleanliness, and diff checks.

## Conclusion

The bounded active-low classifier, GPIO0 ownership, exact short/long routing,
atomic identify cancellation, manual screen advance and display wake, typed
configuration-AP toggle, closed failure isolation, and redaction-safe status
contract are implemented with unit and workflow evidence. This supports
transitioning `UI-003` from `in-progress` to `implemented` without a physical
button or live networking claim.

## Non-claims and residual evidence gap

`UI-003` remains below `verified`. No physical button press, exact LVGL input
timing, live configuration-AP transition, provisioning client, self-test reset
or cancellation, mining, ASIC traffic, voltage, frequency, fan, thermal, or
power behavior was exercised. No origin, hostname, SSID, address, port, USB
identity, credential, pool field, worker, device identifier, private frame, or
raw trace is included in this result.
