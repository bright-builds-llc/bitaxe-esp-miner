---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-06-26T15-47-58
generated_at: 2026-06-26T15:47:58.690Z
---

# Phase 2: Ultra 205 Config And NVS Model - Context

**Gathered:** 2026-06-26
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 2 delivers the typed Ultra 205 configuration and NVS settings model that later firmware, ASIC, Stratum, API, and safety phases can rely on. The phase includes reference-derived Ultra 205 defaults, scoped board/device/ASIC identifiers, NVS key metadata, upstream-compatible default and missing-key semantics, typed validation, settings update decisions, persistence model boundaries, golden fixtures, parity checklist updates, and explicit unverified/deferred status for non-205 boards and safety-critical hardware effects.

This phase does not enable mining, BM1366 initialization, voltage changes, fan or thermal control, HTTP settings handlers, WebSocket telemetry, OTA, or release packaging. It may model values that those surfaces will consume, but hardware effects and user-facing API handlers remain later-phase work until their own evidence exists.

</domain>

<decisions>
## Implementation Decisions

### Ultra 205 Defaults And Board Identity

- **D-01:** Treat `reference/esp-miner/config-205.cvs` as the first Phase 2 golden source for user-visible Ultra 205 defaults. The Rust model must cover at least `hostname=bitaxe`, `stratumurl=public-pool.io`, `stratumport=3333`, `stratumtls=0`, `stratumcert=x`, `stratumuser=<reference wallet>.bitaxe`, `stratumpass=x`, `stratumdiff=1000`, `stratumxnsub=0`, fallback pool values, `asicfrequency=485`, `asicvoltage=1200`, `asicmodel=BM1366`, `devicemodel=ultra`, `boardversion=205`, `rotation=0`, `autofanspeed=1`, `fanspeed=100`, `selftest=1`, and `overheat_mode=0`.
- **D-02:** Expand the current Phase 1 identity-only config into domain types for board version, device family/model, ASIC model, ASIC count, frequency options, voltage options, and board capabilities. Ultra 205 is the active V1 target; other upstream board and ASIC entries should be represented as scoped catalog data but must stay unverified or deferred until each receives evidence.
- **D-03:** Preserve the Ultra 205 pivot from ADR-0014: Phase 2 targets Ultra 205/BM1366 first, while Gamma 601/BM1370 and TPS546-specific behavior remain deferred and cannot inherit Ultra 205 verification.

### NVS Schema And Compatibility Semantics

- **D-04:** Model NVS settings as typed schema entries with separate concepts for NVS key name, stored type, default value, REST/API name, validation minimum, validation maximum, optional indexed-array behavior, and provenance breadcrumb.
- **D-05:** Keep upstream NVS key names exact, including current and legacy keys. Important compatibility details from `nvs_config.c` include namespace `main`, `asicfrequency_f` as the active ASIC frequency key, legacy `asicfrequency` fallback migration, `manualfanspeed` as the active manual fan key, legacy `fanspeed` fallback migration, booleans stored through `u16`, and floats persisted as strings.
- **D-06:** Missing-key behavior should use the upstream default value without writing hardware effects. Corrupt float parsing should be modeled as falling back to the configured default. Actual ESP-IDF NVS erase/reinitialize behavior belongs to the firmware adapter, but the pure model should make the default/reload result testable.
- **D-07:** NVS key length constraints matter. The Rust schema must keep key names within ESP-IDF NVS limits and include tests that protect tricky short names such as `fbsv2authpubk`, `emc_ideality_f`, `emc_beta_comp`, `power_cons_tgt`, `selftest_temp`, `selftest_warm`, and `selftest_max`.

### Validation And Settings Update Model

- **D-08:** Validate raw user/API/NVS values at boundaries into domain values before they reach later firmware logic. Frequency, millivolts, fan duty, temperature targets, hostnames, ports, pool credentials, TLS modes, bool-like values, and board identifiers should reject invalid values with typed errors instead of scattered primitive checks.
- **D-09:** For Phase 2, validation should focus on model correctness and observable rejection semantics. API route implementation is Phase 5, but Phase 2 should expose a pure settings update/apply result that Phase 5 can call without inventing a second validation path.
- **D-10:** Frequency and voltage validation may prove that a value is allowed by the config model, but it must not mark voltage/frequency hardware-control behavior verified. Hardware-control verification remains blocked on later Ultra 205 evidence.

### Persistence Boundary

- **D-11:** Keep persistence semantics split into a pure core and an imperative shell. `crates/bitaxe-config` should own schema/defaults/validation/update decisions and serializable snapshots. `firmware/bitaxe` should later own the ESP-IDF NVS adapter that reads/writes those typed decisions.
- **D-12:** Provide a host-testable persistence abstraction or snapshot roundtrip for Phase 2 so default load, missing-key load, valid update, invalid update, migration, and reload behavior can be tested without ESP-IDF. Add reboot/reload smoke as a later adapter evidence item once the firmware storage adapter exists.
- **D-13:** Do not store secrets or real operator credentials in new fixtures. Use the upstream public defaults from `config-205.cvs` and clearly mark fixture provenance.

### Golden Fixtures And Evidence

- **D-14:** Add reference-derived golden fixtures for Ultra 205 defaults, board/device/ASIC catalog entries, NVS schema rows, legacy migration cases, and representative valid and invalid settings updates. Fixtures should be machine-readable and cite their source files.
- **D-15:** Update `docs/parity/checklist.md` rows CFG-001 through CFG-005 with implementation pointers and evidence as work lands. Rows can move to `implemented` or `verified` only according to ADR-0012: unit/golden evidence can verify pure config parity, while safety-critical voltage/fan/thermal/power effects remain below `verified` until hardware evidence exists.
- **D-16:** Reference breadcrumbs should appear at module or behavior boundaries, not as line-by-line C translation comments. Keep the Rust code independently structured around domain concepts while preserving source evidence.

### the agent's Discretion

The agent may choose the exact module split, newtype names, fixture file format, error enum names, trait names for the host-testable persistence abstraction, and plan granularity, provided the result stays within the functional-core/imperative-shell boundary, uses upstream-derived fixtures rather than guessed constants, preserves typed validation, and keeps hardware effects deferred.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Requirements

- `.planning/ROADMAP.md` - Phase 2 goal, dependencies, CFG requirements, success criteria, verification expectations, and research flags.
- `.planning/REQUIREMENTS.md` - CFG-01 through CFG-06 plus project-wide evidence, safety, and deferred-board requirements.
- `.planning/PROJECT.md` - Core value, Ultra 205-first scope, accepted seed layout, constraints, and key decisions.
- `.planning/STATE.md` - Completed Phase 1 decisions, Ultra 205 pivot notes, and remaining safety-critical evidence blockers.
- `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md` - Prior locked decisions for the safe boot/log foundation and Ultra 205 evidence boundary.

### Upstream Reference Files

- `reference/esp-miner/config-205.cvs` - Golden Ultra 205 default NVS values for Phase 2 fixtures.
- `reference/esp-miner/main/device_config.h` - Upstream ASIC configs, frequency and voltage options, family configs, board catalog, and Ultra 205 board capabilities.
- `reference/esp-miner/main/device_config.c` - Board-version selection, custom-board fallback path, and runtime device config population from NVS.
- `reference/esp-miner/main/nvs_config.h` - NVS key enum, setting types, and settings metadata shape.
- `reference/esp-miner/main/nvs_config.c` - NVS key names, REST names, defaults, min/max values, legacy migrations, default load, typed setters/getters, async save queue, bool storage, and float-string behavior.
- `reference/esp-miner/main/http_server/system_api_json.c` - Current settings read surface and REST field names that later API work must preserve.

### Architecture, Evidence, And Policy

- `.planning/research/SUMMARY.md` - Functional-core/imperative-shell architecture and config-before-ASIC/Stratum dependency rationale.
- `.planning/research/ARCHITECTURE.md` - Pure config crate boundary and NVS/settings risk notes.
- `.planning/research/FEATURES.md` - Config/NVS, API settings, hardware safety, and deferred board feature boundaries.
- `.planning/research/PITFALLS.md` - NVS/API/settings drift pitfalls, NVS key-length warning, range-checked value warning, and evidence guardrails.
- `.planning/research/STACK.md` - Rust/ESP-IDF stack, Bazel/Cargo boundary, and fixture/tooling recommendations.
- `docs/adr/0001-device-user-parity.md` - Device-user parity definition, including settings and NVS behavior.
- `docs/adr/0003-esp-idf-rust-production-stack.md` - ESP-IDF Rust stack decision and platform service boundary.
- `docs/adr/0005-read-only-reference-implementation.md` - Read-only upstream reference policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Parity checklist evidence policy.
- `docs/adr/0008-reference-breadcrumb-comments.md` - Breadcrumb placement policy.
- `docs/adr/0009-monorepo-package-layout.md` - Crate and firmware path ownership.
- `docs/adr/0012-parity-verification-evidence.md` - Verification evidence requirements and hardware-control evidence gate.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - GPL provenance and release guardrails.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205/BM1366 first parity target and deferred Gamma 601 scope.
- `PROVENANCE.md` - Provenance, SPDX, upstream reference, and fixture/source attribution policy.
- `docs/parity/checklist.md` - CFG rows and evidence ledger to update as Phase 2 work lands.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `crates/bitaxe-config/src/lib.rs` currently exposes `Phase1BoardSelection::ultra_205()` with the Phase 1 subset of `config-205.cvs` defaults. Phase 2 should replace or extend this identity-only contract into the typed config/NVS model.
- `crates/bitaxe-core/src/lib.rs` currently has single-variant `BoardTarget::Ultra205` and `AsicTarget::Bm1366` enums plus Phase 1 safe-state types. Phase 2 can either extend these shared domain enums or move richer config-only catalog types into `bitaxe-config` while preserving clear ownership.
- `crates/bitaxe-test-support/src/lib.rs` currently exposes Phase 1 safe-state assertions. It is the natural place for shared fixture helpers and assertions if they are reused across config, API, and firmware tests.
- `docs/parity/checklist.md` already has CFG-001 through CFG-005 rows with current statuses and reference breadcrumbs.

### Established Patterns

- The repo uses pure Rust crates for domain behavior and keeps ESP-IDF effects in `firmware/bitaxe`.
- Unit tests use explicit Arrange, Act, Assert comments and test one concern per test.
- Bazel BUILD files already exist for `crates/bitaxe-config`, `crates/bitaxe-core`, and `crates/bitaxe-test-support`; new source and fixture files should stay visible to both Cargo and Bazel where needed.
- Phase 1 completed with `just build`, `just test`, `just package`, `just parity`, and Ultra 205 safe-state hardware evidence. Phase 2 should preserve those command contracts.

### Integration Points

- `crates/bitaxe-config` owns board/device/ASIC config, NVS schema metadata, validation, update decisions, fixture parsing, and host-testable persistence behavior.
- `firmware/bitaxe` should not gain broad NVS behavior until the pure model exists; later adapter work should translate ESP-IDF NVS reads/writes into typed config snapshots.
- `crates/bitaxe-api` will later consume Phase 2 validation/update results for settings PATCH behavior instead of duplicating validation.
- `tools/parity` and `docs/parity/checklist.md` should surface Phase 2 evidence without treating implementation alone as verification.

</code_context>

<specifics>
## Specific Ideas

- Keep exact upstream values visible in tests and fixtures so review can compare `config-205.cvs` to the Rust output without decoding application logic.
- Model `NvsKeyName`, `RestFieldName`, and typed setting values separately; upstream names intentionally differ.
- Include migration tests for legacy `asicfrequency` -> `asicfrequency_f`, legacy `fanspeed` -> `manualfanspeed`, u16-to-string stratum protocol migrations, and fallback SV2 channel-type key migration where Phase 2 scope reaches those keys.
- Include tests that reject key names longer than ESP-IDF NVS limits and reject invalid settings before any firmware adapter can persist them.
- Keep actual voltage/frequency/fan hardware effects visibly out of scope in code comments, parity notes, and verification reports.

</specifics>

<deferred>
## Deferred Ideas

- Firmware ESP-IDF NVS adapter, real reboot reload smoke, and adapter-level persistence evidence should follow after the pure model is stable.
- Settings HTTP PATCH handlers, API response compatibility, and WebSocket settings/telemetry behavior belong to Phase 5.
- BM1366 safe initialization, frequency transitions, voltage effects, fan, thermal, power, and mining behavior belong to later hardware phases and need hardware evidence.
- OTA, filesystem, static asset, and release packaging effects belong to Phase 7.
- Gamma 601/BM1370 and other non-205 board verification remain deferred until each board has its own evidence set.

</deferred>

---

*Phase: 02-ultra-205-config-and-nvs-model*
*Context gathered: 2026-06-26*
