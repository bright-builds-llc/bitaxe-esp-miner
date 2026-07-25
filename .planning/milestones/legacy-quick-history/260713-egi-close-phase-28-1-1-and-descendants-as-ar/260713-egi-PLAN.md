---
quick_id: 260713-egi
description: "Close Phase 28.1.1 and descendants as archived Won't Do terminal unresolved work; guard historical diagnostics and route GSD exclusively to Phase 30"
mode: quick-full
status: planned
created: 2026-07-13
must_haves:
  truths:
    - "Phase 28.1.1 and descendants 28.1.1.1 through 28.1.1.7 are terminal archived history, while every verification remains gaps_found and STR-09, CFG-07, and ASIC-11 remain pending."
    - "Phase 30 is the only active continuation selected by GSD discuss, chain, push, progress, roadmap, and autonomous/range routing."
    - "Every historical Phase 28.1.1 hardware or capture entrypoint exits 64 with the terminal-closure and Phase 30 routing message before detection, credential access, writes, flashing, reset, monitoring, or hardware interaction."
    - "Archived lookup and lifecycle validation succeed for all eight lineage phases, and the accepted W006 health warnings are documented without recreating active directories or claiming verification passed."
  artifacts:
    - path: ".planning/ROADMAP.md"
      provides: "Checked terminal closure labels, historical plan counts, and conservative Phase 30 input contract"
    - path: ".planning/STATE.md"
      provides: "Phase 30-ready project state and terminal-lineage decision"
    - path: ".planning/milestones/v1.1-phases/28.1.1-bm1366-nonce-production-wire-parity"
      provides: "Archived root history, six finalized debug records, and unresolved closure verification"
    - path: "scripts/phase28.1.1-terminal-closure-guard.sh"
      provides: "Shared effect-free exit-64 guard"
    - path: "scripts/phase28.1.1-terminal-closure-guard-test.sh"
      provides: "Regression proof for all guarded entrypoints and zero active-directory recreation"
    - path: "AGENTS.md"
      provides: "Repository prohibition on reopening the archived lineage"
  key_links:
    - from: "historical Phase 28.1.1 hardware/capture shell entrypoints"
      to: "scripts/phase28.1.1-terminal-closure-guard.sh"
      via: "source and immediate guard invocation before all other initialization"
    - from: ".planning/phases/29-evidence-workflow-automation-closure/29-CONTEXT.md"
      to: ".planning/milestones/v1.1-phases/28.1.1-bm1366-nonce-production-wire-parity/28.1.1-CONTEXT.md"
      via: "canonical archived reference"
    - from: ".planning/ROADMAP.md"
      to: "Phase 30"
      via: "terminal lineage status and conservative no-promotion input contract"
---

# Quick Task 260713-egi Plan

## Task 1: Make the terminal unresolved closure internally truthful and archive-complete

**Files:** `.planning/ROADMAP.md`, `.planning/STATE.md`, `.planning/config.json`, `.codex/tasks/todo.md`, `.planning/phases/28.1.1*/`, `.planning/debug/phase28.1.1-*.md`, `.planning/debug/plan13-*.md`, `.planning/debug/ultra205-serial-session-reuse-failure.md`, `.planning/debug/ultra205-blank-lcd-after-diagnostic-replug.md`, `.planning/milestones/v1.1-phases/`

**Action:**
- Restore root context lifecycle ID `28.1.1-2026-07-09T19-24-27`, refresh root and descendant closure verification metadata, and validate all eight current lifecycles before moving anything.
- Preserve every verification result as `gaps_found`, every historical plan/summary count and exact non-claim, and keep STR-09, CFG-07, and ASIC-11 pending.
- Mark Phase 28.1 and the full lineage checked in the roadmap summary. Label the root `Closed — Won't Do (Unresolved)` and every descendant `Closed — Diagnostic Gaps Deferred`; correct only metadata drift while retaining history.
- Rewrite Phase 30's contract so the lineage is terminal, contains no eligible share evidence, and requires conservative no-promotion unless explicitly supplied new evidence is validated.
- Set STATE to `Phase 30 ready to discuss`, remove stale `_auto_chain_active`, and mark remaining Phase 28.1.1 todo work skipped by the Won't Do decision with a completion review.
- Finalize the two open debug statuses as `closed_wont_do_unresolved`, then move all six related debug records into an archived-root debug-history location.
- Move all eight complete lineage directories to `.planning/milestones/v1.1-phases/` without dropping plans, summaries, verification, UAT, evidence, discussion, or history.
- Document the intentional W006 health exception: active-milestone phases archived before milestone completion. State that repair must not recreate the directories or promote verification.

**Verify:** Run `verify lifecycle` with expected IDs and required plans/verification for all eight before and after archival; assert all eight verification statuses remain `gaps_found`; compare historical plan/summary counts; assert pending requirements/non-claims remain; assert the six debug files exist only inside the archived root; assert no active lineage directory exists.

**Done:** The complete lineage is archived as truthful unresolved history, the roadmap/state/todos/config point exclusively to Phase 30, and no evidence or requirement has been promoted.

## Task 2: Prevent every executable reopening path

**Files:** `AGENTS.md`, `Justfile`, `.planning/phases/29-evidence-workflow-automation-closure/29-CONTEXT.md`, `.planning/phases/29-evidence-workflow-automation-closure/29-VERIFICATION.md`, `scripts/phase28.1.1-roadmap-phase30-immutability.mjs`, `scripts/phase28.1.1-roadmap-phase30-immutability-test.mjs`, `scripts/phase28.1.1-terminal-closure-guard.sh`, `scripts/phase28.1.1-terminal-closure-guard-test.sh`, `scripts/BUILD.bazel`, the launcher and regression sets enumerated below

**Action:**
- Add repo guidance prohibiting explicit discuss, plan, execute, verify, hardware, direct-UART, and pin-manipulation work for the archived lineage. Declare Phase 30 the sole continuation and retain the stricter physical-access rules.
- Update Phase 29's canonical Phase 28.1.1 reference to the archive and refresh Phase 29 verification metadata/evidence after the context-path change without altering its passed outcome or claims.
- Update the roadmap/Phase 30 immutability helper and tests to use archived evidence paths and the new conservative Phase 30 contract.
- Add one shared shell guard that prints the terminal closure and Phase 30 routing message and exits exactly 64. Source/invoke it as the first executable action in every historical lineage hardware/capture launcher, before repo-root discovery, option parsing, imports, traces, locks, reads, or writes. The complete guarded shell set is:
  - `scripts/phase28.1.1-accepted-state-diagnostic.sh`
  - `scripts/phase28.1.1-exact-head-hardware-attempt.sh`
  - `scripts/phase28.1.1-upstream-wire-capture.sh`
  - `scripts/phase28.1.1-wire-parity-capture.sh`
  - `scripts/phase28.1.1.1-below-job-byte-diagnostic.sh`
  - `scripts/phase28.1.1.1-source-work-aligned-capture.sh`
  - `scripts/phase28.1.1.1-upstream-golden-capture.sh`
  - `scripts/phase28.1.1.2-result-path-diagnostic.sh`
  - `scripts/phase28.1.1.3-rx-acquisition-diagnostic.sh`
  - `scripts/phase28.1.1.4-init-sequencing-diagnostic.sh`
  - `scripts/phase28.1.1.5-chip-enumerate-diagnostic.sh`
  - `scripts/phase28.1.1.6-version-rolling-diagnostic.sh`
  - `scripts/phase28.1.1.7-asic-mask-reload-diagnostic.sh`
  - `scripts/diagnose-ultra205-late-attach.sh`
  - `scripts/diagnose-ultra205-uart-capture.sh`
  - `scripts/ultra205-late-attach-broker.sh`
  - `scripts/ultra205-late-attach-worker.sh`
  - `scripts/ultra205-uart-capture-broker.sh`
  - `scripts/ultra205-uart-capture-worker.sh`
  - `scripts/ultra205-transport-qualification.sh`
- Preserve Phase 30-useful pure classifiers, comparators, state modules, evidence denylist, and immutability helpers as executable non-hardware logic; do not guard them.
- Add a Bazel-owned regression enumerating all twenty guarded shell surfaces and both public `Justfile` recipes (`diagnose-ultra205-late-attach` and `diagnose-ultra205-uart-capture`). For each, assert exit 64/message, unchanged active phase directory state, no trace/credential read/write/detection/flash/reset/monitor/hardware marker, and no side effect even for help/dry-run modes.
- Replace obsolete launcher-behavior expectations with terminal-closure assertions in `scripts/phase28.1.1-accepted-state-diagnostic-test.sh`, `scripts/phase28.1.1-exact-head-hardware-attempt-test.sh`, the seven descendant diagnostic tests, `scripts/phase28.1.1.1-upstream-golden-capture-test.sh`, `scripts/diagnose-ultra205-late-attach-test.sh`, and `scripts/diagnose-ultra205-uart-capture-test.sh`; keep useful pure helper/classifier/state coverage in their Node/Perl tests. Update their Bazel data/targets for the shared guard.

**Verify:** Run bash syntax, `shfmt -d`, and `shellcheck` on changed shell files; run the guard regression and affected Bazel tests; prove Phase 29 context references resolve and its refreshed lifecycle validates; prove the immutability test accepts the archive and conservative Phase 30 projection.

**Done:** Explicit project guidance and executable guards make the terminal lineage non-runnable, while Phase 29 and immutability checks remain valid against archived history.

## Task 3: Prove GSD closure behavior and complete the clean git gate

**Files:** all implementation files plus `.planning/quick/260713-egi-close-phase-28-1-1-and-descendants-as-ar/260713-egi-SUMMARY.md`; the independent verifier owns `.planning/quick/260713-egi-close-phase-28-1-1-and-descendants-as-ar/260713-egi-VERIFICATION.md`

**Action:**
- Verify `init yolo-target discuss`, `chain`, and `push` all select Phase 30; roadmap analysis and progress report Phase 30 next; autonomous/range candidates exclude the archive; `audit-uat` has no active 28.1.1 work; and `.planning/debug/` has no related sessions.
- Verify archived `phase-op` lookup and lifecycle validation for 28.1.1 through 28.1.1.7. Run health and assert only the explicitly documented W006 exception is introduced for this lineage; do not repair it.
- Run every guard regression with no hardware or credentials, then affected shell/Bazel tests, `just verify-reference`, and the mandatory Rust sequence in order: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`.
- Finish with roadmap/reference resolution checks, `git diff --check`, frontmatter checks, a simplification/diff review, and explicit assertions that STR-09/CFG-07/ASIC-11 and parity/checklist claims were not promoted.
- The executor creates the quick summary and commits the implementation only after the mandatory Rust sequence passes. A separate verifier then independently reruns the closure checks and creates the quick verification artifact. Before committing that verification artifact and orchestrator-owned STATE/plan/summary docs, rerun the repository's mandatory Rust pre-commit sequence; stage explicit files, ensure a clean tree, and push `main` without rewriting the two existing local commits.

**Verify:** Every command above passes (with only the documented health W006 exception), quick verification status is `passed`, `git status --short` is empty after commit, local `main` is ahead only by the intended commits before push, and `git rev-list --left-right --count HEAD...origin/main` is `0 0` after push.

**Done:** GSD consistently routes to Phase 30, the archived lineage cannot reappear or run, all repository gates pass, and the clean verified closure is committed and pushed.
