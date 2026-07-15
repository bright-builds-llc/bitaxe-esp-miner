---
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
verified: 2026-07-15T16:50:23Z
status: gaps_found
score: "6/10 requirements satisfied"
requirements:
  total: 10
  satisfied: 6
  gaps: 4
  satisfied_ids: [SYS-01, SYS-03, SYS-04, SYS-05, HLT-01, HLT-03]
  gap_ids: [OBS-06, SYS-02, HLT-02, HLT-04]
generated_by: gsd-verifier
lifecycle_mode: interactive
phase_lifecycle_id: 34-2026-07-15T03-26-15
generated_at: 2026-07-15T16:50:23Z
lifecycle_validated: true
overrides_applied: 0
gaps:
  - selected_factory_bytes_not_bound_to_admitted_ota_identity
  - safety_supervisor_checkpoint_stops_after_first_yield
  - concurrent_snapshot_publication_can_regress_revision_order
---

# Phase 34 Verification Report

## Verification Result

Phase 34 has **material software gaps**. The typed provenance, platform-identity, snapshot-correlation, and passive-health foundations are substantive, wired, and well tested, but the phase goal is not achieved by the production paths as written.

All three findings in `34-REVIEW.md` are confirmed:

1. Normal full flashing selects `factory_merged_image`, but admission authenticates identity markers only inside the sibling OTA image and never proves that the factory image contains those admitted app bytes.
2. The running safety supervisor records only its first checkpoint because every step yields and the duplicate-log guard returns before later checkpoint recording.
3. HTTP and live-WebSocket captures reserve revisions before collection and retain them later without a publication-order lock, so concurrent completions can append decreasing revisions.

These defects directly block SYS-02, HLT-02/HLT-04, and OBS-06 respectively. Green unit, source-guard, and Bazel tests do not override the production control flow.

No standards override exists or applies. The repo-local GSD, no-hardware, Phase 35 ownership, evidence-redaction, archived-lineage, functional-core/imperative-shell, verification, testing, and Rust rules materially informed this result.

## Goal Achievement

| Phase success criterion | Status | Evidence |
| --- | --- | --- |
| System-info, live WebSocket, retained logs, and evidence bind to one boot session and monotonic operator-snapshot revision. | GAP | Each capture carries a valid pair, but `runtime_snapshot.rs` reserves at lines 54-55/97-98/282-284 and retains at lines 81/345-355 after releasing the sequence lock. HTTP and WebSocket run in separate contexts, so completion and retained order can regress. The evidence validator correctly rejects that order. |
| Running identity reports semantic/source/reference/package and running-platform facts truthfully or unavailable. | PARTIAL | Runtime and typed platform facts are wired, but the normal selected factory flash candidate is not structurally bound to the OTA image whose source/label/app-SHA markers admission checks. |
| Passive self-test exposes only the seven approved states without starting a self-test. | VERIFIED | `PassiveSelfTestState` is closed and serialization tests cover all seven spellings; the adapter is read-only and source guards reject active self-test/effect calls. |
| Supervisor checkpoint health reflects real progress and stays distinct from unproved task-watchdog participation. | GAP | The pure age evaluator and watchdog separation are correct. The producer returns at `watchdog.rs:59-61` on every step after the first, before `record_supervisor_checkpoint` at line 71, so a live loop becomes falsely stale/unhealthy. |
| No forbidden fixture/host substitution, active health intervention, hardware actuation, credentials, Phase 35, or archived-lineage work occurs. | VERIFIED | Typed availability, source guards, and inspected production adapters preserve the software-only observation boundary. No hardware or prohibited workflow was used during verification. |

**Score:** 6/10 mapped requirements satisfied. Three root defects account for four requirement gaps.

## Plan 34-01 Must-Haves: Canonical Build Provenance

| Must-have | Result | Verification evidence |
| --- | --- | --- |
| One validated provenance stamp is the authority for semantic version, build identity, reference commit, ELF, ESP-IDF descriptor, LCD, API, logs, manifest, and active admission. | PARTIAL | `BuildProvenance`, the Bazel materializer, firmware `build.rs`, runtime projections, and manifest v3 share the stamp. Active admission does not authenticate the selected factory app bytes, so the authority does not reach the final full-flash candidate. |
| The human label is derived from full commit, scoped dirty state, and optional exact tag and is never machine evidence. | VERIFIED | `BuildIdentity::new` derives the four-state matrix; tests reject malformed/contradictory values. Admission compares full fields and does not parse the label for source proof. |
| Relevant dirty changes rebuild firmware and every dirty package is rejected before hardware. | VERIFIED | The checked-in pathspec classifier, declared Bazel inputs, dirty-to-dirty cache test, and `prepare_flash` ordering place identity admission before port resolution and credential handling. |
| Clean release and clean dev packages are eligible only when full commits, embedded SHA, and digests agree. | PARTIAL | Both channels are modeled and tested, but agreement is checked against the OTA sibling and each artifact digest independently; selected factory app equivalence is absent. |
| `build_identity.rs` owns dependency-free parsing, derivation, validation, and stamp serialization. | VERIFIED | The module is substantive and its eight focused tests passed. |
| `build-identity-status.sh`, the Rust materializer, and `build_identity.bzl` retain classifier/authority/transport separation. | VERIFIED | Shell emits only five primitive keys; Starlark transports `ctx.info_file`; Rust rejects unknown/missing/duplicate/contradictory fields. |
| `package_manifest.rs` owns schema-v3 validation without live-Git firmware lookup. | VERIFIED | Manifest construction parses the shared stamp and validates structured provenance; no firmware Git lookup is present. |
| Bazel identity materialization emits one stamp and identity sdkconfig used by firmware and packaging. | VERIFIED | `_build_provenance` produces both outputs; firmware and `firmware_image` depend on the same stamp. |
| Compile-time values feed descriptor, LCD, system-info/WebSocket, retained logs, and reference identity from the same typed stamp. | VERIFIED | `build.rs`, `main.rs`, `runtime_snapshot.rs`, and shared wire projection preserve the separate semantic, label, source, reference, channel, dirty, tag, and app-SHA fields. |
| Active parity/evidence/flash gates reject dirty or inconsistent input before device actions and bind the selected bytes. | GAP | Ordering is correct, but `validate_identity_admission` scans only OTA bytes at `tools/flash/src/main.rs:1056-1068`; default resolution selects factory at lines 1171-1180 and flashes it with `write-bin` at lines 776-789. |

## Plan 34-02 Must-Haves: Coherent Snapshot Identity

| Must-have | Result | Verification evidence |
| --- | --- | --- |
| Every public snapshot carries one opaque session and one nonzero revision assigned once to captured facts. | VERIFIED | `BootSessionId`, `OperatorSnapshotRevision`, and `OperatorSnapshotIdentity` are typed; firmware assigns once before completing each `ApiSnapshot`. |
| Revisions increase without reuse within a boot and restart under a new boot session. | PARTIAL | The allocator is checked and concurrent callers receive unique values, but public/retained completion order can decrease because reservation is not serialized with publication. |
| System-info, WebSocket, retained markers, and evidence preserve the same typed pair. | VERIFIED PER CAPTURE | Shared DTOs and marker rendering copy the attached identity. The cross-capture chronology guarantee remains broken. |
| Projections never substitute fixtures, host checkout, clocks, request counts, or labels for session/revision truth. | VERIFIED | Firmware uses the existing hardware-RNG boot observer and sole sequence; evidence rejects fixture and commit-shaped sessions. |
| `operator_snapshot.rs` owns strict session/revision/sequence/marker types. | VERIFIED | Seven focused tests passed, including malformed sessions, zero, exhaustion, marker spelling, and allocator uniqueness. |
| `runtime_snapshot.rs` is the sole assignment authority before projection. | PARTIAL | It is the sole authority, but it reserves before fact collection and publishes later without ordered completion ownership. |
| `operator_snapshot_evidence.rs` validates supplied correlated projections without hardware. | VERIFIED | Five focused validator/source-guard tests passed; it correctly rejects retained revision regression. |
| `boot_evidence.rs` supplies the existing hardware-RNG session without a second nonce. | VERIFIED | One retained session is constructed from four `esp_random()` words and adapted to `BootSessionId`. |
| Runtime/API wire functions copy, rather than mint, the pair. | VERIFIED | `SystemInfoWire::from_snapshot` and live telemetry share the already-captured `ApiSnapshot`. |
| Retained logs and evidence remain chronologically coherent with public publication. | GAP | `retain_completed_operator_snapshot` is called after unlocked collection; the HTTP handler and WebSocket cadence task can complete in reverse reservation order. |

## Plan 34-03 Must-Haves: Truthful Running Platform

| Must-have | Result | Verification evidence |
| --- | --- | --- |
| Platform facts come from embedded or running-device sources, not host fixtures/checkout state. | VERIFIED | Production collection uses embedded static JSON and read-only ESP-IDF calls; fixture values remain explicitly fixture-only. |
| ESP-IDF, static asset, board 205, BM1366, partition, reset, uptime, heap, and PSRAM are truthful or unavailable. | VERIFIED | `PlatformFact<T>` and closed board/ASIC/reset types cover every field independently; compatibility zero never authenticates availability. |
| Platform facts are captured inside the coherent session/revision snapshot. | VERIFIED PER CAPTURE | One `platform_identity::collect()` result is attached before projection and retention. The separate global chronology gap does not alter each candidate's internal field consistency. |
| `platform_identity.rs` defines typed availability and closed vocabularies. | VERIFIED | Six focused API tests passed. |
| Firmware `platform_identity.rs` is the sole read-only ESP-IDF/runtime adapter. | VERIFIED | The adapter exposes collection only and contains no setter, mutation handle, host process, fixture read, or active device command. |
| The Phase 34 source guard rejects fixture/host substitution and prohibited effects. | VERIFIED | Focused source-guard tests passed. |
| Firmware platform adapter supplies one candidate consumed by `runtime_snapshot.rs`. | VERIFIED | Exactly one collection call populates `snapshot.platform_identity`, then compatibility values derive from that candidate. |
| `ApiSnapshot` carries platform identity under the existing snapshot pair. | VERIFIED | The typed identity and platform candidate coexist in one completed immutable snapshot value. |

## Plan 34-04 Must-Haves: Passive Runtime Health

| Must-have | Result | Verification evidence |
| --- | --- | --- |
| Passive self-test state is closed to idle, blocked, running, passed, failed, canceled, or unavailable. | VERIFIED | Exact vocabulary and serialization tests passed. |
| Supervisor health uses monotonic checkpoint sequence and age so a true freeze becomes stale/unhealthy. | GAP IN PRODUCTION | The pure evaluator passes exact boundary tests. The actual producer freezes after one successful loop iteration because duplicate-log suppression returns before checkpoint recording. |
| Supervisor visibility and ESP task-watchdog participation remain separate; unproved participation is unavailable. | VERIFIED | `RuntimeHealthSnapshot::evaluate` always reports watchdog participation `unavailable` with reason `unproved` independently from supervisor health. |
| Reading/publishing health cannot start self-test, mutate watchdog, create load/fault, or actuate hardware. | VERIFIED | Adapter is read-only and the focused no-effects/source-guard tests passed. |
| `runtime_health.rs` owns pure vocabulary, transition validation, and age derivation. | VERIFIED | Eleven focused core tests passed. |
| `runtime_health_adapter.rs` reads existing lifecycle/checkpoint observations only. | VERIFIED | It clones checkpoint history and invokes the pure evaluator. |
| Shared wire projection exposes one captured `runtimeHealth` value. | VERIFIED PER CAPTURE | `SystemInfoWire` and live telemetry use the same `RuntimeHealthWire`; retained record uses the same snapshot pair and value. |
| Firmware adapter feeds the pure evaluator with effect-free observations. | PARTIAL | The adapter link is correct; its producer history is not recurring because of the early return. |
| Runtime snapshot flattens one revision into public and retained health projections. | VERIFIED PER CAPTURE | Health is collected once and the retained record reuses the snapshot session/revision. Publication ordering remains the OBS-06 gap. |

## Review-Finding Adjudication

### CR-01 — Confirmed, blocking SYS-02 and Plan 34-01 admission

`validate_identity_admission` validates the manifest and artifact digests, then reads only `firmware_ota_image` and performs arbitrary byte-substring searches for source commit, label, and decoded app SHA. `resolve_manifest_flash_image` subsequently prefers `factory_merged_image`, and `flash_command_for_image` writes that separate file at address `0x0`.

The focused test `manifest_v3_uses_factory_artifact_for_full_flash` passed while its fixture defines the OTA with identity markers and defines the factory as unrelated bytes `synthetic factory package`. Its output selected the synthetic factory image for `espflash write-bin`. This is direct automated evidence that current tests admit the defect.

**Impact:** a self-consistent manifest can update the factory digest after replacing or mutating the app region, pass admission, and write bytes not authenticated by the OTA identity checks. Current package generation normally merges the OTA at `0x10000`, but generation convention is not an admission proof.

**Required closure:** validate unique, unambiguous artifact kinds; structurally parse the selected factory image/partition layout before port resolution; require its app partition bytes to equal the admitted OTA image; parse the ESP application descriptor at its defined location rather than accepting marker substrings; and add a regression that mutates the factory app region, updates its manifest digest, and still fails admission.

### WR-01 — Confirmed, blocking HLT-02 and HLT-04

`run_supervisor_step` always supplies `elapsed_ms=25` and `consecutive_steps=4`. `StepSupervisor::decision` therefore always returns `YieldNow` at the 100 ms accumulated interval. The first call logs and records sequence 1. On every later call, `if *logged_yield { return; }` exits before `record_supervisor_checkpoint`.

**Impact:** after 1.5 seconds the live supervisor appears stale and after 5 seconds unhealthy even though its 100 ms loop continues. The checkpoint category, sequence, age, and health are therefore not truthful recurring progress. Pure evaluator tests cannot detect the producer bug.

**Required closure:** suppress only the duplicate log, never checkpoint production; record one checkpoint after every successful non-restart supervisor iteration; and add a production-path regression executing multiple steps and proving sequence advancement plus healthy age while the loop progresses.

### WR-02 — Confirmed, blocking OBS-06

The sole sequence mutex covers only `next_identity`. Fact collection, retained marker append, runtime-health record append, and projection occur after that lock is released. `/api/system/info` runs in the HTTP server context while live telemetry runs from the separate `axeos-live-ws` thread.

**Impact:** capture A can reserve revision 1, pause, then capture B can reserve and retain revision 2 before A retains revision 1. The retained history becomes `2, 1`; the evidence validator correctly emits `operator_snapshot_revision_regression`. The allocator test sorts concurrent results and therefore proves uniqueness, not publication order.

**Required closure:** introduce one ordered completion/publication authority—either serialize reserve/collect/retain/payload issuance, or collect unnumbered candidates then assign and enqueue identity plus retained/public projections atomically at publication. Add a concurrency regression that forces reverse collection completion and proves retained and issued public revisions never decrease.

## Requirements Coverage

| Requirement | Status | Verification |
| --- | --- | --- |
| OBS-06 | GAP | Typed per-capture correlation exists, but concurrent completion can regress retained/public chronology and make valid runtime evidence fail nondeterministically. |
| SYS-01 | SATISFIED | Semantic version and full embedded source commit come from the required compile-time stamp with no runtime host substitution. |
| SYS-02 | GAP | Reference and app-SHA fields exist, but selected factory flash bytes are not authenticated against the admitted OTA/provenance bytes. |
| SYS-03 | SATISFIED | ESP-IDF, embedded static asset, board 205, BM1366, and running partition use typed running/embedded sources. |
| SYS-04 | SATISFIED | Reset reason, uptime, internal heap free/minimum/largest block, and PSRAM facts are read from running ESP-IDF state. |
| SYS-05 | SATISFIED | Typed per-field unavailability prevents zero, fixture, or synthetic compatibility values from authenticating platform facts. |
| HLT-01 | SATISFIED | Closed passive lifecycle vocabulary is projected without starting a self-test. |
| HLT-02 | GAP | Fields exist, but the latest checkpoint stops representing recurring supervisor progress after the first yield. |
| HLT-03 | SATISFIED | Supervisor/checkpoint visibility does not imply task-watchdog participation; unproved remains explicit. |
| HLT-04 | GAP | The pure stale/unhealthy thresholds are correct, but the producer falsely freezes while the supervisor is still live, so unhealthy does not mean actual stalled progress. |

The checked boxes and `Complete` labels in `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, and summaries are planning assertions; the code evidence above supersedes them for verification truth. Those files were intentionally not modified.

## Focused Behavioral Verification

All commands were run at source snapshot `758d7282adca1fc78ee1e40ee0a18e305b216e36`. No hardware, USB, network, credentials, Phase 35, direct UART/pins, or archived-lineage operation was used.

| Command | Result |
| --- | --- |
| `cargo test -p bitaxe-api build_identity` | PASS — 8 identity/stamp/status tests. |
| `cargo test -p bitaxe-api operator_snapshot` | PASS — 7 type/allocator tests; publication ordering is not exercised. |
| `cargo test -p bitaxe-api platform_identity` | PASS — 6 availability/platform tests. |
| `cargo test -p bitaxe-core runtime_health` | PASS — 11 pure evaluator tests; the firmware producer is not exercised. |
| `cargo test -p bitaxe-parity operator_snapshot_evidence` | PASS — 5 validator/source-guard tests, including rejection of revision regression. |
| `cargo test -p bitaxe-parity phase34_source_guard` | PASS — 4 source-guard tests; none proves selected factory app equality, recurring checkpoints, or ordered publication. |
| `cargo test -p bitaxe-flash manifest_v3_uses_factory_artifact_for_full_flash` | PASS — selected an unrelated synthetic factory blob after OTA-only identity validation, confirming CR-01 coverage is missing. |
| `bazel test //scripts:build_identity_status_test //scripts:build_identity_cache_invalidation_test //scripts:package_firmware_test //crates/bitaxe-api:tests //crates/bitaxe-core:tests //tools/flash:tests //tools/parity:tests` | PASS — 7 focused targets, all cache hits. |
| `just verify-reference` | PASS — reference clean at `c1915b0a63bfabebdb95a515cedfee05146c1d50`. |
| `git diff --check` | PASS before this report was written. |

The previously completed all-feature Rust sequence and repository-wide `bazel test //...` are credible regression evidence, but were not repeated because the user explicitly supplied their post-implementation pass and focused verification reproduced the relevant blind spots.

## Gap Closure Work

1. **Bind the selected factory image to admitted provenance before hardware.** Add structural ESP image/partition validation, exact factory-app-to-OTA equality, unique artifact-kind enforcement, descriptor-offset parsing, and tamper regression coverage.
2. **Repair supervisor checkpoint production.** Remove the duplicate-log early return from the progress path and prove multiple real supervisor steps advance sequence and stay healthy.
3. **Serialize snapshot completion/publication.** Assign revisions at an ordered publication boundary or retain the ordering owner through capture and payload issuance; add adversarial reverse-completion concurrency coverage.
4. Re-run the mandatory Rust sequence, focused Cargo/Bazel regressions, `bazel test //...`, canonical build/package/reference checks, and diff review. Then re-run Phase 34 verification and update requirement status only if the production paths pass.

## Human Verification

None. These are deterministic software correctness defects. Hardware or manual observation would neither repair nor reliably disprove them. Phase 35 must remain blocked on a passed Phase 34 re-verification.

## Exact Non-Claims

- This report does not claim Phase 34 passed or that its roadmap/requirements completion marks are correct.
- It does not claim the bytes selected for normal factory flashing are authenticated by current admission.
- It does not claim current supervisor health truthfully distinguishes a live loop from a stalled loop.
- It does not claim operator-snapshot publication is monotonic under concurrent HTTP/WebSocket captures.
- It does not qualify any cached package or current HEAD on hardware.
- It does not supply Phase 35 evidence, CFG-12/EVD closure, parity promotion, active safety verification, mining verification, OTA/recovery proof, credential proof, or other-board proof.
- It does not use or authorize hardware, USB, network, credentials, direct UART/pins, archived Phase 28.1.1 work, or source changes.

## Completion Summary

Phase 34 is **not complete** at the verified production boundary. Six requirements remain supported by code and focused tests; OBS-06, SYS-02, HLT-02, and HLT-04 remain gaps. Closure requires the three software fixes above and fresh verification before Phase 35 may rely on this phase.
