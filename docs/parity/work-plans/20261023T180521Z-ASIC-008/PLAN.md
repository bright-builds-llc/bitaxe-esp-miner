# Parity work plan

- Run ID: `20261023T180521Z-ASIC-008`
- Parity row: `ASIC-008`
- Initial status: `deferred`
- Source commit: `2b839cf19f954bb54ae9829404a411bdd6dbeca0`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-gamma-601-bring-up` (phase E; owner-approved plan
  `gamma-601-phase-e`, 2026-10-23)

## Selection

The Gamma 601 bring-up has completed phases A to D:
- identification and registration;
- the safe boot;
- Wi-Fi;
- the TPS546 configuration and a bounded 1.15 V enable, with the ASIC held
  in reset (`docs/parity/evidence/20261023-gamma601-vout-check.md`).

Phase F will release the BM1370 from reset. Before that, phase E builds
the chip's protocol as a pure core. ASIC-008's Rust-owned target is the
pure `crates/bitaxe-asic` crate. It can advance from `deferred` to
`implemented` without enabling firmware dispatch or making a hardware
claim. ASIC-008 cannot reuse Ultra 205 evidence (ADR-0014, ADR-0041
item 4).

## Scope and non-scope

Advance only `ASIC-008`, to at most `implemented`. Add an independently
designed pure BM1370 protocol core under `crates/bitaxe-asic`, following
the BM1368 core's layout (ASIC-009). It covers:
- command and job framing, through the shared BM13xx framing and CRCs;
- the initialization plan in upstream's order:
  - the version mask, the `CHIP_ID` read and the split for chip detection;
  - the global, addressing and per-chip register writes;
  - the B9/54/3C block;
  - the 6.25 MHz frequency ramp in both directions;
  - the nonce space with the BM1370's hash-counting errata;
  - then, outside the plan as upstream keeps it, the max-baud switch;
- the PLL plan within the BM1370's feedback-divider range (160-239), from
  the shared search generalized to take a range, with the BM1366 path
  unchanged;
- the job interval and the expected hashrate;
- the 82-byte work payload, the 88-byte frame, and job ids in steps of 24
  modulo 128;
- strict 11-byte decoding of nonce results, register reads and chip
  identity:
  - the job lookup;
  - small core, core, ASIC index and version bits;
  - the register map;
  - closed faults;
- provenance-bound golden fixtures, computed by an independent
  reimplementation of upstream's algorithms. The init frames match the
  CRCs in upstream's own hand-dump comments.

Keep `dispatch_catalog_entry` fail closed for BM1370 and retain
`VerificationScope::NotHardwareVerified`. Do not add a firmware UART
adapter, change the stratum job-id step, or activate the Gamma 601's ASIC.

Reference-derived constants and fixtures may contain only protocol facts,
and must record the pinned reference commit and source breadcrumbs. Do
not copy upstream function bodies or prose into MIT-owned Rust files.

This plan authorizes:
- local Rust, fixture and docs edits;
- tests, builds and packages;
- Git commits and pushes.

It authorizes no detector, device, USB or network runtime, flash,
monitor, mining, hardware attempt, external UART, pins or electrical work.

## Implementation

- [ ] A typed BM1370 protocol module with pure init, work and result
      behavior and closed errors; the shared PLL search taking a divider
      range.
- [ ] Provenance-bound golden fixtures and focused Arrange/Act/Assert
      coverage for every implemented surface, plus upstream's own PLL test
      vector.
- [ ] Deferred firmware dispatch kept, with a regression that the BM1370
      remains non-active.
- [ ] An independent review that recomputes the golden values from the
      pinned reference.
- [ ] A source-bound evidence summary and `WORKLOG.md`, commit as
      `SOURCE_COMMIT`, transition only ASIC-008 to `implemented`, sync
      progress, run the final gates, and push.

## Verification and promotion

**Focused checks:**
- `cargo test -p bitaxe-asic bm1370`;
- `cargo test -p bitaxe-asic`;
- `bazel test //crates/bitaxe-asic:tests`;
- `just verify-reference`;
- `just package`.

**Mandatory gates:**
- `cargo fmt --all`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo build --all-targets --all-features`;
- `cargo test --all-features`;
- `bun scripts/bright-builds-check.ts all`;
- `just test`;
- `just verify-redaction`;
- `just parity`;
- `just parity-progress`.

Transition to `implemented` with `unit,golden` only when:
- the full pure protocol surface above and its fixture-backed tests pass;
- firmware dispatch stays deferred;
- the Ultra 205 package stays unaffected;
- every gate passes.

Do not transition to `verified`. That requires phase F's separately
planned Gamma 601 hardware work: a firmware adapter, detector admission,
a hardware smoke and regression, safe-stop evidence, and redaction.

## Non-claims

This plan does not implement or verify:
- firmware dispatch, UART ownership or real chip enumeration;
- initialization timing, frequency or baud effects;
- live work or result traffic;
- voltage, fan or thermal behavior with the ASIC running;
- safe stop or mining;
- any other board or ASIC;
- OTA or recovery, or release readiness.
