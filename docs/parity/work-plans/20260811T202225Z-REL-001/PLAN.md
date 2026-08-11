# Parity work plan

- Run ID: `20260811T202225Z-REL-001`
- Parity row: `REL-001`
- Initial status: `implemented`
- Source commit: `e112c19c1e3aee337a2dfc9dac6c1719cc962f2f`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-rel001-partition-size-normalization-attempt-002`
- Continues plan: `docs/parity/work-plans/20260811T195144Z-REL-001/PLAN.md`

## Selection

The branch is clean and synchronized with `origin/main`, the pinned reference
is clean, the selector reports no open plan, and the prior REL-001 plan is
closed without verification. `CFG-001` is blocked behind a repeated
mining/network-correlation boundary, while `CFG-006` requires unavailable
non-205 hardware. `NET-001` through `NET-003` require qualified reconnect,
provisioning-client, scan, or IPv6 environments that do not have a current
bounded evidence contract. `ASIC-002` through `ASIC-005`, `ASIC-007`,
`STR-001`, `STR-006`, and `STR-007` depend on safety-controlled mining or ASIC
evidence whose last targeted live boundary is closed.

`API-009` still spans mining, physical identify, and live block-notification
effects. `PWR-001` through `PWR-003`, `PWR-005`, `PWR-006`, `THR-001` through
`THR-003`, and `SELF-001` require qualified sensors, actuation, or fault
stimulus. `IO-001` requires controlled transient bus faults, `IO-002` requires
an independent calibrated reference, `UI-001` and `UI-002` require trusted
visual capture, `UI-003` requires physical input, `BAP-002` requires a
compatible accessory, `UI-004` retains broader operator-UAT gaps, and
`STAT-001` through `STAT-003` depend on live mining truth.

REL-001 is the first actionable row. Attempt-001 admitted the exact clean
package and exactly one Ultra 205 but stopped before any device effect because
the new partition comparator treated the ESP-IDF size suffix spellings `8K`
and `8k` as different. This closed discriminator supplies the new information
required for a targeted regression-backed fix and fresh attempt ordinal.

## Scope and non-scope

Fix only canonical partition-table comparison. Parse each non-comment CSV row
into its six exact fields, normalize only the recognized decimal/binary size
unit suffix spelling for comparison, and continue requiring the exact name,
type, subtype, offset, numeric size, order, row count, package digest, and
artifact digest. Prove the fix against the actual checked-in Ultra 205 table
and preserve rejection for field, numeric value, ordering, and unknown-unit
drift. Keep the functional comparison core separate from the effectful capture
shell.

After all software gates pass and the fix is committed and pushed, build the
exact package and run one fresh detector plus conditional attempt-002 capture.
The existing typed OTA transaction remains authoritative: one exact factory
flash with mining disabled, one trusted same-origin factory baseline, a
prearmed same-device reader, one exact OTA application upload, same-device
reacquisition, exact build/boot/partition postconditions, and aggregate-only
public evidence.

The public projection may contain only closed schema/provenance fields,
cryptographic digests, bounded counts, canonical-layout and slot-transition
booleans, disabled mining/hardware-control facts, cleanup, protected modes,
and redaction status. OTA bytes, HTTP bodies, origins, hostnames, ports, USB or
network identities, Wi-Fi values, credentials, commands, and raw traces stay
private.

This plan does not authorize rollback, erase-flash, interrupted update,
OTAWWW, SPIFFS update, recovery upload, arbitrary raw writes, mining, ASIC
initialization or work, voltage/frequency/fan/thermal/power effects, network
discovery, foreign-process termination, direct UART, or pin/pad/header work.
REL-002/REL-003, rollback, destructive recovery, other boards, and release
readiness remain non-claims.

## Implementation

- [ ] Add a pure partition-row parser/comparator that normalizes only accepted
      ESP-IDF size-unit suffix case and keeps every other field exact.
- [ ] Add red/green regressions using the checked-in Ultra 205 partition table
      plus negative field, numeric-size, order, and unknown-unit cases through
      the real automation test/runfiles boundary.
- [ ] Run focused and complete gates, push the clean implementation, freeze its
      exact package, and spend at most one detector plus conditional capture.
- [ ] Independently validate accepted public evidence and promote only
      `REL-001` when every typed acceptance condition passes.

## Verification and promotion

Run the focused automation test and real Bazel runfiles boundary, then in
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
    sensitive-output, no-public-output, and diff checks

After a clean implementation commit is pushed, run exactly:

1. `just package`
2. `test ! -e scratch/rel001-ota-slot/wrapper-002 && (umask 077; mkdir -m 700 -p scratch/rel001-ota-slot/wrapper-002 && just detect-ultra205 > scratch/rel001-ota-slot/wrapper-002/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/rel001-ota-slot/attempt-002 && test ! -e docs/parity/evidence/rel001-ota-slot/partition-layout-projection.json && (umask 077; just capture-partition-layout-evidence --private-root scratch/rel001-ota-slot/attempt-002 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/rel001-ota-slot/wrapper-002/detector.stdout --projection docs/parity/evidence/rel001-ota-slot/partition-layout-projection.json --capture-timeout-seconds 360 > scratch/rel001-ota-slot/wrapper-002/capture.stdout 2> scratch/rel001-ota-slot/wrapper-002/capture.stderr)`

The wrapper and supervisor-owned attempt roots must be absent before use,
mode `0700`, and contain only mode-`0600` files. Detector failure stops before
writes. The capture permits one exact-package factory flash, replacement NVS
containing only owner-supplied Wi-Fi credentials and `mineonboot=false`, normal
USB reset/re-enumeration, bounded receive-only USB and same-origin HTTP, one
exact OTA application upload, and its scheduled software restart. No second
flash or recovery effect is permitted. Preserve the earliest typed failure,
release every owned USB/socket/process resource, withhold evidence on any
failure, and do not retry this ordinal.

Promotion requires exact source/reference/package identity, all six admitted
artifact digests, the canonical eight-row partition contract, one admitted
board 205, a safe factory baseline, reader admission before exactly one
complete OTA upload, the same physical device, service loss/recovery, exact
recovered build, a changed boot session, ordinal `N+1`, software reset,
successful OTA boot validation, `factory` to `ota_0`, disabled mining and
hardware control, complete cleanup, protected modes, redaction, independent
evidence validation, and every gate passing. Otherwise create a non-verifying
closure, keep `REL-001` at `implemented`, and stop without retry.
