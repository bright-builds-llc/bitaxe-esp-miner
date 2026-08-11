# Parity work plan

- Run ID: `20260811T182900Z-API-003`
- Parity row: `API-003`
- Initial status: `implemented`
- Source commit: `52db06a39a7301f77d5611eec8498d5681310a75`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api003-live-multifield-patch`
- Continues plan: `docs/parity/work-plans/20260811T182057Z-API-003/PLAN.md`

## Selection

The branch is clean and synchronized, and the selector reports no open plan.
`CFG-001` remains behind safety-controlled default-frequency and voltage soak
evidence whose unchanged retry is closed at a repeated network-correlation
boundary. `CFG-006` requires unavailable non-205 hardware. `NET-001` through
`NET-003` lack qualified reconnect, provisioning-client, live-scan, and IPv6
environment contracts. `ASIC-002` through `ASIC-005`, `ASIC-007`, `STR-001`,
`STR-006`, and `STR-007` depend on safety-controlled mining evidence whose last
targeted attempt repeated its terminal continuity signature.

`API-003` is again the first actionable row. The predecessor plan closed before
implementation or hardware because theme belongs to `/api/theme`, not the
system-settings schema. The source audit identifies `rotation` as a real,
bounded system setting accepted only at 0, 90, 180, or 270 and projected by
`/api/system/info`. Pairing a generated alternative rotation with a generated
hostname therefore provides two benign real fields for one atomic production
system PATCH and one atomic restoration PATCH.

## Scope and non-scope

Add a typed aggregate-only `bitaxe-settings-patch-evidence-v1` workflow for one
Ultra 205. It will flash one exact clean package, derive the trusted same-origin
target from that session, read baseline hostname and rotation, submit one
`/api/system` PATCH containing generated non-secret hostname and alternative
valid rotation values, confirm both together in one immediate system-info
readback, restore both originals in one PATCH, confirm both together, clean up,
and publish only closed categories, identities, hashes, counts, and booleans.

Do not expose or publish origins, hostnames, rotations, ports, USB/network
identities, credentials, raw HTTP bodies, serial output, settings, or traces.
Do not read pool credentials, restart, mine, control ASIC, voltage, fan,
thermal, or power behavior, scan/discover the network, update, erase, write raw
flash, inject faults, terminate foreign processes, use direct UART, or
manipulate pins. This row does not claim reboot/power-loss durability, live
mutation of credential or safety-control fields, reconnect behavior, mining,
other boards, or release readiness.

## Implementation

- [ ] Add the Rust-owned command/evidence contract, independent validator, and
      synchronized TypeScript contract for the closed projection.
- [ ] Add the aggregate-only capture using admitted flash/monitor and trusted
      origin seams, exactly one combined hostname/rotation mutation PATCH, one
      combined readback, one combined restoration PATCH, and one combined
      restoration readback.
- [ ] Preserve the earliest typed failure through restoration and optional
      exact-package recovery; publish nothing until all identity, safety,
      cleanup, mode, and redaction checks pass.
- [ ] Add behavior-focused unit, failure-category, primary-precedence, no-
      clobber, privacy, and real-child-process tests.
- [ ] Run the full gate, push implementation, admit its exact schema-v3 package,
      then spend the sole detector and conditional capture.

## Verification and promotion

Run focused contract, settings, automation, and real-process targets followed
by, in order:

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

After a clean implementation commit is pushed, build and validate its exact
package and run exactly these bounded commands:

1. `test ! -e scratch/api003-settings-patch/corrected-wrapper-001 && (umask 077; mkdir -m 700 -p scratch/api003-settings-patch/corrected-wrapper-001 && just detect-ultra205 > scratch/api003-settings-patch/corrected-wrapper-001/detector.stdout 2>&1)`
2. Only after command 1 succeeds:
   `test ! -e scratch/api003-settings-patch/corrected-attempt-001 && test ! -e docs/parity/evidence/api003-settings-patch/settings-patch-projection.json && (umask 077; just capture-settings-patch-evidence --private-root scratch/api003-settings-patch/corrected-attempt-001 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api003-settings-patch/corrected-wrapper-001/detector.stdout --projection docs/parity/evidence/api003-settings-patch/settings-patch-projection.json --capture-timeout-seconds 240 > scratch/api003-settings-patch/corrected-wrapper-001/capture.stdout 2> scratch/api003-settings-patch/corrected-wrapper-001/capture.stderr)`

The ignored wrapper and attempt roots must be absent, mode 0700, and contain
only mode-0600 files. Detector failure stops before writes. Capture permits one
exact-package factory flash and normal USB reset, one generated two-field
mutation PATCH/readback, one exact two-field restoration PATCH/readback,
cleanup, and at most one exact-package recovery flash after an initial flash
effect. Preserve the earliest failure; accepted categories are
`hardware_blocked`, `evidence_invalid`, `timeout`, and `process_failed`. No
retry is permitted.

Promotion requires exact source/reference/package identity, one admitted board
205, trusted same-origin HTTP, exactly one atomic two-field mutation with both
values confirmed together, exactly one atomic two-field restoration with both
originals confirmed together, disabled mining and hardware control, cleanup,
private modes, an independently validated redacted projection, and every gate
passing. Otherwise withhold `RESULT.md` and public evidence, create a typed
non-verified closure, keep `API-003` at `implemented`, and stop without retry.
