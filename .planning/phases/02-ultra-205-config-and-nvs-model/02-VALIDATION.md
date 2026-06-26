---
phase: 02
slug: ultra-205-config-and-nvs-model
status: approved
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-26
---

# Phase 02 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust unit tests through `rules_rust` `rust_test`; Cargo tests remain useful for local diagnosis. |
| **Config file** | `crates/bitaxe-config/BUILD.bazel`, `Cargo.toml` |
| **Quick run command** | `bazel test //crates/bitaxe-config:tests` |
| **Full suite command** | `just test` |
| **Estimated runtime** | ~60 seconds for focused crate tests; full suite depends on firmware wrapper cache state. |

---

## Sampling Rate

- **After every task commit:** Run `bazel test //crates/bitaxe-config:tests`
- **After every plan wave:** Run `just test` and `just parity`
- **Before `/gsd-verify-work`:** Run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`, `just test`, and `just parity`
- **Max feedback latency:** 1 task; no three consecutive code tasks may pass without an automated config-model check.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 02-W0-01 | TBD | 0 | CFG-01 | T-02-02 | Ultra 205 defaults are fixture-derived and contain no real operator secrets. | unit + golden | `bazel test //crates/bitaxe-config:tests --test_filter=ultra_205_defaults` | no W0 | pending |
| 02-W0-02 | TBD | 0 | CFG-02 | T-02-05 | Non-205 catalog entries cannot inherit Ultra 205 evidence status. | unit + golden | `bazel test //crates/bitaxe-config:tests --test_filter=board_catalog` | no W0 | pending |
| 02-W0-03 | TBD | 0 | CFG-03 | T-02-03 | NVS active keys, legacy keys, defaults, migrations, bool storage, and float-string behavior match upstream evidence. | unit + golden | `bazel test //crates/bitaxe-config:tests --test_filter=nvs_schema` | no W0 | pending |
| 02-W0-04 | TBD | 0 | CFG-04 | T-02-01 | Invalid frequency, voltage, fan, temperature, hostname, port, and pool values are rejected before reaching hardware/API logic. | unit | `bazel test //crates/bitaxe-config:tests --test_filter=validation` | no W0 | pending |
| 02-W0-05 | TBD | 0 | CFG-05 | T-02-03 | Host-testable load, update, migration, missing-key, and reload decisions are proven without claiming firmware reboot evidence. | unit | `bazel test //crates/bitaxe-config:tests --test_filter=persistence` | no W0 | pending |
| 02-W0-06 | TBD | 0 | CFG-06 | T-02-02 | Golden fixtures include provenance and cover defaults, schema, valid updates, and invalid updates. | golden | `bazel test //crates/bitaxe-config:tests --test_filter=fixtures` | no W0 | pending |

*Status: pending until concrete PLAN/SUMMARY files bind these references to task IDs.*

---

## Wave 0 Requirements

- [ ] `crates/bitaxe-config/src/catalog.rs` - typed board/device/ASIC catalog tests for CFG-02.
- [ ] `crates/bitaxe-config/src/defaults.rs` - Ultra 205 default fixture tests for CFG-01.
- [ ] `crates/bitaxe-config/src/nvs.rs` - NVS schema/default/migration tests for CFG-03.
- [ ] `crates/bitaxe-config/src/validation.rs` - typed validation tests for CFG-04.
- [ ] `crates/bitaxe-config/src/persistence.rs` - pure snapshot/load/update/reload tests for CFG-05.
- [ ] `crates/bitaxe-config/fixtures/*` - provenance-marked golden fixtures for CFG-06.
- [ ] `crates/bitaxe-config/BUILD.bazel` - fixture visibility and test target wiring when fixture files are included.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Firmware ESP-IDF NVS adapter reboot reload smoke | CFG-05 | Out of Phase 2 scope; Phase 2 only proves pure persistence semantics. | Record as pending adapter evidence for the future firmware storage adapter phase. |
| Voltage, fan, thermal, power, and ASIC hardware effects | CFG-01, CFG-04 | Safety-critical hardware effects require later Ultra 205 hardware evidence. | Do not mark these effects verified from Phase 2 unit/golden tests. |

---

## Validation Sign-Off

- [x] All phase requirements have an automated test class or explicit manual-only deferral.
- [x] Sampling continuity requires focused config tests after every task commit.
- [x] Wave 0 covers all currently missing test infrastructure.
- [x] No watch-mode flags.
- [x] Feedback latency is bounded by one task.
- [x] `nyquist_compliant: true` set in frontmatter.

**Approval:** approved 2026-06-26

