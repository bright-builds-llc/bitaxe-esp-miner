# Parity work plan

- Run ID: `20260821T020549Z-BAP-001`
- Parity row: `BAP-001`
- Initial status: `not-started`
- Source commit: `b7336d50204e7e3c6f8152e6b9d14838c4d1ed0e`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-bap001-firmware-interface`

## Selection

The clean synchronized selector returned no open plan and ordered `ASIC-009`,
`ASIC-010`, `SELF-001`, `BAP-002`, then `BAP-001`.

`ASIC-009` and `ASIC-010` have complete pure protocol cores but require
unavailable supported BM1368/BM1397 boards, firmware adapters, detector-gated
hardware regressions, safe-stop evidence, and redaction before another status
transition. `SELF-001` remains dependency- and safety-blocked because no
production-safe complete self-test route or accepted hardware regression
exists. `BAP-002` has a complete pure protocol core but cannot reach verified
without the `BAP-001` firmware owner and live accessory interoperability.

`BAP-001` is the first actionable row. Its missing firmware ownership,
initialization, bounded ingress, request dispatch, and subscription lifecycle
can be implemented and compile-verified without opening a device session,
connecting an accessory, manipulating pins, or claiming electrical behavior.

## Scope and non-scope

Add an independently designed firmware BAP adapter that owns ESP-IDF UART2 on
the pinned reference defaults: GPIO39 transmit, GPIO40 receive, 115200 baud,
eight data bits, no parity, one stop bit, no flow control, and bounded receive
buffers. Use one long-lived owner task rather than shared global queues and
mutexes. The owner must emit the reference initialization banner, preserve
bounded frame assembly, pass complete frames through `BapIngress` and
`plan_command`, write canonical responses, and own AP-mode announcements plus
subscription renewal, change detection, timeout, and cadence.

Keep the runtime split into a pure, host-testable lifecycle core and a thin
ESP-IDF UART shell. Build request/subscription projections only from existing
runtime and settings snapshots. Translate pure setting intents through an
explicit adapter boundary; unsupported or unavailable effect paths fail closed
without echoing credential-bearing values. Logs and diagnostics may contain
only command/category metadata, never raw frames, SSIDs, passwords, pool
fields, users, endpoints, device addresses, or secret setting values.

Wire the adapter into startup with exclusive ownership of `uart2`, GPIO39, and
GPIO40. Initialization failure must be logged as unavailable and must not stop
safe boot, Wi-Fi, HTTP, monitoring, or the mining owner. Do not change the ASIC
UART1 owner, USB serial, I2C, GPIO0 input, safety controls, or boot ordering.

This plan authorizes repository source, fixture, test, documentation, build,
commit, and push work only. It does not authorize detector use, USB/device
access, flash, monitor, accessory attachment, direct external UART, physical
pins/pads/headers, credentials, network discovery, mining, ASIC traffic,
setting mutation on hardware, voltage/frequency/fan/power effects, restart,
fault injection, OTA, or recovery.

## Implementation

- [ ] Add the pure single-owner BAP runtime lifecycle for bounded ingress,
      mode changes, subscription cadence/timeouts, change-only delivery, and
      redaction-safe failures.
- [ ] Add the thin ESP-IDF UART2 adapter and startup ownership for the pinned
      GPIO39/GPIO40 115200/8-N-1 contract.
- [ ] Add behavior-focused unit/source-ownership tests and current firmware
      build evidence while preserving the existing BAP-002 protocol contract.
- [ ] Record commit-bound evidence and transition only `BAP-001` to at most
      `implemented`; leave live accessory interoperability below verified.

## Verification and promotion

Run focused lifecycle and source-ownership tests, the `bitaxe-core` BAP tests,
and the canonical ESP32-S3 firmware build/package. Then run, in order,
`cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`,
`cargo build --all-targets --all-features`, `cargo test --all-features`,
`bun scripts/bright-builds-check.ts all`, `just test`, `just parity`, and
`just parity-progress`, followed by `just verify-redaction`,
`just verify-reference`, sensitive-value review, file-size review, and
`git diff --check`.

Advance `BAP-001` from `not-started` to `implemented` with `unit,workflow`
evidence only if the single-owner lifecycle, exact UART configuration,
startup wiring, bounded parser/dispatcher, subscription behavior, safe failure
handling, canonical firmware build/package, and every repository gate pass.
`verified` additionally requires a separately task-gated live accessory,
detector-gated named-board request/response and subscription evidence,
cleanup, privacy review, and redaction; none are authorized or claimed here.
