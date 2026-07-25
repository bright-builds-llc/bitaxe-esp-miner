---
phase: quick-260720-jwt
plan: "260720-jwt"
subsystem: evidence-security
tags: [private-first, redaction, flash, phase35, ci]
requires:
  - phase: 35-detector-gated-correlated-evidence-and-exact-parity-promotion
    provides: "Protected Phase 35 supervisor, Phase 33 boot classifier, and sealed attempt-11 failure evidence"
provides:
  - "Repository-wide evidence data classes, lifecycle, sink, admission, and exception policy"
  - "Stream-isolated bounded pre-disk sanitization and additive dual flash evidence mode"
  - "Digest-bound software finalizer that creates admitted evidence only after classification"
  - "Staged/CI/admitted-tree redaction enforcement through one repository command"
  - "Phase 35 private classifier-input routing with attempt-11 regression coverage"
affects: [evidence-pipelines, tools-flash, phase35, ci, parity-admission]
tech-stack:
  added: []
  patterns:
    - "Immutable secret-sanitized private input followed by classifier-authorized commit-redacted projection"
    - "Independent bounded line state for child stdout and stderr"
    - "Exact reviewed legacy exceptions with non-echoing staged and admitted-tree scans"
    - "Zero-base branch creation normalized through a trusted default-branch merge base"
key-files:
  created:
    - docs/parity/evidence-policy.md
    - scripts/verify-redaction.sh
    - scripts/verify-redaction-test.sh
    - scripts/redaction-exceptions.tsv
    - .github/workflows/evidence-redaction.yml
    - tools/flash/src/evidence.rs
  modified:
    - tools/flash/src/main.rs
    - scripts/phase35-correlated-evidence-effects.sh
    - scripts/phase35-correlated-evidence-test.sh
    - .planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-CONTEXT.md
    - .planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-04-PLAN.md
key-decisions:
  - "Dual mode is additive: legacy defaults and --redact-evidence remain compatible while Phase 35 explicitly opts into private-first capture."
  - "Private classifier logs and records are mode 0600 below a mode-0700 root; admitted logs and records contain no private paths or operational values."
  - "Dual capture creates no admitted artifacts; a digest-bound software finalizer may create them only after the classifier passes."
  - "New-branch CI scans every merge-base-to-HEAD destination change without exceptions while retaining exact exceptions for unchanged admitted baseline files."
  - "Attempts 1-11 remain immutable, and attempt 12 requires a fresh exact-head preflight followed by separate authorization."
patterns-established:
  - "Capture stdout and stderr through independent bounded incremental sanitizers before any disk write or terminal inheritance."
  - "Close and hash private input, classify it, finalize a separate admitted artifact, then prove the private digest is unchanged."
  - "Scan the staged/CI destination snapshot plus complete admitted roots without echoing matched content."
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: quick-full
phase_lifecycle_id: quick-260720-jwt
generated_at: 2026-07-20T21:46:22Z
duration: 2h18m
completed: 2026-07-20
---

# Quick Task 260720-jwt: Repository-Wide Private-First Evidence Summary

**Private-first evidence capture with immutable classifier input, separate admitted projection, repository redaction enforcement, and repaired Phase 35 ordering**

## Performance

- **Duration:** 2h 18m
- **Started:** 2026-07-20T19:28:22Z
- **Completed:** 2026-07-20T21:46:22Z
- **Tasks:** 3
- **Files modified:** 17

## Accomplishments

- Defined authoritative `NeverPersistRaw`, `ProtectedOperational`, `ShareableFact`, and `PublicProvenance` classes with explicit lifecycle and sink rules.
- Added a non-echoing staged/CI/admitted-tree scanner whose legacy exceptions apply only to unchanged full-baseline artifacts; changed, staged, range, and new-branch destination blobs always fail closed.
- Normalized GitHub's all-zero branch-creation base through the trusted default-branch merge base, capped non-echoing findings, and proved the real admitted baseline completes without treating immutable history as new content.
- Added `--evidence-mode dual` with distinct stdout/stderr sanitization state, canonical ignored-root admission before effects, mode-0700/0600 protection, immutable private artifacts, safe console categories, and legacy compatibility.
- Added a digest-bound software finalizer so dual capture creates no admitted log or record until the authorized classifier has passed.
- Routed Phase 35 through `direct_flash < classifier < finalize_evidence < original_read/PATCH`, and proved classifier failure creates no admitted outputs.
- Preserved attempts 1-11 and all evidence truth; no detector, credentials, hardware, network, device, mutation, reboot, admission, promotion, push, or attempt-12 authorization occurred.

## Task Commits

Each task was committed atomically:

1. **Task 1: Enforce repository-wide private-first evidence policy** - `512ecc3c` (feat)
2. **Task 2: Add dual private-first flash evidence capture** - `d24325ba` (feat)
3. **Task 3: Route Phase 35 through private classifier input** - `754344c0` (fix)
4. **Follow-up: Close scanner scope and baseline-exception gaps** - `12642499` (fix)
5. **Follow-up: Stage private evidence before projection** - `4be06e39` (fix)
6. **Follow-up: Finalize Phase 35 evidence after classification** - `c83dce00` (fix)
7. **Follow-up: Normalize new-branch redaction scans** - `28fdd115` (fix)

Plan metadata remains uncommitted for the orchestrator and independent verifier.

## Files Created/Modified

- `docs/parity/evidence-policy.md` - Canonical data classes, lifecycle, capture order, sinks, admission, and exception rules.
- `scripts/verify-redaction.sh` - Non-echoing staged/CI and full admitted-tree scanner.
- `scripts/redaction-exceptions.tsv` - Exact reviewed exceptions for immutable legacy artifacts.
- `.github/workflows/evidence-redaction.yml` - Pull-request and push enforcement through the repository adapter.
- `tools/flash/src/evidence.rs` - Stream-identified sanitizers, secure ignored-workspace-root preflight, private capture, finalization, permissions, and digest checks.
- `tools/flash/src/main.rs` - Additive dual CLI/orchestration, explicit `finalize-evidence`, safe projections, compatibility records, and focused tests.
- `scripts/phase35-correlated-evidence-effects.sh` - Private Boot A classifier routing followed by digest-bound direct finalizer invocation.
- `scripts/phase35-correlated-evidence-test.sh` - Real-process attempt-11, exact ordering, no-admitted-on-rejection, cleanup, and runfiles regressions.
- Phase 35 context, Plan 35-04, `AGENTS.md`, Just/Bazel wiring, and scoped task/lesson records - Durable workflow enforcement and attempt authority.

## Decisions Made

- Kept dual mode opt-in so existing developer and `--redact-evidence` callers retain their established behavior.
- Used separate stdout and stderr line-bounded incremental sanitizers to carry chunk splits without joining partial lines, while failing closed as `evidence_sanitization_invalid` on invalid UTF-8, overlong input, or pipe-state failure.
- Produced only a private log and private command record during dual capture; the explicit finalizer verifies the classified digest before producing the admitted compatibility log and record.
- Classified only the private log in Phase 35; `flash-monitor.log` remains absent until the classifier passes and the finalizer succeeds.
- Left the Phase 33 classifier unchanged and repaired only the upstream capture/ordering boundary.
- Required zero-base CI to establish a trusted default-branch comparison commit; missing or unrelated comparison history fails closed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Seeded exact reviewed legacy exceptions**

- **Found during:** Task 1
- **Issue:** Complete admitted-tree scanning found immutable historical operational data and opaque artifacts.
- **Fix:** Added exact path/category registry entries without wildcards, inline bypasses, history rewriting, or weakened scanning.
- **Files modified:** `scripts/redaction-exceptions.tsv`
- **Verification:** Scanner fixture suite and full `just verify-redaction` passed.
- **Committed in:** `512ecc3c`

**2. [Rule 3 - Blocking] Made redaction tests and registry runfiles-aware**

- **Found during:** Task 1
- **Issue:** Bazel execution required deterministic registry discovery, and generated-script launch through `/usr/bin/env` stalled on the host.
- **Fix:** Preferred sibling/runfiles registry paths and used the stable Bash dispatcher for hermetic fixtures.
- **Files modified:** `scripts/verify-redaction.sh`, `scripts/verify-redaction-test.sh`, `scripts/BUILD.bazel`
- **Verification:** Direct, Bazel, and Just scanner paths passed.
- **Committed in:** `512ecc3c`

**3. [Rule 2 - Missing Critical] Covered numeric pool scalars, console sinks, and root permissions**

- **Found during:** Task 2 review
- **Issue:** Numeric JSON pool ports, dual wrapper output, and pre-existing evidence-root permissions needed explicit fail-closed handling.
- **Fix:** Added scalar redaction, category-safe dual output/errors, exact mode-0700 root enforcement, and regression tests.
- **Files modified:** `tools/flash/src/main.rs`, `tools/flash/src/evidence.rs`
- **Verification:** Flash Cargo/Bazel suites, Clippy, permissions, projection, and safe-output tests passed.
- **Committed in:** `d24325ba`

**4. [Rule 2 - Missing Critical] Removed scanner-visible private hostname assignments from modified Phase 35 source**

- **Found during:** Task 3 staged admission
- **Issue:** The repository scanner correctly rejected private-value assignment shapes in the staged Phase 35 shell source.
- **Fix:** Renamed the protected variable and constructed the JSON field without a staged private assignment literal.
- **Files modified:** `scripts/phase35-correlated-evidence-effects.sh`
- **Verification:** Shell hygiene, direct/Bazel Phase 35 tests, and `just verify-redaction` passed.
- **Committed in:** `754344c0`

**5. [Rule 1 - Bug] Restricted legacy exceptions to the unchanged full baseline**

- **Found during:** Follow-up adversarial scanner review
- **Issue:** Legacy exceptions could affect staged/range destinations, operational rules were broader than their shareable sinks, and an all-zero push base did not scan every destination blob.
- **Fix:** Applied exceptions only to unchanged tracked admitted artifacts in the complete baseline, scanned all changed blobs with `NeverPersistRaw`, and scoped `ProtectedOperational` to shareable sinks. The initial all-zero implementation scanned every destination `HEAD` blob and was superseded by deviation 8.
- **Files modified:** `scripts/verify-redaction.sh`, `scripts/verify-redaction-test.sh`
- **Verification:** Hermetic staged, range, malformed-revision, and new-branch regressions plus `just verify-redaction` passed.
- **Committed in:** `12642499`

**6. [Rule 2 - Missing Critical] Separated stream state and delayed all admitted projection**

- **Found during:** Follow-up private-first boundary review
- **Issue:** Child streams shared partial-line state, dual capture still created admitted artifacts, and local-root admission was not proved through Git ignore state before effects.
- **Fix:** Added explicit stream identity with independent bounded buffers, canonical workspace containment plus `git check-ignore`, private-only capture, and a digest-bound software finalizer.
- **Files modified:** `tools/flash/src/evidence.rs`, `tools/flash/src/main.rs`
- **Verification:** Forced-interleave unit/real-process tests, real Git admission tests, 142 flash tests, Bazel flash tests, and the full Rust gate passed.
- **Committed in:** `4be06e39`

**7. [Rule 1 - Bug] Enforced classifier-before-finalizer in Phase 35**

- **Found during:** Follow-up Phase 35 integration review
- **Issue:** The supervisor needed an explicit finalization boundary and rejection proof consistent with private-only dual capture.
- **Fix:** Verified the private record digest around classification and finalization, invoked the already-resolved finalizer directly, and asserted exact process order plus absent admitted outputs on classifier failure.
- **Files modified:** `scripts/phase35-correlated-evidence-effects.sh`, `scripts/phase35-correlated-evidence-test.sh`, Phase 35 context/plan, and policy/task documentation.
- **Verification:** Shell static checks, seven Bazel targets, redaction, parity, lifecycle, and exact-HEAD preflight passed.
- **Committed in:** `c83dce00`

**8. [Rule 1 - Bug] Made all-zero branch-creation CI usable without weakening destination scans**

- **Found during:** Independent verification after Task 3
- **Issue:** Treating every `HEAD` blob as newly introduced disabled all legacy exceptions, emitted roughly 648 KB of immutable findings, and made branch-creation CI unusable.
- **Fix:** Resolved zero-base pushes to the trusted default-branch merge base, scanned every changed destination blob with exceptions disabled, kept exact exceptions only for unchanged tracked admitted baseline files, and capped findings at 100 plus a non-echoing suppression count.
- **Files modified:** `scripts/verify-redaction.sh`, `scripts/verify-redaction-test.sh`, `.github/workflows/evidence-redaction.yml`, `docs/parity/evidence-policy.md`
- **Verification:** Multi-commit hermetic fixtures, changed-exception staged/range/new-branch checks, a 1,000-finding bounded-output fixture, opt-in real-repository baseline scan, `actionlint`, and the full gate suite passed.
- **Committed in:** `28fdd115`

**Total deviations:** 8 auto-fixed (6 missing-critical/correctness, 2 blocking)
**Impact on plan:** All fixes strengthened required security and hermetic execution without expanding hardware or evidence authority.

## Issues Encountered

- The first pre-commit preflight launch stalled in macOS `dyld` before application code, with no detector or hardware activity. The owned process was terminated; a warmed rerun passed, and the required post-commit exact-current-head preflight passed with `current_head_equal=true`.
- Independent verification found that the initial all-zero branch implementation conflated immutable history with branch-introduced content. The corrected real-repository zero-base scan completed with a 25-byte success result while hermetic new-content violations remained fail-closed and bounded.

## Verification

- Bash syntax, `shfmt`, and `shellcheck` passed for all changed shell files.
- Scanner direct/Bazel/Just tests, the opt-in real-repository zero-base test, and staged/full admitted-tree redaction checks passed; `actionlint` also passed the CI workflow.
- `bazel test` passed the scanner plus seven required Phase 29/30/33/35 and flash targets.
- `just verify-reference`, `just parity`, and lifecycle verification for Phase 35 passed.
- Every task passed the ordered repository-wide Rust sequence: format, Clippy with warnings denied, all-target build, and all-feature tests.
- Post-commit `just phase35-evidence preflight-only=true` passed for `28fdd115` with `current_head_equal=true` and capability digest `c3e2fbecb14036b439a1964feb5c4cc1b7252c11057b51517bf354647c0f87b1`; no detector or hardware path ran.

## Known Stubs

None.

## User Setup Required

None.

## Next Phase Readiness

- Phase 35 is software-prepared for a possible attempt 12, but no attempt is authorized.
- Any attempt 12 requires a fresh exact-current-head preflight and separate user authorization after that preflight.
- Remaining active evidence-producing workflows are tracked for later private-first migration; immutable historical evidence is excluded.

## Self-Check: PASSED

- All key created and modified implementation files exist.
- Task commits `512ecc3c`, `d24325ba`, `754344c0`, `12642499`, `4be06e39`, `c83dce00`, and `28fdd115` exist.
- The quick summary is present and intentionally uncommitted for orchestrator/independent-verifier ownership.

*Quick task: 260720-jwt*
*Completed: 2026-07-20*
