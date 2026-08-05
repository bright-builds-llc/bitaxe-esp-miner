# Parity work log

## 2026-08-05T00:53:20Z | bootloader diagnostic contract

- Source commit: `db0bef8fbdbf7cbffe64a4c015de1c7ea7b080fc`.
- Actions: Resumed only `API-010`, reduced attempt-008 through protected
  classification, and inspected the pinned espflash 4.5.0 reset/connect source.
- Verification: The exact current signature is generic espflash
  `connection_failed`, bootloader synchronization failed, enumeration remained
  unchanged, the same physical device stayed accessible and holder-free, and
  cleanup completed. Espflash source proves the underlying reset-strategy error
  is available only at debug level before the CLI collapses it.
- Evidence: Closed categories, booleans, safe source provenance, and the pushed
  attempt-008 record only. Protected child output and all device, port,
  USB/process identity, command, and path values remain private.
- Outcome: Private child-local debug capture plus a closed classifier can
  distinguish the remaining hypotheses without changing detector effects.
- Blocker or next safe action: Run the complete plan gate, commit and push this
  immutable plan/task checkpoint, then make the focused classifier loop red.

## 2026-08-05T01:05:00Z | immutable plan gate

- Actions: Ran the complete pre-plan software gate and confirmed the
  deterministic selector points only to this plan.
- Verification: Cargo format, clippy, build, and all-feature tests passed;
  Bright Builds passed; all 35 Bazel tests passed; parity, progress, semantic
  redaction, pinned-reference cleanliness, immutable-output, selector, and
  diff checks passed. The first parity report encountered the known macOS
  `Resource temporarily unavailable (os error 35)` transient; its single
  allowed read-only rerun passed.
- Outcome: The immutable task/plan checkpoint is eligible to commit and push
  before source changes.
- Blocker or next safe action: Commit and push this checkpoint, then make the
  focused bootloader diagnostic classifier test red.

## 2026-08-05T01:20:00Z | diagnostic red and green

- Actions: Added production-shaped pinned-espflash debug transcript tests,
  observed the required missing-type/API compile failure, implemented the
  closed connection signature classifier, and applied the child-local
  `espflash::connection=debug` filter only to detector-owned `board-info`.
- Verification: The tight test now passes four focused tests covering process
  timeout/interruption, all pinned connection variants, unavailable debug,
  safe public detail, operation/command gating, and a real child-process
  environment boundary with mode-0600 output. Complete device-session tests,
  package clippy, focused device-session/flash Bazel tests, format, and diff
  checks pass.
- Simplification review: Reused the existing private child streams and process
  supervisor. Closed connection signatures now precede the legacy transfer
  text heuristic, preventing a reset-stage serial error containing `write`
  from being misclassified as a post-transfer flash failure.
- Outcome: Raw debug stays private; public failure detail contains only a
  closed connection signature plus static recovery guidance.
- Blocker or next safe action: Run the complete repository gate, then commit
  and push the diagnostic implementation before the bounded hardware probe.

## 2026-08-05T01:55:00Z | complete implementation gate

- Actions: Ran the complete implementation gate. Bright Builds first rejected
  `usb/process.rs` at 641 lines against its 628-line ceiling, so the existing
  real-process tests moved mechanically into `usb/process/tests.rs` and Bazel
  source ownership was updated. A second macOS `os error 35` at parity then
  triggered re-planning and read-only host-capacity diagnosis.
- Verification: The focused classifier, complete device-session, dependent
  flash, package clippy, and exact file-length checks passed after the split.
  The restarted complete gate passed Cargo format, all-target/all-feature
  clippy, build, tests and doc tests; Bright Builds; all 35 Bazel tests;
  canonical package; parity and progress; semantic redaction; pinned-reference
  cleanliness; selector; immutable-output; and diff checks. Host diagnosis
  observed 751 processes against an 8,000 per-user limit, zero throttled
  memory pages, and one zombie; a fresh-process parity run passed.
- Outcome: The diagnostic implementation is software-complete and eligible to
  commit and push. No USB or other hardware command has run under this plan.
- Blocker or next safe action: Review the exact diff, commit and push, confirm
  clean synchronization, then run the one authorized protected detector.

## 2026-08-05T02:05:00Z | pushed diagnostic implementation

- Source commit: `520f4113`.
- Actions: Committed and pushed the typed diagnostic implementation to
  synchronized `main` after the complete gate passed.
- Outcome: The software gate in the immutable hardware contract is complete.
- Blocker or next safe action: Commit and push this post-push task checkpoint,
  confirm clean synchronization, then run the one authorized detector.

## 2026-08-05T02:15:00Z | terminal detector result

- Package source commit: `ff50e590f3d6a00c93b23b774c85739428134152`.
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- Actions: Rebuilt the exact clean package and ran the sole protected
  wrapper-009 detector. The detector failed, so the conditional capture was
  not launched.
- Protected reduction: Terminal category `bootloader_connect_failed`, closed
  signature `generic_connection_failure`, seven reset failures, seven
  underlying generic connection outcomes, zero specific connection variants,
  and zero boot-mode or download-mode observations. Both retry-admission and
  final-cleanup summaries saw the same accessible holder-free device with
  three stable samples and unchanged enumeration. The private root contained
  six mode-0600 files and no surviving journal.
- Upstream comparison: Pinned espflash 4.5.0 and managed esptool 4.12.0 use the
  same USB-JTAG/Serial DTR/RTS reset sequence. Espressif documents that this
  interface should enter download mode automatically and identifies manual
  boot-mode entry as the recovery when automatic reset cannot do so.
- Privacy/effects: Wrapper mode 0700 and stdout mode 0600 passed. No raw trace,
  port, device identity, network value, credential, or command was promoted.
  No flash, erase, factory reset, theme mutation, restart request, mining, or
  hardware-control effect occurred. No capture root or public projection was
  created.
- Outcome: `API-010` remains `implemented`; evidence and `RESULT.md` are
  withheld. NVS/factory reset is ruled out because the failure precedes every
  flash or application-setting boundary.
- Terminal blocker: Automatic USB control did not establish a ROM download
  session. Further recovery requires an external normal-connector state change
  or manual boot-mode hardware intervention under a new authorization and
  immutable attempt contract. No retry is eligible in this plan.
