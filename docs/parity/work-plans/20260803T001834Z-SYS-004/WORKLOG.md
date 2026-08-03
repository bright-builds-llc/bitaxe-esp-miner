# Parity work log

## 2026-08-03T00:18:34Z | static-version root-cause investigation

- Source commit: `6c377f4d7439d4cedb1162899f93647577c1a478`
- Actions: Selected the first deterministic candidate, traced upstream version
  initialization and the Rust package/runtime path, and defined one bounded
  exact-package hardware contract.
- Verification: Clean synchronized `main`; pinned reference
  `c1915b0a63bfabebdb95a515cedfee05146c1d50`; `next-item` returned no open plan
  and `SYS-004` first at `implemented`. Upstream `SYSTEM_init_versions` reads
  `/version.txt`; Rust package input contains no `version.txt`, while
  `platform_identity.rs` returns the generic `assets/release.json` name.
- Evidence: `reference/esp-miner/main/system.c`,
  `reference/esp-miner/main/http_server/system_api_json.c`,
  `firmware/bitaxe/src/platform_identity.rs`,
  `firmware/bitaxe/static/www/assets/release.json`, and
  `scripts/package-firmware.sh`.
- Outcome: The remaining static-semantics gap is an implementation defect, not
  merely missing evidence. The targeted fix must make the installed SPIFFS
  payload own the reported version before consuming the one hardware attempt.
- Blocker or next safe action: Run all plan pre-commit gates, commit and push
  this immutable plan/task contract, then implement the software fix without
  exercising hardware.

## 2026-08-03T00:36:00Z | targeted package/runtime repair

- Source commit: implementation worktree based on
  `033002d10dd77ca4be76c70203aa79156724a51a`.
- Actions: Replaced the embedded fallback UI-name projection with a mounted,
  package-owned version marker; added strict pure parsing; staged the marker
  without mutating checked-in assets; and added a typed exact-package private
  capture classifier with a closed public schema.
- Verification: Focused Cargo API and parity tests passed; the package shell
  regression proved byte-exact `version.txt` content; focused Clippy passed;
  and Bazel `//tools/parity:tests` plus `//scripts:package_firmware_test`
  passed. One initial Bazel invocation named the package test target
  incorrectly and performed no tests; the corrected target passed.
- Evidence: `crates/bitaxe-api/src/platform_identity.rs`,
  `firmware/bitaxe/src/platform_identity.rs`, `scripts/package-firmware.sh`,
  `scripts/package-firmware-test.sh`, and
  `tools/parity/src/sys004_version_evidence.rs`.
- Outcome: The root defect is repaired at the package/runtime ownership
  boundary, and stale or mismatched live/static versions now fail closed.
- Blocker or next safe action: Run every mandatory repository gate, review the
  complete diff, commit and push the clean software source, then create the
  exact package and consume the single detector-gated hardware attempt.

## 2026-08-03T00:50:00Z | software verification gate

- Source commit: implementation worktree based on
  `033002d10dd77ca4be76c70203aa79156724a51a`.
- Actions: Ran the exact ordered Rust pre-commit sequence, the complete Bazel
  test graph, Bright Builds checks, parity validation/progress, redaction,
  reference cleanliness, and whitespace validation.
- Verification: Formatting, Clippy with warnings denied, all-target/all-feature
  build, all-feature Cargo tests, Bright Builds with zero findings, and all 82
  Bazel tests passed. Parity remains 34/94 verified (36.2%), redaction passed,
  and the pinned reference is clean. The first parity report completed its
  report but its process ended with transient host `os error 35`; an unchanged
  immediate rerun completed with `validation_errors: none`, followed by every
  remaining gate passing.
- Evidence: Local command results plus the focused entries above; no generated
  package from the dirty verification worktree is admissible for hardware.
- Outcome: The targeted software repair is ready for a clean source commit.
- Blocker or next safe action: Review and commit the exact diff, push `main`,
  then build a fresh clean package before detector or hardware use.

## 2026-08-03T01:02:00Z | attempt-001 detector stop

- Source commit: `0a4475f232cc7d944e69c6425955994bbfc12a9e`.
- Actions: Built the clean exact package, passed the standalone Ultra 205
  detector, issued the Phase 36 preflight capability, consumed attempt 001,
  and invoked the contracted SYS-004 projection.
- Verification: Package label `0a4475f232cc-dev` matched the clean source. The
  broker ledger reports exact-package admission complete, board-205 detector
  `detector_failed`, cleanup complete, no secondary failure, recovery not
  authorized, and no flash/candidate/private capture. The projection rejected
  the absent capture at its private boundary. No credential content or raw USB
  or network value was exposed or committed.
- Evidence: Protected mode-`0700` attempt-001 root with mode-`0600` handle,
  append-only ledger, and `phase36-attempt-seal-v2`; categorical facts only are
  recorded here.
- Outcome: `stop_hardware_blocker` does not apply because the independent
  detector passed and the failure is a source-confirmed broker parser defect.
  `SYS-004` remains `implemented`; no checklist or progress field changed.
- Root cause: `tools/flash detect` canonically renders `port: <value>`, but
  `ProcessTransactionBoundary::run_detector` searches only for `port=<value>`.
  Existing tests mock the detector boundary and never exercise its stdout
  grammar, allowing the mismatch to survive.
- Blocker or next safe action: Commit the separate attempt-002 contract, then
  implement a pure exact-output parser with negative grammar regressions. No
  retry is allowed until the fix passes all gates and is cleanly pushed.

## 2026-08-03T00:47:00Z | broker detector parser repair

- Source commit: implementation worktree based on
  `2e2f30a83a802bf7f11d65baa0b50b1972c0c84a`.
- Actions: Added one pure detector stdout parser, reused it in both Phase 36
  process gates, and removed the duplicated incorrect `port=` extraction.
- Verification: Focused tests accept exactly one absolute or numbered COM
  `port: ` line joined to exactly one `usb_session: ready` marker and reject
  the obsolete spelling, missing readiness, duplicate/empty/relative ports,
  incomplete COM ports, and invalid UTF-8. Focused Clippy passed with warnings
  denied.
- Evidence: `tools/parity/src/phase36_broker/hardware.rs`,
  `tools/parity/src/phase36_broker/hardware/tests.rs`, and
  `tools/parity/src/phase36_broker/hardware_process/process_boundary.rs`.
- Outcome: The source-confirmed attempt-001 boundary has a minimal
  regression-backed repair; no device interaction occurred during the fix.
- Blocker or next safe action: Run every mandatory gate, commit and push the
  corrected source, then build the clean attempt-002 package.

## 2026-08-03T00:49:00Z | attempt-002 stop and final invocation fix

- Source commit: `9f4d56700c42a318e1aef61ee99bffcaf06e4231`.
- Actions: Built the clean package, passed standalone detection and preflight,
  consumed the single authorized retry, ran the typed projection, and traced
  the repeated pre-flash detector stop to nested process construction.
- Verification: Attempt 002 again records exact-package admission complete,
  detector failure, cleanup complete, no recovery authority, no secondary
  failure, no flash, and no capture/candidate. The projection correctly
  rejected the absent private capture. Focused tests prove the broker now sets
  nested `just` to Bazel's workspace directory while retaining the exact
  command and argument.
- Evidence: Protected attempt-002 categorical ledger/seal plus
  `tools/parity/src/phase36_broker/hardware.rs` and its focused tests. No raw
  USB, Wi-Fi, or network data is committed.
- Outcome: The root-cause chain now has two targeted fixes: canonical detector
  output parsing and correct Bazel workspace process context. No device effect
  occurred in either attempt. `SYS-004` remains conservatively `implemented`.
- Terminal blocker: The task contract explicitly forbids a third ordinal, so
  the corrected broker cannot produce exact-package live version evidence in
  this invocation. Commit and push the final software fix, but do not retry,
  transition the checklist, append progress, create `RESULT.md`, or archive
  the unresolved task.

## 2026-08-03T00:58:00Z | final software verification

- Source commit: implementation worktree based on
  `9f4d56700c42a318e1aef61ee99bffcaf06e4231`.
- Actions: Completed the mandatory ordered Rust checks and every repository
  verification gate after the workspace-directory repair.
- Verification: `cargo fmt --all`, warning-denied all-target/all-feature
  Clippy, all-target/all-feature Cargo build, and all-feature Cargo tests
  passed in order. All 82 Bazel tests passed. Bright Builds reported zero
  findings; parity validation reported no errors; progress remained 34 of 94
  active rows verified (36.2%); and redaction, reference integrity, plus diff
  checks passed.
- Evidence: Focused command-construction regression and the complete local
  gate outputs. No hardware command was rerun and no private observation was
  promoted.
- Outcome: The minimal process-context fix is ready to commit and push.
  `SYS-004` remains `implemented`, and its task remains active and unarchived.
- Residual risk: Live exact-package HTTP/WebSocket equality is still unproved.
  The sole blocker is the consumed retry contract; a future explicit attempt
  ordinal is required to cross that evidence boundary.

## 2026-08-03T02:44:37Z | attempt-003 authorization

- Source commit: `f369dbde0cc689b6dc8cd4c76b9fd4fe45d5ad71`.
- Actions: Resumed the only open parity plan after the user explicitly
  authorized `SYS-004` attempt 003, and recorded one exact corrected-broker
  hardware contract in the active task.
- Verification: The worktree is clean on synchronized `main`; the pinned
  reference remains `c1915b0a63bfabebdb95a515cedfee05146c1d50`; and
  `next-item --format json` resumes this plan with no alternate candidates.
  The ordered Rust sequence, all 82 Bazel tests, Bright Builds with zero
  findings, parity and unchanged 34/94 progress, redaction, reference
  integrity, and diff checks all passed.
- Evidence: The task records the five exact commands, private paths and modes,
  safe no-mining effect boundary, typed recovery, cleanup, retry limit, closed
  terminal outcomes, and exact promotion criteria.
- Outcome: Attempt 003 is authorized and software-verified but not yet
  admissible until this contract is cleanly committed and pushed. No detector,
  credential, package, flash, serial, HTTP, WebSocket, or hardware action has
  occurred in this continuation.
- Blocker or next safe action: Commit and push this contract, then build the
  clean exact package before the first detector.
