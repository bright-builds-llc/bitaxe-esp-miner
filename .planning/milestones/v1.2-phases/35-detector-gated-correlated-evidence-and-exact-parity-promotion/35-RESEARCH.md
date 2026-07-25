---
phase: 35
slug: detector-gated-correlated-evidence-and-exact-parity-promotion
status: complete
researched: 2026-07-17
phase_lifecycle_id: 35-2026-07-17T17-00-37
lifecycle_mode: yolo
---

# Phase 35 Research: Detector-Gated Correlated Evidence and Exact Parity Promotion

## Research Question

What must the implementation plans establish so one bounded Ultra 205 run can admit only the exact eligible v1.2 operator-runtime claims while every broader or excluded claim remains a deterministic non-promotion?

## Recommended Architecture

Use the established functional-core/imperative-shell split:

1. A thin Phase 35 shell supervisor owns detector invocation, protected local inputs, flash/monitor subprocesses, the approved normal reboot, bounded timeouts, cleanup, restoration, and raw artifact permissions.
2. A typed Rust admission core parses one content-addressed evidence root, validates two internally coherent boot epochs, enforces provenance and chronology, derives a redacted projection, and produces a closed promotion decision matrix.
3. Promotion is a staged generation: validate all evidence, redaction, inventory, current-head, reference-cleanliness, lifecycle, cleanup, no-actuation, and exclusion decisions before atomically publishing the admitted evidence/checklist projection.

This reuses Phase 33 hardware mechanics and Phase 34 snapshot validation rather than introducing a new hardware transport stack.

## Existing Assets to Reuse

| Asset | Reuse boundary |
| --- | --- |
| `scripts/detect-ultra205.sh` | Sole detector preflight. Require exactly one likely board-205 port and successful non-interactive `board-info` before any target, credential, flash, reset, monitor, HTTP, or promotion action. |
| `scripts/phase33-confirmed-settings-durability.sh` | Reuse or extract detector/package gates, protected workspace permissions, passive monitor contract, process/holder cleanup, approved normal restart, restoration, and failure-injection patterns. Do not duplicate its hardware semantics in Rust. |
| `tools/parity/src/operator_evidence/generation.rs` | Reuse staged generation, validation-before-exchange, and explicit inventory patterns for atomic admission. |
| `tools/parity/src/v12_admission.rs` | Extend the typed admission vocabulary and exclusion reasons; keep lifecycle completion distinct from parity evidence. |
| Phase 34 runtime-health and operator-snapshot modules | Validate each boot epoch independently. Do not weaken their same-session coherence checks to make a cross-reboot bundle pass. |
| Phase 30 no-promotion artifact and parity checklist guards | Preserve STR-09, CFG-07, ASIC-11 and the archived Phase 28.1.1 lineage as authoritative non-promotions. |

## Evidence Model

### One Protected Root, Two Epochs

The approved reboot necessarily changes the boot session. The evidence root therefore contains two internally coherent epoch bundles:

- Boot A: pre-PATCH projection, PATCH response, storage-confirmed immediate readback, and matching coherent system-info/WebSocket/retained-log snapshot.
- Boot B: exactly one approved normal reboot, same-board reacquisition, storage-confirmed hostname persistence, and a second coherent system-info/WebSocket/retained-log snapshot.

The typed join must bind both epochs with:

- one run identifier;
- exact source commit and current-head proof;
- exact pinned reference commit and reference-cleanliness proof;
- package manifest path and digest;
- board category `205`;
- target-lock provenance;
- stable physical-identity digest without exporting the raw identifier;
- boot ordinals `N` and `N+1`;
- approved reset cause/category;
- strictly ordered monotonic chronology;
- response-before-effect ordering;
- storage-confirmed revision/value correlation;
- restoration and cleanup completion.

Boot-local snapshot validators should continue to reject mixed-session evidence. Cross-epoch correlation belongs only in the Phase 35 admission model.

### Content-Addressed Inventory

Create the local evidence root under a mode-0700 gitignored parent and raw files as mode-0600. Each artifact gets a digest and declared role. The manifest must reject:

- missing or extra files;
- duplicate logical roles;
- digest mismatches;
- absolute or traversal paths;
- symlinks;
- changed source/reference/package identity;
- incomplete cleanup or restoration;
- unrecognized schema versions.

A failed root is sealed as a non-promotion. Retries create a new root; never mutate, splice, or reuse a failed root.

### Redacted Shareable Projection

Raw device paths, USB identity, PIDs, origins, targets, SSIDs, IP/MAC values, endpoints, pool inputs, hostname values, credentials, NVS secrets, and tokens stay in the protected root. The shareable projection may contain only categories, booleans, counts, durations, ordinals, approved reason enums, and cryptographic digests needed for correlation.

Redaction is a typed allowlist, not a best-effort scrub. Admission must scan the staged projection for forbidden key names and raw-value canaries before publication.

## Capture Orchestration

The shell supervisor should implement explicit gates:

1. Current-head/package gate: require clean or explicitly permitted source state, exact current commit, pinned clean reference, package manifest and digest, and protected evidence root.
2. Detector/capability gate: run `just detect-ultra205`; require exactly one board-205 candidate and successful board-info.
3. Effect/capture gate: only after gates 1 and 2, load ignored credential paths without reading or printing contents, flash the exact package, capture Boot A, perform the one approved normal reboot, reacquire the same board, capture Boot B, restore the original setting, clean up all serial/process holders, and close the root.

Use at least 360 seconds for each Ultra 205 flash/monitor capture and at least 420 seconds of shell/tool wall clock. The retained-runtime monitor command must use the full passive contract:

`--chip esp32s3 --before no-reset-no-sync --after no-reset --no-reset --non-interactive`

Every exit path after mutation must attempt restoration and then prove restoration and cleanup. Restoration failure, unexpected reset, identity drift, missing bytes, multiple targets, malformed/stale device URL, or holder leakage results in a sealed non-promotion.

## Exact Promotion Model

Use a closed typed decision matrix whose exhaustive output is either:

- `Promote { row_id, evidence_digest }`, or
- `DoNotPromote { scope, reason }`.

Dedicated narrow v1.2 row IDs are preferable when an existing checklist row covers broader semantics than the evidence proves. The allowlist should describe only passive operator-runtime facts directly supported by the eligible root, including the exact CFG-12/EVD-13 hostname durability correlation and the proven Phase 34 snapshot/health projections.

The matrix must explicitly preserve non-promotion for:

- active control or actuation;
- self-test effects;
- independent watchdog intervention;
- mining, Stratum, ASIC work, and Phase 28.1.1 descendants;
- credentials and credential-management claims;
- direct UART, pins, pads, GPIO, test points, probes, jumpers, solder, or injected signals;
- OTA;
- any board other than Ultra 205;
- all broader or unmapped checklist rows;
- lifecycle, test, or administrative completion treated as evidence.

Completeness tests must fail when a new row or exclusion scope lacks a decision. All non-allowlisted existing checklist rows must remain byte-for-byte unchanged by the staged generation.

## Failure and Recovery Rules

- Detector failure: stop before target or credential access; no hardware action.
- Capture failure before mutation: clean process/serial state and seal non-promotion.
- Failure after mutation: restore original setting, prove readback, clean process/serial state, then seal non-promotion.
- Approved reboot failure or same-board reacquisition failure: do not retry inside the root; seal and start a newly planned root.
- Redaction, inventory, provenance, current-head, reference-cleanliness, no-actuation, lifecycle, or cleanup failure: do not publish or update parity.
- Atomic exchange failure: retain the previous admitted generation; staged output remains non-authoritative.

## Planning Decomposition

Plan the work in dependency order:

1. Typed evidence schema, two-epoch correlation evaluator, exhaustive exclusion vocabulary, and fixtures.
2. Thin detector-gated shell supervisor plus simulation/failure-injection coverage.
3. Atomic admission/promotion matrix, dedicated narrow checklist rows, and no-promotion invariants.
4. Final software verification followed by one detector-gated hardware run only if all preconditions pass.

The hardware plan must document the exact recovery path and redaction boundary before it may run.

## Validation Architecture

### Test Layers

| Layer | Purpose | Command family |
| --- | --- | --- |
| Rust unit/fixture tests | Evidence schema, chronology, two-epoch join, identity digest, exhaustive decisions, redaction allowlist, and fail-closed mutations | `cargo test --all-features` plus scoped Bazel parity tests |
| Shell simulation tests | Detector short-circuiting, package lock, permissions, timeout arguments, restart ordering, restoration, process/holder cleanup, and failure sealing | Existing shell-test harness through scoped Bazel targets |
| Source-policy tests | No-effects boundary, forbidden direct-UART/pin paths, secret/raw identifier leakage, exact passive-monitor arguments | Repo-owned source guard targets |
| Integration generation tests | Complete staged generation, inventory, digest validation, atomic exchange, unchanged non-allowlisted rows | Scoped `tools/parity` Bazel tests and `just parity` |
| Hardware evidence | One final exact-current-package Ultra 205 run after detector success | Phase 35 repo-owned evidence command with `capture-timeout-seconds=360` or higher |

### Requirement Coverage

| Requirement | Required proof |
| --- | --- |
| CFG-12 | Original value captured locally, committed PATCH response, storage-confirmed immediate readback, exactly one approved normal reboot, same-board reacquisition, matching storage-confirmed post-reboot value, restoration proof. |
| EVD-10 | Detector and board-info gate precedes every target, credential, flash, reset, monitor, capture, and promotion action. |
| EVD-11 | One protected root binds source/reference/package/board/target-lock/run/epoch provenance and complete inventory digests. |
| EVD-12 | Each boot epoch independently passes coherent system-info/WebSocket/retained-log revision validation and the cross-epoch join is typed. |
| EVD-13 | CFG-12 durability chain is correlated to the exact coherent operator snapshot revisions in both epochs. |
| EVD-14 | Inventory, redaction, cleanup, no-actuation, reference-cleanliness, current-head, restoration, and lifecycle gates all pass before publication. |
| EVD-15 | Exhaustive closed promotion matrix admits only dedicated allowlisted rows and deterministically records every excluded/broad scope as non-promotion. |

### Mandatory Negative Fixtures

Include fixtures for zero/multiple detector candidates, board-info failure, wrong board, source drift, reference drift, package digest mismatch, missing/extra inventory files, symlinks/path traversal, chronology inversion, duplicate checkpoints, mixed boot sessions, skipped/additional reboot, identity drift, missing storage confirmation, value mismatch, restoration failure, leaked holder, forbidden raw field, lifecycle-only proof, actuation evidence, and an unclassified checklist row.

### Sampling Contract

- After each task: run the narrowest affected Bazel/Rust/shell test target.
- After each plan: run all Phase 35 scoped tests plus `just parity`.
- Before final verification/commit: run the repository Rust pre-commit gate in order and the phase lifecycle verifier.
- Hardware evidence is never a substitute for failing software tests and software completion is never parity evidence.

## Sources

- `.planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-CONTEXT.md`
- `.planning/ROADMAP.md`
- `.planning/REQUIREMENTS.md`
- `.planning/STATE.md`
- `.planning/phases/31-operator-claim-and-telemetry-contract/31-CONTEXT.md`
- `.planning/phases/33-confirmed-settings-durability/33-CONTEXT.md`
- `.planning/phases/34-provenance-runtime-health-and-coherent-operator-snapshot/34-CONTEXT.md`
- `.planning/phases/34-provenance-runtime-health-and-coherent-operator-snapshot/34-VERIFICATION.md`
- `.planning/phases/34-provenance-runtime-health-and-coherent-operator-snapshot/34-SECURITY.md`
- `scripts/detect-ultra205.sh`
- `scripts/phase33-confirmed-settings-durability.sh`
- `tools/parity/src/v12_admission.rs`
- `tools/parity/src/operator_evidence/generation.rs`
- `AGENTS.md`
- `AGENTS.bright-builds.md`
- `standards/core/architecture.md`
- `standards/core/testing.md`
- `standards/core/verification.md`
- `standards/languages/rust.md`
