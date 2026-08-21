# BAP-001 Firmware Interface Evidence

## Provenance

| Field | Value |
| --- | --- |
| Parity row | `BAP-001` |
| Immutable plan | `docs/parity/work-plans/20260821T020549Z-BAP-001/PLAN.md` |
| Plan SHA-256 | `a4c7f4b8bc317a798596aaecadc5e357b408a06210bc51d2a11f964199d448cc` |
| Plan commit | `9c7a386c19c373e6ee3e0df8e0557e808b8fbb42` |
| Implementation commit | `80f88df1799be63e3a71c291ad015f89c65cd8ae` |
| Reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| Hardware attempts | none |

## Implemented behavior

The firmware owns one ESP-IDF UART2 driver configured from the pinned
reference defaults: GPIO39 transmit, GPIO40 receive, 115200 baud, eight data
bits, no parity, one stop bit, no flow control, and bounded 1,024-byte driver
buffers. Startup transfers those exact peripherals once and treats adapter
initialization failure as unavailable without replacing safe boot.

One named owner thread emits the reference initialization banner, assembles
fragmented input only between `$` and a line terminator, discards overlong
frames, and routes complete bounded messages through the existing `BapIngress`
and `plan_command` functional core. The pure lifecycle owns duplicate
suppression, AP/connected mode, five-second AP announcements, subscription
renewal, absolute due times, coalescing, five-minute expiry, unsubscribe, and
deferred setting acknowledgements.

Read requests and supported subscription updates project existing operator
runtime truth. Change-only subscription delivery compares canonical in-memory
frames and never logs their values. Setting intents do not receive an
acknowledgement: the firmware returns the closed `set_failed` response because
no BAP-specific persistence, restart, or safety-critical actuation owner is
authorized by this plan. Diagnostics contain only command/failure categories
and setting names, never raw frames or values.

## Verification

The following passed against the implementation:

- four focused pure lifecycle tests for fragmented/oversized framing,
  subscription renewal/cadence/timeout/unsubscribe, deferred setting
  acknowledgement, and coalesced AP announcements;
- two source-ownership/privacy tests proving one UART2 owner, the exact pin and
  115200/8-N-1 configuration, startup wiring, core handoff, and value-free
  diagnostics;
- all twelve existing BAP protocol-core tests;
- `bazel test //firmware/bitaxe:bap_runtime_tests //firmware/bitaxe:bap_source_ownership_tests //crates/bitaxe-core:tests`;
- `just package`, producing the canonical six-file Ultra 205 package;
- `cargo fmt --all`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo build --all-targets --all-features`;
- `cargo test --all-features`;
- `bun scripts/bright-builds-check.ts all` with zero findings;
- `just test`, with all 50 Bazel tests passing;
- `just parity` with no validation errors and `just parity-progress`;
- `just verify-redaction`, `just verify-reference`, file-size review,
  sensitive-value review, and `git diff --check`.

## Conclusion and non-claims

`BAP-001` has a Rust-owned firmware initialization, UART, request, and
subscription lifecycle and supports `implemented` with `unit,workflow`
evidence. No accessory, detector, device session, USB transaction, flash,
monitor, physical UART connection, pin manipulation, credentials, external
network, mining, ASIC traffic, setting mutation, hardware control, restart,
fault injection, OTA, or recovery was used.

This evidence does not verify electrical UART behavior, live request/response
exchange, live subscriptions, accessory interoperability, Wi-Fi-password
subscription delivery, setting persistence, restart behavior, or any
safety-critical actuation. Those require a separately authorized task,
detector-gated named hardware and accessory, protected evidence, cleanup, and
redaction before `verified` is eligible.
