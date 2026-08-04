# NET-003 work result

- Parity row: `NET-003`
- Final status: `implemented`
- Implementation commit: `9b6375c5102d9e1389c918318c8773e3975cc6ff`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none

## Evidence and verification

The pure API contract bounds scan responses to 20 networks, preserves the
upstream numeric ESP-IDF authentication-mode values, and projects station IPv6
addresses with an interface zone only for link-local addresses. Its fixtures
use synthetic network names and contain no observed radio, endpoint, address,
credential, port, or device data.

The firmware retains its sole ESP-IDF Wi-Fi driver behind one boot-lifetime
owner. Scan admission is exclusive and non-waiting. An AP-only owner is moved
temporarily into mixed mode for the scan and restored to its original AP-only
configuration afterward, including when scanning fails. The access-gated HTTP
route returns the bounded public response on success and the exact upstream
`WiFi scan failed` body on an internal failure; diagnostics expose only closed
failure categories.

The boot-lifetime IPv6 subscription filters events to the exact station
network interface, requests a link-local address, projects link-local and
global assignments into the existing public system-info field, and never logs
the address. The runtime snapshot carries that projection without introducing
a second Wi-Fi owner.

The following gates passed on the implementation commit:

- five focused scan/auth-mode/IPv6 contract tests;
- six firmware source-ownership, route, restoration, lifecycle, and privacy
  tests;
- 23 focused API-comparator tests using a checked-in synthetic scan fixture;
- real `xtensa-esp32s3-espidf` firmware builds through `just build`;
- `cargo fmt --all`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo build --all-targets --all-features`;
- `cargo test --all-features`;
- `bun scripts/bright-builds-check.ts all`;
- `just test` with all 30 Bazel test targets passing;
- API comparison with 107 schema, 53 captured-response, and 38 static-route
  checks and no validation errors;
- `just parity` and `just parity-progress`;
- `just verify-redaction`, `just verify-reference`, and `git diff --check`.

## Conclusion

Bounded Wi-Fi scanning, explicit authentication-mode reporting, AP-only scan
restoration, and station IPv6 publication are implemented with unit, workflow,
firmware-build, and synthetic API-comparison evidence. The implementation adds
no dependency and preserves the existing functional-core and imperative-shell
boundary.

## Non-claims and residual evidence gap

`NET-003` remains `implemented`, not `verified`. No connected device, observed
network, or live IPv6 assignment was used. The evidence therefore does not
prove radio scan results, connection preservation during a scan, authentication
values from real access points, station link-local/global IPv6 assignment, or
the live HTTP response. Credentials, external requests, mining, ASIC traffic,
voltage/fan/power effects, OTA, recovery, direct UART, and pins were not used or
claimed.
