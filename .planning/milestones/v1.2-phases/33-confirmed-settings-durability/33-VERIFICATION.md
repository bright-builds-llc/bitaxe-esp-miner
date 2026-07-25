---
phase: 33-confirmed-settings-durability
verified: 2026-07-15T03:20:00Z
status: passed
score: 8/8 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 33-2026-07-14T01-50-49
generated_at: 2026-07-15T03:20:00Z
lifecycle_validated: true
overrides_applied: 0
gaps: []
---

# Phase 33 Verification Report

## Verification Result

Phase 33 has **passed** its remapped software and evidence-readiness boundary. The current source passes the complete software, build, package, reference-clean, source-guard, and simulation verification surface. All eight consolidated software must-haves are verified.

The sole hardware run is credible historical evidence for exact source/package `a6304553343...`: it records the A/N to B/N+1 transition, software reset, unique fresh origin, stable physical identity, matching hostname digests, passive-monitor cleanup, and hostname restoration. It does not qualify current firmware and produces no parity promotion. CFG-12 is not counted in the Phase 33 score; it remains pending under Phase 35, where one final detector-gated exact-current-package run must jointly close CFG-12 and EVD-13.

No verification override exists or applies.

No additional Phase 33 hardware attempt is permitted.

## Goal Achievement

| # | Observable truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Invalid known hostname input fails generically and atomically, without persistence or deferred effects. | VERIFIED | `v12_settings.rs` performs broad validation before authorization; `settings.rs` tests rejection with no adapter/effect calls. |
| 2 | The exact valid hostname-only mutation is the sole authorized Phase 33 settings write, with compatibility classification completed first. | VERIFIED | Closed `V12SettingsDecision` and exhaustive known/unknown/unsupported/mixed request tests. |
| 3 | PATCH success requires one serialized write, commit, strict read-only reload, typed reconciliation, confirmed publication, and effect acquisition in that order; same-value and concurrent requests traverse the same durable chain. | VERIFIED | Pure protocol tests plus `EspNvsSettingsAdapter` transaction ownership and phase source guards. |
| 4 | Immediate API/operator settings state is projected from the confirmed storage snapshot, with no request overlay. | VERIFIED | `ConfirmedHostnameSnapshot`, `ConfirmedSnapshotCell`, firmware adapter publication, and runtime snapshot reload wiring. |
| 5 | Unknown, unsupported, compatibility-only, and mixed requests remain inert: no settings writes, hostname effects, restart acquisition, secret exposure, or unrelated actuation. | VERIFIED | Compatibility decision tests, persistence-call assertions, source guards, and redacted response/evidence models. |
| 6 | The software simulation and host wrapper fail closed before hardware use and enforce detector, canonical flash/package, passive-monitor, restart, cleanup, restoration, and redaction contracts. | VERIFIED | Shell simulation, Bazel test, wrapper source guards, and current build/package checks all pass. |
| 7 | Phase evidence tooling is redacted, protected, non-promotional, and preserves cleanup/restoration outcomes without exposing local identifiers or secrets. | VERIFIED | Evidence README contract, fixture-backed redaction tests, source guards, and the unchanged tracked historical summary. |
| 8 | Typed RTC boot ordinal and origin classification provide substantive, wired A/N to B/N+1 classifier readiness rather than log-shape-only markers. | VERIFIED | Boot identity/evidence modules are wired into firmware and parity tooling and covered by unit, shell, and Bazel tests. |

**Score:** 8/8 must-haves verified.

## Required Artifacts

| Artifact | Expected role | Result |
| --- | --- | --- |
| `crates/bitaxe-api/src/v12_settings.rs` | Closed compatibility-first request decision | VERIFIED — substantive, exhaustive, and used by the HTTP route. |
| `crates/bitaxe-api/src/settings.rs` | Pure durable-hostname protocol and effect ownership | VERIFIED — ordering, same-value, failure, and concurrency behavior are tested. |
| `crates/bitaxe-config/src/persistence.rs` | Typed strict-reload reconciliation contract | VERIFIED — exact stored hostname is required for confirmation. |
| `crates/bitaxe-config/src/confirmed_snapshot.rs` | Confirmed snapshot publication cell | VERIFIED — retains confirmed state on poison and redacts debug output. |
| `firmware/bitaxe/src/settings_adapter.rs` | Serialized NVS transaction, reload, reconciliation, and publication | VERIFIED — the lock spans writable open through confirmed publication. |
| `firmware/bitaxe/src/http_api.rs` | Compatibility routing, durable execution, response, and deferred effect/restart ownership | VERIFIED in software — current implementation compiles and tests pass; its post-run changes create the exact-source hardware gap. |
| `crates/bitaxe-api/src/boot_identity.rs` | Typed boot/identity comparison | VERIFIED — substantive and covered by unit tests. |
| `crates/bitaxe-api/src/phase33_evidence.rs` | Redacted evidence classifier/model | VERIFIED — rejects ineligible states and sensitive raw fields. |
| `firmware/bitaxe/src/rtc_boot_ordinal.rs` | Retained boot ordinal | VERIFIED — firmware-wired and unchanged since the qualified source. |
| `firmware/bitaxe/src/boot_evidence.rs` | Boot marker production | VERIFIED — firmware-wired and unchanged since the qualified source. |
| `firmware/bitaxe/src/wifi_adapter.rs` | Fresh origin evidence | VERIFIED — firmware-wired and unchanged since the qualified source. |
| `scripts/phase33-confirmed-settings-durability.sh` | Detector-gated one-attempt hardware wrapper | VERIFIED in simulation/source guards — no hardware command was run during verification. |
| `scripts/phase33-confirmed-settings-durability-test.sh` | Fail-closed host simulation | VERIFIED — direct execution passes, including failure and cleanup scenarios. |
| `tools/parity/src/phase33_source_guard.rs` | Cross-language ordering and safety source guards | VERIFIED — 11 focused tests pass. |
| `docs/evidence/phase-33/README.md` | Tracked evidence and redaction contract | VERIFIED — explicitly requires exact build/package and blocks CFG-12 on any gate failure. |
| `docs/evidence/phase-33/hardware-summary.md` | Historical redacted hardware proof | VERIFIED AS HISTORICAL ONLY — complete and credible for `a630455`, non-promotional, and not current-firmware evidence. |

The plan artifact declarations are prose rather than machine-readable artifact objects, so the generic GSD artifact/key-link extractor reported zero structured entries. The table above records the required manual existence, substance, and wiring checks.

## Key-Link Verification

| Link | Result | Evidence |
| --- | --- | --- |
| PATCH body → compatibility-first V12 decision | WIRED | `http_api.rs` invokes the closed decision before opening the persistence adapter. |
| Exact hostname decision → pure durable transaction plan | WIRED | `Hostname` and `execute_hostname_update` form the only authorized mutation path. |
| Transaction plan → serialized ESP-IDF NVS write/commit/reload/reconcile | WIRED | `settings_adapter.rs` holds transaction ownership through strict read-only confirmation. |
| Confirmed snapshot → immediate runtime/API projection | WIRED | The confirmed cell is published by the adapter and read during runtime snapshot reload. |
| HTTP response → already-owned deferred settings/restart effect | WIRED IN CURRENT SOFTWARE | Current code acquires process-lifetime queue ownership before returning success and releases it after response handling. |
| RTC ordinal + reset reason + fresh origin + stable identity → evidence classifier | WIRED | Typed firmware markers and parity classifier produce the qualified A/N to B/N+1 record. |
| Wrapper → detector → canonical package flash → passive monitor → cleanup/restoration | WIRED | Simulation and source guards enforce the full contract. |
| Historical hardware summary → current firmware qualification | INTENTIONALLY NOT WIRED | The summary binds `a630455`; exact-source rules prohibit using it for later firmware, and Phase 35 owns a future current-package chain. |

## Data-Flow Trace

The current software path is complete:

1. The HTTP adapter parses the request into the compatibility-first V12 decision.
2. Only the exact hostname-only decision creates a typed `Hostname` mutation.
3. The pure settings protocol asks the firmware adapter to write, commit, reload, reconcile, and publish while holding one transaction owner.
4. The confirmed snapshot cell becomes the immediate runtime/API projection source; request values are not overlaid onto the response.
5. The deferred settings/restart owner is acquired before success and released only after the response boundary.

The historical hardware trace is internally complete for `a630455`: detector-approved identity → canonical package flash → A/N immediate readback → normal software restart → B/N+1 passive capture → unique fresh origin → same identity → matching digest → cleanup → restoration. It stops at the source provenance boundary and is retained only as historical non-promotional proof.

## Requirements Coverage

| Requirement | Status | Verification |
| --- | --- | --- |
| CFG-09 | SATISFIED | Invalid known hostname input is generic, atomic, and effect-free. |
| CFG-10 | SATISFIED | Success follows serialized write/commit/reload/typed-reconciliation/publication. |
| CFG-11 | SATISFIED | Immediate operator/API state comes from confirmed storage truth without overlay. |
| CFG-13 | SATISFIED | Compatibility-only, unknown, unsupported, and mixed input remains inert and redacted. |

## Deferred Requirement Ownership

| Requirement | Owner | Status | Closure contract |
| --- | --- | --- | --- |
| CFG-12 | Phase 35 | PENDING | One final detector-gated exact-current-package run must jointly close CFG-12 and EVD-13 through correlated pre-PATCH, storage-confirmed immediate, and same-board post-reboot hostname observations. Administrative remapping supplies no physical evidence. |

## Behavioral Verification

All commands below were run against software snapshot `c20aa1d`; only planning artifacts changed between that snapshot and this superseding report. No hardware, credentials, serial access, or protected raw evidence was used:

| Command | Result |
| --- | --- |
| `cargo test -p bitaxe-api -p bitaxe-config` | PASS — 167 API tests and 49 config tests passed. |
| `cargo test -p bitaxe-parity phase33_source_guard` | PASS — 11 Phase 33 source-guard tests passed. |
| `bash scripts/phase33-confirmed-settings-durability-test.sh` | PASS — fail-closed simulation, signals, cleanup, redaction, and restoration scenarios passed. |
| `bazel test //scripts:phase33_confirmed_settings_durability_test //crates/bitaxe-api:tests //crates/bitaxe-config:tests //tools/parity:tests --test_filter=phase33` | PASS — all four targets passed. |
| `cargo fmt --all -- --check` | PASS. |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS. |
| `cargo build --all-targets --all-features` | PASS. |
| `cargo test --all-features` | PASS. |
| `just build` | PASS — canonical firmware ELF target built. |
| `just package` | PASS — Ultra 205 package and image artifacts built. |
| `just verify-reference` | PASS — pinned reference is clean at `c1915b0a63bfabebdb95a515cedfee05146c1d50`. |
| `git diff --check` | PASS. |

## Exact-Source and Non-Promotion Assessment

The exact-source boundary remains real and binding:

- `f307336` changes confirmed-snapshot poison behavior and ownership.
- `49c5bca` replaces per-request deferred work with a process-lifetime settings/restart queue and acquires restart ownership before response.
- `a0e4d19` preserves durable PATCH success when the best-effort hostname-effect worker is unavailable.
- Later commits also harden/redact the host wrapper and tracked evidence paths.

The NVS write/commit/reload/reconcile chain remains strongly covered by current software tests, and the typed boot/origin modules remain classifier-ready. That does not establish physical current-package behavior across response delivery, deferred restart execution, ESP-IDF reboot, and post-boot readback. Therefore the `a630455` device run does not qualify current firmware, CFG-12 remains pending for Phase 35, and no parity row is promoted.

## Boundary Analysis

- **Deferred requirement:** CFG-12 has credible historical proof for `a630455` but remains pending for Phase 35's exact-current-package run.
- **Potentially misleading test:** the source guard proves lexical and structural ordering; it cannot prove that the ESP-IDF response is physically delivered before the newly queued restart or that the current firmware survives the reboot with the same stored hostname.
- **Current-source limitation:** the post-review process-lifetime deferred queue and best-effort worker behavior have no eligible current-source device execution across a real software reboot.
- **Provenance result:** material firmware files changed after the sole hardware-qualified commit, so current-firmware qualification and promotion remain prohibited even though all Phase 33 software gates pass.

## Anti-Pattern Scan

No blocker-grade placeholders, unimplemented branches, TODO/FIXME markers, swallowed errors, or debug logging were found in the Phase 33 implementation surface. The later physical qualification boundary is explicitly assigned to Phase 35 rather than represented as completed Phase 33 evidence.

## Deferred Work and Adjacent Phases

Phase 34 owns coherent boot-session/revisioned snapshot/provenance/health work. Phase 35 owns CFG-12, final correlated evidence admission, and parity promotion. Its final exact-current-package chain must close CFG-12 and EVD-13 together. Existing `a630455` evidence remains explicitly historical and non-promotional.

## Human Verification

None requested. Physical qualification is not a Phase 33 completion gate, and the verifier does not ask the user to rerun or reproduce the device procedure.

## Completion Summary

Current Phase 33 software behavior is implemented and independently verified across the pure transaction, firmware integration, build/package, source-guard, and fail-closed simulation surfaces. All eight remapped must-haves pass with CFG-09, CFG-10, CFG-11, and CFG-13 satisfied. CFG-12 remains visibly pending under Phase 35; the credible `a630455` trace supplies no current-firmware qualification or parity promotion. No additional Phase 33 hardware attempt is permitted.
