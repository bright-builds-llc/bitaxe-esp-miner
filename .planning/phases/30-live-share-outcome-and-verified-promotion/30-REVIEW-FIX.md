---
phase: 30-live-share-outcome-and-verified-promotion
fixed_at: 2026-07-13T17:35:14Z
review_path: .planning/phases/30-live-share-outcome-and-verified-promotion/30-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 1
status: all_fixed
---

# Phase 30: Code Review Fix Report

**Fixed at:** 2026-07-13T17:35:14Z
**Source review:** `.planning/phases/30-live-share-outcome-and-verified-promotion/30-REVIEW.md`
**Iteration:** 1

## Fixed Issues

### CR-01: Verified promotion was self-attested by checklist text

**Status:** fixed
**Commit:** `03db3f7` (`fix(30): bind promotion to structured evidence`)
**Files modified:** `tools/parity/src/main.rs`, `tools/parity/BUILD.bazel`, `BUILD.bazel`
**Applied fix:** The parity report now reads only the exact committed Phase 30 `conclusion.md` path for verified STR-09, CFG-07, or ASIC-11 rows. It parses unique closed fields, rejects missing, unreadable, malformed, no-promotion, and mismatched row bundles, and requires explicit current-source, detector, same-chain, provenance, redaction, raw-artifact, hardware, eligible-outcome, and row-specific proof before admission. Checklist notes are breadcrumbs only.
**Verification:** Seven Phase 30 Rust tests cover committed no-promotion rejection through the report path, matching positive structured artifacts, missing artifacts, malformed closed values, mismatched row bundles, and unchanged conservative rows.

### WR-01: The final conclusion was outside the automated contract

**Status:** fixed
**Commit:** `03db3f7` (`fix(30): bind promotion to structured evidence`)
**Files modified:** `BUILD.bazel`, `scripts/BUILD.bazel`, `scripts/phase30-no-promotion-contract-test.sh`, `tools/parity/BUILD.bazel`
**Applied fix:** The root package exports `conclusion.md`; both the shell contract and parity Bazel target declare it as data. The shell contract verifies all seven no-promotion fields, three `not_promoted_pending` rows, the completion-versus-verification statement, and all exact non-claims, then includes the conclusion in the protected redaction aggregate.
**Verification:** `bash -n`, ShellCheck when available, `//scripts:phase30_no_promotion_contract_test`, and `//tools/parity:tests` passed.

## Verification

- `cargo fmt --all` passed.
- `cargo clippy --all-targets --all-features -- -D warnings` passed.
- `cargo build --all-targets --all-features` passed.
- `cargo test --all-features` passed.
- `bazel test //scripts:phase30_no_promotion_contract_test //tools/parity:tests` passed.
- `just parity` passed with `validation_errors: none`.
- `just verify-reference` passed.
- `git diff --check` passed.
- STR-09, CFG-07, and ASIC-11 checklist statuses were not changed; `.planning/REQUIREMENTS.md` was not changed.

## Skipped Issues

### IN-01: Extract Phase 30 policy from the large command file

Skipped because the requested scope was Critical and Warning findings only. No refactor was performed.

***

_Fixed: 2026-07-13T17:35:14Z_
_Fixer: gsd-code-fixer_
_Iteration: 1_
