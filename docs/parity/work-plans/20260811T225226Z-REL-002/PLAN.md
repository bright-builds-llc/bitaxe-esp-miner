# Parity work plan

- Run ID: `20260811T225226Z-REL-002`
- Parity row: `REL-002`
- Initial status: `implemented`
- Source commit: `f83d38443192446fc58178385e4dbb3af46fae7d`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-rel002-retained-marker-attempt-004`
- Continues plan: `docs/parity/work-plans/20260811T223031Z-REL-002/PLAN.md`

## Selection

The branch and pinned reference are clean and synchronized, and the selector
reports no open plan. Earlier unfinished rows retain the concrete boundaries
recorded in the REL-002 predecessors: `CFG-001` and ASIC/Stratum need qualified
safety-controlled mining; `CFG-006` needs unavailable non-205 hardware;
network rows need reconnect/provisioning/scan/IPv6 environments; and the API,
power, thermal, self-test, IO, UI, BAP, and statistics rows need unavailable
actuation, calibrated, visual, input, accessory, fault, or live-mining proof.
`REL-002` remains first actionable because attempt-003 proved its strict reset-
aborted upload and retained protocol-error status but exposed one exact host
predicate mismatch before the probe and rollback stages.

## Scope and non-scope

Change only the retained-log predicate used by the SDK-config rollback
workflow. Recognize the firmware's canonical API-visible marker
`firmware_ota_status=Protocol Error`; do not recognize the UART-only
`firmware_ota_update=protocol_error` spelling as retained evidence. Keep the
reset-before-FIN transport, 90-second initial monitor, six-attempt baseline
readiness, device-session transactions, recovery behavior, public schema, and
all identity, cleanup, and privacy checks unchanged.

Add production-shaped orchestration coverage proving the canonical marker
advances into the probe and rollback stages, the UART-only spelling closes as
`interruption_not_observed` with exact-package recovery and no public evidence,
missing/malformed/unrelated retained logs fail closed, and public output never
contains operational values. Existing reset, readiness, session, rollback,
recovery, evidence, mode, and redaction tests must continue to pass.

After a clean pushed implementation, run one fresh detector and at most one
conditional attempt-004. Allowed effects are one exact normal factory flash,
replacement NVS with only owner-supplied Wi-Fi and `mineonboot=false`, bounded
receive-only USB and same-origin HTTP, one strict reset-aborted partial OTA,
one complete rollback-probe OTA, its scheduled restart, one normal probe
restart for native rollback, and one exact normal recovery flash only if final
restoration cannot otherwise be confirmed. Recovery never creates success.

Public evidence remains only the closed redacted
`bitaxe-sdkconfig-rollback-evidence-v1` projection. Origins, hostnames, ports,
USB/network/process identities, Wi-Fi values, credentials, HTTP bodies,
firmware bytes, commands, and raw traces remain private. This plan does not
authorize OTAWWW, SPIFFS update, erase-flash, arbitrary raw writes, bootloader
or partition-table corruption, power interruption, mining, ASIC work, pool
access, voltage, frequency, fan, thermal/power control, discovery, foreign-
process termination, direct UART, pins, pads, headers, GPIO, probes, jumpers,
soldering, or injected signals. Other boards, `REL-003`, recovery-page upload,
and release readiness remain non-claims.

## Implementation

- [ ] Replace the UART-only retained-log predicate with the canonical retained
      firmware OTA status marker.
- [ ] Add positive, negative, recovery, no-evidence, and privacy regressions at
      the real rollback orchestration seam.
- [ ] Run the complete software gate, push the exact implementation, and build
      normal/probe artifacts from that clean commit.
- [ ] Spend exactly one fresh detector and at most one conditional attempt-004.
- [ ] Validate and promote only `REL-002` when every abort, same-device,
      rollback, restoration, cleanup, mode, and redaction fact passes.

## Verification and promotion

Run the focused rollback orchestration and interrupted-upload tests, then in
order:

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
2. `test ! -e scratch/rel002-sdkconfig-rollback/wrapper-004 && (umask 077; mkdir -m 700 -p scratch/rel002-sdkconfig-rollback/wrapper-004 && just detect-ultra205 > scratch/rel002-sdkconfig-rollback/wrapper-004/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/rel002-sdkconfig-rollback/attempt-004 && test ! -e docs/parity/evidence/rel002-sdkconfig-rollback/sdkconfig-rollback-projection.json && (umask 077; just capture-sdkconfig-rollback-evidence --private-root scratch/rel002-sdkconfig-rollback/attempt-004 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --rollback-probe-image bazel-bin/firmware/bitaxe/esp-miner-rollback-probe.bin --rollback-probe-metadata bazel-bin/firmware/bitaxe/rollback-probe.json --wifi-credentials wifi-credentials.json --detector-output scratch/rel002-sdkconfig-rollback/wrapper-004/detector.stdout --projection docs/parity/evidence/rel002-sdkconfig-rollback/sdkconfig-rollback-projection.json --capture-timeout-seconds 600 > scratch/rel002-sdkconfig-rollback/wrapper-004/capture.stdout 2> scratch/rel002-sdkconfig-rollback/wrapper-004/capture.stderr)`

Fresh private and public paths must be absent. Wrapper/attempt roots are ignored
mode-`0700` directories with mode-`0600` files. Detector failure stops before
writes; any conditional capture start consumes attempt-004. Preserve the
earliest typed failure through cleanup and optional exact-package recovery.
Accepted non-success categories remain `package_invalid`, `process_failed`,
`timeout`, `hardware_blocked`, `evidence_invalid`,
`interruption_not_observed`, `probe_boot_failed`, `rollback_not_observed`, and
`recovery_failed`. Release every resource and never retry this ordinal.

Promotion requires exact provenance/digests and rollback settings, one admitted
board 205, safe factory baseline, one reset-before-FIN partial request with the
canonical retained protocol abort and unchanged normal build, same-device
`ota_0` probe boot at `N+1`, pending validation, one normal restart, exact
factory rollback at the next ordinal, disabled mining/hardware control,
cleanup, modes, redaction, independent validation, and all gates passing.
Otherwise create a truthful closure, withhold public evidence, and keep
`REL-002` implemented.
