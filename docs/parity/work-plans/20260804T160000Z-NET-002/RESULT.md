# NET-002 work result

- Parity row: `NET-002`
- Final status: `implemented`
- Implementation commit: `126cf24b7205f3bdac0a0a70fc022a422973c392`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none

## Evidence and verification

The pure configuration-network contract derives the upstream-shaped
`Bitaxe_XXXX` SSID from the final two AP MAC octets and constructs bounded DNS
responses without parsing names into logs. Standard IN/A questions receive the
AP IPv4 address with the pinned 300-second TTL. Non-A queries receive an empty
standard response, responses and nonstandard opcodes are ignored, and malformed,
compressed, reserved-label, truncated, over-count, or oversized packets fail
through closed categories.

The firmware's sole Wi-Fi owner now initializes the ESP-IDF network stack even
without station credentials. It configures an open, visible, channel-1 AP with
ten-client capacity; uses AP-only mode for missing or invalid credentials; uses
mixed AP+STA mode during station admission; retains the AP and captive DNS when
station admission fails; and switches to station-only mode after success. The
existing hostname, connected-origin, RSSI, public snapshot, and production
readiness behavior remain on the successful station path.

A boot-lifetime UDP/53 thread owns the captive DNS socket only while the
configuration network is retained. Its diagnostics contain closed categories
and I/O error kinds, never query names, peer addresses, response bytes, AP
identifiers, credentials, or network endpoints.

The following gates passed on the implementation commit:

- eight focused configuration-SSID and DNS packet tests;
- four firmware source-ownership and lifecycle-order tests;
- focused `bitaxe-api` Cargo and Bazel tests;
- real `xtensa-esp32s3-espidf` firmware builds through `just build`;
- `cargo fmt --all`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo build --all-targets --all-features`;
- `cargo test --all-features`;
- `bun scripts/bright-builds-check.ts all`;
- `just test` with all 30 Bazel test targets passing;
- `just parity` and `just parity-progress`;
- `just verify-redaction`, `just verify-reference`, identifier scans, and
  `git diff --check`.

## Conclusion

Configuration SoftAP startup, station-admission fallback, connected-mode AP
shutdown, and bounded wildcard captive DNS are implemented with unit and
workflow evidence. The implementation adds no new dependency and preserves the
functional-core/imperative-shell boundary.

## Non-claims and residual evidence gap

`NET-002` remains `implemented`, not `verified`. No connected device or
provisioning client was used, so SSID visibility, association, DHCP, wildcard
DNS over the radio, captive redirect, settings access, station handoff, failed
credential fallback, and later reconnect behavior are unproved. Wi-Fi scanning,
IPv6, credentials, external requests, mining, voltage/fan/power effects, OTA,
recovery, direct UART, and pins were not used or claimed.
