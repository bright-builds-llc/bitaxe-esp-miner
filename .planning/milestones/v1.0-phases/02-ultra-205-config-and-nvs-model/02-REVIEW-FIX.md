---
phase: 02-ultra-205-config-and-nvs-model
fixed_at: 2026-06-26T18:34:29Z
review_path: .planning/phases/02-ultra-205-config-and-nvs-model/02-REVIEW.md
iteration: 4
findings_in_scope: 7
fixed: 7
skipped: 0
status: all_fixed
---

# Phase 02: Code Review Fix Report

**Fixed at:** 2026-06-26T18:34:29Z
**Source review:** .planning/phases/02-ultra-205-config-and-nvs-model/02-REVIEW.md
**Iterations:** 4

**Summary:**
- Findings in scope: 7
- Fixed: 7
- Skipped: 0
- Final review status: clean

## Fixed Issues

### WR-01: Runtime settings reject upstream-valid custom voltage/frequency values

**Files modified:** `crates/bitaxe-config/src/settings.rs`
**Commit:** ca65f2a
**Applied fix:** Removed Ultra 205 catalog option gates from the generic settings PATCH validation path and added a regression test proving schema-valid custom `frequency=486` and `coreVoltage=1199` are accepted.

### WR-02: Active float writes use migration formatting instead of upstream active-save formatting

**Files modified:** `crates/bitaxe-config/src/settings.rs`, `crates/bitaxe-config/src/nvs.rs`, `crates/bitaxe-config/src/persistence.rs`, `crates/bitaxe-config/fixtures/settings-updates.json`
**Commit:** 583f8f7
**Applied fix:** Added active float serialization with six fixed decimals for settings writes, updated active write fixture and test expectations to `485.000000`, and preserved legacy migration expectations at `485`.

### WR-03: Golden fixture coverage is claimed but most fixtures are not exercised

**Files modified:** `crates/bitaxe-config/src/golden_tests.rs`, `crates/bitaxe-config/src/lib.rs`, `crates/bitaxe-config/BUILD.bazel`
**Commit:** 5f4d767
**Applied fix:** Added fixture-backed golden tests for Ultra 205 defaults, board catalog data, NVS schema rows, migration decisions, active compatibility writes, and corrupt loaded values; wired the new test module and CSV parser into the Bazel target.

### WR-04: Rejected settings updates skipped migration-aware reload state

**Files modified:** `crates/bitaxe-config/src/persistence.rs`
**Commit:** 396a5ba
**Applied fix:** Rejected settings patches now report current loaded values through the same migration-aware reload path while preserving the original snapshot and emitting no write/erase operations.

### WR-05: Golden schema comparison ignored missing optional fields

**Files modified:** `crates/bitaxe-config/src/golden_tests.rs`, `crates/bitaxe-config/fixtures/nvs-schema.json`
**Commit:** 396a5ba
**Applied fix:** Schema golden tests now compare defaults, min, max, and array size exactly, and the fixture records all modeled defaults/ranges.

### WR-06: Mixed-case SV2 migration target and BM1370XP catalog name diverged from upstream

**Files modified:** `crates/bitaxe-config/src/nvs.rs`, `crates/bitaxe-config/fixtures/nvs-migrations.json`, `crates/bitaxe-config/src/catalog.rs`, `crates/bitaxe-config/fixtures/catalog.json`, `crates/bitaxe-config/src/golden_tests.rs`, `docs/parity/evidence/phase-02-ultra-205-config-nvs-model.md`
**Commit:** e1cb097
**Applied fix:** Mixed-case `fbSv2ChanType` migration now follows upstream first-missing-key order, BM1370XP is represented as an internal profile id while `model()` returns upstream `BM1370`, and the evidence mapping no longer references a nonexistent parity checklist row.

### WR-07: Mixed-case SV2 migration metadata did not match behavior

**Files modified:** `crates/bitaxe-config/src/nvs.rs`
**Commit:** 83484c5
**Applied fix:** `migration_rules()` now names `sv2chantype` as the primary mixed-case legacy target and has a regression test tying the metadata to the representative migration decision.

## Verification

- `cargo fmt --all` passed before each fix commit.
- `cargo clippy --all-targets --all-features -- -D warnings` passed before each fix commit.
- `cargo build --all-targets --all-features` passed before each fix commit.
- `cargo test --all-features` passed before each fix commit.
- `bazel test //crates/bitaxe-config:tests` passed after the final fix commits.
- `just parity` passed with `validation_errors: none`.
- `git status --short reference/esp-miner` produced no output.
- Final standard-depth re-review status: `clean` with 0 findings.

---

_Fixed: 2026-06-26T18:34:29Z_
_Fixer: the agent (gsd-code-fixer)_
_Iterations: 4_
