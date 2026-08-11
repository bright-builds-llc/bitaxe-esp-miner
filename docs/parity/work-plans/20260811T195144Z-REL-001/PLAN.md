# Parity work plan

- Run ID: `20260811T195144Z-REL-001`
- Parity row: `REL-001`
- Initial status: `implemented`
- Source commit: `a5328d1b72e06d24e9f3a151b55bd738881201da`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-rel001-live-ota-slot-transition`

## Selection

The branch is clean, synchronized with `origin/main`, the pinned reference is
clean, and the deterministic selector reports no open plan. `CFG-001` is
closed at a repeated network-correlation boundary with no unchanged mining
soak retry. `CFG-006` requires unavailable non-205 hardware. `NET-001` through
`NET-003` require controlled access-point failure/recovery, provisioning,
scan, or IPv6 environments that have no qualified repository contract.
`ASIC-002` through `ASIC-005`, `ASIC-007`, `STR-001`, `STR-006`, and `STR-007`
depend on safety-controlled mining evidence whose last targeted attempt
repeated its terminal continuity boundary.

`API-009` cannot prove its complete command-effect set without active mining,
a physical identify observation, and a live block-notification state.
`PWR-001` through `PWR-003`, `PWR-005`, `PWR-006`, `THR-001` through
`THR-003`, and `SELF-001` require qualified sensors, actuation, or fault
stimulus. `IO-001` requires controlled transient bus faults, while `IO-002`
requires an independent calibrated voltage reference. `UI-001` and `UI-002`
require trusted physical visual capture, `UI-003` requires recorded physical
input, `BAP-002` requires a compatible accessory, and `UI-004` retains broader
live mutation, upload, and responsive operator-UAT gaps. `STAT-001` through
`STAT-003` require live BM1366 or mining-history truth unavailable beyond the
closed mining boundary. `LOG-001` is already verified.

`REL-001` is therefore the first actionable row. Its exact partition-table
offsets and sizes, package artifacts, factory boot, and SPIFFS mount already
have unit, workflow, API-comparison, and device evidence. One normal packaged
OTA transition can safely close the remaining selected-slot seam by proving
the exact factory package selects, writes, boots, validates, and reports
`ota_0` on the same admitted Ultra 205.

## Scope and non-scope

Add a typed aggregate-only `bitaxe-partition-layout-evidence-v1` workflow. It
will flash one exact clean Ultra 205 factory package with mining disabled,
derive one trusted same-origin target from the admitted monitor session,
confirm the exact package, safe state, and `runningPartition=factory`, arm the
existing receive-only same-device supervisor, upload that package's exact OTA
application image once to `/api/system/OTA`, and prove the recovered device
reports the same source/reference/application identity, a new boot session,
ordinal `N+1`, software reset, successful OTA boot validation, and
`runningPartition=ota_0`.

Extend the device-session transaction with a private
`esp-device-session-ota-intent-v1` and `ota-live` command. The action remains
inside the same process that admits physical USB identity, obtains three
stable accessible holder-free samples, arms the serial reader before the
request, sends exactly one bounded binary upload, reacquires the same device,
and closes its HTTP/build/boot/postcondition quorum. Preserve the existing
fixture `reboot` and live `reboot-live` interfaces.

The public projection contains only closed schema/provenance fields,
cryptographic digests, bounded artifact and observation counts, exact
partition-contract and slot-transition booleans, safe-state facts, cleanup,
private-mode, and redaction status. OTA bytes, HTTP bodies, origins, hostnames,
ports, USB and network identities, Wi-Fi values, credentials, commands, and
raw serial/process traces remain beneath the protected private root.

This work does not roll back firmware, erase flash, interrupt an update,
update the SPIFFS partition, perform recovery upload, write arbitrary raw
partitions, mine, initialize or submit ASIC work, actuate voltage, frequency,
fan, thermal, or power controls, scan or discover networks, terminate foreign
processes, use direct UART, or manipulate pins. REL-002/REL-003 rollback,
large-erase, interrupted-update, OTAWWW/static-partition, other-board, and
release-readiness behavior remain explicit non-claims.

## Implementation

- [ ] Add a bounded binary-body exchange to the shared strict HTTP transport
      with exact request-progress evidence and no payload disclosure.
- [ ] Add the typed OTA intent, `ota-live` CLI, upload-before-reacquisition
      transaction, partition postcondition, and behavior-focused unit and
      real-child-process regressions while preserving reboot compatibility.
- [ ] Add the Rust-owned REL-001 evidence schema/validator, synchronized
      TypeScript contract, private-first capture, CLI/Just surface, exact
      package/partition checks, safe-state checks, no-clobber publication, and
      privacy/failure regressions.
- [ ] Run all mandatory software gates, push the implementation, freeze its
      exact package, and spend at most one detector plus conditional capture.
- [ ] Independently validate the closed projection and promote only `REL-001`
      when every acceptance condition passes.

## Verification and promotion

Run focused HTTP-transport, device-session, automation, evidence-validator,
package, and real-child-process tests followed by, in order:

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
11. selector, immutable-plan, task-uniqueness, sensitive-output,
    reference-cleanliness, and diff checks

After a clean implementation commit is pushed, run exactly these bounded
commands:

1. `just package`
2. `test ! -e scratch/rel001-ota-slot/wrapper-001 && (umask 077; mkdir -m 700 -p scratch/rel001-ota-slot/wrapper-001 && just detect-ultra205 > scratch/rel001-ota-slot/wrapper-001/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/rel001-ota-slot/attempt-001 && test ! -e docs/parity/evidence/rel001-ota-slot/partition-layout-projection.json && (umask 077; just capture-partition-layout-evidence --private-root scratch/rel001-ota-slot/attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/rel001-ota-slot/wrapper-001/detector.stdout --projection docs/parity/evidence/rel001-ota-slot/partition-layout-projection.json --capture-timeout-seconds 360 > scratch/rel001-ota-slot/wrapper-001/capture.stdout 2> scratch/rel001-ota-slot/wrapper-001/capture.stderr)`

The wrapper and attempt roots must be absent before use, mode `0700`, and
contain only mode-`0600` files. Detector failure stops before writes. The
capture permits one exact-package factory flash, replacement NVS containing
only owner-supplied Wi-Fi credentials and `mineonboot=false`, normal USB reset
and re-enumeration, bounded receive-only USB and same-origin HTTP observation,
one exact OTA application upload, and its scheduled software restart. No
second flash, rollback, erase, interruption, or recovery effect is permitted.
If failure follows a completed upload, leave the exact package in whichever
valid factory/OTA slot ESP-IDF selected, preserve the earliest typed failure,
release owned USB/process resources, withhold public evidence, and stop.

Map non-ready device-session outcomes to `hardware_blocked`, malformed or
missing projections to `evidence_invalid`, child timeout to `timeout`, and
launch/child failures to `process_failed`. Exactly one capture is permitted;
no unchanged retry is authorized.

Promotion requires exact source/reference/package identity, one admitted
board 205, exact package partition-table and artifact digests, a safe factory
baseline, reader admission before exactly one complete OTA upload, the same
physical device, service loss and recovery, a changed boot session, ordinal
`N+1`, software reset, exact recovered build identity, successful boot
validation, `factory` to `ota_0`, disabled mining and hardware control,
complete socket/process/USB cleanup, correct private modes, an independently
validated redacted projection, and every mandatory gate passing. Otherwise
withhold `RESULT.md` and public evidence, create a typed non-verified closure,
keep `REL-001` at `implemented`, and stop without retry.
