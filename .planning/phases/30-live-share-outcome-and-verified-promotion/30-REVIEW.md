---
phase: 30-live-share-outcome-and-verified-promotion
reviewed: 2026-07-13T17:21:29Z
generated_at: 2026-07-13T17:21:29Z
depth: standard
status: issues_found
generated_by: gsd-code-reviewer
lifecycle_mode: yolo
phase_lifecycle_id: 30-2026-07-13T16-24-26
diff_base: 90b23be
files_reviewed: 7
files_reviewed_list:
  - BUILD.bazel
  - docs/parity/checklist.md
  - docs/parity/evidence/phase-30-live-share-outcome-and-verified-promotion/disposition.md
  - docs/parity/evidence/phase-30-live-share-outcome-and-verified-promotion/conclusion.md
  - scripts/BUILD.bazel
  - scripts/phase30-no-promotion-contract-test.sh
  - tools/parity/src/main.rs
findings:
  critical: 1
  warning: 1
  info: 1
  total: 3
---

# Phase 30: Code Review Report

**Reviewed:** 2026-07-13T17:21:29Z
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

The committed Phase 30 state is conservative: STR-09, CFG-07, and ASIC-11 remain `implemented` and pending, the archived result remains `gaps_found`, and the two Phase 30 documents contain no private runtime values found by the existing denylist. The current targeted tests pass.

However, the new promotion guard does not bind a verified checklist row to a real evidence artifact. It accepts promotion assertions copied directly into the row text, even while the repository's actual `conclusion.md` says no promotion and no new evidence. This leaves the central false-promotion risk unresolved. The final conclusion is also outside the persistent Phase 30 shell/redaction contract, so later drift in that artifact is not covered automatically.

Material guidance applied: repo-local terminal archived-lineage and evidence-redaction rules from `AGENTS.md`; `AGENTS.bright-builds.md`; `standards-overrides.md`; `standards/index.md`; and the architecture, code-shape, testing, verification, and Rust standards. No local override applies.

## Critical Issues

### CR-01: Verified promotion is self-attested by checklist text instead of authenticated evidence

**File:** `tools/parity/src/main.rs:1289-1375`

**Issue:** `phase30_missing_shared_terms`, `phase30_missing_row_terms`, and `phase30_has_eligible_share_outcome` search only `row_haystack(row)`. The report path reads the checklist but never loads the cited Phase 30 artifact. Consequently, a future edit can mark STR-09, CFG-07, or ASIC-11 `verified`, paste the expected strings into the notes, and make `just parity` accept the row without a matching committed promotion artifact. The positive fixtures demonstrate the disconnect: they pass with `phase30_promotion_disposition: promoted` and `new_evidence_input: explicit` in the synthetic row even though the real `conclusion.md` currently records `no_promotion_no_eligible_evidence` and `new_evidence_input: none`. The directory substring check also accepts a nonexistent or unrelated filename. This defeats the phase's core requirement that promotion require explicit eligible evidence and can manufacture safety-critical parity.

**Fix:** Make Phase 30 promotion admission consume a structured, committed artifact rather than treating the checklist note as the evidence payload. Resolve an exact allowed artifact path, fail if it is missing, parse exact closed fields, and require its row-specific proof bundle plus current-source/provenance/redaction gates before accepting `verified`. Keep the checklist as a breadcrumb only. Add an integration regression that evaluates a verified checklist row against the current no-promotion `conclusion.md` and proves rejection; add a positive fixture backed by a temporary complete artifact, plus missing-file, mismatched-row, and malformed-value cases.

## Warnings

### WR-01: The final conclusion is outside the automated contract and redaction aggregate

**File:** `scripts/phase30-no-promotion-contract-test.sh:7-15,106-111`

**Issue:** The shell contract requires and scans `disposition.md`, Phase 28.1 validation, and the three checklist rows, but it neither requires nor scans `conclusion.md`. The Bazel target likewise omits the conclusion from `data`, and the root package does not export it. The current conclusion passed a one-off denylist run, but future changes could remove a `not_promoted_pending` row, change a no-promotion field or non-claim, or introduce a private path/network/credential value while `//scripts:phase30_no_promotion_contract_test` remains green.

**Fix:** Export `conclusion.md`, add it to the shell test's Bazel data, require its seven disposition fields, all three `not_promoted_pending` rows, completion-versus-verification statement, and exact non-claims, and include it in the mode-0600 redaction aggregate passed through both the shared denylist and explicit category scans.

## Info

### IN-01: Phase 30 adds another policy cluster to a 2,997-line command file

**File:** `tools/parity/src/main.rs:1229-1379,2485-2768`

**Issue:** The Phase 30 validator, token vocabulary, fixtures, and eight tests add roughly 449 lines to an already oversized `main.rs`. The new logic is cohesive and has a natural module boundary, while keeping it in the command entry file makes evidence-admission policy harder to audit and increases the chance that future changes miss related tests.

**Fix:** After the functional gate is corrected, extract Phase 30 admission and its tests into a focused `phase30_promotion.rs` plus child test module, leaving `main.rs` as the thin command/report shell.

## Verification

- `cargo test -p bitaxe-parity --all-features phase30_` passed: 8 tests.
- `bazel test //scripts:phase30_no_promotion_contract_test //tools/parity:tests` passed (cached).
- `bash -n scripts/phase30-no-promotion-contract-test.sh` passed.
- ShellCheck passed for `scripts/phase30-no-promotion-contract-test.sh` when available.
- `git diff --check 90b23be --` over the seven reviewed files passed.
- The promoted-evidence denylist passed over both Phase 30 evidence documents; the current documents contain no matched local path, URL, IP, MAC, credential, or raw-value token.

***

_Reviewed: 2026-07-13T17:21:29Z_
_Reviewer: gsd-code-reviewer_
_Depth: standard_
