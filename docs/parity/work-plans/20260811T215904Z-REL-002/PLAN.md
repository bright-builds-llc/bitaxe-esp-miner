# Parity work plan

- Run ID: `20260811T215904Z-REL-002`
- Parity row: `REL-002`
- Initial status: `implemented`
- Source commit: `96d86057ff5af311a1cc6b4cea8800f3a561c8e3`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-rel002-reset-before-fin-attempt-002`
- Continues plan: `docs/parity/work-plans/20260811T214737Z-REL-002/PLAN.md`

## Selection

The branch and pinned reference are clean, `main` is synchronized with
`origin/main`, and the selector reports no open plan after the truthful closure
of the disproved post-FIN design. Earlier candidates remain blocked by the
same concrete boundaries recorded in the predecessor plans: `CFG-001` and the
ASIC/Stratum lineage require safety-controlled mining; `CFG-006` needs
non-205 hardware; network rows need qualified reconnect/provisioning/scan/IPv6
environments; API, power, thermal, self-test, IO, UI, BAP, and statistics rows
need unavailable actuation, calibrated, visual, input, accessory, fault, or
live-mining evidence. `REL-002` remains the first actionable row because a
direct Node 24 trace now proves the exact transport sequence absent from
attempt-001: flush the strict prefix without FIN, then force a TCP reset.

## Scope and non-scope

Change only interrupted-upload completion. Write the admitted HTTP headers and
exact 4,096-byte strict image prefix without calling `end`. After the write
callback, keep the connection fully open for exactly 100 milliseconds so the
small prefix can reach the peer, then issue one `resetAndDestroy`. Resolve only
after the local socket emits `close` and only when prefix flush and reset were
both observed. A timeout, connection failure, peer close before reset, or
pre-reset error must destroy the owned socket and reject.

Replace the cooperative real-child test with an `allowHalfOpen: true` peer
that records the full prefix while the helper is still pending, never closes
itself, observes no EOF, and terminates only from the reset. Add focused tests
for origin and strict-prefix validation, pre-flush failure, timeout cleanup,
the one-request bound, and exact declared-versus-transmitted lengths. Preserve
the existing rollback orchestration, primary-failure precedence, recovery,
private-mode, redaction, evidence, CLI, and contract behavior.

After a clean pushed implementation, reuse the still-unconsumed attempt-002
ordinal and paths from the superseded software-only plan. Run one fresh
detector and at most one conditional capture. It may install the exact normal
factory package with replacement NVS containing only owner-supplied Wi-Fi and
`mineonboot=false`; send one bounded reset-aborted application OTA request;
upload one complete rollback probe; accept its scheduled software restart;
issue one normal probe restart to permit native rollback; and use one exact
normal recovery flash only if final restoration cannot otherwise be confirmed.
Recovery cannot turn failure into success evidence.

Public evidence remains the closed redacted
`bitaxe-sdkconfig-rollback-evidence-v1` projection. Origins, hostnames, ports,
USB/network/process identities, Wi-Fi values, credentials, HTTP bodies,
firmware bytes, commands, and raw traces stay private. This plan does not
authorize OTAWWW, SPIFFS update, erase-flash, arbitrary raw writes, bootloader
or partition-table corruption, power interruption, mining, ASIC work, pool
access, voltage, frequency, fan, thermal or power control, network discovery,
foreign-process termination, direct UART, pins, pads, headers, GPIO, probes,
jumpers, soldering, or injected signals. Other boards, `REL-003`, recovery-page
upload, and release readiness remain non-claims.

## Implementation

- [ ] Flush one exact partial HTTP request without FIN, wait the bounded
      delivery grace, issue one TCP reset, and resolve only after local close.
- [ ] Prove the boundary with a non-cooperative real child plus validation,
      error, timeout, cleanup, and exact-prefix tests.
- [ ] Run the complete software gate, push the exact clean implementation, and
      build normal and isolated rollback-probe artifacts from that commit.
- [ ] Spend exactly one fresh detector and at most one conditional attempt-002.
- [ ] Validate and promote only `REL-002` when every abort, same-device,
      rollback, restoration, cleanup, mode, and redaction fact passes.

## Verification and promotion

Run focused automation transport, rollback capture, contract-validator, CLI,
and redaction tests, then in order:

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

1. `bazel build //firmware/bitaxe:firmware_image //firmware/bitaxe:rollback_probe_image`
2. `test ! -e scratch/rel002-sdkconfig-rollback/wrapper-002 && (umask 077; mkdir -m 700 -p scratch/rel002-sdkconfig-rollback/wrapper-002 && just detect-ultra205 > scratch/rel002-sdkconfig-rollback/wrapper-002/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/rel002-sdkconfig-rollback/attempt-002 && test ! -e docs/parity/evidence/rel002-sdkconfig-rollback/sdkconfig-rollback-projection.json && (umask 077; just capture-sdkconfig-rollback-evidence --private-root scratch/rel002-sdkconfig-rollback/attempt-002 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --rollback-probe-image bazel-bin/firmware/bitaxe/esp-miner-rollback-probe.bin --rollback-probe-metadata bazel-bin/firmware/bitaxe/rollback-probe.json --wifi-credentials wifi-credentials.json --detector-output scratch/rel002-sdkconfig-rollback/wrapper-002/detector.stdout --projection docs/parity/evidence/rel002-sdkconfig-rollback/sdkconfig-rollback-projection.json --capture-timeout-seconds 600 > scratch/rel002-sdkconfig-rollback/wrapper-002/capture.stdout 2> scratch/rel002-sdkconfig-rollback/wrapper-002/capture.stderr)`

The private roots and public output must be absent before use. Wrapper and
attempt roots are ignored mode-`0700` directories with mode-`0600` files.
Detector failure stops before writes; any conditional capture start consumes
attempt-002. Preserve the earliest typed failure through cleanup and optional
exact-package recovery. Accepted non-success categories are `package_invalid`,
`process_failed`, `timeout`, `hardware_blocked`, `evidence_invalid`,
`interruption_not_observed`, `probe_boot_failed`, `rollback_not_observed`, and
`recovery_failed`. Release every owned resource and do not retry the ordinal.

Promotion requires exact normal/probe provenance and digests, rollback-enabled
SDK settings, one admitted board 205, a safe factory baseline, one strict
partial request with reset-before-FIN and a retained device protocol abort
without reboot/build change, same-device `ota_0` probe boot at `N+1`, pending
validation, one normal restart, same-device exact factory rollback at the next
ordinal, disabled mining/hardware control, cleanup, protected modes, redaction,
independent projection validation, and all gates passing. Otherwise create a
truthful closure, withhold public evidence, and keep `REL-002` implemented.
