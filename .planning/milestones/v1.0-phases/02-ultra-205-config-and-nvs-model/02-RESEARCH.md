# Phase 02: Ultra 205 Config And NVS Model - Research

**Researched:** 2026-06-26
**Domain:** Rust domain modeling for Ultra 205 board config, NVS schema, validation, fixtures, and persistence decisions
**Confidence:** HIGH for local reference-derived config/NVS findings; MEDIUM-HIGH for fixture/build integration details because the exact module split is still planner discretion.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

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

### Deferred Ideas (OUT OF SCOPE)

- Firmware ESP-IDF NVS adapter, real reboot reload smoke, and adapter-level persistence evidence should follow after the pure model is stable.
- Settings HTTP PATCH handlers, API response compatibility, and WebSocket settings/telemetry behavior belong to Phase 5.
- BM1366 safe initialization, frequency transitions, voltage effects, fan, thermal, power, and mining behavior belong to later hardware phases and need hardware evidence.
- OTA, filesystem, static asset, and release packaging effects belong to Phase 7.
- Gamma 601/BM1370 and other non-205 board verification remain deferred until each board has its own evidence set.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CFG-01 | Ultra 205 defaults match the reference config for device model, board version, ASIC model, ASIC frequency, ASIC voltage, pool defaults, fan defaults, and self-test defaults. | Use `reference/esp-miner/config-205.cvs` as the golden source and assert every user-visible default listed in D-01. [VERIFIED: reference/esp-miner/config-205.cvs] |
| CFG-02 | Board, device, and ASIC identifiers are represented as typed Rust domain values, including non-205 upstream boards as scoped but not hardware-verified entries. | Model upstream `AsicConfig`, `FamilyConfig`, and `DeviceConfig` concepts as Rust catalog data with explicit verification scope per board. [VERIFIED: reference/esp-miner/main/device_config.h] |
| CFG-03 | NVS key names, default values, missing-key behavior, and migration behavior match upstream observable behavior for V1 settings. | Model the `Settings` table, namespace `main`, legacy fallback migrations, default load rules, bool-as-u16 storage, and float-as-string behavior from upstream. [VERIFIED: reference/esp-miner/main/nvs_config.c] |
| CFG-04 | Runtime settings use typed validation for ranges and units such as frequency, millivolts, temperatures, fan duty, hostnames, ports, and pool credentials. | Parse raw values into domain newtypes and reuse upstream min/max/rest-name metadata for setting-level rejection semantics. [VERIFIED: reference/esp-miner/main/nvs_config.c; reference/esp-miner/main/http_server/http_server.c] |
| CFG-05 | Settings changed through user-facing surfaces persist and reload across reboot with upstream-compatible semantics. | Phase 2 should prove host-testable load/update/reload decisions and produce adapter commands; firmware NVS adapter/reboot smoke remains deferred by D-12. [VERIFIED: .planning/phases/02-ultra-205-config-and-nvs-model/02-CONTEXT.md] |
| CFG-06 | Reference-derived golden fixtures cover Ultra 205 defaults, NVS schemas, and representative valid/invalid settings updates. | Add machine-readable fixtures with source path, reference commit, and license/provenance metadata; parse CSV-shaped defaults with a structured parser. [VERIFIED: reference/esp-miner/config-205.cvs; PROVENANCE.md] |
</phase_requirements>

## Summary

Phase 2 should be implemented as a pure Rust config model in `crates/bitaxe-config`, not as firmware NVS I/O. [VERIFIED: .planning/phases/02-ultra-205-config-and-nvs-model/02-CONTEXT.md; standards/core/architecture.md] The crate should own typed board/catalog data, Ultra 205 defaults, NVS schema metadata, validation, pure update decisions, migration/default-load semantics, and serializable snapshots. [VERIFIED: .planning/phases/02-ultra-205-config-and-nvs-model/02-CONTEXT.md]

The pinned upstream files are sufficient to plan this phase without broader ecosystem exploration. [VERIFIED: reference/esp-miner/config-205.cvs; reference/esp-miner/main/device_config.h; reference/esp-miner/main/nvs_config.c; reference/esp-miner/main/http_server/http_server.c] The main planning risk is subtle compatibility drift: upstream uses different seed/default keys and active NVS keys, stores booleans as `u16`, stores floats as strings, migrates legacy keys, and exposes API/rest names that differ from NVS key names. [VERIFIED: reference/esp-miner/main/nvs_config.c; reference/esp-miner/main/http_server/system_api_json.c]

**Primary recommendation:** Extend `crates/bitaxe-config` into a typed functional core with reference-derived fixtures, `thiserror` validation errors, `serde`/`serde_json` snapshots, and `csv` as a dev-only parser for the CSV-shaped upstream fixture. [VERIFIED: Cargo.toml; cargo info serde; cargo info serde_json; cargo info thiserror; cargo info csv]

## Project Constraints (from AGENTS.md)

- Prefer `AGENTS.md` as the root instruction file and also read `AGENTS.bright-builds.md`, `standards-overrides.md`, and relevant standards before plan/review/implementation/audit work. [VERIFIED: AGENTS.md; AGENTS.bright-builds.md]
- Keep `reference/esp-miner` pinned and read-only; verification should fail when it is locally modified. [VERIFIED: AGENTS.md; docs/adr/0005-read-only-reference-implementation.md]
- Use Bazel as the canonical automation graph and `just` as the human command surface. [VERIFIED: AGENTS.md; Justfile; MODULE.bazel]
- Preserve functional core / imperative shell boundaries: pure logic belongs in crates, ESP-IDF/NVS/FreeRTOS/hardware effects belong in `firmware/bitaxe`. [VERIFIED: AGENTS.md; standards/core/architecture.md]
- Use typed domain values and parse raw boundary data early; make illegal states unrepresentable where practical. [VERIFIED: standards/core/architecture.md; standards/languages/rust.md]
- For Rust modules, prefer `foo.rs` plus `foo/` over `foo/mod.rs` when creating multi-file modules. [VERIFIED: standards/languages/rust.md]
- Prefer `thiserror` for library errors and `anyhow` for application/CLI errors. [VERIFIED: AGENTS.md; standards/languages/rust.md]
- Do not use `unwrap()`; prefer `?` propagation or `expect()` only when panic is impossible. [VERIFIED: AGENTS.md]
- Use `let...else` for guard-style extraction when it improves Rust control flow. [VERIFIED: AGENTS.md; standards/languages/rust.md]
- Prefix internal `Option<T>` names with `maybe_` when the value may be absent. [VERIFIED: AGENTS.md; standards/languages/rust.md]
- Unit tests must focus on one concern and clearly use Arrange, Act, Assert comments unless trivially obvious. [VERIFIED: AGENTS.md; standards/core/testing.md]
- Before commits in this Rust repo, run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`; this research artifact is not being committed because the user explicitly reserved git finalization for the parent wrapper. [VERIFIED: AGENTS.md]
- Do not store real operator secrets in fixtures; reference-derived fixture data needs provenance and license posture review. [VERIFIED: .planning/phases/02-ultra-205-config-and-nvs-model/02-CONTEXT.md; PROVENANCE.md]

## Standard Stack

### Core

| Library / Component | Version | Purpose | Why Standard |
|---------------------|---------|---------|--------------|
| `crates/bitaxe-config` | `0.1.0` local | Own config catalog, NVS schema, validation, defaults, update decisions, and host-testable snapshots. | It already owns Phase 1 Ultra 205 identity/defaults and is the accepted pure config boundary. [VERIFIED: crates/bitaxe-config/Cargo.toml; crates/bitaxe-config/src/lib.rs; 02-CONTEXT.md] |
| `crates/bitaxe-core` | `0.1.0` local | Share stable board/ASIC/domain identifiers when those identifiers cross crate boundaries. | It already owns `BoardTarget::Ultra205` and `AsicTarget::Bm1366`; do not duplicate cross-crate identity enums. [VERIFIED: crates/bitaxe-core/src/lib.rs] |
| `serde` | `1.0.228`, published 2025-09-27 | Derive serialization/deserialization for snapshots and fixture structs. | Workspace pin matches current crates.io max stable version and supports deriving typed fixture shapes. [VERIFIED: Cargo.toml; cargo info serde; crates.io API] |
| `serde_json` | `1.0.150`, published 2026-05-21 | Store and compare golden snapshots and representative settings update cases. | Workspace pin matches current crates.io max stable version and is already used by repo host tools. [VERIFIED: Cargo.toml; cargo info serde_json; crates.io API] |
| `thiserror` | `2.0.18`, published 2026-01-18 | Define library-grade validation/load/update error enums. | Workspace pin matches current crates.io max stable version and repo rules prefer `thiserror` for library errors. [VERIFIED: Cargo.toml; cargo info thiserror; AGENTS.md; crates.io API] |
| `csv` | `1.4.0`, published 2025-10-17 | Dev/test-only parsing of upstream CSV-shaped `config-205.cvs` fixture. | It avoids ad hoc parsing of structured fixture data and supports serde-backed records. [VERIFIED: cargo info csv; reference/esp-miner/config-205.cvs] |

### Supporting

| Library / Component | Version | Purpose | When to Use |
|---------------------|---------|---------|-------------|
| `crates/bitaxe-test-support` | `0.1.0` local | Shared fixture assertions and helpers if multiple crates later consume config fixtures. | Use only for reusable test helpers; keep production config logic in `bitaxe-config`. [VERIFIED: crates/bitaxe-test-support/src/lib.rs] |
| `tools/parity` | `0.1.0` local | Report checklist status and evidence gaps. | Use after config evidence lands to update and validate CFG rows. [VERIFIED: tools/parity/Cargo.toml; Justfile] |
| `tempfile` | `3.27.0`, published 2026-03-11 | Host-only persistence roundtrip tests if file-backed snapshots are useful. | Use as a dev-dependency only when tests need temporary directories/files. [VERIFIED: Cargo.toml; cargo info tempfile; crates.io API] |
| `esp-idf-svc` / `esp-idf-sys` | `0.52.1` / `0.37.2` workspace pins | Later firmware NVS adapter surface. | Do not use in Phase 2 pure model; use in later `firmware/bitaxe` adapter work. [VERIFIED: Cargo.toml; 02-CONTEXT.md] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Static typed schema plus fixtures | Generate Rust schema directly from `nvs_config.c` | Code generation adds tooling and GPL/provenance complexity before the schema stabilizes. [VERIFIED: PROVENANCE.md; reference/esp-miner/main/nvs_config.c] |
| `csv` dev-dependency | Hand-parse `config-205.cvs` in tests | Hand parsing is brittle and conflicts with the repo preference for structured data handling where reasonable. [VERIFIED: cargo info csv; AGENTS.md] |
| `serde_json` fixtures | `toml` fixtures | JSON matches the existing workspace dependency set; adding TOML is not necessary for Phase 2. [VERIFIED: Cargo.toml] |
| Pure snapshot/reload simulation | Real ESP-IDF NVS adapter now | Real NVS I/O is explicitly deferred; Phase 2 should return adapter commands and prove pure semantics. [VERIFIED: 02-CONTEXT.md] |

**Installation / manifest changes:**

Prefer manual manifest edits so `bitaxe-config` reuses existing workspace pins where they already exist. [VERIFIED: Cargo.toml]

```toml
# crates/bitaxe-config/Cargo.toml
[dependencies]
bitaxe-core = { path = "../bitaxe-core" }
serde = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
csv = "1.4.0"
serde_json = { workspace = true }
```

If production code does not need serde derives, keep `serde` as a dev-dependency and limit serialization to fixture/snapshot test types. [VERIFIED: Cargo.toml; 02-CONTEXT.md]

**Version verification:** `cargo search`, `cargo info`, and the crates.io API were used on 2026-06-26 to verify `serde 1.0.228`, `serde_json 1.0.150`, `thiserror 2.0.18`, `tempfile 3.27.0`, and `csv 1.4.0`; the crates.io API shows `anyhow 1.0.103` is newer than the workspace `anyhow 1.0.102`, but Phase 2 should not add `anyhow` to `bitaxe-config` because this is library code. [VERIFIED: cargo info; crates.io API; AGENTS.md]

## Architecture Patterns

### Recommended Project Structure

```text
crates/bitaxe-config/
├── Cargo.toml
├── BUILD.bazel
├── fixtures/
│   ├── ultra-205-defaults.csv          # reference-derived, provenance-marked
│   ├── nvs-schema.json                 # schema rows and source breadcrumbs
│   ├── nvs-migrations.json             # legacy-key scenarios
│   └── settings-updates.json           # representative valid/invalid updates
└── src/
    ├── lib.rs                          # public API and module exports
    ├── catalog.rs                      # ASIC/family/board catalog data
    ├── defaults.rs                     # Ultra 205 default snapshot
    ├── nvs.rs                          # key names, stored types, migrations
    ├── settings.rs                     # runtime setting values and update decisions
    ├── validation.rs                   # domain newtypes and typed errors
    └── persistence.rs                  # host-testable load/update/reload model
```

Expose fixture files to both Cargo and Bazel when tests use `include_str!` or runtime fixture reads. [VERIFIED: crates/bitaxe-config/BUILD.bazel; MODULE.bazel] If the planner chooses integration tests under `tests/`, add explicit Bazel `rust_test` targets instead of relying on Cargo-only discovery. [VERIFIED: crates/bitaxe-config/BUILD.bazel]

### Pattern 1: Typed Schema Rows

**What:** Represent each setting as data with `NvsKeyName`, `StoredType`, `SettingDefault`, optional `RestFieldName`, range metadata, optional array behavior, and source breadcrumb. [VERIFIED: reference/esp-miner/main/nvs_config.h; reference/esp-miner/main/nvs_config.c]

**When to use:** Use for every upstream setting row that Phase 2 models, including board identity, pool, frequency, voltage, fan, display, self-test, and migration-relevant keys. [VERIFIED: reference/esp-miner/main/nvs_config.c]

**Example:**

```rust
// Source: reference/esp-miner/main/nvs_config.c
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoredType {
    Str,
    U16,
    I32,
    U64,
    FloatString,
    BoolAsU16,
}

pub struct SettingSchema {
    pub key: NvsKeyName,
    pub stored_type: StoredType,
    pub rest_name: Option<RestFieldName>,
    pub min: i32,
    pub max: i32,
    pub provenance: &'static str,
}
```

### Pattern 2: Parse Boundary Values Once

**What:** Convert raw string/number/bool input into domain values such as `BoardVersion`, `AsicFrequencyMhz`, `CoreVoltageMv`, `FanDutyPercent`, `TemperatureCelsius`, `Hostname`, `Port`, and `PoolCredential` before later firmware/API logic sees them. [VERIFIED: standards/core/architecture.md; standards/languages/rust.md]

**When to use:** Use at NVS load, fixture load, and future API settings update boundaries. [VERIFIED: reference/esp-miner/main/http_server/http_server.c]

**Example:**

```rust
// Source: ESP-IDF NVS docs and Bright Builds Rust standards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NvsKeyName(String);

impl NvsKeyName {
    pub fn parse(value: impl Into<String>) -> Result<Self, ConfigValidationError> {
        let value = value.into();
        if !value.is_ascii() || value.len() > 15 {
            return Err(ConfigValidationError::InvalidNvsKeyName { value });
        }

        Ok(Self(value))
    }
}
```

### Pattern 3: Pure Update Decisions

**What:** Validate a proposed settings patch and return either typed rejection details or an ordered set of NVS writes/migration writes that a later adapter can apply. [VERIFIED: 02-CONTEXT.md; reference/esp-miner/main/http_server/http_server.c]

**When to use:** Use for Phase 2 valid/invalid settings fixture tests and later Phase 5 settings PATCH implementation. [VERIFIED: .planning/ROADMAP.md; 02-CONTEXT.md]

**Example:**

```rust
// Source: reference/esp-miner/main/http_server/http_server.c
pub enum SettingsUpdateDecision {
    Accepted {
        snapshot: SettingsSnapshot,
        writes: Vec<NvsWrite>,
    },
    Rejected {
        errors: Vec<ConfigValidationError>,
    },
}
```

### Pattern 4: Scope Verification Separately From Catalog Presence

**What:** Catalog entries can exist for upstream boards/ASICs while their verification status remains `deferred` or `unverified`. [VERIFIED: 02-CONTEXT.md; docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md]

**When to use:** Use for Gamma 601/BM1370, TPS546-specific behavior, and every non-205 board row imported from `device_config.h`. [VERIFIED: reference/esp-miner/main/device_config.h; docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md]

### Anti-Patterns to Avoid

- **One `Config` struct full of primitives:** Raw strings and integers would force every caller to remember units, ranges, defaults, and storage quirks. [VERIFIED: standards/core/architecture.md; standards/languages/rust.md]
- **Firmware-first NVS implementation:** It would bury compatibility decisions inside ESP-IDF calls and make default/migration/reload behavior expensive to test. [VERIFIED: 02-CONTEXT.md; standards/core/architecture.md]
- **Treating CSV seed keys as active NVS keys:** `config-205.cvs` seeds `asicfrequency`, while upstream active runtime key is `asicfrequency_f` with legacy fallback handling. [VERIFIED: reference/esp-miner/config-205.cvs; reference/esp-miner/main/nvs_config.c]
- **Marking voltage/frequency hardware behavior verified from config tests:** Config tests can verify values and validation only; hardware-control parity needs later hardware evidence. [VERIFIED: docs/adr/0012-parity-verification-evidence.md; 02-CONTEXT.md]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CSV fixture parsing | Split lines and commas manually | `csv` dev-dependency | The upstream defaults are CSV-shaped and a parser avoids brittle test code. [VERIFIED: reference/esp-miner/config-205.cvs; cargo info csv] |
| Library error formatting | Stringly `Err(String)` or `anyhow` in `bitaxe-config` | `thiserror` enums | Repo rules prefer typed library errors and `thiserror` is already pinned/current. [VERIFIED: AGENTS.md; Cargo.toml; cargo info thiserror] |
| NVS key limits | Comments saying keys "must be short" | `NvsKeyName` constructor plus tests | ESP-IDF 5.5.4 limits NVS keys and namespace names to 15 characters. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/storage/nvs_flash.html] |
| Settings validation | Repeated primitive checks in API/firmware | Domain newtypes and schema-driven range validation | Upstream centralizes type/range checks through settings metadata and the PATCH handler; Rust should parse once. [VERIFIED: reference/esp-miner/main/nvs_config.c; reference/esp-miner/main/http_server/http_server.c] |
| Persistence behavior | Direct ESP-IDF NVS reads/writes in tests | In-memory snapshot/reload model returning `NvsWrite` decisions | Phase 2 must be host-testable without ESP-IDF and adapter reboot smoke is deferred. [VERIFIED: 02-CONTEXT.md] |
| Board verification status | Boolean `supported` flag only | Explicit `VerificationScope` / `EvidenceStatus` | Non-205 catalog presence must not imply hardware verification. [VERIFIED: docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md; docs/parity/checklist.md] |

**Key insight:** The hard part is not storing values; it is preserving observable compatibility across CSV seed defaults, active NVS key names, REST field names, migrations, validation ranges, missing-key defaults, and evidence status. [VERIFIED: reference/esp-miner/config-205.cvs; reference/esp-miner/main/nvs_config.c; reference/esp-miner/main/http_server/system_api_json.c; docs/adr/0012-parity-verification-evidence.md]

## Common Pitfalls

### Pitfall 1: Confusing Seed Defaults With Runtime Schema

**What goes wrong:** The Rust model copies `config-205.cvs` keys and misses active runtime keys such as `asicfrequency_f` and `manualfanspeed`. [VERIFIED: reference/esp-miner/config-205.cvs; reference/esp-miner/main/nvs_config.c]
**Why it happens:** The CSV seed file is the first visible default source, but `nvs_config.c` owns active settings metadata and migrations. [VERIFIED: reference/esp-miner/config-205.cvs; reference/esp-miner/main/nvs_config.c]
**How to avoid:** Keep separate fixture types for seed defaults, active NVS schema, and migration cases. [VERIFIED: 02-CONTEXT.md]
**Warning signs:** Tests assert `asicfrequency` as the only frequency key or `fanspeed` as the only manual fan key. [VERIFIED: reference/esp-miner/main/nvs_config.c]

### Pitfall 2: Overrunning ESP-IDF NVS Key Limits

**What goes wrong:** A renamed key, generated indexed key, or compatibility alias exceeds ESP-IDF's 15-character key limit. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/storage/nvs_flash.html]
**Why it happens:** Several upstream keys are already near or at the limit, including `asicfrequency_f` and `fbstratumdecode` at 15 characters. [VERIFIED: reference/esp-miner/main/nvs_config.c]
**How to avoid:** Enforce key length in `NvsKeyName::parse` and test every schema row plus generated indexed names. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/storage/nvs_flash.html]
**Warning signs:** Tests only check hand-picked "tricky short names" and skip full schema iteration. [VERIFIED: reference/esp-miner/main/nvs_config.c]

### Pitfall 3: Losing Float/String Compatibility

**What goes wrong:** Frequency is persisted as a numeric float in the Rust model instead of a string-backed NVS value. [VERIFIED: reference/esp-miner/main/nvs_config.c]
**Why it happens:** Rust has a native `f32`, but ESP-IDF 5.5.4 NVS documents integer, string, and blob storage and notes float/double as future possible types. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/storage/nvs_flash.html]
**How to avoid:** Model `StoredType::FloatString`; parse corrupt strings to default; write floats using upstream-compatible string formatting semantics where practical. [VERIFIED: reference/esp-miner/main/nvs_config.c]
**Warning signs:** Tests use binary float persistence or do not include corrupt-float fallback. [VERIFIED: reference/esp-miner/main/nvs_config.c]

### Pitfall 4: Treating Pure Persistence As Real Reboot Evidence

**What goes wrong:** Snapshot roundtrip tests are reported as adapter-level reboot persistence. [VERIFIED: docs/adr/0012-parity-verification-evidence.md]
**Why it happens:** Phase 2 needs reload semantics, but the ESP-IDF NVS adapter is explicitly deferred. [VERIFIED: 02-CONTEXT.md]
**How to avoid:** Label Phase 2 evidence as unit/golden for pure model behavior and leave real reboot reload smoke as pending adapter evidence. [VERIFIED: 02-CONTEXT.md; docs/adr/0012-parity-verification-evidence.md]
**Warning signs:** CFG-005 moves to `verified` with no firmware evidence path or caveat. [VERIFIED: docs/parity/checklist.md; docs/adr/0012-parity-verification-evidence.md]

### Pitfall 5: GPL/Provenance Ambiguity In Fixtures

**What goes wrong:** Reference-derived fixture files are committed as MIT-only with no source path, upstream commit, or license posture note. [VERIFIED: PROVENANCE.md]
**Why it happens:** Golden data can look like neutral constants even when it is derived from GPL-covered upstream files. [VERIFIED: PROVENANCE.md]
**How to avoid:** Add fixture metadata and conservative SPDX/provenance notes for reference-derived fixture files. [VERIFIED: PROVENANCE.md]
**Warning signs:** New fixture files contain upstream-derived schema/default data but no source breadcrumb. [VERIFIED: PROVENANCE.md; docs/adr/0008-reference-breadcrumb-comments.md]

## Code Examples

Verified patterns from local standards and reference behavior:

### Settings Schema With Provenance

```rust
// Source: reference/esp-miner/main/nvs_config.c
pub const ASIC_FREQUENCY: SettingSchema = SettingSchema {
    key: NvsKeyName::from_static("asicfrequency_f"),
    stored_type: StoredType::FloatString,
    rest_name: Some(RestFieldName::from_static("frequency")),
    min: 1,
    max: u16::MAX as i32,
    provenance: "reference/esp-miner/main/nvs_config.c:NVS_CONFIG_ASIC_FREQUENCY",
};
```

### Migration Case

```rust
// Source: reference/esp-miner/main/nvs_config.c
pub enum MigrationRule {
    LegacyU16ToFloatString {
        legacy_key: NvsKeyName,
        active_key: NvsKeyName,
    },
    LegacyU16Mirror {
        legacy_key: NvsKeyName,
        active_key: NvsKeyName,
    },
}
```

### Boundary Validation

```rust
// Source: standards/languages/rust.md
impl TryFrom<u16> for FanDutyPercent {
    type Error = ConfigValidationError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value > 100 {
            return Err(ConfigValidationError::OutOfRange {
                field: "manualFanSpeed",
                min: 0,
                max: 100,
                actual: i64::from(value),
            });
        }

        Ok(Self(value))
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `asicfrequency` stored as `u16` | `asicfrequency_f` stored as string-backed float, with legacy `asicfrequency` fallback/mirror | Upstream comment says since v2.10.0 | Rust must model active and legacy keys. [VERIFIED: reference/esp-miner/main/nvs_config.c] |
| `fanspeed` manual fan key | `manualfanspeed`, with legacy `fanspeed` fallback/mirror | Upstream comment says since v2.11.0 | Rust must migrate and preserve legacy writes for compatibility. [VERIFIED: reference/esp-miner/main/nvs_config.c] |
| Stratum protocol as numeric `u16` | `stratumprot` / `fbstratumprot` as strings `SV1` or `SV2` | Current upstream migration code | Rust migration fixtures should cover numeric-to-string conversion. [VERIFIED: reference/esp-miner/main/nvs_config.c; reference/esp-miner/main/global_state.h] |
| SV2 channel type as numeric `u16` | `sv2chantype` / `fbsv2chantype` as strings `standard` or `extended` | Current upstream migration code | Rust migration fixtures should cover current key and legacy `fbSv2ChanType`. [VERIFIED: reference/esp-miner/main/nvs_config.c; reference/esp-miner/components/stratum_v2/include/sv2_protocol.h] |
| Float/double as imagined native NVS values | Store floats through strings for this phase | ESP-IDF 5.5.4 docs list float/double as possible future types, not current storage types | Rust should not invent native NVS float persistence. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/storage/nvs_flash.html] |

**Deprecated/outdated:**

- Gamma 601-first assumptions in older `.planning/research/*` docs are superseded by ADR-0014 and current Phase 2 context. [VERIFIED: docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md; .planning/ROADMAP.md]
- Treating `Phase1BoardSelection` as the config model is outdated for Phase 2; it is explicitly identity-only and side-effect-free. [VERIFIED: crates/bitaxe-config/src/lib.rs; 02-CONTEXT.md]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Fixture files copied or derived from upstream can be checked in when they carry conservative provenance/SPDX metadata. [ASSUMED] | Standard Stack, Common Pitfalls | If legal review requires a different fixture strategy, the planner must switch to reading the pinned reference file directly or isolate fixture licensing differently. |

## Open Questions (RESOLVED)

1. **RESOLVED: How should reference-derived fixtures be licensed file-by-file?**
   - What we know: PROVENANCE requires conservative treatment for upstream-derived expression and says intentionally ported/incorporated GPL expression should not be MIT-only. [VERIFIED: PROVENANCE.md]
   - What's unclear: Exact SPDX expression for generated fixture files is not legally decided in this phase research. [ASSUMED]
   - Resolution: Accepted for Phase 2 planning. Plans must add explicit fixture provenance metadata and avoid MIT-only claims for reference-derived fixtures until legal/release review. [VERIFIED: PROVENANCE.md]

2. **RESOLVED: Should config fixtures live in `bitaxe-config` or `bitaxe-test-support`?**
   - What we know: `bitaxe-config` owns the behavior and `bitaxe-test-support` owns reusable test helpers. [VERIFIED: crates/bitaxe-config/src/lib.rs; crates/bitaxe-test-support/src/lib.rs; 02-CONTEXT.md]
   - What's unclear: Reuse pressure from API/firmware tests will increase in later phases. [ASSUMED]
   - Resolution: Accepted for Phase 2 planning. Start fixtures in `crates/bitaxe-config/fixtures`; move shared fixture loaders to `bitaxe-test-support` only when a second crate needs them. [VERIFIED: standards/core/code-shape.md]

3. **RESOLVED: How much of the full NVS schema should Phase 2 model?**
   - What we know: D-04 requires typed schema entries, D-14 requires NVS schema fixtures, and CFG-03 covers V1 settings. [VERIFIED: 02-CONTEXT.md; .planning/REQUIREMENTS.md]
   - What's unclear: Whether scoreboard/theme/display-only settings should receive full validation behavior now or only schema/default metadata. [ASSUMED]
   - Resolution: Accepted for Phase 2 planning. Model the full upstream row metadata now, but prioritize validation/update fixtures for settings named in D-01 plus migration-sensitive keys. [VERIFIED: reference/esp-miner/main/nvs_config.c; 02-CONTEXT.md]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Rust toolchain | `crates/bitaxe-config` implementation and tests | yes | `rustc 1.88.0-nightly` | Use repo-pinned ESP/Rust setup if local toolchain drifts. [VERIFIED: rustc --version; Cargo.toml] |
| Cargo | Dependency metadata and local crate tests | yes | `cargo 1.88.0-nightly` | Use Bazel/Just for canonical verification. [VERIFIED: cargo --version; Justfile] |
| Bazel | Canonical test/build graph | yes | `9.1.1` | No fallback; repo requires Bazel graph. [VERIFIED: bazel --version; MODULE.bazel] |
| `just` | Human command surface | yes | `1.48.0` | Direct Bazel commands if needed. [VERIFIED: just --version; Justfile] |
| `espflash` | Later hardware flash/monitor evidence | yes | `4.0.1` | Not required for Phase 2 pure model tests. [VERIFIED: espflash --version; 02-CONTEXT.md] |
| crates.io access | Version checks and adding `csv` dev dependency | yes | Registry queries succeeded | Vendoring is not needed for this phase. [VERIFIED: cargo search; cargo info] |

**Missing dependencies with no fallback:** None found for Phase 2 research and pure-model planning. [VERIFIED: command probes]

**Missing dependencies with fallback:** None found. [VERIFIED: command probes]

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust unit tests through `rules_rust` `rust_test`; Cargo tests remain useful for local diagnosis. [VERIFIED: crates/bitaxe-config/BUILD.bazel; MODULE.bazel] |
| Config file | `crates/bitaxe-config/BUILD.bazel`, `Cargo.toml`. [VERIFIED: crates/bitaxe-config/BUILD.bazel; Cargo.toml] |
| Quick run command | `bazel test //crates/bitaxe-config:tests` [VERIFIED: crates/bitaxe-config/BUILD.bazel] |
| Full suite command | `just test` [VERIFIED: Justfile] |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| CFG-01 | Ultra 205 defaults match `config-205.cvs` | unit + golden | `bazel test //crates/bitaxe-config:tests --test_filter=ultra_205_defaults` | Partial now in `crates/bitaxe-config/src/lib.rs`; Wave 0 should add full fixture coverage. [VERIFIED: crates/bitaxe-config/src/lib.rs] |
| CFG-02 | Typed board/device/ASIC catalog with non-205 scoped entries | unit + golden | `bazel test //crates/bitaxe-config:tests --test_filter=board_catalog` | Missing; Wave 0 should add catalog module/tests. [VERIFIED: reference/esp-miner/main/device_config.h] |
| CFG-03 | NVS key/default/migration behavior | unit + golden | `bazel test //crates/bitaxe-config:tests --test_filter=nvs_schema` | Missing; Wave 0 should add NVS schema and migration tests. [VERIFIED: reference/esp-miner/main/nvs_config.c] |
| CFG-04 | Typed validation rejects invalid ranges/units | unit | `bazel test //crates/bitaxe-config:tests --test_filter=validation` | Missing; Wave 0 should add validation module/tests. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] |
| CFG-05 | Pure persistence load/update/reload semantics | unit | `bazel test //crates/bitaxe-config:tests --test_filter=persistence` | Missing; Wave 0 should add snapshot/reload tests only, not firmware adapter tests. [VERIFIED: 02-CONTEXT.md] |
| CFG-06 | Golden fixtures for defaults/schema/update cases | golden | `bazel test //crates/bitaxe-config:tests --test_filter=fixtures` | Missing; Wave 0 should add fixture files and Bazel visibility. [VERIFIED: 02-CONTEXT.md] |

### Sampling Rate

- **Per task commit:** `bazel test //crates/bitaxe-config:tests` plus affected crate tests. [VERIFIED: crates/bitaxe-config/BUILD.bazel]
- **Per wave merge:** `just test` and `just parity`. [VERIFIED: Justfile]
- **Phase gate:** `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`, `just test`, `just parity`, and diff review confirming `reference/esp-miner` is unchanged. [VERIFIED: AGENTS.md; Justfile; docs/adr/0005-read-only-reference-implementation.md]

### Wave 0 Gaps

- [ ] `crates/bitaxe-config/src/catalog.rs` - covers CFG-02. [VERIFIED: reference/esp-miner/main/device_config.h]
- [ ] `crates/bitaxe-config/src/defaults.rs` - covers CFG-01. [VERIFIED: reference/esp-miner/config-205.cvs]
- [ ] `crates/bitaxe-config/src/nvs.rs` - covers CFG-03. [VERIFIED: reference/esp-miner/main/nvs_config.c]
- [ ] `crates/bitaxe-config/src/validation.rs` - covers CFG-04. [VERIFIED: reference/esp-miner/main/http_server/http_server.c]
- [ ] `crates/bitaxe-config/src/persistence.rs` - covers CFG-05. [VERIFIED: 02-CONTEXT.md]
- [ ] `crates/bitaxe-config/fixtures/*` - covers CFG-06 with source/provenance metadata. [VERIFIED: 02-CONTEXT.md; PROVENANCE.md]
- [ ] `crates/bitaxe-config/BUILD.bazel` - add fixture visibility for Bazel tests when fixtures are included/read. [VERIFIED: crates/bitaxe-config/BUILD.bazel]

## Security Domain

### Applicable ASVS Categories

The table uses the GSD-required ASVS category labels; controls are mapped to this firmware config phase rather than web-session implementation. [VERIFIED: .planning/config.json]

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no for Phase 2 | Do not implement auth here; API auth belongs to Phase 5. [VERIFIED: .planning/ROADMAP.md] |
| V3 Session Management | no for Phase 2 | No sessions are introduced by a pure config crate. [VERIFIED: 02-CONTEXT.md] |
| V4 Access Control | no for Phase 2 | Access control belongs to future API/firmware adapters; config code should expose typed decisions only. [VERIFIED: .planning/ROADMAP.md; 02-CONTEXT.md] |
| V5 Input Validation | yes | Use schema-driven parsing and domain newtypes for NVS/API boundary values. [VERIFIED: standards/core/architecture.md; reference/esp-miner/main/http_server/http_server.c] |
| V6 Cryptography | limited | Do not hand-roll crypto; store stratum cert/pubkey fields as validated strings only and leave crypto/protocol behavior to later Stratum/API phases. [VERIFIED: reference/esp-miner/main/nvs_config.c; .planning/ROADMAP.md] |

### Known Threat Patterns for Config/NVS Model

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Invalid voltage/frequency/fan settings crossing into hardware control | Tampering / Elevation of Privilege | Parse into bounded domain types and keep hardware effects deferred until safety phases. [VERIFIED: 02-CONTEXT.md; docs/adr/0012-parity-verification-evidence.md] |
| Real Wi-Fi or operator pool credentials in fixtures | Information Disclosure | Use upstream public defaults only and mark fixture provenance. [VERIFIED: 02-CONTEXT.md; PROVENANCE.md] |
| NVS key mismatch causing silent setting loss | Tampering / Denial of Service | Preserve exact upstream active and legacy keys with migration fixtures. [VERIFIED: reference/esp-miner/main/nvs_config.c] |
| Corrupt float strings breaking config load | Denial of Service | Fall back to configured default on corrupt float parse. [VERIFIED: reference/esp-miner/main/nvs_config.c] |
| Non-205 boards inheriting Ultra 205 evidence | Spoofing / Safety governance failure | Track verification scope separately from catalog entries. [VERIFIED: docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md] |

## Sources

### Primary (HIGH confidence)

- `reference/esp-miner/config-205.cvs` - Ultra 205 seed/default fixture values. [VERIFIED: local read]
- `reference/esp-miner/main/device_config.h` - ASIC configs, family configs, board catalog, and Ultra 205 board capabilities. [VERIFIED: local read]
- `reference/esp-miner/main/device_config.c` - board selection and custom-board fallback from NVS values. [VERIFIED: local read]
- `reference/esp-miner/main/nvs_config.h` - NVS enum, stored types, and `Settings` shape. [VERIFIED: local read]
- `reference/esp-miner/main/nvs_config.c` - NVS namespace, schema rows, defaults, migrations, load semantics, setters, bool storage, and float-string storage. [VERIFIED: local read]
- `reference/esp-miner/main/http_server/system_api_json.c` - settings/system read field names and values. [VERIFIED: local read]
- `reference/esp-miner/main/http_server/http_server.c` - settings PATCH validation and update behavior. [VERIFIED: local read]
- `.planning/phases/02-ultra-205-config-and-nvs-model/02-CONTEXT.md` - locked Phase 2 decisions and deferred scope. [VERIFIED: local read]
- `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, `.planning/STATE.md` - CFG requirements, phase scope, and current project state. [VERIFIED: local read]
- `docs/adr/0012-parity-verification-evidence.md` and `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - evidence gates and Ultra 205 pivot. [VERIFIED: local read]
- `PROVENANCE.md` - fixture/source provenance and license posture policy. [VERIFIED: local read]
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, `standards/languages/rust.md` - repo rules and Bright Builds standards. [VERIFIED: local read]

### Official / Registry (HIGH confidence)

- ESP-IDF 5.5.4 NVS docs - key/namespace length, supported value types, string limits, and no native float/double storage yet: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32/api-reference/storage/nvs_flash.html [CITED: official docs]
- crates.io / `cargo info` - `serde 1.0.228`, `serde_json 1.0.150`, `thiserror 2.0.18`, `tempfile 3.27.0`, `csv 1.4.0`, and `anyhow 1.0.103` latest registry metadata. [VERIFIED: cargo info; crates.io API]

### Secondary (MEDIUM confidence)

- `.planning/research/ARCHITECTURE.md`, `.planning/research/PITFALLS.md`, `.planning/research/FEATURES.md`, `.planning/research/SUMMARY.md` - prior project research; use architecture/pitfall guidance, but ignore Gamma-first content superseded by ADR-0014. [VERIFIED: local read]

### Tertiary (LOW confidence)

- None.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - recommended crates are local or registry-verified; `csv` is the only new likely dependency and is dev/test-only. [VERIFIED: Cargo.toml; cargo info csv]
- Architecture: HIGH - phase context, repo standards, and current crate boundaries all agree on pure config core plus deferred firmware adapter. [VERIFIED: 02-CONTEXT.md; standards/core/architecture.md; crates/bitaxe-config/src/lib.rs]
- Pitfalls: HIGH - key migrations, storage encodings, and key limits were verified against the pinned reference tree and ESP-IDF 5.5.4 docs. [VERIFIED: reference/esp-miner/main/nvs_config.c; ESP-IDF 5.5.4 docs]
- Security: MEDIUM - validation and secret-fixture controls are clear, but full API auth/access-control controls are future-phase concerns. [VERIFIED: .planning/ROADMAP.md; 02-CONTEXT.md]

**Research date:** 2026-06-26
**Valid until:** 2026-07-26 for repo-local reference findings; re-check crate versions and ESP-IDF docs before planning if dependency changes are delayed beyond 30 days.
