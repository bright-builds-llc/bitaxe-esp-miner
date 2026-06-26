# Phase 2 Ultra 205 Config And NVS Model Evidence

**Date:** 2026-06-26
**Scope:** Phase 2 pure Rust config/default/schema/validation/persistence model
**Reference:** `reference/esp-miner` at `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Scope Conclusion

Phase 2 verifies pure config/default/schema/validation/persistence behavior only; firmware ESP-IDF NVS adapter reboot evidence and voltage/fan/thermal/power/ASIC hardware effects are not verified by this evidence.

The fixtures used by this phase are derived from public upstream defaults and schema metadata. No private Wi-Fi credentials or operator pool credentials were added.

## `bazel test //crates/bitaxe-config:tests`

**Result:** Passed during Plan 02-04 Task 1 and the Phase 2 gate.

**Covers:** Ultra 205 defaults, board catalog evidence scope, NVS schema, migrations, corrupt float fallback, settings update decisions, fixtures, and pure persistence load/update/reload behavior.

## `just test`

**Result:** Passed during the Plan 02-04 Phase 2 gate.

**Covers:** Repo-owned test entrypoint routed through Bazel.

## `just parity`

**Result:** Passed during the Plan 02-04 Phase 2 gate.

**Covers:** Checklist parsing, reference cleanliness guard, reference commit reporting, and invalid verified-claim checks.

## `cargo fmt --all`

**Result:** Passed before Plan 02-04 task commits and during the Phase 2 gate.

## `cargo clippy --all-targets --all-features -- -D warnings`

**Result:** Passed before Plan 02-04 task commits and during the Phase 2 gate.

## `cargo build --all-targets --all-features`

**Result:** Passed before Plan 02-04 task commits and during the Phase 2 gate.

## `cargo test --all-features`

**Result:** Passed before Plan 02-04 task commits and during the Phase 2 gate.

## Evidence Mapping

| Requirement | Evidence | Boundary |
| --- | --- | --- |
| CFG-001 | `crates/bitaxe-config/src/defaults.rs`, `crates/bitaxe-config/fixtures/ultra-205-defaults.csv`, `bazel test //crates/bitaxe-config:tests` | Pure default values only. |
| CFG-002 | `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` | Gamma 601/BM1370 remains deferred and receives no verification from Ultra 205 evidence. |
| CFG-003 | `crates/bitaxe-config/src/catalog.rs`, `crates/bitaxe-config/fixtures/catalog.json` | Catalog representation only; non-205 entries are not hardware-verified. |
| CFG-004 | `crates/bitaxe-config/src/nvs.rs`, `crates/bitaxe-config/src/persistence.rs`, `crates/bitaxe-config/fixtures/nvs-schema.json`, `crates/bitaxe-config/fixtures/nvs-migrations.json` | Pure schema, default-load, migration, and snapshot reload semantics only. |
| CFG-005 | `crates/bitaxe-config/src/settings.rs`, `crates/bitaxe-config/src/persistence.rs`, `crates/bitaxe-config/fixtures/settings-updates.json` | Pure update/reload decisions only; API PATCH route and firmware NVS adapter evidence remain later work. |
| CFG-06 | All Phase 2 fixtures under `crates/bitaxe-config/fixtures/` | Fixture and golden coverage for defaults, catalog, NVS schema, migrations, and settings update cases. |
