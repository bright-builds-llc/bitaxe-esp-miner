---
phase: 36-substantive-evidence-admission-and-exact-re-promotion
verified: 2026-07-24T18:06:15Z
status: gaps_found
score: "9/13 must-haves verified"
roadmap_score: "2/5 success criteria verified"
reviewed_commit: f1cb6101f2c384acaffe0b8523097433ff0f04cc
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 36-2026-07-23T15-20-53
generated_at: 2026-07-24T18:06:15Z
lifecycle_validated: true
overrides_applied: 0
gaps:
  - truth: "The authoritative admitted inventory contains substantive sensor and runtime-health facts joined to the exact operator snapshot."
    status: failed
    reason: "The current authoritative typed projection has null sensor_substance, snapshot_join, and runtime_health fields; the snapshot and runtime-health rows are therefore demoted rather than substantively supported."
    artifacts:
      - path: docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion/typed-fact-projection.json
        issue: "sensor_substance, snapshot_join, and runtime_health are null"
      - path: docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion/decision-matrix.json
        issue: "V12-OPERATOR-SNAPSHOT-205 and V12-RUNTIME-HEALTH-205 are demoted with typed insufficiency reasons"
    missing:
      - "Eligible immutable API, WebSocket, and retained-log companion documents containing actual sensor state/value/stamp/failure facts and runtime-health/checkpoint facts from one exact boot and snapshot revision"
      - "A later Phase 36 gap-closure plan before any new exact-current-source hardware acquisition is authorized"
  - truth: "The authoritative admitted inventory contains an independently observed same-device runtime identity correlated to the exact source, reference, ELF, and flashed package."
    status: failed
    reason: "The runtime_identity projection is null and V12-PACKAGE-IDENTITY-205 is demoted as runtime_identity_observation_insufficient."
    artifacts:
      - path: docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion/typed-fact-projection.json
        issue: "runtime_identity is null"
      - path: docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion/decision-matrix.json
        issue: "Exact package identity is not a supported row"
    missing:
      - "An eligible immutable device-session ledger or independently complete terminal result/projection pair joined to the exact package manifest"
      - "A later Phase 36 gap-closure plan before any new exact-current-source hardware acquisition is authorized"
  - truth: "The authoritative evidence proves no actuation through an independent bounded effect ledger or equally strong closed observation."
    status: failed
    reason: "The independent_effect projection is null. The validators correctly give supervisor-authored assertions zero authority, but the current generation contains no independent effect observation."
    artifacts:
      - path: docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion/typed-fact-projection.json
        issue: "independent_effect is null"
      - path: tools/parity/src/phase36_evidence/effects.rs
        issue: "The implementation can validate an independent ledger, but no eligible ledger is present in the authoritative generation"
    missing:
      - "An immutable, bounded, complete, independently owned effect ledger covering the admitted session"
      - "A later Phase 36 gap-closure plan before any new exact-current-source hardware acquisition is authorized"
  - truth: "Final project, requirement, Phase 34 Nyquist, and milestone-audit documents state the exact Phase 36 outcome."
    status: partial
    reason: "The checklist reflects the Phase 36 correction and the planning documents correctly avoid false completion, but Phase 34 validation, PROJECT.md, REQUIREMENTS.md, and the v1.2 milestone audit have not yet been reconciled to the authoritative Phase 36 preserve/demote result."
    artifacts:
      - path: .planning/phases/34-provenance-runtime-health-and-coherent-operator-snapshot/34-VALIDATION.md
        issue: "Five task rows remain pending"
      - path: .planning/REQUIREMENTS.md
        issue: "All five Phase 36 requirements remain generically Pending rather than recording the verified claim-specific outcomes"
      - path: .planning/v1.2-MILESTONE-AUDIT.md
        issue: "Still describes the pre-Phase-36 Phase 35 gaps rather than the authoritative successor result"
    missing:
      - "Reconcile the final documents without marking SYS-02, EVD-11, EVD-12, or EVD-14 complete"
      - "Record EVD-15 as satisfied only if the independent review and security gates also pass"
---

# Phase 36: Substantive Evidence Admission and Exact Re-Promotion Verification Report

**Phase Goal:** Every affected v1.2 promotion is supported by typed admission of the substantive sensor, runtime-health, runtime-identity, and no-actuation facts it claims, with fail-closed correction and exact re-evaluation of prior evidence.

**Verified commit:** `f1cb6101f2c384acaffe0b8523097433ff0f04cc`

**Status:** gaps_found

**Re-verification:** No — initial verification

## Verdict

Phase 36 implements a substantive, fail-closed admission and correction system, and its current public successor is internally consistent. It does not, however, contain the authoritative facts needed to close four of the five Phase 36 requirements.

The current successor supports only `V12-HOSTNAME-205`. It correctly demotes package identity, coherent operator snapshot, and runtime health. It also records deterministic non-promotion for every excluded scope. Those conservative corrections are valid, but task completion and passing mechanism tests do not turn absent authoritative evidence into goal achievement.

No later milestone phase exists to absorb these gaps. Any new hardware acquisition requires a separately authorized later Phase 36 gap-closure plan; this verification did not use hardware, USB, serial, networking, target discovery, or credentials.

## Goal Achievement

### Roadmap Success Criteria

| # | Roadmap truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Admission validates actual sensor truth, values, stamps, failure states, runtime health, checkpoint facts, and the sensor/snapshot boot join. | ✗ FAILED | The validators and mutation tests exist, but the authoritative projection has `sensor_substance: null`, `snapshot_join: null`, and `runtime_health: null`; Level 4 data flow is hollow. |
| 2 | The admitted inventory contains a runtime-derived same-device identity observation correlated to the exact source, reference, ELF, and flashed package. | ✗ FAILED | `runtime_identity` is null and `V12-PACKAGE-IDENTITY-205` is demoted as `runtime_identity_observation_insufficient`. |
| 3 | No-actuation proof comes from an independent bounded effect ledger or equally strong observation. | ✗ FAILED | `independent_effect` is null. The implementation rejects supervisor self-attestation, but no eligible independent observation flows into the authoritative generation. |
| 4 | Claim-specific facts control atomic correction and Attempt 31 is re-evaluated without private-evidence rewrite or implicit hardware escalation. | ✓ VERIFIED | The matrix preserves hostname, demotes exactly three unsupported rows, excludes all broader scopes, fingerprints Phase 35, and publishes atomically. Phase 35 evidence is unchanged from HEAD. The offline path has no hardware broker. |
| 5 | Phase 34/project/requirements/audit reconciliation occurs only after substantive closure passes fresh verification. | ✓ VERIFIED | The documents have not been falsely promoted or marked complete before this verification. Their final reconciliation remains a downstream gap because substantive closure did not pass. |

**Roadmap score:** 2/5 success criteria verified

### Merged Observable Truths

ROADMAP success criteria were merged with non-duplicative PLAN frontmatter truths. PLAN truths that merely restate the five roadmap criteria were not counted twice.

| # | Observable truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Actual sensor/runtime-health substance and an exact snapshot join flow into the authoritative admission. | ✗ FAILED | Three null projection fields and two typed demotions. |
| 2 | Independently observed same-device runtime identity flows into the authoritative admission. | ✗ FAILED | Null runtime identity and typed identity demotion. |
| 3 | Independent no-actuation observation flows into the authoritative admission. | ✗ FAILED | Null independent effect observation. |
| 4 | Claim-specific correction is atomic, read-only with respect to Attempt 31, and does not authorize hardware. | ✓ VERIFIED | Promotion, offline, publication, and rollback tests pass. |
| 5 | Administrative truth is gated on substantive fresh verification. | ✓ VERIFIED | Requirements remain pending and the milestone remains open. |
| 6 | A successor schema fingerprints without rewriting or reinterpreting the Phase 35 v1 root. | ✓ VERIFIED | Phase 35 root/generation digests are bound in the successor; the Phase 35 evidence tree has no HEAD diff. |
| 7 | Missing, contradictory, self-attested, and privacy-unsafe inputs have named fail-closed fixtures. | ✓ VERIFIED | Phase 36 evidence suite and explicit mutation catalog pass fresh. |
| 8 | Synthetic protected-root process tests prove protected inputs cannot reach shareable output. | ✓ VERIFIED | `//scripts:phase36_evidence_test` passes fresh. |
| 9 | Aggregate immutable insufficiency preserves component-specific insufficiency categories. | ✓ VERIFIED | Contract tests pass, and the offline reducer independently retains snapshot, health, identity, and effect categories. |
| 10 | The shareable successor contains a versioned typed projection from which its current decisions can be reconstructed. | ✓ VERIFIED | Projection schema is complete, redaction-safe, hash-bound, and expresses absent current facts as null rather than invented digests. |
| 11 | Every affected row has explicit prerequisites and an exact claim-fact digest. | ✓ VERIFIED | Four affected-row decisions each carry a distinct lowercase digest; the evaluator rejects wrong-claim reuse. |
| 12 | Final documents state the exact supported, corrected, and insufficient outcomes. | ✗ FAILED | The checklist is exact, but Phase 34 validation, PROJECT, REQUIREMENTS, and milestone audit still require Phase 36 reconciliation. |
| 13 | Administrative completion cannot override typed insufficiency or a failed verifier. | ✓ VERIFIED | Current formal status remains in progress/pending, and this report records `gaps_found`. |

**Score:** 9/13 must-haves verified

## Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `tools/parity/src/phase36_evidence.rs` and modules | Closed successor admission contract | ✓ VERIFIED | Substantive parsers, protected-root authority, closed insufficiency types, evaluator identity closure, and fail-closed reducers are implemented and tested. |
| `tools/parity/fixtures/phase36/` | Synthetic eligible and mutation fixtures | ✓ VERIFIED | Used by the fresh evidence and process suites. |
| `scripts/phase36-evidence-test.sh` | Protected-input/redaction/no-hardware process harness | ✓ VERIFIED | Fresh Bazel target passes. |
| `tools/parity/src/phase36_evidence/substance.rs` | Sensor/runtime-health facts and exact provenance join | ✓ VERIFIED as mechanism | Substantive and negative tests pass; the authoritative current data source is absent. |
| `tools/parity/src/phase36_evidence/runtime_identity.rs` | Device-session replay and exact-package join | ✓ VERIFIED as mechanism | Runtime ledger replay and terminal-pair validation exist; the authoritative current data source is absent. |
| `tools/parity/src/phase36_evidence/effects.rs` | Independent effect completeness reducer | ✓ VERIFIED as mechanism | Complete/missing/prohibited effects are covered; the authoritative current data source is absent. |
| `tools/parity/src/phase36_promotion.rs` and modules | Claim-specific matrix and exact non-claim preservation | ✓ VERIFIED | Zero-through-four promotion tests, digest checks, and unrelated-row preservation pass. |
| `tools/parity/src/operator_evidence/generation/phase36.rs` and transaction module | Atomic successor/checklist publication and rollback | ✓ VERIFIED | Fresh generation suite exercises failure boundaries and crash recovery. |
| `docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion/` | Authoritative shareable successor | ⚠ HOLLOW FOR FOUR REQUIREMENTS | Hash-consistent five-file generation; only hostname facts are populated. The generated `checklist.md` snapshot is an implementation deviation from the PLAN's four-document wording, used for crash recovery and covered by tests. |
| `docs/parity/checklist.md` | Exact current supported/corrected rows and preserved non-claims | ✓ VERIFIED | Byte-identical to the generation snapshot; parity validation reports no errors. |
| `.planning/v1.2-MILESTONE-AUDIT.md` | Reconciled milestone outcome | ✗ STALE | Still records the Phase 35 audit rather than the Phase 36 successor outcome. |

## Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| Phase 36 evidence envelope | Immutable Phase 35 root | Root/generation digest and protected role graph | ✓ WIRED | Successor binds root `0401e7…` and superseded manifest digest `cb76a7…`; Phase 35 tree is unchanged. |
| `substance.rs` | Operator snapshot validator | Exact API/WebSocket/retained revision and boot join | ✓ WIRED | Capability and negative tests pass; current authoritative inputs do not populate it. |
| `runtime_identity.rs` | Device-session production model | Ordered `SessionState::apply` replay and terminal projection comparison | ✓ WIRED | `tools/device-session/src/model.rs` is included in the Phase 36 evidence evaluator identity. |
| `effects.rs` | Independent effect observation | Closed ordered eight-effect ledger | ✓ WIRED | Supervisor Boolean has no authority; no current ledger is present. |
| Claim prerequisites | Promotion matrix | Claim-specific digests and missing-reason reducer | ✓ WIRED | Three missing facts become three exact demotions; hostname is preserved. |
| Publisher | Successor root and checklist | Validated staging, filesystem sync, atomic exchange, rollback/recovery | ✓ WIRED | Fresh publication tests pass and canonical checklist equals the generation snapshot. |
| Current authoritative facts | Four requirement-bearing claims | Typed projection | ✗ HOLLOW | Sensor, join, health, identity, and independent-effect fields are all null. |

## Data-Flow Trace (Level 4)

| Artifact / claim | Data variable | Source | Produces real authoritative data | Status |
| --- | --- | --- | --- | --- |
| Hostname durability | `hostname_durability` | Authenticated Phase 35 public generation | Yes | ✓ FLOWING |
| Sensor substance | `sensor_substance` | Optional immutable API/WebSocket/retained companions | No current eligible companions | ✗ DISCONNECTED |
| Snapshot join | `snapshot_join` | Same companion set and exact boot/revision join | No current eligible companions | ✗ DISCONNECTED |
| Runtime health | `runtime_health` | Same companion set with checkpoint/lifecycle facts | No current eligible companions | ✗ DISCONNECTED |
| Runtime identity | `runtime_identity` | Optional exact package plus device-session evidence | No current eligible session evidence | ✗ DISCONNECTED |
| Independent no-actuation | `independent_effect` | Optional independently owned bounded ledger | No current eligible ledger | ✗ DISCONNECTED |
| Promotion decisions | `scope_decisions` | Claim-specific optional validated facts | Yes; missing facts deterministically demote | ✓ FLOWING |

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Fresh Phase 36 evidence, promotion, generation, and protected-process behavior | `bazel test --nocache_test_results //tools/parity:phase36_evidence_tests //tools/parity:phase36_promotion_tests //tools/parity:operator_evidence_generation_tests //scripts:phase36_evidence_test` | 4/4 targets passed; all executed fresh | ✓ PASS |
| Canonical parity checklist validation | `just parity` | `validation_errors: none`; exact Phase 36 preserve/demote rows rendered | ✓ PASS |
| Phase lifecycle provenance | `gsd-tools.cjs verify lifecycle 36 --require-plans --raw` | `valid` | ✓ PASS |
| Successor document integrity | SHA-256 comparison against `manifest.json` and checklist `cmp` | All projection/matrix/verdict/checklist hashes match; checklist snapshots identical | ✓ PASS |
| Phase 35 immutability | `git diff --quiet HEAD -- docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion` | No diff | ✓ PASS |

## Requirements Coverage

| Requirement | Source plans | Status | Evidence |
| --- | --- | --- | --- |
| SYS-02 | 36-02, 36-03, 36-04 | ✗ BLOCKED | The package parser is implemented, but `runtime_identity` is null and package identity is demoted. The running firmware is not independently correlated in the authoritative admission. |
| EVD-11 | 36-01 through 36-04 | ✗ BLOCKED | Exact source/reference/package structure exists, but the independently observed runtime identity required by the requirement is absent. |
| EVD-12 | 36-01 through 36-04 | ✗ BLOCKED | Sensor substance and exact snapshot join are null; the snapshot row is demoted. |
| EVD-14 | 36-01 through 36-04 | ✗ BLOCKED | Inventory, redaction, cleanup mechanics, reference fingerprints, and atomic publication are implemented, but independent no-actuation evidence is absent. |
| EVD-15 | 36-02 through 36-04 | ✓ SATISFIED | Only hostname is supported; identity, snapshot, and health are deterministically demoted; all eleven excluded scopes are explicit `do_not_promote` decisions; unrelated rows and non-claims are preserved. |

No Phase 36 requirement is orphaned. All five are declared in Phase 36 plans and mapped to Phase 36 in `REQUIREMENTS.md`.

## Anti-Patterns Found

| File | Pattern | Severity | Impact |
| --- | --- | --- | --- |
| Phase 36 source set | TODO/FIXME/placeholder/empty implementation scan | ℹ️ None | No substantive stub marker was found. The `mktemp` match in the process harness is legitimate test setup. |
| Public Phase 36 generation | PLAN described four public documents; implementation owns a fifth `checklist.md` snapshot | ⚠️ Warning | The snapshot enables authoritative crash recovery and is hash-bound/redaction-safe, but the formal artifact description should be updated or explicitly accepted. It does not close the missing evidence facts. |

## Human Verification Required

None for this verdict. The goal gaps are directly established by the authoritative null fields and typed demotions. Hardware acquisition is not a human spot-check for this report and remains prohibited until a later Phase 36 gap-closure plan explicitly authorizes it.

## Gaps Summary

The code can validate the required sensor, health, runtime-identity, and effect observations, and it fails closed when those observations are missing. The current authoritative generation proves that they are missing:

- `sensor_substance`, `snapshot_join`, and `runtime_health` are null;
- `runtime_identity` is null;
- `independent_effect` is null;
- only `V12-HOSTNAME-205` remains supported;
- package identity, operator snapshot, and runtime health are demoted with exact typed reasons.

Therefore SYS-02, EVD-11, EVD-12, and EVD-14 remain blocked. EVD-15 is satisfied by the precise correction and preserved non-claims. The planning and milestone documents may now be reconciled to this `gaps_found` result, but they must not administratively promote the four blocked requirements.

_Verified: 2026-07-24T18:06:15Z_

_Verifier: the agent (gsd-verifier)_
