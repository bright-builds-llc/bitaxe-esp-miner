# Parity work plan

- Run ID: `20260811T214737Z-REL-002`
- Parity row: `REL-002`
- Initial status: `implemented`
- Source commit: `46f34ac101a15c2fabae3417e119cb0118afff0a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-rel002-force-close-attempt-002`
- Continues plan: `docs/parity/work-plans/20260811T210329Z-REL-002/PLAN.md`

## Selection

The branch is clean and synchronized with `origin/main`, the pinned reference
is clean, and the deterministic selector reports no open plan. `CFG-001`
repeats the safety-controlled mining and network-correlation boundary, while
`CFG-006` requires unavailable non-205 hardware. `NET-001` through `NET-003`
require qualified reconnect, provisioning-client, scan, or IPv6 environments.
`ASIC-002` through `ASIC-005`, `ASIC-007`, `STR-001`, `STR-006`, and `STR-007`
depend on the closed safety-controlled mining lineage.

`API-009` requires mining actuation, an active block-dismiss case, and trusted
physical identify rendering. `PWR-001` through `PWR-003`, `PWR-005`,
`PWR-006`, `THR-001` through `THR-003`, and `SELF-001` require qualified
sensors, actuation, or fault stimulus. `IO-001` requires controlled transient
bus faults, `IO-002` requires an independent calibrated reference, `UI-001`
and `UI-002` require trusted visual capture, `UI-003` requires physical input,
`BAP-002` requires a compatible accessory, `UI-004` retains operator-UAT and
deferred OTAWWW gaps, and `STAT-001` through `STAT-003` depend on live mining
truth. `REL-002` is the first actionable row because attempt-001 isolated one
specific host transport defect: the interrupted-upload helper returned after
write-side flush without forcing or observing full socket closure.

## Scope and non-scope

Replace only that false completion boundary. After writing the admitted strict
image prefix, the helper must half-close in byte order, allow a bounded
250-millisecond grace for the peer to close, reset the connection if it
remains live, and resolve only after the local socket emits `close`. A timeout
or pre-flush transport failure must destroy the owned socket and reject. The
returned closed observation and existing rollback workflow remain unchanged.

Add a real child-process regression with `allowHalfOpen: true`. The child must
retain its response half after receiving EOF, record the exact declared and
transmitted lengths only after connection closure, and prove the helper cannot
return while the peer remains live. Focused tests must also cover validation,
timeout cleanup, pre-flush error propagation, one partial request, and exact
strict-prefix bounds. In-process orchestration, evidence, recovery,
primary-failure, private-mode, and redaction tests remain authoritative and
must continue to pass.

After a clean pushed implementation, run one new detector and at most one
conditional attempt-002 transaction. It may install the exact normal factory
package with replacement NVS containing only owner-supplied Wi-Fi credentials
and `mineonboot=false`; send one bounded truncated application OTA request;
upload one complete admitted rollback probe; accept its scheduled software
restart; issue one normal probe restart to permit native ESP-IDF rollback; and
use one exact normal factory recovery flash only when final normal restoration
cannot otherwise be confirmed. Recovery cannot convert failure into evidence.

The public `bitaxe-sdkconfig-rollback-evidence-v1` projection may contain only
closed schema/provenance fields, cryptographic artifact digests, bounded
counts, interruption/abort/boot/rollback booleans, disabled mining and
hardware-control facts, cleanup, protected modes, and redaction facts.
Origins, hostnames, ports, USB/network/process identities, Wi-Fi values,
credentials, HTTP bodies, firmware bytes, commands, and raw traces remain
private.

This plan does not authorize OTAWWW or SPIFFS update, erase-flash, arbitrary
raw writes, bootloader or partition-table corruption, power interruption,
mining, ASIC initialization or work, pool access, voltage, frequency, fan,
thermal or power control, network discovery, foreign-process termination,
direct UART, pins, pads, headers, GPIO, probes, jumpers, soldering, or injected
signals. `REL-003`, recovery-page upload, other boards, and release readiness
remain non-claims.

## Implementation

- [ ] Make interrupted upload completion mean observed full local socket
      closure, with bounded FIN grace and forced reset when the peer stays
      half-open.
- [ ] Replace the child regression's cooperative peer close with a real
      half-open peer and add focused error, timeout, cleanup, and strict-prefix
      coverage.
- [ ] Run focused and complete software gates, push the exact implementation,
      and build the normal and isolated probe artifacts from that clean commit.
- [ ] Spend exactly one fresh detector and at most one conditional attempt-002.
- [ ] Independently validate the projection and promote only `REL-002` when
      every interruption, rollback, restoration, cleanup, and privacy fact
      passes.

## Verification and promotion

Run focused interrupted-upload, rollback orchestration, contract-validator,
CLI, and redaction tests, then in order:

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
2. `test ! -e scratch/rel002-sdkconfig-rollback/wrapper-002 && (umask 077; mkdir -m 700 -p scratch/rel002-sdkconfig-rollback/wrapper-002 && just detect-ultra205 > scratch/rel002-sdkconfig-rollback/wrapper-002/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/rel002-sdkconfig-rollback/attempt-002 && test ! -e docs/parity/evidence/rel002-sdkconfig-rollback/sdkconfig-rollback-projection.json && (umask 077; just capture-sdkconfig-rollback-evidence --private-root scratch/rel002-sdkconfig-rollback/attempt-002 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --rollback-probe-image bazel-bin/firmware/bitaxe/esp-miner-rollback-probe.bin --rollback-probe-metadata bazel-bin/firmware/bitaxe/rollback-probe.json --wifi-credentials wifi-credentials.json --detector-output scratch/rel002-sdkconfig-rollback/wrapper-002/detector.stdout --projection docs/parity/evidence/rel002-sdkconfig-rollback/sdkconfig-rollback-projection.json --capture-timeout-seconds 600 > scratch/rel002-sdkconfig-rollback/wrapper-002/capture.stdout 2> scratch/rel002-sdkconfig-rollback/wrapper-002/capture.stderr)`

The wrapper and attempt roots must be absent before use, ignored, mode `0700`,
and contain only mode-`0600` files. Detector failure stops before writes. Any
conditional capture start consumes attempt-002. Preserve the earliest typed
failure through cleanup and optional exact-package recovery; recovery status
is secondary. Accepted non-success categories are `package_invalid`,
`process_failed`, `timeout`, `hardware_blocked`, `evidence_invalid`,
`interruption_not_observed`, `probe_boot_failed`, `rollback_not_observed`, and
`recovery_failed`. Release every owned USB, socket, file, and child-process
resource in every path and do not retry this ordinal.

Promotion requires exact normal and probe source/reference identity, admitted
artifact digests, rollback-enabled SDK settings, one admitted board 205, a
safe factory baseline, exactly one partial upload followed by observed full
host teardown and device protocol abort without reboot/build change,
reader-admitted same-device `ota_0` probe boot at ordinal `N+1`, the pending-
validation marker, one normal restart, same-device exact normal factory
rollback at the following ordinal, disabled mining and hardware control,
cleanup, protected modes, redaction, independent evidence validation, and all
gates passing. Otherwise create a truthful non-verifying closure, keep
`REL-002` at `implemented`, withhold final evidence, and stop without retry.
