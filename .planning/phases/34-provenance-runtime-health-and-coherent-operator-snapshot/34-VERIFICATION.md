---
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
verified: "2026-07-17T14:05:28Z"
status: passed
score: 10/10 requirements satisfied
requirement_score: 10/10
roadmap_truth_score: 5/5
plan_score: 11/11
generated_by: gsd-verifier
verification_mode: fresh_goal_backward
lifecycle_mode: interactive
phase_lifecycle_id: 34-2026-07-15T03-26-15
generated_at: "2026-07-17T14:05:28Z"
lifecycle_validated: true
reviewed_source_commit: fdd3c7ab3547165fecf35b3267184fc5098b5599
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: "9/10"
  mode: fresh_goal_backward
  gaps_closed:
    - "SYS-02 now requires the descriptor-bearing first segment to be non-empty DROM."
    - "SYS-02 now rejects pairwise direct destination overlap and ESP32-S3 D/IRAM physical-alias overlap."
  gaps_remaining: []
  regressions: []
---

# Phase 34: Provenance, Runtime Health, and Coherent Operator Snapshot Verification Report

**Phase Goal:** An Ultra 205 operator can inspect one internally coherent snapshot of read-only telemetry, confirmed settings, truthful identity, and passive runtime health across system-info, WebSocket, retained logs, and evidence projections.

**Reviewed source:** `fdd3c7ab3547165fecf35b3267184fc5098b5599`

**Verified:** 2026-07-17T14:05:28Z

**Status:** `passed`

**Verification mode:** Fresh goal-backward verification. The prior report was used only as historical gap context; all ten requirements were re-evaluated from the current code and executable behavior.

## Verdict

Phase 34 achieves its goal. All ten requirements, all five roadmap success criteria, and all eleven plans are verified at the reviewed source commit. The two earlier SYS-02 reproductions and the two final Plan 34-11 layout gaps are closed at the pure parser, OTA/factory package, parsed dry/non-dry CLI, immutable snapshot, manifest/ELF, and pre-effect boundaries.

No override was applied. No hardware, serial port discovery, USB access, credentials, network access, OTA, Phase 35 work, or archived Phase 28.1.1 lineage work was used.

## Goal Achievement

### Observable Truths

| # | Roadmap truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | System-info, live WebSocket, retained log, and evidence bind to one opaque boot session and monotonic snapshot revision. | ✓ VERIFIED | `BootSessionId` and checked `OperatorSnapshotRevision` are assigned by the serialized publisher; the same completed snapshot is issued to system-info/live routes and atomically retained before issuance. Evidence validation rejects mixed sessions, regressions, missing markers, and mismatched redacted projections. |
| 2 | Runtime identity and platform facts are truthful or explicitly unavailable. | ✓ VERIFIED | Canonical provenance is parsed once, compiled into firmware without a Git fallback, projected with the ESP application ELF SHA, and bound through schema-v3 package admission. ESP-IDF/static asset, board, ASIC, partition, reset, uptime, and heap facts come from the firmware adapter or a typed unavailable state. |
| 3 | Passive self-test exposes only the closed lifecycle vocabulary and starts no submode. | ✓ VERIFIED | The pure runtime-health model owns the exact seven-value vocabulary. The firmware adapter only reads existing state and currently reports `unavailable`; no active self-test call or effect surface is reachable from collection. |
| 4 | Supervisor checkpoint facts age to stale/unhealthy and remain distinct from task-watchdog proof. | ✓ VERIFIED | The recurring supervisor producer advances checkpoint sequence/time independently of one-shot log suppression. The pure evaluator derives healthy/stale/unhealthy from age and reports task-watchdog participation as `unavailable` with reason `unproved`. |
| 5 | Phase 34 remains read-only and does not broaden evidence or hardware scope. | ✓ VERIFIED | Collection adapters are observation-only; source guards and no-effects tests reject fixture substitution and active effects. This verification used only host tests, packaging, artifact inspection, and an inert `/dev/null` dry run. |

**Roadmap score:** 5/5 truths verified

## Requirements Coverage

| Requirement | Description | Status | Evidence |
| --- | --- | --- | --- |
| OBS-06 | One boot session and monotonic revision across system-info, live WebSocket, retained log, and evidence | ✓ SATISFIED | `operator_snapshot.rs:16,89,142,189`; `operator_snapshot_publication.rs:105`; `runtime_snapshot.rs:121,145,340-376`; `wire.rs:44-50,200-210,325`; `operator_snapshot_evidence.rs:81-158,252-397`. Reverse-completion, concurrent, retention, issuance, evidence-coherence, and route-wiring tests passed. |
| SYS-01 | Embedded semantic version and source commit without host-checkout substitution | ✓ SATISFIED | `build_identity.rs:31-249` strictly derives/parses identity. `build.rs` requires the generated provenance stamp; `main.rs:187-231` reads compile-time values and retains them; `runtime_snapshot.rs:542-548` projects them. Build-status, cache-invalidation, Cargo, and source-guard tests passed. |
| SYS-02 | Correlate running firmware, reference commit, and flashed package identity | ✓ SATISFIED | The schema-v3 manifest binds the unique firmware ELF SHA to `app_elf_sha256`; active admission rehashes the actual ELF before reading OTA/factory bytes; the exact factory slice must equal the fully parsed OTA; execution uses only a private immutable byte snapshot. Canonical envelope, descriptor, overlap, digest, and pre-effect coverage is detailed below. |
| SYS-03 | Truthful ESP-IDF/static asset, board 205, BM1366, and running partition identity | ✓ SATISFIED | `platform_identity.rs:20-40` collects each read-only fact once; typed `PlatformFact` values preserve unavailable reasons. The same candidate is completed into the issued snapshot. |
| SYS-04 | Truthful decoded reset, uptime, and heap facts | ✓ SATISFIED | `platform_identity.rs:58-110` uses ESP-IDF reset, monotonic timer, and heap APIs; unknown/zero values become unavailable. Reset decoding and zero/unavailable regressions passed. |
| SYS-05 | Explicit unavailable state; no fixture or placeholder substitution | ✓ SATISFIED | `PlatformFact<T>` is a closed available/unavailable sum type and `PlatformIdentity::fixture_only` authenticates nothing. Runtime source guards reject host-checkout and placeholder substitution; evidence validation rejects commit-shaped boot sessions and synthetic fixture sessions. |
| HLT-01 | Passive self-test lifecycle without starting a hardware self-test | ✓ SATISFIED | `runtime_health.rs:10-36` owns the exact vocabulary. `runtime_health_adapter.rs:6-16` performs only read/copy/evaluate work and supplies `PassiveSelfTestState::Unavailable`. The no-effects Bazel target passed. |
| HLT-02 | Supervisor availability plus bounded checkpoint category, sequence, and age | ✓ SATISFIED | `watchdog.rs:17-31,143-183` owns bounded recurring checkpoint history; `runtime_health.rs:99-184,210-264` validates category/sequence/time and projects age. Wire and retained records use the same completed snapshot. |
| HLT-03 | Checkpoint visibility is distinct from ESP task-watchdog participation | ✓ SATISFIED | `TaskWatchdogParticipation` is independent of `CheckpointHealth`; evaluation always reports `unavailable`/`unproved` in Phase 34. Exact vocabulary and “healthy does not imply watchdog” tests passed. |
| HLT-04 | Frozen progress ages to stale or unhealthy | ✓ SATISFIED | `runtime_health.rs:243-251` derives thresholds from checkpoint age; boundary, frozen-sequence, recovery, regression, and recurring-producer tests passed. |

**Requirement score:** 10/10 satisfied

There are no orphaned Phase 34 requirements: the roadmap and requirements traceability map exactly OBS-06, SYS-01 through SYS-05, and HLT-01 through HLT-04 to this phase.

## SYS-02 Closure Audit

### Earlier reproduced gaps

| Reproduction | Current closure | Boundary proof |
| --- | --- | --- |
| WR-01: a recomputed package redirected entry/load addresses, including zero load, while preserving outer hashes | `ValidatedEsp32S3Image::validate` checks exact ESP32-S3 header values, checked segment arithmetic, admitted memory envelopes, mapped congruence, executable entry containment, descriptor/MMU identity, checksum, appended SHA, and exact EOF. Zero-length segments retain no-range semantics. | Pure parser and complete OTA/factory tests passed. Parsed dry-run zero-load and parsed non-dry mapped-mismatch tests assert no port, credential, snapshot, command, execution, USB, or flash effect. |
| WR-02: a different ELF plus honestly recomputed artifact digest contradicted top-level application identity | Manifest construction and validation require exactly one `firmware_elf` and equality with `app_elf_sha256`. Active admission reads and rehashes the actual ELF and checks that relationship before OTA/factory reads. | Parsed dry and non-dry changed-ELF tests assert `firmware_elf_app_sha_mismatch`, no later artifact read, no snapshot, no port/credential lookup, and no execution. The current package’s computed ELF SHA equals both manifest fields. |

### Final Plan 34-11 layout gaps

| Gap | Parser invariant | Full-boundary proof |
| --- | --- | --- |
| Descriptor-bearing segment family was unconstrained | `ValidatedSegmentLayout::try_new` first requires segment 0 to exist, be non-empty, contain the full descriptor, and be DROM; descriptor fields remain anchored at payload offset zero. | Independently resealed descriptor-in-DRAM/IRAM/IROM/RTC fixtures fail through standalone OTA and exact factory-embedded OTA validation. Parsed dry and non-dry paths return only `app_descriptor_segment_not_drom` before every effect sentinel. |
| Segments could overlap directly or through D/IRAM aliases | Every non-empty segment becomes a checked half-open range. All numeric pairs must be disjoint; IRAM ranges are normalized with pinned `SOC_I_D_OFFSET` before DRAM/IRAM alias comparison. Exact adjacency is accepted and zero-length segments create no range. | Resealed direct and alias-overlap fixtures fail through OTA, factory, and parsed dry/non-dry paths. Positive direct/alias adjacency and zero-length cases pass. |

### Admission and execution chain

| Stage | Verified invariant | Status |
| --- | --- | --- |
| Manifest producer | Schema v3, canonical provenance, unique ELF/OTA/factory kinds, actual ELF digest equals top-level application SHA | ✓ VERIFIED |
| Active manifest consumer | Current clean provenance equality, unique artifacts, actual ELF digest equality before later image reads | ✓ VERIFIED |
| OTA parser | Exact supported ESP32-S3 envelope, typed segment layout, descriptor identity, checksum/digest/trailer | ✓ VERIFIED |
| Factory package | Exact `0x10000` app partition, embedded OTA parsed independently and byte-equal to admitted OTA | ✓ VERIFIED |
| Parsed CLI | Both dry-run and non-dry malformed-package paths fail before ports, credentials, rendering, snapshots, USB, execution, flash, or OTA | ✓ VERIFIED |
| Immutable execution | Non-dry execution materializes already-admitted factory bytes once in an owner-held mode-0600 temporary file; replacement of package paths cannot change child bytes; cleanup is tested on success and failure | ✓ VERIFIED |
| Canonical positive | Current `fdd3c7ab...` package has one ELF, one OTA, one factory artifact; ELF relationship and exact factory embedding hold; inert parsed dry-run admits it | ✓ VERIFIED |

## Required Artifacts

| Artifact group | Expected | Status | Details |
| --- | --- | --- | --- |
| Canonical build identity | One strict provenance authority transported into firmware/package inputs | ✓ VERIFIED | `build_identity.rs`, `scripts/build-identity-status.sh`, `scripts/build_identity.bzl`, `firmware/bitaxe/build.rs`, and manifest construction are substantive and wired. |
| Coherent operator snapshot | One boot-lifetime publisher assigns and carries identity through collection, retention, and issuance | ✓ VERIFIED | `operator_snapshot.rs`, `operator_snapshot_publication.rs`, `runtime_snapshot.rs`, HTTP routes, and retained-pair storage are substantive and wired. |
| Truthful platform facts | Read-only ESP-IDF adapter plus closed unavailable representation | ✓ VERIFIED | `platform_identity.rs` in API and firmware is wired into candidate collection and wire projection. |
| Passive runtime health | Pure evaluator, recurring producer, read-only adapter, API/log projection | ✓ VERIFIED | Core health, supervisor checkpoint producer, firmware adapter, wire DTO, and retained record are wired. |
| Package identity admission | Exact parser, package relationship, active consumer, immutable execution snapshot | ✓ VERIFIED | `esp32s3_image.rs`, `package_admission.rs`, `main.rs`, and `package_manifest.rs` form one closed admission chain. |
| Evidence coherence | Host validator rejects mixed/incomplete/duplicate/regressing snapshot identities | ✓ VERIFIED | `operator_snapshot_evidence.rs` and Phase 34 source guards are substantive and covered by passing tests. |

The automated PLAN artifact checker reported 34/34 declared artifacts passing for Plans 34-04 through 34-11. Plans 34-01 through 34-03 use legacy frontmatter not surfaced by that checker; their artifacts were verified manually at existence, substance, wiring, and data-flow levels.

## Key Link Verification

| From | To | Via | Status |
| --- | --- | --- | --- |
| Workspace status | Firmware/API/LCD/log/package | One parsed `BuildProvenance` stamp and compile-time environment; no runtime Git query | ✓ WIRED |
| Hardware RNG boot nonce | System-info/live/retained/evidence | One boot-session authority plus serialized checked revision allocator | ✓ WIRED |
| Runtime settings/telemetry/platform/health candidate | HTTP and WebSocket issue functions | Candidate collection precedes lock; one immutable completion is retained, then actually issued | ✓ WIRED |
| Supervisor producer | Runtime-health wire/log record | Recurring checkpoint history is copied into the pure age evaluator and completed snapshot | ✓ WIRED |
| Firmware ELF and schema-v3 manifest | OTA/factory parser | Unique actual ELF SHA, descriptor SHA, OTA bytes, and embedded factory slice must agree | ✓ WIRED |
| Admitted factory bytes | Child `espflash write-bin` | Private owner-held immutable snapshot path, never a re-read package path | ✓ WIRED |

The generic key-link grep cannot infer Rust module imports from plan prose and reports false negatives for path-to-module links. Manual symbol and call-chain tracing above verifies the actual links.

## Data-Flow Trace

| Output | Data source | Flow | Status |
| --- | --- | --- | --- |
| `/api/system/info` | Runtime projection, confirmed settings, platform adapter, health adapter, boot session | `publish_projected_system_info` → one completed snapshot → retained pair → HTTP issue closure | ✓ FLOWING |
| `/api/ws/live` | Same candidate sources and publisher | `publish_projected_live_telemetry_payload` → one completed snapshot → retained pair → connect/cadence issue closure | ✓ FLOWING |
| Retained correlation | Same completed identity and health value | exact marker + runtime-health record → one `RetainedPair` → one mutex acquisition/atomic append | ✓ FLOWING |
| Evidence projection | Captured system-info, live JSON, retained record | strict duplicate-aware parsers → chronology/session/membership/redaction checks | ✓ FLOWING |
| Package/flash identity | Built ELF, generated OTA, factory image, manifest | actual hashes + typed image parser → owned factory bytes → private execution snapshot | ✓ FLOWING |

## Behavioral Spot-Checks

| Behavior | Command/check | Result | Status |
| --- | --- | --- | --- |
| Pure API, health, package, flash, and evidence behavior | `cargo test -p bitaxe-flash -p bitaxe-api -p bitaxe-core -p bitaxe-parity -p xtask` | 597 passed, 0 failed | ✓ PASS |
| Firmware adapters and Bazel ownership | Eight focused Bazel targets for API/core, runtime health, no-effects, checkpoint producer, retained pair, flash, parity, and xtask | 8/8 passed (cached) | ✓ PASS |
| Build identity and package shell contracts | Three focused Bazel script targets | 3/3 passed (cached) | ✓ PASS |
| Current firmware/package construction | `just package` | ESP32-S3 release firmware and schema-v3 package built successfully | ✓ PASS |
| Current package identity | `jq`, SHA-256, and exact factory-slice comparison | Reviewed source commit matched; one ELF/OTA/factory each; actual ELF SHA matched both manifest fields; factory embedded exact OTA | ✓ PASS |
| Canonical parsed admission | `cargo run -q -p bitaxe-flash -- flash --board 205 --manifest ... --port /dev/null --dry-run` | Admitted the unique factory image and rendered only the inert dry-run command | ✓ PASS |
| Formatting and shell quality | `cargo fmt --all -- --check`; `shellcheck`; `shfmt -d`; `git diff --check` | All clean | ✓ PASS |

## Anti-Patterns Found

| File | Pattern | Severity | Impact |
| --- | --- | --- | --- |
| `tools/parity/src/phase34_source_guard.rs` | The word `placeholder` appears only as a forbidden-source token in a source guard | ℹ️ Info | Not a stub; it actively rejects placeholder substitution. |

No Phase 34 goal path contains a TODO/FIXME/placeholder implementation, empty handler, hardcoded empty user-visible data source, ignored response, or effect-only logging stub.

## Human Verification Required

None. The Phase 34 contract is deliberately software-only and every must-have is deterministically verifiable through source tracing, host execution, package construction, and inert admission. Hardware evidence belongs to later scoped work and was neither required nor authorized here.

## Gaps Summary

No gaps remain. SYS-02’s historical entry/load and ELF-identity reproductions remain closed, and Plan 34-11 closes the descriptor-segment and pairwise destination/alias layout gaps without weakening adjacency, zero-length, package, immutable-execution, or pre-effect guarantees. OBS-06 and the other eight requirements show no regression.

## Verification Complete

**Status:** passed

**Score:** 10/10 requirements satisfied

All must-haves are verified. Phase 34 achieves its goal at reviewed source commit `fdd3c7ab3547165fecf35b3267184fc5098b5599`.

***

_Verified: 2026-07-17T14:05:28Z_

_Verifier: the agent (gsd-verifier)_
