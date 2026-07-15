---
phase: 33-confirmed-settings-durability
status: all_fixed
findings_in_scope: 1
fixed: 1
skipped: 0
iteration: 3
generated_at: 2026-07-15T02:01:42Z
---

# Phase 33 Code Review Fix Report

The sole warning finding in the iteration-3 `33-REVIEW.md` was fixed. No critical or informational findings were in scope.

## Iteration 3 Fix

### WR-01: Tracked paths cannot satisfy raw-root admission through ignore rules

- Commit: `ce98a53` (`fix(33): reject tracked raw evidence roots`)
- Replaces the `git check-ignore --no-index` admission shortcut with a named untracked-and-ignored invariant.
- Uses `git ls-files --cached --stage` with a literal pathspec to reject an exact tracked entry, any tracked descendant, gitlinks, and ambiguous index states before applying the ordinary ignore check.
- Fails closed when either Git probe errors or cannot establish the required state.
- Preserves canonicalization, symlink rejection, repository-root rejection, outside-root mode checks, directory creation modes, and pre-operation ordering.
- Adds a temporary-Git-repository regression with a force-added ignored subtree and proves rejection occurs before any fake detector, flash, classifier, identity, monitor, or HTTP-capable operation.

## Cumulative Resolution

- Iteration 1 fixed five warnings in commits `4dcf1b7`, `f307336`, and `49c5bca`: protected raw-root containment, restoration-safe finalization, production-path simulation fidelity, poisoned-snapshot retention, and response-before-effect worker ownership.
- Iteration 2 fixed two successor warnings in commits `a0e4d19` and `5e67223`: confirmed settings success across best-effort worker failure and redacted confirmed-snapshot diagnostics.
- Iteration 3 fixed the final tracked-but-ignored admission warning in commit `ce98a53`.
- Across all three iterations, eight warning findings were fixed and none were skipped.

## Verification

- The required Rust pre-commit sequence passed in order: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- `bash -n`, `shellcheck`, and `shfmt -d` passed for the Phase 33 wrapper, behavioral test, and fake fixture scripts.
- `bash scripts/phase33-confirmed-settings-durability-test.sh` passed, including the new tracked-but-ignored rejection and the existing safe, unsafe, symlink, cancellation, restoration, fake-backed failure, and sensitive-output cases.
- Bazel tests passed for `//scripts:phase33_confirmed_settings_durability_test` and `//tools/parity:tests` with the Phase 33 source-guard filter.
- `just build`, `just package`, and `just verify-reference` passed at source commit `ce98a53`.
- `git diff --check` passed, and the implementation commit contains only the Phase 33 wrapper and its behavioral test.
- `docs/evidence/phase-33/hardware-summary.md` remains byte-unchanged from commit `323e5e4` and passes the sensitive-output denylist.

## Preservation And Residual Risk

- No hardware, USB serial session, credentials, protected raw evidence, or raw device trace was accessed.
- No hardware proof, evidence status, public response shape, boot identity contract, or restart ownership guarantee was changed or promoted.
- This fix adds host/fake behavioral proof and build/package proof only; it does not replace or promote the existing Phase 33 hardware evidence.

This report is intentionally left uncommitted for the parent GSD workflow to consume.
