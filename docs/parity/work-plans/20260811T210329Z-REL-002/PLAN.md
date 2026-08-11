# Parity work plan

- Run ID: `20260811T210329Z-REL-002`
- Parity row: `REL-002`
- Initial status: `implemented`
- Source commit: `e77f595c7fd1af5063a4f55f121cedad13404967`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-rel002-rollback-interruption-attempt-001`

## Selection

The branch is clean and synchronized with `origin/main`, the pinned reference
is clean, and the deterministic selector reports no open plan. `CFG-001` is
closed at its repeated safety-controlled mining/network-correlation boundary,
while `CFG-006` requires unavailable non-205 hardware. `NET-001` through
`NET-003` require qualified reconnect, provisioning-client, scan, or IPv6
environments that do not have bounded evidence contracts. `ASIC-002` through
`ASIC-005`, `ASIC-007`, `STR-001`, `STR-006`, and `STR-007` depend on the
closed safety-controlled mining lineage.

`API-009` still requires mining actuation, an active block-dismiss case, and
trusted physical identify rendering. `PWR-001` through `PWR-003`, `PWR-005`,
`PWR-006`, `THR-001` through `THR-003`, and `SELF-001` require qualified
sensors, actuation, or fault stimulus. `IO-001` requires controlled transient
bus faults, `IO-002` requires an independent calibrated reference, `UI-001`
and `UI-002` require trusted visual capture, `UI-003` requires physical input,
`BAP-002` requires a compatible accessory, `UI-004` retains operator-UAT and
deferred OTAWWW gaps, and `STAT-001` through `STAT-003` depend on live mining
truth. `REL-002` is the first row with a complete bounded path to a status
change after `REL-001` proved the canonical table, factory baseline, OTA
upload, `ota_0` execution, and successful validation on this Ultra 205.

## Scope and non-scope

Add a build-isolated rollback probe derived from the same clean source and
pinned reference as the normal package. The probe may differ only by an
explicit compile-time test mode that leaves a newly booted OTA image in ESP-IDF
`pending_verify`, publishes a non-sensitive retained marker, keeps mining and
hardware control disabled, and otherwise starts the normal HTTP service. The
normal release target must never inherit this mode.

Add one typed host transaction that first installs the exact normal factory
package with replacement NVS containing only owner-supplied Wi-Fi credentials
and `mineonboot=false`. It must prove a stable normal baseline, send exactly
one deliberately truncated firmware upload to the inactive OTA slot, prove
that the server aborted the OTA handle without rebooting or changing build,
then send exactly one complete admitted probe image. It must use the canonical
reader-armed device-session transaction to prove the probe boot on the same
physical device at ordinal `N+1` in `ota_0`. A second canonical device-session
transaction may issue one ordinary restart; ESP-IDF must then reject the
unvalidated probe and recover the exact normal factory build on the same
device at the next ordinal. Confirm the final normal package, safe state, and
cleanup before publishing evidence.

The public `bitaxe-sdkconfig-rollback-evidence-v1` projection may contain only
closed schema/provenance fields, cryptographic artifact digests, bounded
counts, interruption/abort/boot/rollback booleans, disabled mining and
hardware-control facts, cleanup, protected-mode, and redaction facts. Origins,
hostnames, ports, USB or network identities, Wi-Fi values, credentials, HTTP
bodies, firmware bytes, commands, process identifiers, and raw serial traces
remain private.

This plan does not authorize OTAWWW, SPIFFS update, erase-flash, arbitrary raw
writes, corrupted partition tables or bootloaders, power interruption, forced
boot failure, mining, ASIC initialization or work, voltage/frequency/fan/
thermal/power effects, network discovery, foreign-process termination, direct
UART, or pin/pad/header work. `REL-003`, recovery-page upload, other boards,
and release readiness remain non-claims.

## Implementation

- [ ] Add the isolated rollback-probe build and prove the normal build cannot
      inherit its pending-validation behavior.
- [ ] Extend the typed device-session postcondition narrowly so an admitted
      OTA image may have a different expected ELF digest, without weakening
      normal reboot or same-package OTA checks.
- [ ] Add a typed interrupted-upload and two-session rollback capture with
      primary-failure precedence, exact-package recovery, protected private
      artifacts, closed public evidence, and real-child-process regressions.
- [ ] Run focused and complete software gates, push the implementation, and
      spend at most one detector plus conditional hardware attempt.
- [ ] Independently validate the projection and promote only `REL-002` when
      every interruption and rollback acceptance fact passes.

## Verification and promotion

Run focused firmware boot-validation, build-mode, HTTP interruption,
device-session, capture, contract-validator, CLI, and redaction tests, then in
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

After a clean implementation commit is pushed, run exactly:

1. `bazel build //firmware/bitaxe:firmware_image //firmware/bitaxe:rollback_probe_image`
2. `test ! -e scratch/rel002-sdkconfig-rollback/wrapper-001 && (umask 077; mkdir -m 700 -p scratch/rel002-sdkconfig-rollback/wrapper-001 && just detect-ultra205 > scratch/rel002-sdkconfig-rollback/wrapper-001/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/rel002-sdkconfig-rollback/attempt-001 && test ! -e docs/parity/evidence/rel002-sdkconfig-rollback/sdkconfig-rollback-projection.json && (umask 077; just capture-sdkconfig-rollback-evidence --private-root scratch/rel002-sdkconfig-rollback/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --rollback-probe-image bazel-bin/firmware/bitaxe/esp-miner-rollback-probe.bin --rollback-probe-metadata bazel-bin/firmware/bitaxe/rollback-probe.json --wifi-credentials wifi-credentials.json --detector-output scratch/rel002-sdkconfig-rollback/wrapper-001/detector.stdout --projection docs/parity/evidence/rel002-sdkconfig-rollback/sdkconfig-rollback-projection.json --capture-timeout-seconds 600 > scratch/rel002-sdkconfig-rollback/wrapper-001/capture.stdout 2> scratch/rel002-sdkconfig-rollback/wrapper-001/capture.stderr)`

The wrapper and attempt roots must be absent before use, ignored, mode `0700`,
and contain only mode-`0600` files. Detector failure stops before writes. The
capture permits one exact normal factory flash, replacement NVS as described
above, bounded receive-only USB and same-origin HTTP, one truncated OTA request
whose prefix is bounded by the repo-owned command, one complete probe upload,
its scheduled software restart, and one normal restart of the still-pending
probe to trigger bootloader rollback. If final normal-package restoration
cannot be confirmed, one exact normal factory recovery flash is permitted;
recovery never converts a failed attempt into evidence.

Preserve the earliest typed failure through recovery and report recovery only
as secondary safe booleans. Accepted non-success categories are
`package_invalid`, `process_failed`, `timeout`, `hardware_blocked`,
`evidence_invalid`, `interruption_not_observed`, `probe_boot_failed`,
`rollback_not_observed`, and `recovery_failed`. Release every owned USB,
socket, file, and child-process resource in every path. Do not retry this
ordinal.

Promotion requires exact normal and probe source/reference identity, admitted
artifact digests, canonical rollback-enabled SDK settings, one admitted board
205, a safe factory baseline, one partial upload with an observed protocol
abort and no reboot/build change, reader admission before one complete probe
upload, same-device `ota_0` probe boot at ordinal `N+1`, an explicit pending-
validation marker, one normal restart, same-device exact normal factory
recovery at the following ordinal, disabled mining and hardware control,
complete cleanup, protected modes, redaction, independent evidence validation,
and every gate passing. Otherwise create a non-verifying closure, keep
`REL-002` at `implemented`, withhold final evidence, and stop without retry.
