---
phase: "34"
slug: provenance-runtime-health-and-coherent-operator-snapshot
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-07-15"
---

# Phase 34 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Rust unit/integration tests, Bash behavior tests, Bazel test graph |
| **Config file** | `Cargo.toml`, `MODULE.bazel`, package-local `BUILD.bazel` files |
| **Quick run command** | `cargo test -p bitaxe-core -p bitaxe-api` plus the affected focused Bazel target |
| **Full suite command** | `bazel test //...` |
| **Estimated runtime** | ~180 seconds excluding firmware cross-build/package |

## Sampling Rate

- **After every task commit:** Run the task's focused Cargo, Bash, or Bazel behavior test.
- **After every plan wave:** Run the affected Bazel targets and `git diff --check`.
- **Before phase verification:** The mandatory Rust sequence, `bazel test //...`, `just build`, `just package`, and `just verify-reference` must be green.
- **Max feedback latency:** 180 seconds for host checks; canonical ESP-IDF build/package may exceed this bound and runs at plan completion.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 34-01-01 | 01 | 1 | SYS-01, SYS-02 | T-34-01, T-34-12 | The shared Rust authority derives all four labels and a strict semantic/source/reference provenance stamp from scoped primitive inputs. | unit/shell/Bazel | `cargo test -p bitaxe-api build_identity && bash scripts/build-identity-status-test.sh && bazel test //scripts:build_identity_status_test` | ❌ W0 | ⬜ pending |
| 34-01-02 | 01 | 1 | SYS-01, SYS-02 | T-34-02, T-34-03 | Bazel transports one Rust-materialized stamp; relevant dirty-to-dirty edits rebuild and the output-local ESP-IDF descriptor uses its label. | integration/Bazel/build | `bazel test //scripts:build_identity_status_test //scripts:build_identity_cache_invalidation_test && bazel build //firmware/bitaxe:build_provenance_stamp && just build` | ❌ W0 | ⬜ pending |
| 34-01-03 | 01 | 1 | SYS-01, SYS-02 | T-34-03, T-34-04 | LCD/API/WebSocket/log surfaces expose separate semantic, source, reference, label, and ELF-SHA fields without parsing presentation text. | unit/integration | `cargo test -p bitaxe-core && cargo test -p bitaxe-api system_info && cargo test -p bitaxe-api runtime_projection && cargo test -p bitaxe-parity phase34_identity_runtime_source_guard` | ❌ W0 | ⬜ pending |
| 34-01-04 | 01 | 1 | SYS-02 | T-34-01, T-34-04 | Manifest v3 and tools/flash validate full source/reference/ELF/package identity and reject dirty input before port resolution. | integration/Bazel | `cargo test -p xtask package_manifest && cargo test -p bitaxe-flash identity_admission && cargo test -p bitaxe-parity release_gate && cargo test -p bitaxe-parity release_evidence && bash scripts/package-firmware-test.sh && bazel test //scripts:package_firmware_test //tools/xtask:tests //tools/flash:tests //tools/parity:tests && just package` | ❌ W0 | ⬜ pending |
| 34-02-01 | 02 | 2 | OBS-06 | T-34-05, T-34-08 | Domain correlation pair rejects zero, malformed, and regressing revisions. | unit | `cargo test -p bitaxe-api operator_snapshot && cargo test -p bitaxe-api direct_system_info_projection && cargo test -p bitaxe-api projection_system_info_preserves_axeos_fields && bazel test //crates/bitaxe-api:tests` | ✅ | ⬜ pending |
| 34-02-02 | 02 | 2 | OBS-06 | T-34-08, T-34-09 | Firmware assigns one boot session/revision and retains the same marker across projections. | unit/integration | `cargo test -p bitaxe-parity phase34_operator_snapshot_runtime_source_guard && bazel test //crates/bitaxe-api:tests //tools/parity:tests && just build` | ✅ | ⬜ pending |
| 34-02-03 | 02 | 2 | OBS-06 | T-34-10, T-34-11 | Evidence parser rejects mixed, duplicate, partial, or effectful correlation paths without hardware. | source guard/integration | `cargo test -p bitaxe-parity operator_snapshot_evidence && cargo test -p bitaxe-parity operator_evidence && bazel test //tools/parity:tests` | ✅ | ⬜ pending |
| 34-03-01 | 03 | 3 | SYS-03, SYS-04, SYS-05 | T-34-13, T-34-14 | Platform contracts represent every required fact as proved or explicitly unavailable. | unit/serialization | `cargo test -p bitaxe-api platform_identity && cargo test -p bitaxe-api operator_snapshot && cargo test -p bitaxe-api wire` | ✅ | ⬜ pending |
| 34-03-02 | 03 | 3 | SYS-03, SYS-04, SYS-05 | T-34-13, T-34-15 | Read-only ESP-IDF facts are captured once per coherent revision with no host/fixture substitution. | integration/build/source guard | `cargo test -p bitaxe-api platform_identity && cargo test -p bitaxe-parity phase34_source_guard && just build` | ✅ | ⬜ pending |
| 34-04-01 | 04 | 4 | HLT-01, HLT-02, HLT-03, HLT-04 | T-34-16, T-34-17, T-34-18, T-34-19 | Passive state/health projection performs no self-test, watchdog, load, fault, or hardware effect. | unit/source guard | `cargo test -p bitaxe-core -p bitaxe-api && bazel test //crates/bitaxe-core:all //crates/bitaxe-api:all //firmware/bitaxe:runtime_health_tests //firmware/bitaxe:runtime_health_no_effects_test && git diff --check` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

## Wave 0 Requirements

- [ ] Typed build-identity unit tests and Bazel target.
- [ ] Workspace-status shell behavior test with temporary Git repositories.
- [ ] LCD/system-info/live-WebSocket identity projection tests.
- [ ] Manifest v3 and dirty-admission tests.
- [ ] Cache invalidation regression proving a second dirty source edit changes the firmware action inputs.

Existing Cargo/Bazel/shell infrastructure covers later Phase 34 plans.

## Manual-Only Verifications

None for Plan 01. The application descriptor, embedded commit, manifest, and dirty admission are inspected with host tooling. Physical qualification is deferred to Phase 35.

## Validation Sign-Off

- [ ] All tasks have `<automated>` verification or Wave 0 dependencies.
- [ ] Sampling continuity: no three consecutive tasks without automated verification.
- [ ] Wave 0 covers all missing references.
- [ ] No watch-mode flags.
- [ ] Feedback latency is under 180 seconds for host checks.
- [ ] `nyquist_compliant: true` is set in frontmatter after all planned tests exist.

**Approval:** pending
