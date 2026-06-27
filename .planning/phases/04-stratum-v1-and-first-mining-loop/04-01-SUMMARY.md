---
phase: 04-stratum-v1-and-first-mining-loop
plan: "01"
subsystem: stratum
tags: [rust, stratum-v1, json-rpc, serde-json, bazel]

requires:
  - phase: 03-bm1366-asic-protocol-and-safe-initialization
    provides: "Typed BM1366 ASIC command/result boundaries that later Stratum mining work will target"
provides:
  - "Typed Stratum v1 JSON-RPC request IDs, errors, client serializers, server parser, and protocol fixtures"
  - "Active bitaxe-stratum runtime status replacing the Phase 1 deferred marker"
  - "Bazel-visible fixture and dependency wiring for bitaxe-stratum"
affects: [phase-04-stratum, phase-05-api-telemetry, parity-evidence]

tech-stack:
  added: [thiserror, serde, serde_json]
  patterns:
    - "Parse pool JSON at the crate boundary into typed Stratum v1 domain values"
    - "Use provenance-rich reference-derived fixtures without copying GPL source expression"

key-files:
  created:
    - crates/bitaxe-stratum/src/error.rs
    - crates/bitaxe-stratum/src/jsonrpc.rs
    - crates/bitaxe-stratum/src/v1.rs
    - crates/bitaxe-stratum/src/v1/messages.rs
    - crates/bitaxe-stratum/fixtures/v1/protocol-cases.json
  modified:
    - Cargo.lock
    - MODULE.bazel.lock
    - crates/bitaxe-stratum/Cargo.toml
    - crates/bitaxe-stratum/BUILD.bazel
    - crates/bitaxe-stratum/src/lib.rs

key-decisions:
  - "Stratum v1 parser rejects unknown methods, invalid hex fields, malformed params, malformed responses, and oversized extranonce2 lengths before mining state can consume pool data."
  - "Protocol tests and fixtures use synthetic usernames/passwords only, keeping real pool credentials out of source and evidence."
  - "TDD RED failures were run but not committed because AGENTS.md requires passing Rust verification before every commit."

patterns-established:
  - "Stratum v1 request and response behavior is represented by typed enums and structs rather than raw serde_json::Value outside the parser boundary."
  - "Reference breadcrumbs live at the v1 module and messages module boundaries, with fixture metadata carrying source path, reference commit, license posture, derivation, and checklist IDs."

requirements-completed: [STR-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 04-2026-06-27T13-17-33
generated_at: 2026-06-27T14:18:29Z

duration: 10 min
completed: 2026-06-27
---

# Phase 04 Plan 01: Stratum V1 Protocol Core Summary

**Typed Stratum v1 JSON-RPC parser and serializer core with provenance-backed fixtures**

## Performance

- **Duration:** 10 min
- **Started:** 2026-06-27T14:07:45Z
- **Completed:** 2026-06-27T14:18:29Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Replaced the deferred Stratum runtime marker with an active Stratum v1 pure-core status.
- Added typed Stratum v1 errors, JSON-RPC request IDs, client serializers, server parser, and response/domain models.
- Added metadata-rich protocol fixtures covering client, pool notification, difficulty, extranonce, version-mask, submit, success, and error cases.
- Wired `bitaxe-stratum` Cargo and Bazel dependencies for `thiserror`, `serde`, `serde_json`, fixtures, and generated Bazel module lock metadata.

## Task Commits

Each task was committed atomically:

1. **Task 1: Establish typed Stratum v1 module and error surface** - `d0be610` (feat)
2. **Task 2: Implement Stratum v1 message parsing and serialization** - `ea612e9` (feat)

## Files Created/Modified

- `Cargo.lock` - Records new `bitaxe-stratum` dependency edges for `serde`, `serde_json`, and `thiserror`.
- `MODULE.bazel.lock` - Refreshes crate-universe lock metadata for the updated Stratum crate dependencies.
- `crates/bitaxe-stratum/Cargo.toml` - Adds workspace dependencies required by the protocol core.
- `crates/bitaxe-stratum/BUILD.bazel` - Adds new Stratum sources, fixture compile data, and crate deps.
- `crates/bitaxe-stratum/src/lib.rs` - Exports `error`, `jsonrpc`, `v1`, and active runtime status.
- `crates/bitaxe-stratum/src/error.rs` - Defines `StratumV1Error`.
- `crates/bitaxe-stratum/src/jsonrpc.rs` - Defines `StratumRequestId`.
- `crates/bitaxe-stratum/src/v1.rs` - Adds the Stratum v1 module facade and breadcrumbs.
- `crates/bitaxe-stratum/src/v1/messages.rs` - Implements typed client serializers, server parser, domain values, and focused tests.
- `crates/bitaxe-stratum/fixtures/v1/protocol-cases.json` - Records provenance-rich Stratum v1 protocol cases.

## Decisions Made

- Raw pool JSON now terminates at `parse_server_message`, which returns typed Stratum v1 domain messages or typed errors.
- Oversized extranonce2 lengths are rejected instead of silently clamped, satisfying the plan threat model for malformed pool input.
- TDD RED states were verified locally but not committed, preserving the repo rule that every Rust commit follows passing format, clippy, build, and test checks.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Enforced fail-closed oversized extranonce2 parsing**
- **Found during:** Task 2 (Implement Stratum v1 message parsing and serialization)
- **Issue:** The task action mentioned capping extranonce2 length, while the threat model required malformed and oversized pool input to be rejected before state mutation.
- **Fix:** `parse_extranonce2_len` rejects values above `MAX_EXTRANONCE_2_LEN` with `StratumV1Error::InvalidField`.
- **Files modified:** `crates/bitaxe-stratum/src/v1/messages.rs`
- **Verification:** `cargo test -p bitaxe-stratum stratum_v1_protocol --all-features`; `bazel test //crates/bitaxe-stratum:tests`; full Rust pre-commit sequence.
- **Committed in:** `ea612e9`

**2. [Rule 2 - Missing Critical] Kept TDD RED failures out of git history**
- **Found during:** Task 1 and Task 2 TDD execution
- **Issue:** The generic GSD TDD flow calls for RED commits, but AGENTS.md requires passing Rust format, clippy, build, and tests before every commit.
- **Fix:** Ran RED failures locally, implemented GREEN behavior, and committed only passing task states.
- **Files modified:** Task-owned Stratum files only.
- **Verification:** Task-scoped RED failures were observed before implementation; all final pre-commit checks passed before each task commit.
- **Committed in:** `d0be610`, `ea612e9`

**Total deviations:** 2 auto-fixed (2 missing critical)
**Impact on plan:** Both adjustments enforce repo safety and threat-model correctness without expanding beyond the planned Stratum v1 protocol core.

## Issues Encountered

None.

## Known Stubs

None.

## Authentication Gates

None.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p bitaxe-stratum stratum_v1_contract --all-features` - passed
- `cargo test -p bitaxe-stratum stratum_v1_protocol --all-features` - passed
- `bazel test //crates/bitaxe-stratum:tests` - passed
- `cargo fmt --all` - passed before each task commit
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before each task commit
- `cargo build --all-targets --all-features` - passed before each task commit
- `cargo test --all-features` - passed before each task commit
- `git status --short reference/esp-miner` - clean

## Next Phase Readiness

Ready for Plan 04-02. The Stratum v1 parser/serializer core is in place for fake-pool lifecycle, runtime counters, and later mining-loop state work.

## Self-Check: PASSED

- Found `.planning/phases/04-stratum-v1-and-first-mining-loop/04-01-SUMMARY.md`
- Found task commit `d0be610`
- Found task commit `ea612e9`

---
*Phase: 04-stratum-v1-and-first-mining-loop*
*Completed: 2026-06-27*
