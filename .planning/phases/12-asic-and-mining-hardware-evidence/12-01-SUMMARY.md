---
phase: 12
plan: 01
subsystem: parity-evidence
tags: [asic, mining, evidence, redaction, ultra-205]
requires:
  - phase: 11
    provides: "Ultra 205 detector-gated safety evidence pattern and redaction review precedent"
provides:
  - "Phase 12 ASIC/mining evidence runbook and claim matrix"
  - "Generated artifact contract for detector, safe-boot, chip-detect, mining, soak, and parity-promotion evidence packs"
  - "Secret redaction review template for Phase 12 hardware artifacts"
affects: [docs-parity, checklist-promotion, hardware-evidence, phase-12]
tech-stack:
  added: []
  patterns:
    - "Tiered evidence ladder before checklist promotion"
    - "Evidence-pack contract with redaction review before generated artifacts are committed"
key-files:
  created:
    - docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md
    - docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/README.md
    - docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/redaction-review.md
  modified: []
key-decisions:
  - "Phase 12 evidence is split into detector/safe boot, chip-detect, diagnostic work/result, controlled mining smoke, bounded soak, and parity-promotion tiers."
  - "Generated logs and JSON must pass an explicit redaction review before being cited or committed."
patterns-established:
  - "Checklist rows can only promote when the exact claim has matching evidence tier metadata."
  - "Unsupported ASIC/mining tiers are recorded as hardware evidence pending instead of inferred from safe boot or unit tests."
requirements-completed: [ASIC-07, STR-06, STR-07, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-06-30T00-13-19
generated_at: 2026-06-30T01:07:42Z
duration: 12 min
completed: 2026-06-30
---

# Phase 12 Plan 01: Evidence Contract Summary

**Tiered ASIC and mining evidence contract with redaction rules before any live Phase 12 hardware command runs**

## Performance

- **Duration:** 12 min
- **Started:** 2026-06-30T00:55:00Z
- **Completed:** 2026-06-30T01:07:42Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Created the Phase 12 runbook with hardware gate, stop conditions, allowed/prohibited command sets, evidence ladder, claim matrix, execution log, promotion rules, residual risks, and current pending conclusion.
- Added the artifact README defining six evidence packs and the rule that generated JSON/log/probe artifacts are not hand-edited.
- Added the secret redaction review template covering pool, worker, Wi-Fi, private endpoint, NVS, API token, private IP, and pasted-terminal-secret categories.

## Task Commits

1. **Task 1: Write the Phase 12 runbook and claim matrix** - `6c270e3`
2. **Task 2: Add artifact and redaction templates** - `e0f0d90`

## Files Created/Modified

- `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` - Phase 12 ledger, evidence ladder, claim matrix, and checklist promotion rules.
- `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/README.md` - Generated artifact contract and evidence-pack semantics.
- `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/redaction-review.md` - Secret review checklist and initial pending conclusion.

## Decisions Made

Evidence promotion is tiered so chip-detect, diagnostic work/result, controlled mining smoke, and bounded soak do not inherit each other's proof.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

The first redaction template pass used capitalized checklist labels for some exact categories. The template now includes a machine-checkable category sentence with the exact plan wording.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for `12-02`: parity tooling can now use this evidence contract to reject unsupported verified ASIC/mining rows.

*Phase: 12-asic-and-mining-hardware-evidence*
*Completed: 2026-06-30*
