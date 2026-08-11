# Parity work plan

- Run ID: `20260811T231354Z-REL-002`
- Parity row: `REL-002`
- Initial status: `implemented`
- Source commit: `17e5e96b2761e74661b14f3f3cc598b0bec9fc78`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-rel002-retained-boot-log-attempt-005`
- Continues plan: `docs/parity/work-plans/20260811T225226Z-REL-002/PLAN.md`

## Selection

The branch and pinned reference are clean and synchronized, and the selector
reports no open plan. Earlier unfinished rows retain the concrete dependency,
environment, safety, actuation, calibrated, visual, accessory, fault, non-205,
or live-mining boundaries documented by the REL-002 predecessors. `REL-002`
remains first actionable because attempt-004 proved the interrupted upload and
the complete same-device probe boot, then exposed one bounded evidence-source
mismatch before native rollback: semantic boot lines emitted before USB
re-enumeration were required from a serial reader attached after re-enumeration
instead of from the API-visible retained log that intentionally stores them.

## Scope and non-scope

After the ready probe device session and exact HTTP identity check, fetch
`/api/system/logs` into a protected private probe-log artifact and require the
exact complete retained lines `ota_boot_validation=rollback_probe_pending` and
the passive safe-state marker. After the ready rollback device session and
exact final HTTP identity check, fetch a protected private final-log artifact
and require the exact complete retained passive safe-state line. Keep typed
serial delivery and correlation mandatory through both device-session
projections, but remove semantic boot-line parsing from the late serial files.

Map unavailable or missing probe retained-log evidence to `probe_boot_failed`
and final retained-log evidence to `rollback_not_observed`, preserving those
primary categories through exact-package recovery. Add production-shaped tests
where late serial artifacts contain no semantic markers while retained logs
succeed, plus missing probe marker, missing probe safe state, missing final safe
state, fetch failure, recovery, primary precedence, no-public-evidence, exact-
artifact, and privacy coverage. Preserve every other reset, readiness,
same-device, build, ordinal, partition, recovery, cleanup, schema, and
redaction check.

After a clean pushed implementation, run one fresh detector and at most one
conditional attempt-005. Allowed effects are one exact normal factory flash,
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

- [ ] Validate probe-pending and safe-state boot semantics from the exact
      API-visible retained log after HTTP identity admission.
- [ ] Validate final safe-state boot semantics from the exact retained log
      while preserving typed late-serial delivery correlation.
- [ ] Add success, missing-marker, fetch-failure, recovery, precedence,
      no-evidence, and privacy regressions at the real orchestration seam.
- [ ] Run the complete software gate, push the exact implementation, and build
      normal/probe artifacts from that clean commit.
- [ ] Spend exactly one fresh detector and at most one conditional attempt-005.
- [ ] Validate and promote only `REL-002` when every abort, same-device,
      rollback, restoration, cleanup, mode, and redaction fact passes.

## Verification and promotion

Run the canonical automation target and focused retained-log regressions, then
in order:

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
2. `test ! -e scratch/rel002-sdkconfig-rollback/wrapper-005 && (umask 077; mkdir -m 700 -p scratch/rel002-sdkconfig-rollback/wrapper-005 && just detect-ultra205 > scratch/rel002-sdkconfig-rollback/wrapper-005/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/rel002-sdkconfig-rollback/attempt-005 && test ! -e docs/parity/evidence/rel002-sdkconfig-rollback/sdkconfig-rollback-projection.json && (umask 077; just capture-sdkconfig-rollback-evidence --private-root scratch/rel002-sdkconfig-rollback/attempt-005 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --rollback-probe-image bazel-bin/firmware/bitaxe/esp-miner-rollback-probe.bin --rollback-probe-metadata bazel-bin/firmware/bitaxe/rollback-probe.json --wifi-credentials wifi-credentials.json --detector-output scratch/rel002-sdkconfig-rollback/wrapper-005/detector.stdout --projection docs/parity/evidence/rel002-sdkconfig-rollback/sdkconfig-rollback-projection.json --capture-timeout-seconds 600 > scratch/rel002-sdkconfig-rollback/wrapper-005/capture.stdout 2> scratch/rel002-sdkconfig-rollback/wrapper-005/capture.stderr)`

Fresh private and public paths must be absent. Wrapper/attempt roots are ignored
mode-`0700` directories with mode-`0600` files. Detector failure stops before
writes; any conditional capture start consumes attempt-005. Preserve the
earliest typed failure through cleanup and optional exact-package recovery.
Accepted non-success categories remain `package_invalid`, `process_failed`,
`timeout`, `hardware_blocked`, `evidence_invalid`,
`interruption_not_observed`, `probe_boot_failed`, `rollback_not_observed`, and
`recovery_failed`. Release every resource and never retry this ordinal.

Promotion requires exact provenance/digests and rollback settings, one admitted
board 205, safe factory baseline, one reset-before-FIN partial request with the
canonical retained protocol abort and unchanged normal build, same-device
`ota_0` probe boot at `N+1`, exact retained pending/safe boot semantics, one
normal restart, exact factory rollback at the next ordinal with retained safe-
state semantics, disabled mining/hardware control, cleanup, modes, redaction,
independent validation, and all gates passing. Otherwise create a truthful
closure, withhold public evidence, and keep `REL-002` implemented.
