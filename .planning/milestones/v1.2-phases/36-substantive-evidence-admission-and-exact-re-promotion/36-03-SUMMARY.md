---
phase: 36-substantive-evidence-admission-and-exact-re-promotion
plan: "03"
subsystem: evidence-promotion
tags: [rust, typed-evidence, atomic-publication, provenance, fail-closed]
requires:
  - phase: 36-02
    provides: Typed sensor, health, runtime-identity, and independent-effect admission
  - phase: 35
    provides: Immutable admitted evidence root and historical generation
provides:
  - Claim-specific Phase 36 promotion matrix with exact fact digests
  - Atomic successor publication with rollback at every injected boundary
  - Offline Attempt 31 re-evaluation with typed insufficiency and no hardware fallback
  - Conservative public correction preserving only the independently valid hostname claim
affects: [36-04, parity-checklist, evidence-policy]
tech-stack:
  added: []
  patterns:
    - Closed per-claim promotion decisions instead of aggregate eligibility
    - Immutable predecessor fingerprint linked to one authoritative successor
    - Explicit-path offline classification followed by atomic public correction
key-files:
  created:
    - tools/parity/src/phase36_promotion.rs
    - tools/parity/src/operator_evidence/generation/phase36.rs
    - tools/parity/src/operator_evidence/generation/phase36/transaction.rs
    - tools/parity/src/phase36_offline.rs
    - docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion/manifest.json
  modified:
    - tools/parity/src/phase36_promotion/evaluator.rs
    - tools/parity/src/phase36_promotion/types.rs
    - docs/parity/checklist.md
    - scripts/phase35-promotion-contract-test.sh
    - scripts/phase36-evidence-test.sh
key-decisions:
  - "A Phase 35 hostname carry-forward requires the immutable manifest, projection, admitted verdict, and exact hostname promotion in the historical decision matrix."
  - "Missing Attempt 31 effect evidence corrects the three Attempt 31-dependent claims but cannot erase the independently admitted Phase 35 hostname decision."
  - "Canonical requirement completion remains unchanged for Plan 04 independent verification."
patterns-established:
  - "Successor chain: immutable predecessor digest plus predecessor checklist fingerprint -> typed decision matrix -> atomic successor generation and checklist."
  - "Conservative reevaluation: absent protected companions produce exact component insufficiencies, one aggregate insufficiency, and no fallback action."
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 36-2026-07-23T15-20-53
generated_at: 2026-07-24T14:55:00Z
duration: 24min
completed: 2026-07-24
---

# Phase 36 Plan 03: Claim-Specific Correction and Atomic Re-Promotion Summary

**Claim-specific evidence decisions now publish through an atomic successor chain, preserving the valid hostname claim while correcting three unsupported Attempt 31 claims without hardware fallback.**

## Performance

- **Duration:** 24 minutes
- **Started:** 2026-07-24T14:31:23Z
- **Completed:** 2026-07-24T14:55:00Z
- **Tasks:** 3
- **Files changed:** 29

## Accomplishments

- Added closed `Promote`, `Demote`, `Preserve`, and `DoNotPromote` decisions whose positive outcomes bind exact lowercase claim-fact digests and whose checklist projection preserves every unrelated row.
- Added a four-document Phase 36 publisher that validates the complete generation, synchronizes staged bytes, atomically exchanges generation and checklist state, and restores both outputs byte-identically at every injected failure boundary.
- Added an explicit-path offline Attempt 31 command that snapshots only caller-supplied protected companions, runs pure classifiers, verifies source bytes remain unchanged, and has no detector, target discovery, credential, network, flash, monitor, serial, reboot, or hardware path.
- Published `immutable_artifacts_insufficient` with all four exact component categories, preserved `V12-HOSTNAME-205`, and corrected package identity, operator snapshot, and runtime health to `implemented`.
- Proved the public successor starts from the exact Phase 35 checklist fingerprint and references byte-identical Phase 35 generation artifacts and root digest.

## Task Commits

1. **Task 1: Implement the claim-specific decision matrix and exact checklist projection** - `167f34a4`
2. **Task 2: Add atomic successor generation and checklist publication** - `9a79a08c`
3. **Task 3: Re-evaluate immutable Attempt 31 and publish the exact correction** - `7ff72f4e`

## Files Created and Modified

- `tools/parity/src/phase36_promotion/` - Closed claim scopes, prerequisite validation, fact-digest binding, checklist projection, and historical-generation resolver.
- `tools/parity/src/operator_evidence/generation/phase36.rs` and `phase36/transaction.rs` - Validated staging, filesystem synchronization, atomic exchange, rollback, and failure injection.
- `tools/parity/src/phase36_offline.rs` - Explicit-path protected companion snapshots, pure classification, immutable-byte verification, and terminal public correction.
- `tools/parity/src/phase36_evidence/substance.rs` - Component-specific sensor and health admission used by the offline reevaluator.
- `scripts/phase36-evidence-test.sh` - Real-process sufficient and per-component insufficient cases plus effectful-token denylist.
- `scripts/phase35-promotion-contract-test.sh` - Immutable Phase 35 to authoritative Phase 36 successor-chain validation.
- `docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion/` - Shareable typed projection, decision matrix, verdict, and manifest.
- `docs/parity/checklist.md` - Exact four-row Phase 36 projection; every unrelated row and non-claim remains byte-identical.

## Decisions Made

- The historical hostname claim is reconstructed only from the complete public Phase 35 generation, including the matrix entry that proves `V12-HOSTNAME-205` was promoted under the same root.
- Phase 35 hostname durability already owns its admitted storage, reload, reboot, restoration, and cleanup evidence. Missing Attempt 31 independent-effect material therefore conditions only the three new Attempt 31-dependent claims.
- The absence of safely supplied protected companions is a terminal classification result: publish exact insufficiency and stop without searching for inputs or brokering later hardware.
- Plan 04 retains canonical requirement reconciliation after fresh independent verification.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Declared immutable public evidence as Bazel inputs**

- **Found during:** Task 3 focused Bazel verification
- **Issue:** Hermetic Rust and real-process tests used Phase 35 manifest, verdict, projection, and matrix files that were not all declared in `compile_data` or test data.
- **Fix:** Added the exact public files to the relevant Bazel targets and root exports.
- **Verification:** All four focused Bazel targets and the 73-target `just test` suite pass.
- **Committed in:** `7ff72f4e`

**2. [Rule 1 - Bug] Required the historical matrix before preserving hostname**

- **Found during:** Task 3 source review
- **Issue:** The public-generation loader validated the Phase 35 projection and verdict but did not prove that the historical matrix actually promoted the hostname row.
- **Fix:** Digest-validated the matrix and required exactly one matching historical promotion for `V12-HOSTNAME-205`; added a semantic-tamper regression.
- **Verification:** The offline unit suite rejects a self-consistent generation whose hostname decision is changed.
- **Committed in:** `7ff72f4e`

**3. [Rule 1 - Bug] Prevented Attempt 31 effect absence from erasing Phase 35 hostname evidence**

- **Found during:** First real offline publication
- **Issue:** One shared independent-effect prerequisite demoted all four rows, contradicting the task's requirement to preserve hostname when its existing admitted facts remain valid.
- **Fix:** Kept the historical hostname decision bound only to its independently admitted durability facts while applying Attempt 31 effect insufficiency to the other three claims.
- **Verification:** Focused evaluator tests and the published verdict support only `V12-HOSTNAME-205`.
- **Committed in:** `7ff72f4e`

**4. [Rule 1 - Bug] Made promotion tests independent of the current projected checklist**

- **Found during:** Task 3 focused Rust verification after public correction
- **Issue:** Evaluator tests treated the repository's mutable current checklist as a fixed all-verified fixture, so the publisher's own correction invalidated their setup.
- **Fix:** Constructed explicit all-verified test input for decision-count and one-row-change tests.
- **Verification:** All seven focused promotion tests and 371 parity crate tests pass.
- **Committed in:** `7ff72f4e`

**5. [Rule 3 - Blocking] Updated the historical contract for an authoritative successor**

- **Found during:** Full `just test`
- **Issue:** The Phase 35 contract required its historical four-row projection to remain the current checklist forever and could not admit a valid successor.
- **Fix:** Preserved every Phase 35 artifact check, then added exact predecessor digest, before/after checklist fingerprint, successor file digest, hostname preservation, and three-demotion checks.
- **Verification:** `phase35_promotion_contract_test` and the full 73-target Bazel suite pass.
- **Committed in:** `7ff72f4e`

**Total deviations:** 5 auto-fixed (3 Rule 1 bugs, 2 Rule 3 blocking issues).

**Impact on plan:** The fixes close provenance and integration gaps required for the planned conservative successor; they add no hardware, credential, network, or broader parity scope.

## Issues Encountered

- The first local publication exposed the shared-effect over-demotion. The exact Phase 35 checklist bytes were restored, the corrected generation was republished once, and the final manifest proves its `checklist_fingerprint_before` equals the immutable Phase 35 manifest's checklist digest.
- The initial full Bazel run had one successor-awareness failure; after the contract fix, the rerun passed all 73 tests.

## Verification

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- Focused Phase 36 Rust and four-target Bazel suites
- `just test` — 73/73 tests passed
- `just parity` — `validation_errors: none`
- `just verify-reference` — pinned reference clean
- `just verify-redaction` — passed
- Phase 35 manifest, verdict, matrix, and projection match commit `9a79a08c` byte-for-byte
- Checklist diff contains exactly the four authorized V12 rows
- `git diff --check`

## Known Stubs

None.

## Authentication Gates

None.

## User Setup Required

None.

## Next Phase Readiness

- Plan 04 can independently verify the successor generation, conservative checklist projection, full regression evidence, and canonical requirement disposition.
- No hardware run or protected evidence discovery occurred; future evidence collection remains separately gated.

## Self-Check

PASSED.

- All key implementation and public-generation files exist.
- Task commits `167f34a4`, `9a79a08c`, and `7ff72f4e` resolve.
- Phase 36 lifecycle validation is valid with lifecycle ID `36-2026-07-23T15-20-53` and mode `yolo`.

***

*Phase: 36-substantive-evidence-admission-and-exact-re-promotion*
*Completed: 2026-07-24*
