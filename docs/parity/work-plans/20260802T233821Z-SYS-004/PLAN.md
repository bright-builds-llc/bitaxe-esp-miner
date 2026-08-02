# Parity work plan

- Run ID: `20260802T233821Z-SYS-004`
- Parity row: `SYS-004`
- Initial status: `in-progress`
- Source commit: `f1591ffc19e4fac28f5271228477372b6f4c7d55`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-sys004-version-reporting`

## Selection

The clean `main` worktree matched `origin/main`, and
`bazel run //tools/parity:report -- next-item --format json` reported no open
plan. Candidate order was evaluated without using private soak evidence or
broadening standing hardware authority.

- `CFG-001` still lacks promotable safety-critical default-profile voltage and
  frequency evidence. The private default-profile soak explicitly forbids
  parity promotion and ended at a repeated continuity boundary.
- `CFG-005`, `API-003`, and `API-009` still lack their complete live PATCH or
  claim-specific command-effect evidence; hostname-only durability, restart,
  mining actuation, and display identify do not prove the broader rows.
- `NET-001` still lacks reconnect/fallback timing and IPv6 parity, and the
  latest private network-continuity evidence is non-promotable and failed.
- `ASIC-002` through `ASIC-005`, `ASIC-007`, `STR-001`, `STR-006`, `STR-007`,
  `PWR-001` through `PWR-003`, `PWR-005`, `PWR-006`, `THR-001` through
  `THR-003`, and `SELF-001` retain explicit live hardware, safety-control,
  fault, production-mining, or exact-soak gaps. The consumed private soak
  attempts cannot close them.
- `API-002` still lacks full field-level live parity. `LOG-001` has live raw
  WebSocket evidence but not the explicitly required live
  `/api/system/logs` download check.
- `REL-001`, `REL-002`, and `REL-003` retain selected-partition, rollback,
  recovery, OTAWWW, and release-readiness gaps.
- `SAFE-10`, `SAFE-11`, `CFG-07`, `ASIC-09` through `ASIC-12`, `STR-08`,
  `STR-09`, `SAFE-12`, and `SAFE-13` require promotable live mining/safety
  evidence that the active terminal soak records explicitly do not provide.
- `V12-OPERATOR-SNAPSHOT-205` and `V12-RUNTIME-HEALTH-205` retain their exact
  Phase 36 substance corrections; the recent OTA result lists both as
  non-claims.

`SYS-004` is the first actionable row. The checklist still says its API
version surface is missing, while current code now projects upstream
`version`, `axeOSVersion`, and `idfVersion`, adds canonical semantic/source/
reference/build provenance, and has unit, API-compare, package, runtime
attestation, and historical live system-info evidence. This plan reconciles
that implementation only. It does not infer exact-current live API proof.

The repository task workflow, evidence privacy and hardware boundaries in
`AGENTS.md`, managed verification/testing/Rust standards, and the active
lessons on exact-boundary evidence and non-promotable hardware artifacts
govern this work.

## Scope and non-scope

Scope is limited to auditing the current version-reporting path from canonical
build provenance and read-only ESP-IDF/static-asset facts through
`ApiSnapshot` and `SystemInfoWire`, running focused regressions, recording a
row-specific worklog, and moving only `SYS-004` from `in-progress` to
`implemented` if the evidence passes.

No firmware, API, package, reference, hardware, credential, network, detector,
flash, monitor, HTTP, WebSocket, OTA, mining, safety-control, direct-UART, or
pin effect is authorized. No other checklist row may change. `verified` is out
of scope because no exact-current-package live `/api/system/info` observation
has yet bound the complete version trio and extended provenance surface.

## Implementation

- [ ] Trace the upstream version sources and the current canonical Rust
      projection through build provenance, platform identity, snapshot, and
      wire serialization.
- [ ] Run focused behavior regressions for build identity, system-info wire
      fields, package-manifest identity, and runtime boot attestation.
- [ ] Record the exact evidence and non-claims in an append-only `WORKLOG.md`,
      then commit it before changing the checklist.
- [ ] Transition only `SYS-004` to `implemented` with truthful evidence,
      targets, and notes, then synchronize progress from the evidence commit.

## Verification and promotion

Focused checks:

- `cargo test -p bitaxe-api build_identity --all-features`
- `cargo test -p bitaxe-api system_info_wire_serializes_upstream_field_names_and_encodings --all-features`
- `cargo test -p xtask package_manifest --all-features`
- `cargo test -p bitaxe-api runtime_boot_attestation --all-features`
- `bazel test //crates/bitaxe-api:tests //tools/xtask:tests`

Required plan, evidence, and finalization gates:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test`
- `just parity`
- `just parity-progress`
- `just verify-redaction`
- `just verify-reference`
- `git diff --check`

Promotion to `implemented` requires all focused checks, exact upstream field
mapping, canonical source/reference/build identity flow, current package and
runtime-attestation guards, historical live Ultra 205 observation of
`version`, `axeOSVersion`, and `idfVersion`, and a passing transition receipt.
Promotion to `verified` is prohibited in this run. It requires a later bounded
task with exact-current-package live API evidence and an explicit decision on
the static-asset version semantics; historical API output or boot markers alone
are insufficient.
