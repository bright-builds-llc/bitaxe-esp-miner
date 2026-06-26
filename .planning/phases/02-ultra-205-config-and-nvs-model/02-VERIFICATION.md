---
phase: 02-ultra-205-config-and-nvs-model
verified: 2026-06-26T18:42:08Z
status: passed
score: "12/12 must-haves verified"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-06-26T15-47-58
generated_at: 2026-06-26T18:42:08Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 2: Ultra 205 Config And NVS Model Verification Report

**Phase Goal:** Users and firmware can rely on upstream-compatible Ultra 205 settings, defaults, validation, persistence, and scoped board identity.
**Verified:** 2026-06-26T18:42:08Z
**Status:** passed
**Re-verification:** No - initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Ultra 205 defaults expose hostname, pool, ASIC, fan, self-test, device, and board values from `config-205.cvs`. | VERIFIED | `defaults.rs` returns exact values for `bitaxe`, `public-pool.io`, `solo.ckpool.org`, BM1366, 485 MHz, 1200 mV, board 205, fan 100, self-test true, and overheat false; `golden_tests.rs` compares these to `fixtures/ultra-205-defaults.csv`. |
| 2 | Board/device/ASIC catalog represents Ultra 205 as the active evidence target and keeps non-205 upstream boards not hardware-verified. | VERIFIED | `catalog.rs` includes board `205` as `VerificationScope::ActiveUltra205`; all other catalog entries use `NotHardwareVerified`, including Gamma 601/BM1370 and TPS546 boards. |
| 3 | Reference-derived fixtures carry provenance metadata and no real operator Wi-Fi or private pool credentials. | VERIFIED | CSV/JSON fixtures include `source`, `reference_commit`, and GPL posture metadata; Wi-Fi rows are empty and pool data matches upstream public defaults. |
| 4 | Config crate exposes exact upstream NVS namespace, key names, stored types, REST names, defaults, ranges, and indexed-key behavior. | VERIFIED | `nvs.rs` defines `NVS_NAMESPACE = "main"`, 15-byte key parsing, `StoredType` variants, `RestFieldName`, `all_settings_schema()`, scoreboard `array_size: 20`, and fixture-backed schema tests. |
| 5 | Legacy NVS migrations produce typed write/erase decisions without touching ESP-IDF NVS. | VERIFIED | `migration_decisions()` emits inert `NvsWrite`/`NvsErase` decisions for `asicfrequency`, `fanspeed`, protocol, SV2 channel, and mixed-case legacy keys; no ESP-IDF NVS calls exist in `nvs.rs`. |
| 6 | Missing keys and corrupt float strings load as configured defaults in the pure model. | VERIFIED | `load_setting_value()` falls back to schema defaults; tests prove `asicfrequency_f="bad"` loads as `485.0` and missing snapshot keys load Ultra 205 defaults. |
| 7 | Invalid frequency, voltage, fan, temperature, hostname, port, pool, TLS, bool-like, and board identifier inputs are rejected before firmware/API use. | VERIFIED | `validation.rs` and `settings.rs` enforce typed ranges, enum values, string lengths, bool-as-u16, NVS key limits, and active board scope. Invalid fixture cases reject with no writes. |
| 8 | Accepted settings updates produce typed NVS write decisions using the same schema and migration behavior. | VERIFIED | `apply_settings_patch()` walks `all_settings_schema()`, emits `NvsWrite` values, and adds legacy mirrors for frequency and manual fan writes. |
| 9 | Voltage and frequency validation does not claim hardware-control parity. | VERIFIED | Validation returns inert domain values only; parity checklist keeps CFG-001 implemented, not hardware-verified, for frequency/voltage use. |
| 10 | Pure config snapshots can load defaults, apply valid updates, reject invalid updates, migrate legacy keys, and reload without ESP-IDF. | VERIFIED | `persistence.rs` uses `NvsSnapshot`, `load_snapshot`, `apply_patch_to_snapshot`, and `reload_snapshot`; tests cover default load, valid roundtrip, rejected updates, legacy migration, and corrupt float fallback. |
| 11 | Parity checklist CFG rows record implementation pointers and evidence without claiming firmware NVS adapter or hardware-control verification. | VERIFIED | `docs/parity/checklist.md` links CFG rows to Phase 2 evidence, keeps Gamma 601 deferred, and states API PATCH route, firmware NVS adapter, and hardware effects remain later evidence. |
| 12 | Phase verification commands prove config tests and parity reporting remain healthy. | VERIFIED | Current checkout passes `cargo fmt --all -- --check`, Clippy, Cargo build/test, `bazel test //crates/bitaxe-config:tests`, `just parity`, and reference cleanliness check. |

**Score:** 12/12 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/bitaxe-config/fixtures/ultra-205-defaults.csv` | Ultra 205 golden defaults with provenance | VERIFIED | Exists with exact defaults, reference commit, source, license posture, and empty Wi-Fi fields. |
| `crates/bitaxe-config/src/defaults.rs` | Typed `Ultra205Defaults` API | VERIFIED | Exports `Ultra205Defaults` and `ultra_205_defaults`; golden tests compare against CSV fixture. |
| `crates/bitaxe-config/src/catalog.rs` | Typed board/device/ASIC catalog | VERIFIED | Exports catalog entries, ASIC profiles, capabilities, and verification scopes. |
| `crates/bitaxe-config/src/nvs.rs` | Typed NVS schema and migration model | VERIFIED | Defines namespace, key types, storage types, schema rows, migration rules, load defaults, and tests. |
| `crates/bitaxe-config/fixtures/nvs-schema.json` | Reference-derived schema fixture | VERIFIED | Fixture rows are parsed and compared to `all_settings_schema()`. |
| `crates/bitaxe-config/fixtures/nvs-migrations.json` | Reference-derived migration cases | VERIFIED | Fixture cases are parsed and compared to migration/compatibility/load behavior. |
| `crates/bitaxe-config/src/validation.rs` | Boundary validation newtypes/errors | VERIFIED | Implements typed validators for ranges, enums, hostnames, keys, bool-like values, and board scope. |
| `crates/bitaxe-config/src/settings.rs` | Pure settings update API | VERIFIED | Implements REST-name patch handling, schema-driven validation, and inert NVS write decisions. |
| `crates/bitaxe-config/fixtures/settings-updates.json` | Valid/invalid update cases | VERIFIED | Fixture drives accepted/rejected update tests. |
| `crates/bitaxe-config/src/persistence.rs` | Snapshot load/update/reload model | VERIFIED | Implements `NvsSnapshot`, `PersistenceDecision`, `load_snapshot`, `apply_patch_to_snapshot`, and `reload_snapshot`. |
| `docs/parity/evidence/phase-02-ultra-205-config-nvs-model.md` | Phase 2 command evidence and scoped conclusions | VERIFIED | Contains required command headings and explicit pure-config-only scope statement. |
| `docs/parity/checklist.md` | CFG evidence ledger | VERIFIED | CFG rows cite Phase 2 evidence while preserving deferred/hardware boundaries. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `defaults.rs` | `fixtures/ultra-205-defaults.csv` | Golden test | VERIFIED | `golden_tests.rs` uses `include_str!("../fixtures/ultra-205-defaults.csv")` and compares `ultra_205_defaults()` output. |
| `catalog.rs` | `reference/esp-miner/main/device_config.h` | Module breadcrumb | VERIFIED | `catalog.rs` starts with `Reference: reference/esp-miner/main/device_config.h`. |
| `nvs.rs` | `reference/esp-miner/main/nvs_config.c` | Module breadcrumbs | VERIFIED | `nvs.rs` documents source breadcrumbs for settings and migration behavior. |
| `nvs.rs` | `fixtures/nvs-schema.json` | Fixture/golden tests | VERIFIED | `golden_tests.rs` loads `nvs-schema.json` and compares rows to `all_settings_schema()`. |
| `settings.rs` | `nvs.rs` | Schema-driven validation and writes | VERIFIED | `settings.rs` imports and uses `all_settings_schema`, `SettingSchema`, `StoredType`, and compatibility writes. |
| `settings.rs` | `reference/esp-miner/main/http_server/http_server.c` | PATCH validation breadcrumb | VERIFIED | `settings.rs` includes the upstream HTTP server breadcrumb. |
| `persistence.rs` | `settings.rs` | Applies settings writes to snapshot | VERIFIED | `apply_patch_to_snapshot()` calls `apply_settings_patch()` and applies accepted writes. |
| `docs/parity/checklist.md` | Phase 2 evidence doc | Evidence links | VERIFIED | CFG rows link to `evidence/phase-02-ultra-205-config-nvs-model.md`. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `defaults.rs` | `ULTRA_205_DEFAULTS` | `config-205.cvs`-derived constants plus CSV golden test | Yes - exact upstream seed defaults | VERIFIED |
| `catalog.rs` | `BOARD_CATALOG` | `device_config.h`-derived constants plus JSON golden test | Yes - typed board/ASIC catalog | VERIFIED |
| `nvs.rs` | `all_settings_schema()` rows | `nvs_config.c` fixture and schema constants | Yes - schema rows with defaults/ranges/rest names | VERIFIED |
| `settings.rs` | `writes` in `SettingsUpdateDecision` | `all_settings_schema()` plus raw REST patch values | Yes - accepted patches emit typed write decisions | VERIFIED |
| `persistence.rs` | `PersistenceDecision.values/writes/erases` | `NvsSnapshot`, migration decisions, schema defaults, settings writes | Yes - snapshot reload data flows through pure model | VERIFIED |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Config crate tests pass | `bazel test //crates/bitaxe-config:tests` | Passed | PASS |
| Full Cargo tests pass | `cargo test --all-features` | Passed, including 40 `bitaxe-config` tests | PASS |
| Parity report accepts checklist | `just parity` | Passed with `validation_errors: none` | PASS |
| Reference tree remains clean | `git status --short reference/esp-miner` | No output | PASS |
| Formatting is clean | `cargo fmt --all -- --check` | Passed | PASS |
| Lint is clean | `cargo clippy --all-targets --all-features -- -D warnings` | Passed | PASS |
| Build is clean | `cargo build --all-targets --all-features` | Passed | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| CFG-01 | 02-01 | Ultra 205 defaults match reference config. | SATISFIED | `Ultra205Defaults`, CSV fixture, unit tests, and golden tests cover device, board, ASIC, pool, fan, self-test, frequency, and voltage defaults. |
| CFG-02 | 02-01 | Board/device/ASIC identifiers are typed, including scoped non-205 entries. | SATISFIED | `BoardCatalogEntry`, `AsicProfile`, and `VerificationScope`; non-205 test enforces `NotHardwareVerified`. |
| CFG-03 | 02-02 | NVS key names, defaults, missing-key behavior, and migrations match upstream observable behavior. | SATISFIED | `nvs.rs`, schema/migration fixtures, corrupt-float fallback tests, and snapshot load tests. |
| CFG-04 | 02-03 | Runtime settings use typed validation for ranges and units. | SATISFIED | `validation.rs`, `settings.rs`, invalid update fixtures, typed errors, and no-write rejection behavior. |
| CFG-05 | 02-04 | Settings persist and reload with upstream-compatible semantics. | SATISFIED (phase-scoped) | Pure `NvsSnapshot` load/update/reload semantics are implemented and tested; firmware NVS adapter, real reboot smoke, and API PATCH route are explicitly outside Phase 2 evidence. |
| CFG-06 | 02-01, 02-02, 02-03, 02-04 | Reference-derived golden fixtures cover defaults, NVS schemas, and valid/invalid updates. | SATISFIED | CSV/JSON fixtures exist with provenance and are exercised by fixture-backed tests. |

No orphaned Phase 2 requirements were found: PLAN frontmatter and `.planning/REQUIREMENTS.md` both account for CFG-01 through CFG-06.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| n/a | n/a | No blocking stub patterns found | n/a | `TODO/FIXME/placeholder` and hardware-effect scans found no blocking implementation stubs. Grep hits were scoped comments, checklist notes, or harmless match arms. |
| `crates/bitaxe-config/src/nvs.rs` | n/a | Large central schema file | Info | File is 1580 lines, above the Bright Builds refactor trigger. It is a central schema registry with fixture-backed coverage; not a Phase 2 goal gap. |

### Human Verification Required

None. Phase 2 is a pure config/default/schema/validation/persistence model. Firmware ESP-IDF NVS adapter reboot smoke, API PATCH route behavior, voltage/fan/thermal/power effects, and ASIC hardware behavior are intentionally deferred to later phases and not required for this phase's pass status.

### Gaps Summary

No goal-blocking gaps found. The implementation satisfies the Phase 2 pure Rust config model contract, keeps reference and evidence boundaries explicit, and avoids overclaiming firmware adapter or hardware-control parity.

---

_Verified: 2026-06-26T18:42:08Z_
_Verifier: the agent (gsd-verifier)_
