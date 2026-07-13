---
phase: 31-operator-claim-and-telemetry-contract
status: passed
score: "26/26 must-haves verified"
verified: 2026-07-13
generated_by: gsd-verifier
generated_at: 2026-07-13T21:53:38Z
lifecycle_mode: yolo
phase_lifecycle_id: 31-2026-07-13T19-47-51
lifecycle_validated: true
verification_result: passed
requirements:
  - OBS-01
  - CFG-08
review_status: clean
security_status: passed
---

# Phase 31: Operator Claim and Telemetry Contract Verification

## Verdict

Phase 31 passed. The implementation establishes independently truthful observation states, keeps compatibility numerics subordinate to stamped truth, makes request-side consumers read-only, limits Phase 31 settings authority to an effect-free hostname capability, and admits only the two exact Phase 31 claims. Review fixes `2463764` and `9f66dd9` close the two provenance/projection warnings without adding a producer, persistence effect, hardware action, evidence promotion, or archived-lineage operation.

## Goal Achievement

| Success criterion | Result | Evidence |
| --- | --- | --- |
| Power and thermal facts distinguish `fresh`, `stale`, `unavailable`, and `fault` independently from numeric compatibility values. | VERIFIED | `Observation<T>` is a four-variant state-carrying enum. Six operator facts carry separate `ObservationTruthWire` values. Mixed-state and fresh-zero-versus-unavailable-zero tests pass, and nonfresh or unstamped values project numeric zero without becoming fresh. |
| Producer-owned sequence and monotonic acquisition metadata cannot be refreshed by reads or projections. | VERIFIED | Successful publication advances a boot-scoped source sequence once. Stale and fault transitions retain the exact last-good stamp. `ObservationStore::read`, system-info, statistics, and WebSocket projections copy stored state without changing metadata. Repeated-read and projection tests pass. |
| `hostname` is the complete Phase 31 v1.2 PATCH allowlist. | VERIFIED | `V12SettingsChange` has only `Hostname(Hostname)`. The pure classifier authorizes only an exact validated hostname-only object; every other schema field, unknown field, mixed payload, credential, control, mining/self-test field, and empty payload remains compatibility-only. No persistence executor or live HTTP handler was changed; Phase 33 retains that ownership. |
| Unsupported control, evidence, and broad claims cannot become eligible Phase 31 outcomes. | VERIFIED | `V12PromotableClaim` contains only `ObservationTruthContract` and `HostnamePatchAllowlist`. Every excluded category is a typed `V12ExclusionReason`; wrong-row, missing-evidence, string, schema-growth, and broad-promotion tests fail closed. The parity checklist is byte-unchanged across the phase range. |

**Goal score:** 4/4 success criteria verified.

## Requirements Coverage

| Requirement | Status | Verification |
| --- | --- | --- |
| OBS-01 | SATISFIED | The requirements checkbox and traceability row are complete; Plans 31-01 and 31-02 carry the ID. Four-state truth, independent facts, compatibility-zero separation, producer stamps, and immutable consumer projections are implemented and tested. |
| CFG-08 | SATISFIED | The requirements checkbox and traceability row are complete; Plan 31-03 carries the ID. Hostname is the only constructible Phase 31 settings capability and all broader fields remain ineligible for v1.2 promotion. |

The GSD phase initializer's null requirement field is not authoritative here; `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`, and plan frontmatter consistently bind OBS-01 and CFG-08 to Phase 31.

## Must-Haves Audit

All 12 truths, eight required artifacts, and six key links from the three plans are verified.

### Observation Truth Core

- `crates/bitaxe-safety/src/observation.rs` owns the closed observation variants, domain-wrapped boot session/sequence/monotonic time, typed reasons, success transition, and stamp-preserving stale/fault transitions.
- `crates/bitaxe-safety/src/power.rs` and `thermal.rs` consume that contract. Power safety remains fail closed, and temperature/tachometer truth remains independently representable.
- Compatibility accessors are projections only. Numeric zero is not stored as observation truth and cannot authenticate freshness.

### API And Firmware Consumer Boundary

- `crates/bitaxe-api/src/observation.rs` maps observation state and stamp separately from numeric compatibility fields and preserves metadata during fact projection.
- `firmware/bitaxe/src/safety_adapter/observation_store.rs` starts unavailable and exposes a copy-only read plus producer-named complete replacement boundary; its source-boundary regression finds no ESP-IDF, I2C, GPIO, fan, voltage, or reset surface.
- `firmware/bitaxe/src/runtime_snapshot.rs` reads the stored observation snapshot instead of collecting safety telemetry during a request.
- Review fix `2463764` makes retained Phase 27 compatibility data explicitly unavailable because it lacks eligible producer provenance. Review fix `9f66dd9` suppresses all six nonfresh or unstamped operator numerics even if the legacy aggregate status is mutable or says fresh.
- `SystemInfoWire`, live WebSocket payloads, and statistics consume the same immutable snapshot projection; no consumer advances sequence or acquisition time.

### Settings And Claim Admission

- `crates/bitaxe-api/src/v12_settings.rs` is an effect-free classifier with one authority variant. Its tests cover all current schema fields, mixed and secret-bearing inputs, generic errors, redaction-safe diagnostics, and absence of persistence/effect symbols.
- `tools/parity/src/v12_admission.rs` owns the two exact eligible claims and 18 typed exclusion/admission-failure reasons. OBS-01 and CFG-08 evidence is row-scoped and cannot authenticate the other claim.
- `tools/parity/src/main.rs` invokes the closed Phase 31 validation. `just parity` returns `validation_errors: none`, and `docs/parity/checklist.md` is unchanged.

## Security And Review Result

The final standard-depth review is clean: 0 critical, 0 warning, and 0 info findings in `31-REVIEW.md`. `31-REVIEW-FIX.md` records both prior warnings fixed, zero skipped, and `status: all_fixed`.

OWASP ASVS Level 1 was the phase baseline. Every High-severity STRIDE threat has a verified mitigation:

| Threats | Mitigation result |
| --- | --- |
| T-31-01, T-31-02, T-31-07 | Closed state variants, producer success transition, exact stamp retention, and fail-closed power/thermal tests pass. |
| T-31-03, T-31-08, T-31-09, T-31-10 | Explicit truth/numeric separation, immutable repeated reads, mixed-state projection, unavailable initialization, source-boundary scan, and firmware build pass. |
| T-31-04, T-31-11 | Exact hostname-only construction, exhaustive broader-field rejection, redaction-safe diagnostics, and absence of persistence/effect authority pass. |
| T-31-05, T-31-12 | Closed two-claim admission, exhaustive typed exclusions, wrong-row/missing-evidence/string/schema-growth rejection, and checklist preservation pass. |

Medium threats T-31-06 and T-31-13 also pass through cross-session ordering tests and stable generic settings-error tests.

## Independent Command Evidence

| Command or check | Result |
| --- | --- |
| `cargo fmt --all` | PASS; completed without changing the worktree. |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS. |
| `cargo build --all-targets --all-features` | PASS. |
| `cargo test --all-features` | PASS; complete host workspace tests and doc tests passed. |
| `cargo test -p bitaxe-safety --all-features observation` | PASS; 12 focused tests. |
| `cargo test -p bitaxe-api --all-features safety_telemetry` | PASS; 14 focused tests. |
| `cargo test -p bitaxe-api --all-features projection` | PASS; 19 focused tests. |
| `cargo test -p bitaxe-api --all-features settings_v12` | PASS; 8 focused tests. |
| `cargo test -p bitaxe-parity --all-features phase31_` | PASS; 7 focused tests. |
| `bazel test //crates/bitaxe-safety:tests //crates/bitaxe-api:tests //tools/parity:tests` | PASS; 3/3 affected targets. |
| `just build` | PASS; canonical `xtensa-esp32s3-espidf` release firmware built against ESP-IDF `v5.5.4`. Eight reported firmware dead-code warnings are pre-existing and outside the host Clippy `-D warnings` surface. |
| `just test` | PASS; all 53 Bazel test targets passed. Repository script targets used deterministic fixtures only; no hardware command was invoked. |
| `just parity` | PASS; `validation_errors: none`. |
| `just verify-reference` | PASS; reference clean at `c1915b0a63bfabebdb95a515cedfee05146c1d50`. |
| Reference and promotion diff | PASS; `git diff --exit-code 1ed2226..HEAD -- reference/esp-miner docs/parity/checklist.md` returned clean, and the reference submodule worktree is clean. |
| Requirement traceability | PASS; OBS-01 and CFG-08 are checked, mapped to Phase 31 as Complete, and present in plan frontmatter. |
| `gsd-tools verify schema-drift 31 --raw` | PASS; `drift_detected: false`, `blocking: false`. |
| `gsd-tools verify lifecycle 31 --expect-id 31-2026-07-13T19-47-51 --expect-mode yolo --require-plans --raw` | PASS; `valid`. |
| `git diff --check 1ed2226..HEAD` and current `git diff --check` | PASS after documentation-only whitespace fix `6eb3e41`. |

The exact mandatory Rust gate ran separately and in the required order: format, Clippy, build, then test.

## Scope And Safety Audit

Verification used committed source, planning artifacts, pure host tests, Bazel builds, packaging checks, parity validation, and reference guards only. It did not detect or access a board, USB or serial device, credentials, ignored local evidence, network target, or hardware state. It did not flash, monitor, reset, actuate fan/voltage/power/ASIC controls, start self-test or mining behavior, run OTA/recovery, invoke direct UART/pin work, operate on the archived Phase 28.1.1 lineage, or promote checklist evidence.

No Phase 31 commit changes the reference tree, parity checklist, committed evidence, config persistence implementation, or live HTTP settings handler. The settings capability remains pure and its effectful integration is explicitly deferred to Phase 33.

## Exact Non-Claims

- Phase 31 does not prove live INA260/EMC2101 acquisition, a real producer cadence, sensor-failure recovery, or hardware freshness; Phase 32 owns those outcomes.
- Phase 31 does not write, commit, reload, reconcile, expose, or reboot-test hostname persistence; Phase 33 owns those outcomes.
- Phase 31 does not establish Phase 34's coherent global operator-snapshot revision, system provenance, retained-log correlation, or passive runtime-health evidence.
- Phase 31 does not provide detector-gated Phase 35 evidence or promote any parity checklist row.
- Active control, self-test effects, watchdog intervention/load, mining or archived lineage, credentials, direct UART/pins, OTA/recovery, other boards, telemetry history, UI/display/input/BAP, and broad production/verified claims remain ineligible.

## Residual Risks

- Until Phase 32 installs the sole real producer, firmware operator telemetry intentionally remains unavailable rather than manufacturing freshness from retained legacy samples.
- `ObservationStore::replace` is a pure host-testable primitive; firmware restricts mutation through its producer-named adapter, but later producer integration must preserve that ownership contract.
- The roadmap's prose rollup still says `0/5 phases complete; 1/27 requirements complete` even though its Phase 31 row and both authoritative requirement rows are complete. This is a non-blocking derived-summary inconsistency, not a Phase 31 behavior or traceability gap.

## Final Status

`verification_result: passed`. Phase 31 achieves OBS-01 and CFG-08 at typed contract, host-test, firmware-build, and workflow-validation depth while preserving every deferred effect and evidence boundary.

***

Material guidance applied: repo-local Phase 31 scope, ESP-IDF boundary, archived-lineage prohibition, hardware/credential/evidence restrictions, and frontmatter rules in `AGENTS.md`; the Bright Builds workflow in `AGENTS.bright-builds.md`; no active override in `standards-overrides.md`; and the architecture, code-shape, testing, verification, operability, and Rust standards under `standards/`.
