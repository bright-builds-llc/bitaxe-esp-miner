---
phase: 15-bm1366-mining-evidence-completion
fixed_at: 2026-07-01T05:19:51Z
review_path: .planning/phases/15-bm1366-mining-evidence-completion/15-REVIEW.md
iteration: 1
findings_in_scope: 7
fixed: 7
skipped: 0
status: all_fixed
---

# Phase 15: Code Review Fix Report

**Fixed at:** 2026-07-01T05:19:51Z
**Source review:** `.planning/phases/15-bm1366-mining-evidence-completion/15-REVIEW.md`
**Iteration:** 1

**Summary:**

- Findings in scope: 7
- Fixed: 7
- Skipped: 0

## Fixed Issues

### WR-01: Detector Logs Do Not Substantiate Board-Info Claims

**Files modified:** `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion.md`, `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke.md`, `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/bounded-soak.md`
**Commit:** 8d6f88b
**Commit status:** fixed
**Applied fix:** Downgraded mining-smoke and bounded-soak detector claims to port-only evidence and updated the final ledger to keep omitted board-info transcripts pending for those packs.
**Verification:** Read back edited sections, confirmed stale overclaim patterns no longer matched, and ran `mdformat --check` on the changed Markdown files.

### WR-02: Redaction Review Is Marked Passed With Unchecked Checklist Items

**Files modified:** `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md`
**Commit:** 7b65b20
**Commit status:** fixed
**Applied fix:** Checked completed redaction controls and marked absent API/WebSocket/pasted/manual artifact classes as explicit N/A checklist entries.
**Verification:** Read back the checklist and ran `mdformat --check` on the changed Markdown file.

### WR-03: Environment Variables Can Trigger Live Probes Outside The Manifested Command

**Files modified:** `scripts/phase15-controlled-mining.sh`, `scripts/phase15-controlled-mining-test.sh`
**Commit:** fbb1bb9
**Commit status:** fixed: requires human verification
**Applied fix:** Removed implicit `DEVICE_URL` authorization from the environment and added a regression test proving env-only `DEVICE_URL` plus pool variables remains controlled no-share unless `--device-url` is present.
**Verification:** Read back changed shell sections, ran `bash -n` on both shell files, ran `shfmt -l -d`, and ran `bash scripts/phase15-controlled-mining-test.sh`.

### WR-04: Mining Allow Validation Does Not Bind Surfaces To Claim Tiers Or Require The Command Filter

**Files modified:** `tools/parity/src/mining_allow.rs`
**Commit:** 3c4d50b
**Commit status:** fixed: requires human verification
**Applied fix:** Added the Phase 15 surface/claim-tier matrix, required the allowed-command filter for document validation, rejected unapproved wrapper command shapes and prohibited command tokens, and added unit coverage for each path.
**Verification:** Read back changed Rust sections and ran `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.

### WR-05: STR-008 Validation Can Accept A Controlled No-Share Overclaim

**Files modified:** `tools/parity/src/main.rs`
**Commit:** 3e9925d
**Commit status:** fixed: requires human verification
**Applied fix:** Made verified ASIC/mining rows reject live blocker terms, required STR-008 to show accepted/rejected share evidence or an explicitly approved bounded controlled-no-share soak, and added reject/accept unit tests.
**Verification:** Read back changed Rust sections and ran `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.

### WR-06: Frequency Transition Can Be Promoted Without A Hardware-Control Guard

**Files modified:** `tools/parity/src/main.rs`
**Commit:** 95c4d22
**Commit status:** fixed: requires human verification
**Applied fix:** Classified frequency transition as safety-critical, included `ASIC-007` in ASIC/mining and active-control validation, required bounded frequency-transition hardware-regression evidence, and added positive/negative unit tests.
**Verification:** Read back changed Rust sections and ran `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.

### WR-07: API Probe Failure Can Abort Instead Of Recording Pending Evidence

**Files modified:** `scripts/phase15-controlled-mining.sh`, `scripts/phase15-controlled-mining-test.sh`
**Commit:** 4e7f192
**Commit status:** fixed: requires human verification
**Applied fix:** Pre-created API probe temp files before curl, redacted only existing temp paths, logged pending curl failures, and added a live-prerequisite regression test where curl exits before writing output.
**Verification:** Read back changed shell sections, ran `bash -n` on both shell files, ran `shfmt -l -d`, and ran `bash scripts/phase15-controlled-mining-test.sh`.

## Skipped Issues

None - all findings were fixed.

_Fixed: 2026-07-01T05:19:51Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_
