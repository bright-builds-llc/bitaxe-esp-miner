# Parity work plan

- Run ID: `20260804T170000Z-NET-003`
- Parity row: `NET-003`
- Initial status: `not-started`
- Source commit: `f052c59a6ce155bfa034b6c777da3de7b9ba9b61`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-net003-scan-ipv6`

## Selection

The deterministic selector reported no open plan after `NET-002` closed.
Implemented candidates still require their separately documented hardware,
configuration, API, mining, safety-effect, release, or recovery evidence.
In-progress display, input, and statistics rows likewise cannot be promoted by
reclassifying existing artifacts. `NET-003` is the first remaining
implementation-actionable not-started row.

The pinned reference exposes a private-network-gated
`GET /api/system/wifi/scan` endpoint returning at most 20 visible networks with
`ssid`, `rssi`, and numeric `authmode`. Its Wi-Fi owner serializes scans and
reports failures as `WiFi scan failed`. After station IPv4 admission, it creates
a link-local IPv6 address and publishes `IP_EVENT_GOT_IP6` observations to the
existing system-info `ipv6` field, adding an interface zone only to link-local
addresses.

## Scope and non-scope

Add pure, bounded scan wire types and explicit ESP-IDF-compatible auth-mode
mapping in `bitaxe-api`. Add pure IPv6 projection that distinguishes
link-local addresses from global/ULA addresses and appends a positive interface
index only when link-local.

Retain the ESP-IDF Wi-Fi object behind one boot-lifetime owner. Admit one scan
at a time without waiting behind an active scan, use `scan_n::<20>`, temporarily
enable the station interface when provisioning is AP-only, and restore the
original AP configuration even when scanning fails. Register the existing
access-gated scan route and return the exact public success/error shape without
logging SSIDs, BSSIDs, addresses, peers, or response bodies.

After successful station IPv4 admission, request an IPv6 link-local address and
retain one typed `IpEvent` subscription. Accept only IPv6 events for the station
netif, derive the implementation index from that exact netif, update the shared
Wi-Fi snapshot, and emit category-only diagnostics. Preserve existing IPv4,
hostname, RSSI, configuration AP, captive DNS, and connected-origin behavior.

Do not add live network discovery evidence, external requests, credential
handling changes, long-running reconnect orchestration, mining, ASIC traffic,
voltage/fan/power effects, OTA, recovery, direct UART, or pin manipulation.
Never place observed SSIDs, BSSIDs, IPv4/IPv6 values, interface identifiers, or
credentials in committed evidence.

## Implementation

- [ ] Add bounded scan response and IPv6 projection contracts with focused
      Arrange/Act/Assert unit and serialization tests.
- [ ] Add exclusive Wi-Fi scan ownership, AP-only temporary mixed-mode
      admission, exact restoration, and category-only failure handling.
- [ ] Register the access-gated HTTP route and station-only IPv6 event
      publication without exposing network identifiers in logs.
- [ ] Add source-ownership regressions, build the real ESP32-S3 firmware, and
      record focused plus repository-wide verification in `WORKLOG.md`.
- [ ] Create `RESULT.md` only if every bounded software criterion passes;
      transition only `NET-003` and leave radio/API hardware evidence pending.

## Verification and promotion

Run focused `bitaxe-api` Cargo/Bazel tests, firmware source-ownership tests, and
the real ESP32-S3 firmware build. Then run, in order, `cargo fmt --all`, strict
all-target/all-feature Clippy, all-target/all-feature build, all-feature tests,
Bright Builds checks, `just test`, `just parity`, `just parity-progress`,
redaction, reference cleanliness, sensitive-identifier scans, and diff checks.

Transition only `NET-003` from `not-started` to `implemented` with
`unit,workflow` evidence if the bounded wire contract, one-at-a-time scan,
AP-mode restoration, access-gated route, station-only IPv6 publication,
category-only diagnostics, real firmware build, and all gates pass. A later
detector-gated session must prove visible scan results, failure behavior,
connection preservation, and link-local/global IPv6 API observations before
`verified` can be considered.
