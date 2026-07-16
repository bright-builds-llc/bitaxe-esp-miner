---
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
status: gaps_found
score: "9/10 requirements satisfied"
generated_by: gsd-verifier
lifecycle_mode: interactive
phase_lifecycle_id: 34-2026-07-15T03-26-15
generated_at: 2026-07-16T21:19:23Z
lifecycle_validated: true
---

# Phase 34 Verification

## Verification Result

**Status:** `gaps_found`
**Score:** 9/10 requirements satisfied

Nine Phase 34 requirements are supported by production-source inspection and executable software-only evidence. `SYS-02` remains unsatisfied because the canonical ESP32-S3 application-image admission path does not enforce two required layout invariants: the descriptor-bearing first segment is not required to be DROM, and decoded segment destinations are not checked for pairwise direct or D/IRAM alias overlap.

The repo-local guidance, Bright Builds workflow, architecture, testing, verification, code-shape, and Rust standards materially informed this result. No active standards override applies. This verification performed no hardware, device, credential, network-discovery, flash, monitor, OTA, UART, pin, or archived Phase 28 lineage work.

## Requirements

| Requirement | Status | Verification basis |
| --- | --- | --- |
| `SYS-01` | **verified** | Strict typed build identity and provenance are parsed from the required build stamp, embedded at compile time without a Git fallback, and projected into runtime identity. Contradictory or malformed identity inputs fail closed. |
| `SYS-02` | **gaps_found** | Schema-v3 packaging, ELF/application digest binding, entry and load-address validation, descriptor validation, trailer validation, unique factory selection, and immutable execution snapshots are present. The parser does not require the descriptor-bearing first segment to be DROM and does not reject pairwise direct or D/IRAM alias overlap, so the declared canonical producer envelope is incomplete. |
| `SYS-03` | **verified** | Board `205`, BM1366 ASIC identity, ESP-IDF version, firmware identity, and running-partition facts use typed, closed production adapters with explicit availability. |
| `SYS-04` | **verified** | Reset reason, monotonic uptime, internal-heap free/minimum/largest-block values, and PSRAM capacity/free values are read through production platform adapters. |
| `SYS-05` | **verified** | Platform facts use an explicit available/unavailable tagged representation. Zero values and unknown reset codes do not authenticate unavailable evidence. |
| `OBS-06` | **verified** | The single production publisher orders revision allocation, completion, fallible retained-pair publication, and issuance under one mutex. Collection occurs before the lock; retention and issuance failures remain distinct; poison and reentrancy paths fail closed without collapsing revision identity. |
| `HLT-01` | **verified** | The core model exposes exactly the seven passive health states. Production currently reports unavailable where proof is absent and introduces no task-start or other effectful control surface. |
| `HLT-02` | **verified** | Recurring producer-owned checkpoints advance category, sequence, and monotonic time; snapshots derive freshness age from checkpoint history. |
| `HLT-03` | **verified** | Task-watchdog participation is modeled separately from producer liveness and remains explicitly unavailable when unproved. The snapshot path does not mutate task-watchdog state. |
| `HLT-04` | **verified** | Freshness derives healthy/stale/unhealthy states from cadence thresholds, and production tests show recurring checkpoint sequence advancement is independent of duplicate-yield log suppression. |

## Plan Must-Have Rollup

| Plan | Result | Verification conclusion |
| --- | --- | --- |
| `34-01` | **verified in declared scope** | Canonical build identity, runtime projections, schema-v3 package identity, and pre-effect dirty-build admission are present. The later `SYS-02` layout gap still applies. |
| `34-02` | **verified** | Typed boot-session, revision, and correlation semantics are present; completion-order publication was subsequently strengthened by `34-07`. |
| `34-03` | **verified** | Platform truth and explicit unavailable-state modeling are implemented through production adapters. |
| `34-04` | **verified** | Passive health vocabulary, freshness age, and watchdog separation are implemented, with the recurring-checkpoint correction supplied by `34-06`. |
| `34-05` | **verified** | Required package artifacts, OTA/factory structural validation, and application-byte binding are present. |
| `34-06` | **verified** | Recurring producer checkpoints advance independently of one-time or duplicate-suppressed logging. |
| `34-07` | **verified** | Completion-ordered publication, retained-pair semantics, and real issuance are implemented in the production publisher. |
| `34-08` | **verified in declared scope** | The basic executable envelope, closed factory selection, and immutable execution snapshot are present; these checks alone do not close the later canonical-layout requirements. |
| `34-09` | **verified** | Retained previous/current publication is atomic and fallible, with retention and issuance represented as distinct failure channels. |
| `34-10` | **gaps_found** | Entry/load-address and packaged-ELF contradiction gaps are closed, but canonical layout enforcement remains incomplete because descriptor-segment family and pairwise destination non-overlap are not enforced. |

## SYS-02 Functional Conformance

The current production chain is coherent from package construction through schema-v3 validation, active admission, and immutable execution snapshotting. The packaged ELF digest is bound to the application digest, and active admission rechecks the actual ELF before reading OTA or factory application bytes. The earlier zero or redirected entry/load-address gap and the contradictory packaged-ELF gap are closed.

The canonical package produced at the current commit is accepted and independently described as a valid ESP32-S3 image by the managed ESP-IDF tooling. That is necessary positive evidence, but it does not prove rejection coverage for every declared invariant.

Two functional invariants are absent from `esp32s3_image` admission:

1. The descriptor is decoded at offset zero of the first non-empty payload, but that descriptor-bearing segment is not required to belong to the DROM memory family.
2. Segment destinations are validated individually, but the complete segment set is not checked for pairwise overlap, including direct numeric overlap and D/IRAM physical alias overlap.

Because both invariants are part of the canonical ESP-IDF producer envelope required by Plan `34-10`, their absence blocks `SYS-02` and Phase 34 completion.

## Executable Evidence

| Evidence | Result |
| --- | --- |
| `just package` at `2cd1bd05b0af1cf435b3bc6857f8b99c9a072a08` | **passed**; produced the real schema-v3 firmware package. Existing firmware dead-code warnings were unchanged and non-blocking. |
| Managed ESP-IDF `esptool.py image_info` inspection of the canonical application image | **passed**; reported a valid six-segment ESP32-S3 image with valid checksum/hash and expected descriptor/MMU metadata. |
| `cargo test -p bitaxe-api` | **passed**, 208 tests. |
| `cargo test -p bitaxe-core` | **passed**, 21 tests. |
| `cargo test -p bitaxe-flash` | **passed**, 100 tests. |
| Focused `xtask` package-manifest tests | **passed**, 8 tests. |
| Focused Phase 34 source-guard tests | **passed**, 5 tests. |
| Focused operator-snapshot evidence tests | **passed**, 6 tests. |
| Focused Bazel production and tool targets | **passed** for retained-pair, supervisor-checkpoint, operator-snapshot publication, flash, parity, xtask, and API targets. |
| Canonical software-only flash dry-run using the generated package and `/dev/null` | **passed**; admission completed without execution. |
| `just verify-reference` | **passed**; pinned reference tree was clean. |

The committed `34-REVIEW.md` independently reaches the same `SYS-02` conclusion and records both missing invariants as blocking warnings. No source change after the reviewed implementation invalidates that review; current `HEAD` adds the review artifact only.

## Gaps Requiring Closure

1. Require the descriptor-bearing first non-empty application segment to be DROM while preserving the existing descriptor-at-offset-zero validation.
2. Reject pairwise direct destination overlap and D/IRAM physical alias overlap across decoded non-empty segments. Preserve half-open interval semantics so adjacent segments remain valid and zero-length segments contribute no range.
3. Add focused parser, full-admission, and pre-effect regression coverage for both invariants, then rerun the Phase 34 code review and lifecycle-bound verification.

## Human Verification

None required. Phase 34 remains software-only, and the outstanding gaps are deterministically verifiable through source inspection and executable tests.

## Non-Claims

- This result does not claim hardware, USB, flash, monitor, OTA execution, Wi-Fi, pool, mining, ASIC, thermal, fan, voltage, or watchdog-registration evidence.
- Acceptance of the clean canonical package is not treated as proof that malformed or noncanonical layouts are rejected.
- Administrative artifact completeness is not treated as requirement verification.
- Phase 35 is not authorized to proceed while `SYS-02` remains unsatisfied.

## Next Action

Implement the two narrow `SYS-02` layout checks and their regression coverage, then run a fresh code review and `gsd-verifier` pass for lifecycle `34-2026-07-15T03-26-15`. Until that evidence passes, keep Phase 34 at `gaps_found` and Phase 35 blocked.
