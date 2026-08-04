# UI-001 work result

- Parity row: `UI-001`
- Final status: `implemented`
- Implementation commit: `3cc76504b44c36e502c24bc40fc9281cc173c78c`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none

## Evidence and verification

The pure display contract admits only the exact Ultra 205 SSD1306 128x32
panel, the four upstream rotations, the two inversion states, and bounded
upstream timeout semantics. The edge-triggered power policy handles always-on,
activity-only wake, positive inactivity timeouts, priority visibility, exact
boundaries, and regressed clocks without repeatedly issuing physical commands.

The configuration adapter projects the confirmed NVS snapshot without schema
coercion, applies upstream defaults only to missing or empty defaultable values,
and fails closed for malformed storage or unsupported values. Firmware reads
that snapshot before initializing the panel, retains one logical display owner
for configuration, rendering, and power, and uses the shared absolute-deadline
scheduler. A display failure disables only display work and preserves sensor
acquisition.

The following gates passed on the implementation tree and immediately before
the implementation commit:

- twelve focused core display tests;
- four focused configuration projection tests;
- display command-order and production ownership Bazel tests;
- the real ESP32-S3 firmware Bazel build;
- `cargo fmt --all`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo build --all-targets --all-features`;
- `cargo test --all-features`;
- `bun scripts/bright-builds-check.ts all` with zero findings;
- `just test` with all 34 Bazel test targets passing;
- `just parity` with no validation errors and `just parity-progress` reporting
  the pre-transition baseline of 39 of 94 active rows verified (41.5%);
- `just verify-redaction`, `just verify-reference`, immutable-plan, and diff
  checks.

## Conclusion

Exact configuration projection, one-time driver initialization, retained
runtime ownership, absolute refresh cadence, edge-only power service, and
failure isolation are implemented with unit and workflow evidence. This
supports transitioning `UI-001` from `in-progress` to `implemented` without a
physical display claim.

## Non-claims and residual evidence gap

`UI-001` remains below `verified`. No live panel orientation, inversion,
timeout, wake, brightness, rendering, operator-visible behavior, physical
button, mining, ASIC traffic, pool interaction, voltage, fan, thermal, or power
behavior was exercised. UI-002 owns carousel and notification content; UI-003
owns physical input behavior. No origin, hostname, SSID, address, port, USB
identity, credential, pool field, worker, device identifier, or raw trace is
included in this result.
