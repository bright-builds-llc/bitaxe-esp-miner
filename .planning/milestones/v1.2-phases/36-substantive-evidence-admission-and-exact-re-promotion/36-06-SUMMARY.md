---
phase: 36-substantive-evidence-admission-and-exact-re-promotion
plan: "06"
subsystem: evidence
tags: [hardware-attempt, ultra205, broker, runfiles, privacy, non-promotion]
requires:
  - phase: 36-05
    provides: exact-package broker, typed ledger, capture harness, and preflight capability
provides:
  - one closed exact-current-source hardware-mode invocation with no retry
  - immutable private pre-effect failure record and public root digest
  - deployed-runfiles repair for the Phase 36 effect adapter
  - truthful no-candidate handoff that blocks offline promotion
affects: [36-07, phase36-hardware-attempt, evidence-promotion]
tech-stack:
  added: []
  patterns:
    - execute hardware brokers through Bazel deployment targets with complete runfiles
    - preserve a pre-effect typed blocker without manufacturing device evidence
key-files:
  created:
    - .planning/phases/36-substantive-evidence-admission-and-exact-re-promotion/36-06-SUMMARY.md
  modified:
    - Justfile
    - scripts/phase36-substantive-evidence.sh
    - scripts/phase36-substantive-evidence-test.sh
key-decisions:
  - "Treat effect_adapter_unavailable as the authoritative pre-effect blocker and do not consume a second hardware command."
  - "Publish no candidate and make no parity claim because detector, device, session, and substantive facts were never observed."
  - "Route future Phase 36 supervisor invocations through the deployed Bazel target so the effect adapter is present in runfiles."
patterns-established:
  - "One-shot failure: repair the software boundary for future correctness, but never retry the consumed hardware command in the same plan."
  - "Pre-effect privacy: retain only protected command records in the immutable private root and disclose closed categories plus opaque digests."
requirements-completed: []
requirements-pending: [SYS-02, EVD-11, EVD-12, EVD-14]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 36-2026-07-23T15-20-53
generated_at: 2026-07-24T22:36:54Z
duration: 14min
completed: 2026-07-24
---

# Phase 36 Plan 06: Exact-Current Hardware Attempt Summary

**One exact-current hardware-mode invocation stopped at a deployed-adapter boundary before detector or device access, preserving an immutable non-promotional record and repairing the production runfiles path without retry**

## Performance

- **Duration:** 14 min
- **Started:** 2026-07-24T22:22:30Z
- **Completed:** 2026-07-24T22:36:54Z
- **Tasks:** 2
- **Tracked files modified:** 4
- **Hardware command invocations:** 1
- **Detector invocations:** 0
- **Device effects:** 0

## Accomplishments

- Rebuilt the exact `a899bbfa` Ultra 205 package and passed the complete software-only preflight, ordered Rust gate, full Just/Bazel suite, parity, reference, redaction, and identity-binding checks before hardware authority was consumed.
- Invoked the authorized Phase 36 hardware mode exactly once. It stopped before private-child creation with the closed broker category `phase36_hardware_private_attempt_failed` and discriminator `effect_adapter_unavailable`.
- Preserved the protected pre-effect record as an immutable mode-0700 root with mode-0600 files and public root digest `44c102722d9098bb17d67715cb41fd119500cd67c4129b1c03a2107a43f974c0`.
- Repaired the production command to execute the deployed Bazel supervisor with complete runfiles, preserved exact nested broker failure categories, and strengthened the real-process regression for deployed adapter availability.
- Kept Phase 35 byte-identical with root digest `0401e7b485df2d1ccfc67e63845f98b6217816a184901bf0595d03af3219757d`; Plan 36-04 and substantive canonical reconciliation content remained unchanged, while only normal `STATE.md` and `ROADMAP.md` execution tracking advanced.

## Task Commits

1. **Task 1: Freeze exact-current-source package and pass every no-hardware preflight gate** - no tracked commit; package and protected preflight artifacts are ignored local outputs.
2. **Task 2: Execute and seal one exact broker-owned Ultra 205 evidence attempt** - `68b43bcd` (fix)

## Files Created/Modified

- `Justfile` - runs the Phase 36 supervisor through its deployed Bazel target so the complete runfiles tree is available.
- `scripts/phase36-substantive-evidence.sh` - preserves a nested closed broker category instead of replacing it with a generic output error.
- `scripts/phase36-substantive-evidence-test.sh` - verifies the deployed effect adapter exists before the fake-detector transaction and repairs the previously unclosed shell conditional.
- `.planning/phases/36-substantive-evidence-admission-and-exact-re-promotion/36-06-SUMMARY.md` - records the one-shot pre-effect blocker and exact non-claims.

The planned commit-redacted candidate was not created because no detector, device, session, or substantive fact crossed the pre-effect boundary.

## Decisions Made

- The authoritative boundary signature is `phase36_hardware_private_attempt_failed` plus `effect_adapter_unavailable`.
- The single hardware-mode command was consumed even though the broker stopped before detector access. No second invocation, fresh ordinal, timing change, or post-fix retry was permitted.
- No offline seal or candidate was synthesized because the existing trusted classifier has no pre-effect sealing mode and manufacturing device facts would violate the evidence contract.
- Plan 36-07 cannot admit or publish evidence from this result because `36-GAP-HARDWARE-CANDIDATE.json` does not exist.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Deployed the effect adapter on the production command path**

- **Found during:** Task 2 hardware invocation
- **Issue:** The Just command built only the parity report and then ran the source-tree supervisor directly. The broker therefore could not resolve the deployed `phase36_hardware_effect` adapter and stopped before creating its child or invoking the detector.
- **Fix:** Routed the command through `//scripts:phase36_substantive_evidence`, whose Bazel runfiles include the report and effect adapter.
- **Files modified:** `Justfile`, `scripts/phase36-substantive-evidence-test.sh`
- **Verification:** The fresh deployed-runfiles process test reaches its fake detector, records exactly one fake detector invocation, creates a typed non-promotion seal, and proves zero later fake effects.
- **Committed in:** `68b43bcd`

**2. [Rule 1 - Bug] Preserved nested broker failure categories**

- **Found during:** Task 2 failure classification
- **Issue:** The supervisor parsed only lines beginning exactly with `category=`, while the Rust command emitted `Error: category=...`; the public result was incorrectly collapsed to `phase36_hardware_broker_output_invalid`.
- **Fix:** Added a bounded lower-snake-case category extractor that accepts the trusted error prefix without emitting free-form text.
- **Files modified:** `scripts/phase36-substantive-evidence.sh`
- **Verification:** Shell syntax/format checks and the complete process suite pass.
- **Committed in:** `68b43bcd`

**3. [Rule 1 - Bug] Repaired the real-process regression shell syntax**

- **Found during:** Post-failure deployed-runfiles verification
- **Issue:** The fake-detector invocation had a stray `if` without a matching `then`; cached Bazel results had masked the invalid current source.
- **Fix:** Restored the intended environment-scoped command invocation and added an explicit executable check for the deployed adapter.
- **Files modified:** `scripts/phase36-substantive-evidence-test.sh`
- **Verification:** `bash -n`, `shfmt -d`, and the fresh no-cache deployed process test pass.
- **Committed in:** `68b43bcd`

**Total deviations:** 3 auto-fixed bugs.

**Impact on plan:** The fixes close the discovered production deployment and classification defects for future attempts. They do not retroactively create a sealed broker child, detector fact, device fact, candidate, or promotion authority, and they did not authorize a retry.

## Issues Encountered

- The one authorized command stopped at `effect_adapter_unavailable` before detector admission. The protected root contains only the preflight handle and protected wrapper records; no supervisor-owned attempt child, broker ledger, seal, or candidate exists.
- Offline candidate inspection closed at `attempt_child_invalid`, which is expected because no candidate-bearing child was created.
- Cleanup was vacuous and complete: no detector, adapter, serial, network, flash, monitor, or device process started, and no related process remained.

## Authentication Gates

None.

## User Setup Required

None.

## Verification

- Exact package build passed for the attempt source and again for the repair commit.
- Ordered Rust gate passed: `cargo fmt --all`, Clippy with warnings denied, all-target/all-feature build, and all-feature tests.
- `bash -n` and `shfmt -d` passed for both changed shell scripts.
- Fresh no-cache `//scripts:phase36_substantive_evidence_test`, `//tools/parity:phase36_broker_tests`, and `//tools/parity:phase36_evidence_tests` passed after the repair.
- `just test` passed all 75 targets; `just parity`, `just verify-reference`, `just verify-redaction`, and `git diff --check` passed.
- Credential prerequisite was checked only with `test -s`; its contents, metadata, values, and digests were never read or emitted.
- Hardware command count is 1; detector count and device-effect count are both 0.
- The private root is immutable, has mode 0700 with mode-0600 files, and has digest `44c102722d9098bb17d67715cb41fd119500cd67c4129b1c03a2107a43f974c0`.
- The candidate is absent. Phase 35 bytes/root and Plan 36-04 are unchanged; only normal `STATE.md` and `ROADMAP.md` execution tracking advanced.

## Known Stubs

None. The missing candidate is the truthful result of the closed pre-effect blocker, not a placeholder.

## Next Phase Readiness

- Plan 36-07 has no eligible candidate to classify or publish and must remain blocked/no-promotion.
- Any future hardware attempt requires a new plan, fresh exact-current package and preflight, and explicit progress-gated authority. This plan authorizes no retry.
- The deployed-runfiles repair is fully software-verified for any separately authorized future attempt.

## Self-Check: PASSED

- Summary file exists and commit `68b43bcd` is present.
- The commit-redacted candidate is absent as required by the typed pre-effect blocker.
- Staged redaction and diff checks passed.
- The immutable private-root digest, Phase 35 root digest, and closed invocation counts were revalidated without exposing protected values.
