# Parity work plan

- Run ID: `20260804T160000Z-NET-002`
- Parity row: `NET-002`
- Initial status: `not-started`
- Source commit: `bb007f5f82c86437974616dea5701bfd2973a095`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-net002-provisioning-network`

## Selection

The deterministic selector reported no open plan after `SYS-005` closed. The
implemented candidates retain their previously audited hardware, broad
configuration/network/API, mining, safety-effect, recovery, other-board, or
installed-UI evidence gaps. The in-progress display, input, and statistics
rows likewise need physical/UI or live mining observations that the current
evidence does not supply. None can be promoted by reclassifying an existing
artifact.

`NET-002` is the first implementation-actionable candidate. The Rust firmware
currently starts only station mode and leaves networking unavailable when
credentials are missing. The pinned reference instead starts an open Ultra 205
configuration AP, retains it for absent or failed station setup, serves
wildcard IPv4 DNS answers for captive-portal discovery, and disables the AP
after successful station admission.

## Scope and non-scope

Add a pure, bounded DNS query parser and response builder in `bitaxe-api`.
Accept only standard IPv4 host queries, preserve the transaction and question
section, answer wildcard IN/A questions with the AP address and the pinned
300-second TTL, reject malformed or oversized packets, and emit no raw query
names or network identifiers.

Refactor the firmware Wi-Fi owner to derive the upstream-shaped configuration
SSID from the AP MAC, configure an open channel-1 AP with ten-client capacity,
start AP-only mode when credentials are absent, start mixed AP+STA mode when
credentials exist, retain provisioning after failed station admission, and
disable the AP after successful station admission. Bind a single boot-lifetime
UDP/53 owner to the AP address only while provisioning is active. Preserve the
existing station hostname, connected-origin, RSSI, and public snapshot paths.

Do not add Wi-Fi scanning, IPv6 reporting, reconnect-event orchestration,
credential logging, network discovery, external requests, live provisioning
client behavior, mining, voltage/fan/power effects, OTA, recovery, direct UART,
or pin manipulation. Do not copy GPL-covered implementation expression from
the reference DNS server; use only its observable wire and lifecycle behavior
as evidence.

## Implementation

- [ ] Add typed captive-DNS response planning with strict packet bounds,
      canonical header/question handling, wildcard IN/A answers, and complete
      malformed/non-A/nonstandard tests.
- [ ] Add the ESP-IDF AP/mixed-mode startup transaction and one UDP/53 owner,
      preserving provisioning on absent/failed station admission and disabling
      it after a successful connection.
- [ ] Add source-ownership and startup-state regressions, build the real
      ESP32-S3 firmware, and record focused plus repository-wide verification
      in `WORKLOG.md`.
- [ ] Create `RESULT.md` only if every bounded software acceptance criterion
      passes; transition only `NET-002` and leave live client evidence pending.

## Verification and promotion

Run focused `bitaxe-api` Cargo/Bazel tests, firmware source-ownership tests, and
the real ESP32-S3 firmware build. Then run, in order, `cargo fmt --all`, strict
all-target/all-feature Clippy, all-target/all-feature build, all-feature tests,
Bright Builds checks, `just test`, `just parity`, `just parity-progress`,
redaction, reference cleanliness, and diff checks.

Transition only `NET-002` from `not-started` to `implemented` with
`unit,workflow` evidence if packet handling, AP configuration, fallback state,
single DNS ownership, connected-mode AP shutdown, no-sensitive-output guards,
real firmware build, and all gates pass. A later detector-gated client session
must prove SSID visibility, DHCP reachability, wildcard DNS, captive redirect,
settings access, station handoff, and fallback behavior before `verified` can
be considered.
