# Parity work plan

- Run ID: `20260820T060848Z-ASIC-009`
- Parity row: `ASIC-009`
- Initial status: `not-started`
- Source commit: `e687dfa54bfbea580d396bffe2d5299733c7aad5`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-asic009-bm1368-core`

## Selection

The clean `main` worktree equals `origin/main`, the pinned reference is clean,
and the selector reports no open plan. Candidate order begins `SELF-001`,
`BAP-002`, `STAT-003`, then `ASIC-009`.

`SELF-001` remains unavailable because no production-safe full self-test route
exists for its hardware regression. `BAP-002` remains dependency- and safety-
blocked by unfinished `BAP-001` firmware/UART lifecycle plus unauthorized
external electrical UART work. `STAT-003` remains environment-blocked after
attempt-004's distinct `network_unavailable` result; its active task prohibits
an unchanged retry without an objective protected pool/network recovery signal.

`ASIC-009` is the first actionable remaining row. Its Rust-owned target is the
pure `crates/bitaxe-asic` crate. BM1368 boards are unavailable for hardware
evidence, but pure implementation can still advance the row from `not-started`
to `implemented` without enabling firmware dispatch or making a hardware claim.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
`standards-overrides.md`, managed architecture, code-shape, verification,
testing, and Rust standards, the active tracker/checklist, and bounded lessons
for evidence, authorization, hardware retries, units, and agent-runtime timing.
Active lesson inputs total 31,758 bytes, so every heading was inventoried and
priority complete blocks plus the complete global file were loaded; remaining
non-priority repository blocks were omitted under the deterministic budget. The
August lesson-audit baseline remains current and no new audit trigger is due.

## Scope and non-scope

Advance only `ASIC-009` to at most `implemented`. Add an independently designed
pure BM1368 protocol core under `crates/bitaxe-asic` covering:

- command/job framing and CRC use;
- chip identity, default/max baud, version mask, register writes, chain
  inactive/addressing, difficulty, frequency ramp, and nonce-space init plans;
- the 82-byte work payload, 88-byte frame, 24-step job sequence, and packed
  result-job lookup;
- strict 11-byte job/register result decoding, register classification, core/
  address checks, version bits, and submit-nonce byte order;
- checked-in reference-derived golden fixture metadata and exact behavioral
  regressions.

Keep `dispatch_catalog_entry` fail closed for BM1368 and retain
`VerificationScope::NotHardwareVerified`. Do not add a firmware UART adapter,
activate any non-205 board, or reuse BM1368 behavior as BM1366 evidence.

Reference-derived constants and fixtures may contain only protocol facts and
must record the pinned reference commit and exact source breadcrumbs. Do not
copy upstream function bodies or prose expression into MIT-owned Rust files.

This plan authorizes local Rust/fixture/docs edits, tests, build/package, Git
commit, and push only. It authorizes no credential or protected-attempt access,
detector, device/USB/network runtime, flash, monitor, mining, restart, recovery,
hardware attempt, fault injection, external UART/BAP, pins, or electrical work.

## Implementation

- [ ] Add a typed BM1368 protocol module with pure init/work/result behavior
      and closed errors.
- [ ] Add provenance-bound golden fixtures and focused Arrange/Act/Assert
      coverage for every implemented surface.
- [ ] Preserve deferred firmware dispatch and add explicit regression coverage
      that BM1368 remains non-active without hardware evidence.
- [ ] Produce a source-bound evidence summary and `WORKLOG.md`, commit as
      `SOURCE_COMMIT`, transition only ASIC-009 to `implemented`, sync progress,
      run final gates, and push while leaving the active task unarchived.

## Verification and promotion

Focused verification is:

- `cargo test -p bitaxe-asic bm1368`
- `cargo test -p bitaxe-asic dispatch_non_v1_asic_families_are_deferred_without_hardware_scope`
- `bazel test //crates/bitaxe-asic:tests`
- `just verify-reference`
- `just package`

The mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`, plus fixture provenance,
sensitive-value, reference cleanliness, generated/build source declarations,
source-diff, and final diff checks.

Transition to `implemented` with `unit,golden` only when the full pure protocol
surface above and its fixture-backed tests pass, firmware dispatch remains
deferred, the current Ultra 205 package remains unaffected, and all mandatory
gates pass. Do not transition to `verified`; that requires a separately planned
supported BM1368 board, firmware adapter, detector admission, hardware smoke/
regression, safe-stop evidence, and redaction.

On success create `WORKLOG.md` and the evidence summary, commit implementation
and evidence as `SOURCE_COMMIT`, transition only `ASIC-009` to `implemented`
with `unit,golden`, sync progress, leave this task active with the exact hardware
and firmware gaps, run final gates, and push. On failure create `CLOSURE.md`,
leave ASIC-009 `not-started`, and do not sync unchanged progress.

## Non-claims

This plan does not implement or verify firmware dispatch, UART ownership, real
chip enumeration, initialization timing, frequency/baud effects, live work or
result traffic, voltage/fan/thermal behavior, safe stop, any BM1368 board,
BM1397/BM1370, other boards/ASICs, mining, OTA/recovery, or release readiness.
