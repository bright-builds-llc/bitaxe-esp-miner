# Parity work plan

- Run ID: `20260812T025425Z-NET-001`
- Parity row: `NET-001`
- Initial status: `implemented`
- Source commit: `8673c9089ee9f31542d8847b104d04509c33c681`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-net001-reconnect-lifecycle-attempt-001`

## Selection

The branch and pinned reference are clean and synchronized, and the canonical
selector reports no open plan and lists `NET-001` first. No row was skipped.
The connected Ultra 205 and owner Wi-Fi input make this row actionable without
mining or hardware-control actuation.

Inspection confirmed the row-local gap. The Rust adapter applies the persisted
station credentials and hostname and either reaches a connected boot or falls
back once to the configuration AP, but it installs no post-boot station-
disconnect lifecycle. The pinned reference enables the configuration AP on a
disconnect, publishes a reason and retry ordinal, waits 5,000 ms, retries,
resets the ordinal after DHCP, and suppresses a retry while a provisioning
client is attached. Existing boot evidence cannot prove this boundary.

## Scope and non-scope

Add a pure reconnect state machine and a thin ESP-IDF event/worker adapter.
Handle station disconnect, roaming, configuration-AP client attachment,
bounded retry scheduling, connection-launch failure, and IPv4 reacquisition.
Expose redaction-safe reason categories and retry ordinals; retain the sole
Wi-Fi driver owner. On an ordinary disconnect, enable the existing
configuration AP and captive DNS immediately, wait the upstream 5,000 ms,
issue one nonblocking station reconnect request, repeat while disconnected,
and return to client-only mode after DHCP. Notify the production coordinator
of network loss and recovery without enabling mining.

Add a one-shot private `netreconprobe` NVS marker to the exact evidence flash.
After HTTP readiness, firmware must erase and commit the marker before it arms
the probe, then disconnect only its own station association once. The real
event lifecycle must record fallback, the 5,000-ms retry boundary, DHCP
recovery, and 15,000 ms of stable post-reconnect service. Rebooting before the
marker is consumed may retrigger the probe, so failure recovery may perform one
exact-package flash with the ordinary Wi-Fi seed and no marker. Recovery never
creates success.

Add a typed `capture-network-reconnect-evidence` host workflow. It must admit
the detector output and exact package, run the probe-bearing flash-monitor
transaction once, capture returned serial bytes directly into the protected
attempt root, derive exactly one same-session origin, require exact retained
probe markers and a final HTTP system-info/log quorum, and emit only the closed
redacted `bitaxe-network-reconnect-evidence-v1` projection. Public evidence may
contain provenance digests, reason category, retry/stability durations and
ordinals, equality/connected/fallback/safety/cleanup/mode booleans, and no raw
operational values.

The private transaction may contain Wi-Fi values, hostnames, device/network/
USB/process identities, origins, HTTP bodies, commands, and serial bytes; none
may enter public output, committed evidence, diagnostics, or debug formatting.
This plan does not authorize router changes, RF suppression, credential
mutation after boot, network discovery, erase-flash, arbitrary raw writes,
OTA, recovery upload, power interruption, foreign-process termination,
mining, ASIC initialization or work, pool traffic, voltage, frequency, fan,
thermal or power control, self-test, direct UART, pins, pads, headers, GPIO,
probes, jumpers, soldering, or injected signals. `NET-002`, `NET-003`, other
boards, multi-AP roaming performance, and release readiness remain non-claims.

The design follows repo-local task, USB, privacy, and evidence guidance plus
the Bright Builds architecture, code-shape, verification, testing, and Rust
standards: pure decisions stay outside the ESP-IDF shell, boundary inputs are
typed, nullable names remain explicit, and behavior tests use focused
Arrange/Act/Assert structure.

## Implementation

- [ ] Add and unit-test the pure reconnect state machine, including repeated
      cycles, reason classification, the exact delay, roaming, AP-client
      suppression/resumption, launch failure, DHCP reset, and overflow-safe
      ordinals.
- [ ] Add the thin ESP-IDF event subscriptions and one lifecycle worker while
      preserving the single Wi-Fi owner, fallback DNS, client-only recovery,
      nonblocking callbacks, typed retained markers, and coordinator wakeups.
- [ ] Add clear-before-effect probe admission plus exact flash/NVS generation,
      CLI, source-ownership, privacy, and real-boundary regressions.
- [ ] Add the typed host capture, projection validator, public contract,
      `just` surface, recovery/precedence/mode/redaction tests, and a real child-
      process regression proving monitor stdout is consumed without an
      invented evidence file.
- [ ] Run the complete software gate, push the exact implementation, and build
      the normal package from that clean commit.
- [ ] Spend exactly one fresh detector and at most one conditional attempt-001.
- [ ] Validate and promote only `NET-001` when every lifecycle, exact-package,
      final-state, cleanup, mode, and redaction fact passes.

## Verification and promotion

Run focused `bitaxe-core`, firmware-host/source-ownership, flash, automation,
contract, and redaction targets, then in order:

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
11. selector, immutable-plan, task-uniqueness, reference-cleanliness,
    sensitive-output, private-mode, no-public-output, and diff checks

After the exact implementation commit is clean and pushed, run exactly:

1. `bazel build //firmware/bitaxe:firmware_image`
2. `test ! -e scratch/net001-reconnect/wrapper-001 && (umask 077; mkdir -m 700 -p scratch/net001-reconnect/wrapper-001 && just detect-ultra205 > scratch/net001-reconnect/wrapper-001/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/net001-reconnect/attempt-001 && test ! -e docs/parity/evidence/net001-reconnect/network-reconnect-projection.json && (umask 077; just capture-network-reconnect-evidence --private-root scratch/net001-reconnect/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/net001-reconnect/wrapper-001/detector.stdout --projection docs/parity/evidence/net001-reconnect/network-reconnect-projection.json --capture-timeout-seconds 90 > scratch/net001-reconnect/wrapper-001/capture.stdout 2> scratch/net001-reconnect/wrapper-001/capture.stderr)`

Fresh private and public paths must be absent. Wrapper/attempt roots are
ignored mode-`0700` directories with mode-`0600` files. Detector failure stops
before writes; any conditional capture start consumes attempt-001. Preserve
the earliest typed failure through marker cleanup, resource cleanup, and at
most one ordinary exact-package recovery flash. Accepted non-success
categories are `package_invalid`, `process_failed`, `timeout`,
`hardware_blocked`, `evidence_invalid`, `reconnect_not_observed`,
`reconnect_timing_invalid`, `service_recovery_failed`, and `recovery_failed`.
Release every owned resource and never retry this ordinal.

Promotion requires exact source/reference/package identity, exactly one
detector-admitted board 205, clear-before-effect probe consumption, one
post-boot station disconnect, immediate configuration-AP fallback, a closed
reason category, exactly one first retry no earlier than 5,000 ms, DHCP
reacquisition in the same boot, retry ordinal reset, client-only recovery,
15,000 ms of stable connected service, final HTTP and retained-log quorum,
`mineonboot=false`, disabled mining and hardware control, complete USB/process
cleanup, private modes, redaction, independent projection validation, and all
gates passing. Otherwise create a truthful closure, withhold public evidence,
and keep `NET-001` at `implemented`.
