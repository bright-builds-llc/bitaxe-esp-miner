# Parity work plan

- Run ID: `20260820T064119Z-ASIC-010`
- Parity row: `ASIC-010`
- Initial status: `not-started`
- Source commit: `2fc51b4c0b3e3a0d4c59c318ad368dd62738392c`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-asic010-bm1397-core`

## Selection

The clean `main` worktree equals `origin/main`, the pinned reference is clean,
and the deterministic selector reports no open plan. Its candidate order is
`ASIC-009`, `SELF-001`, `BAP-002`, `STAT-003`, `ASIC-010`, then `BAP-001`.

`ASIC-009` has completed its authorized pure software surface but remains
hardware-blocked until a supported BM1368 board, firmware adapter, detector
contract, and redacted hardware regression exist. `SELF-001` remains
unavailable because no production-safe full self-test route exists. `BAP-002`
depends on unfinished `BAP-001` firmware/accessory UART lifecycle and live
interoperability; external UART or pin work is not authorized. `STAT-003`
remains environment-blocked after attempt-004 stopped at a distinct protected
pool/network-unavailable boundary, and its task prohibits an unchanged retry.

`ASIC-010` is therefore the first actionable row. Its Rust-owned target is the
pure `crates/bitaxe-asic` crate. Max/BM1397 hardware is unavailable, but a pure
implementation can advance the row from `not-started` to `implemented` without
activating firmware dispatch or making any hardware claim.

Material guidance includes `AGENTS.md`, `AGENTS.bright-builds.md`, the empty
effective overrides, architecture, code-shape, verification, testing, and Rust
standards, plus the active tracker/checklist and priority evidence,
authorization, hardware-retry, unit, and agent-runtime lessons. The active
lesson inputs total 31,758 bytes and exceed both deterministic startup limits;
all headings, the complete global file, and complete priority repository blocks
were loaded. Non-priority repository blocks were omitted under the bounded
policy. The existing August audit baseline is current and no distinct audit
trigger is due.

## Scope and non-scope

Advance only `ASIC-010` to at most `implemented`. Independently design a pure
BM1397 core under `crates/bitaxe-asic` covering:

- command/job framing with the shared CRC5 and CRC16 algorithms;
- chip identity, read/inactive/address commands, exact init register writes,
  difficulty, default/max baud, and the BM1397-specific PLL/frequency write
  sequence and ramp plan;
- the fixed 146-byte one/four-midstate work payload, 152-byte frame, and
  modulo-128 four-step job sequence;
- strict nine-byte job/register result decoding, valid-job and address checks,
  midstate-index version rolling, register classification, nonce byte order,
  and previous-nonce duplicate suppression; and
- provenance-bound golden fixtures and exact behavioral regressions.

Represent the upstream BM1397 version-mask setter truthfully as a no-frame
placeholder while retaining version rolling through precomputed midstates and
result correlation. Initialize unused midstate slots deterministically rather
than reproducing upstream uninitialized memory. Keep `dispatch_catalog_entry`
fail closed for BM1397 with `VerificationScope::NotHardwareVerified`.

Do not add a firmware UART adapter, activate board 102/Max, share BM1397 status
with BM1366 evidence, or change user-visible catalog defaults. Reference facts
must record the pinned commit and exact source breadcrumbs; do not copy upstream
function bodies or prose expression into MIT-owned Rust source.

This plan authorizes local Rust/fixture/docs edits, deterministic tests,
build/package, Git commit, and push only. It authorizes no credentials,
protected attempt roots, detector, USB/device/network runtime, flash, monitor,
mining, restart, recovery, hardware attempt, fault injection, external
UART/BAP, pins, or electrical work.

## Implementation

- [ ] Add typed BM1397 framing, init/frequency, work, result, and closed-error
      behavior in the pure ASIC crate.
- [ ] Add pinned-reference golden fixtures and focused Arrange/Act/Assert
      regressions for one/four-midstate work, version rolling, duplicate
      suppression, register reads, frequency sequencing, and invalid inputs.
- [ ] Preserve deferred firmware dispatch and prove the current Ultra 205
      package path remains unaffected.
- [ ] Produce a source-bound summary and `WORKLOG.md`, transition only
      `ASIC-010` to `implemented` with `unit,golden`, sync progress, run every
      final gate, push, and leave this task active and unarchived.

## Verification and promotion

Focused verification is:

- `cargo test -p bitaxe-asic bm1397`
- `cargo test -p bitaxe-asic`
- `cargo test -p bitaxe-asic dispatch_non_v1_asic_families_are_deferred_without_hardware_scope`
- `bazel test //crates/bitaxe-asic:tests`
- `just verify-reference`
- `just package`

The mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`, plus fixture provenance,
sensitive-value, reference-cleanliness, generated/build-source, source-diff,
receipt-hash, task-uniqueness, and final diff checks.

Transition to `implemented` with `unit,golden` only when the complete pure
surface above and its fixture tests pass, BM1397 dispatch remains deferred, the
Ultra 205 package remains buildable, and all gates pass. Do not transition to
`verified`; that requires a separately planned supported BM1397 board, firmware
adapter, detector admission, hardware smoke/regression, safe stop, and
redaction.

On success create `WORKLOG.md` and the evidence summary, commit implementation
and evidence as `SOURCE_COMMIT`, transition only `ASIC-010`, sync progress,
leave the task active with the exact firmware/hardware gaps, final-gate, and
push. On terminal failure create `CLOSURE.md`, preserve the truthful row status,
and do not sync an unchanged checklist.

This plan does not implement or verify firmware dispatch, UART ownership, real
chip enumeration, initialization timing, frequency or baud effects, live
work/results, voltage/fan/thermal behavior, safe stop, any BM1397 board,
BM1368/BM1370 breadth, mining, OTA/recovery, or release readiness.
