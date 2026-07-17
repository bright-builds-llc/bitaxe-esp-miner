---
phase: 35-detector-gated-correlated-evidence-and-exact-parity-promotion
plan: "02"
subsystem: parity-evidence
tags: [bash, bazel, evidence-supervisor, detector-gate, restoration, redaction]
requires:
  - phase: 35-detector-gated-correlated-evidence-and-exact-parity-promotion
    plan: "01"
    provides: Typed correlated evidence admission and closed redacted projection
provides:
  - Detector-gated exact-package correlated evidence supervisor
  - Fail-closed restoration, cleanup, non-promotion sealing, and private-root lifecycle
  - Exhaustive hermetic failure injection for every supervisor boundary
affects: [phase-35-hardware-qualification, phase-35-parity-promotion, CFG-12, EVD-10, EVD-13]
tech-stack:
  added: []
  patterns: [capability-gated imperative shell, explicit child-status preservation, category-only fixture recording]
key-files:
  created:
    - scripts/phase35-correlated-evidence.sh
    - scripts/phase35-correlated-evidence-root.sh
    - scripts/phase35-correlated-evidence-effects.sh
    - scripts/phase35-correlated-evidence-document.sh
    - scripts/phase35-correlated-evidence-fixture.sh
    - scripts/phase35-correlated-evidence-test.sh
  modified:
    - scripts/BUILD.bazel
    - Justfile
key-decisions:
  - "Full mode owns exactly one detector invocation after exact-package admission; preflight mode owns zero detector invocations and permits no effects."
  - "Credential-path validation occurs only after the detector proves exactly one board-205 candidate, and raw values never enter shareable output."
  - "Every post-mutation exit explicitly attempts restoration before cleanup and seals failed roots as non-promotable and non-reusable."
  - "Supervisor responsibilities are split across root, effect, and document helpers while one thin entrypoint owns lifecycle order."
patterns-established:
  - "Effect status: helpers preserve child-command failure explicitly instead of relying on Bash errexit through conditional call stacks."
  - "Hermetic recovery matrix: every post-PATCH failure proves restoration-before-cleanup; every pre-PATCH failure proves cleanup without restoration."
requirements-completed: [CFG-12, EVD-10, EVD-11, EVD-12, EVD-13, EVD-14]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 35-2026-07-17T17-00-37
generated_at: 2026-07-17T22:10:31Z
duration: 30min
completed: 2026-07-17
---

# Phase 35 Plan 02: Detector-Gated Correlated Evidence Supervisor Summary

**One exact-package supervisor now owns the sole detector gate, two-epoch storage proof, approved reboot, restoration, cleanup, and typed admission while a hermetic matrix proves every failure remains non-promotional.**

## Performance

- **Duration:** 30 min
- **Started:** 2026-07-17T21:40:44Z
- **Completed:** 2026-07-17T22:10:31Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Added `just phase35-evidence` and its Bazel-owned supervisor with exact-package Gate 1, sole board-205 detector Gate 2, post-detector credential-path handling, exact Boot A/PATCH/readback/reboot/Boot B ordering, restoration, cleanup, live rechecks, and typed validator admission.
- Kept local evidence under a fresh mode-0700 root with mode-0600 files, chained nine ordered events to the root contract, emitted only digest/category output, and sealed every failed root as non-promotable and non-reusable.
- Added a category-only hermetic fixture and exhaustive failure matrix covering package/detector/capability drift, target and capture failures, mutation and reboot boundaries, identity and epoch mismatches, rechecks, restoration, cleanup leaks, redaction, validator rejection, and exact success ordering.
- Proved preflight invokes no detector or effects, detector failure touches no target or credential command, full success invokes exactly one detector/reboot/restoration/cleanup/validator, and no test accesses USB, hardware, credentials, or the network.

## Task Commits

1. **Task 1: Add the detector-gated correlated evidence supervisor** - `666ab74a` (feat)
2. **Task 2: Add simulation and failure-injection coverage for every supervisor boundary** - `d7c1c870` (test)

## Files Created/Modified

- `scripts/phase35-correlated-evidence.sh` - Owns CLI validation and the strict package/detector/capture/recovery lifecycle.
- `scripts/phase35-correlated-evidence-root.sh` - Owns private-root creation, exact-package capability admission, digest helpers, and checkpoint recording.
- `scripts/phase35-correlated-evidence-effects.sh` - Owns detector, flash, capture, PATCH, reboot, restoration, cleanup, and exit finalization effects.
- `scripts/phase35-correlated-evidence-document.sh` - Constructs the exact inventory, epochs, event chain, evidence root, live rechecks, and validator boundary.
- `scripts/phase35-correlated-evidence-fixture.sh` - Replaces every external effect with a category-only deterministic stub and validates root linkage.
- `scripts/phase35-correlated-evidence-test.sh` - Exercises the complete fail-closed matrix and exact successful ordering without hardware or network access.
- `scripts/BUILD.bazel` - Exposes the local supervisor binary and hermetic shell test.
- `Justfile` - Routes `just phase35-evidence` exclusively through the Bazel supervisor target.

## Decisions Made

- Kept preflight limited to exact-package admission and current-source proof so it cannot discover a port, inspect a credential path, or perform an effect.
- Required the full workflow to own the only detector invocation, then pass only the resulting capability state to later steps; nested helpers do not rediscover hardware.
- Treated restoration and cleanup as root-closing evidence, not best-effort teardown. Any failure in either fact prevents admission and produces a non-promotion seal.
- Used separate focused shell modules so the entrypoint remains easy to audit for lifecycle ordering and every file remains within repository code-shape guidance.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Split the supervisor into focused helper modules**

- **Found during:** Task 1 (detector-gated supervisor).
- **Issue:** A single supervisor file exceeded repository code-shape guidance and made security-critical ordering harder to review.
- **Fix:** Kept one 186-line lifecycle entrypoint and moved package/root, effect/recovery, and document/validator responsibilities into three focused helper modules.
- **Files modified:** `scripts/phase35-correlated-evidence.sh`, `scripts/phase35-correlated-evidence-root.sh`, `scripts/phase35-correlated-evidence-effects.sh`, and `scripts/phase35-correlated-evidence-document.sh`.
- **Verification:** All production and test shell files are below 500 lines and pass `bash -n`, `shfmt`, and `shellcheck`.
- **Committed in:** `666ab74a`.

**2. [Rule 1 - Bug] Preserved child-command failures across conditional shell call stacks**

- **Found during:** Task 2 package-drift failure injection.
- **Issue:** Bash disables implicit `errexit` inside functions called from `||` conditions, so trailing permission updates could mask failed package, flash, capture, or validator child commands.
- **Fix:** Added explicit failure branches that secure partial output and return nonzero before any later step.
- **Files modified:** `scripts/phase35-correlated-evidence-root.sh`, `scripts/phase35-correlated-evidence-effects.sh`, and `scripts/phase35-correlated-evidence-document.sh`.
- **Verification:** Package, runtime, capture, and validator injection cases now fail, clean up, and seal their roots; the complete Phase 35 matrix passes.
- **Committed in:** `d7c1c870`.

**3. [Rule 1 - Bug] Rejected zero-byte and malformed setting snapshots**

- **Found during:** Task 2 zero-byte capture injection.
- **Issue:** A missing setting digest could fall through as an optional value and incorrectly satisfy the pre-PATCH comparison.
- **Fix:** Required a present lowercase 64-hex digest before comparing the snapshot to the expected value.
- **Files modified:** `scripts/phase35-correlated-evidence-document.sh`.
- **Verification:** Zero-byte and pre-PATCH mismatch cases fail before mutation, invoke cleanup without restoration, and leave a non-promotion seal.
- **Committed in:** `d7c1c870`.

**4. [Rule 1 - Bug] Corrected malformed GSD roadmap and decision updates**

- **Found during:** Plan metadata closeout.
- **Issue:** The standard roadmap command shifted the Phase 35 row into the wrong columns, and the decision command could not locate this repository's phase-scoped decision headings.
- **Fix:** Restored the four-column Phase 35 row, updated the exact requirement total, and inserted the Plan 02 decisions in the existing phase-scoped state format.
- **Files modified:** `.planning/ROADMAP.md` and `.planning/STATE.md`.
- **Verification:** Phase 35 reads `In Progress (2/4 plans)`, project progress reads 26/27 requirements, current position is Plan 2 of 4 completed, and the next action is Plan 35-03.
- **Committed in:** Final plan metadata commit.

**Total deviations:** 4 auto-fixed (3 bugs, 1 missing critical code-shape adjustment).
**Impact on plan:** All changes tighten correctness and auditability within the planned supervisor and test scope; no hardware or promotion scope was added.

## Issues Encountered

- The initial failure matrix caught Bash conditional-context behavior that static shell checks cannot detect. Explicit status preservation resolved it without changing the workflow contract.

## Known Stubs

None.

## User Setup Required

None - this plan is software-only and requires no hardware, credentials, network access, or external service configuration.

## Next Phase Readiness

- The next Phase 35 plan can invoke the supervisor only through its documented exact-package and detector gates.
- Hardware qualification remains a later plan-owned action; this plan performed no USB, serial, flash, monitor, HTTP, credential-file, device, or network operation.
- Parity promotion still requires a real eligible same-chain evidence root and the later owning plan; software success here does not promote any live claim by itself.

## Self-Check: PASSED

- All six created scripts and the Bazel/Justfile wiring exist.
- Task commits `666ab74a` and `d7c1c870` exist in repository history.
- Phase 35 supervisor, Phase 33 durability regression, and parity Bazel tests pass.
- Mandatory Rust format, strict Clippy, all-target build, and all-feature test gates pass.
- Reference cleanliness, parity validation, shell static checks, permissions assertions, redaction canaries, and `git diff --check` pass.

***

*Phase: 35-detector-gated-correlated-evidence-and-exact-parity-promotion*
*Completed: 2026-07-17*
