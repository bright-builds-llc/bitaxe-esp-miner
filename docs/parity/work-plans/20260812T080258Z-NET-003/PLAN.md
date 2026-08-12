# Parity work plan

- Run ID: `20260812T080258Z-NET-003`
- Parity row: `NET-003`
- Initial status: `implemented`
- Source commit: `e97c0042358cee66f67d603fef321d1c5ca5b280`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-net003-live-scan-address-attempt-001`

## Selection

The branch and pinned reference are clean, synchronized, and conflict-free.
The canonical selector reports no open plan and ranks `NET-003` first, so no
candidate was skipped. The connected Ultra 205, owner Wi-Fi input, existing
bounded scan route, and newly stable on-device display make one read-only live
network observation actionable without mining or hardware-control actuation.

The row-local implementation is already complete: one retained ESP-IDF Wi-Fi
owner serializes scans, caps the response at 20 records, projects numeric auth
modes, restores AP-only mode when needed, requests a station link-local
address, and publishes station-only v6 observations. The remaining gap is live
proof that the exact package returns a production-shaped scan response while
remaining connected in the same boot and reports a valid, stable station v6
address through `/api/system/info`.

## Scope and non-scope

Add a typed `capture-network-scan-evidence` host transaction and a closed Rust
validator for `bitaxe-network-scan-evidence-v1`. After one exact-package
flash-monitor reaches a trusted origin, capture private system-info before and
after exactly one same-origin `GET /api/system/wifi/scan`. Require exact
source/reference/app identity, one boot session, connected client-only state,
monotonic uptime, one through 20 production-shaped records, bounded signal
strengths, supported numeric auth modes, and the same valid station v6 address
before and after the scan. Classify the address only as `link_local`,
`unique_local`, or `global` in public evidence.

The public projection may contain source/reference/package/workflow digests,
closed schema/category strings, aggregate record counts and scan duration,
and exact-build/same-session/connected/client-only/shape/auth/signal/address-
family/address-kind/stability/safety/cleanup/mode/redaction booleans. It must
not contain observed network names, addresses or address digests, hostnames,
origins, ports, interface or USB identities, BSSIDs, channels, credentials,
HTTP bodies, serial bytes, process traces, or display contents. All raw values
stay beneath the ignored mode-`0700` private attempt root in mode-`0600` files.

The only allowed effects are one exact normal package flash with the existing
opaque Wi-Fi seed, repo-owned USB reset/re-enumeration, passive serial receive,
same-origin HTTP reads, and exactly one bounded Wi-Fi scan request. On any
failure after the flash effect begins, the workflow may perform at most one
ordinary exact-package recovery flash with the same opaque Wi-Fi input;
recovery never creates success. Success requires no recovery because the scan
does not mutate settings or persistent state.

This plan does not authorize credential changes, router changes, network
association changes, arbitrary network discovery from the host, repeated
scans, erase-flash, raw writes, OTA, recovery upload, restart, power
interruption, foreign-process termination, mining, ASIC initialization or
work, pool traffic, voltage, frequency, fan, thermal or power control,
self-test, direct UART, pins, pads, headers, GPIO, probes, jumpers, soldering,
or injected signals. Global-v6 availability, hidden-network semantics,
multi-AP roaming performance, scan failure injection, other boards, and
release readiness remain non-claims.

The design keeps production radio and HTTP work in the imperative shell and
all response/address admission in pure helpers. It uses closed typed failures,
preserves the earliest failure through recovery, and follows the repository's
architecture, code-shape, verification, testing, Rust, TypeScript, privacy,
evidence, and autonomous hardware rules.

## Implementation

- [ ] Add the Rust-owned evidence schema, validator binary, generated
      TypeScript contract, command identity, Bazel wiring, and semantic
      redaction admission.
- [ ] Add the typed host capture, exact-package and detector plumbing, private
      modes, one-scan transaction, address/response admission, recovery
      precedence, and public projection publication.
- [ ] Add focused valid/failure/category/privacy tests, including real
      production-shaped scan data and link-local, unique-local, and global
      address classification without public raw values.
- [ ] Run the complete software gate, push the exact implementation, and build
      the normal package from that clean commit.
- [ ] Spend exactly one fresh detector and at most one conditional attempt-001.
- [ ] Independently validate and promote only `NET-003` when every exact-
      package, live scan, connection, address, safety, cleanup, mode, and
      redaction fact passes.

## Verification and promotion

Run focused automation, contract, HTTP, API, scan, firmware ownership, and
redaction targets, then in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`
9. `just verify-redaction`
10. `just verify-reference`
11. generated-contract, selector, immutable-plan, task-uniqueness,
    reference-cleanliness, sensitive-output, private-mode, fresh-path, and
    diff checks

After the exact implementation commit is clean and pushed, run exactly:

1. `bazel build //firmware/bitaxe:firmware_image`
2. `test ! -e scratch/net003-scan/wrapper-001 && (umask 077; mkdir -m 700 -p scratch/net003-scan/wrapper-001 && just detect-ultra205 > scratch/net003-scan/wrapper-001/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/net003-scan/attempt-001 && test ! -e docs/parity/evidence/net003-scan/network-scan-projection.json && (umask 077; just capture-network-scan-evidence --private-root scratch/net003-scan/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/net003-scan/wrapper-001/detector.stdout --projection docs/parity/evidence/net003-scan/network-scan-projection.json --capture-timeout-seconds 240 > scratch/net003-scan/wrapper-001/capture.stdout 2> scratch/net003-scan/wrapper-001/capture.stderr)`

Fresh wrapper, private, and public paths must be absent. Detector failure stops
before the flash; any conditional capture start consumes attempt-001. Accepted
non-success categories are `process_failed`, `timeout`, `hardware_blocked`,
`evidence_invalid`, and `service_recovery_failed`. Preserve the earliest typed
failure through resource cleanup and the one allowed recovery flash, release
every owned process and USB resource, withhold public evidence, and never retry
this ordinal.

Promotion requires exact source/reference/app identity, one detector-admitted
board 205, passive safe-state boot, one trusted origin, one boot session,
connected client-only system-info before and after one successful scan, one to
20 exact-shape records, valid signal and numeric auth values, a valid and
unchanged station v6 address classified without publication, monotonic uptime,
disabled mining and hardware control, complete cleanup, private modes,
redaction, independent Rust validation, and all gates passing. Otherwise
create a truthful closure, keep `NET-003` at `implemented`, and select no fresh
ordinal until a targeted fix or new diagnostic information exists.
