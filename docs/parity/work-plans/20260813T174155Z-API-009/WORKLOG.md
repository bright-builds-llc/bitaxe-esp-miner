# Parity work log

## 2026-08-13T17:41:55Z | deadline-contract diagnosis

- Source commit: `0a7b239877642adf5f33684402dd92344de88f35`.
- Actions: Closed and pushed attempt-012, re-ran the clean selector, and
  compared the parent process timeout with every bounded child phase.
- Verification: The 810-second parent literal is only 30 seconds above the
  child's 600-second observation plus 180-second terminal grace, while bounded
  flash, NVS, USB recovery, monitor admission, cleanup, and process termination
  also consume the parent budget. The timeout branch does not call the existing
  recovery-fact reader.
- Evidence: Public source and redaction-safe attempt-012 closure facts only. No
  protected artifact, credential, detector, USB, device, or network access.
- Outcome: Root cause confirmed as an orchestration deadline contract that is
  shorter than the complete child transaction.
- Blocker or next safe action: Verify, commit, and push this immutable
  software-only plan before editing the timeout owner.

## 2026-08-13T17:46:00Z | immutable-plan verification

- Plan SHA-256:
  `89f3013e6da22e7210a430f4e4ca4bf840463c564c7aa737d078ada4ab7363a4`.
- Verification: Formatting, strict Clippy, all-target build, all-feature tests,
  Bright Builds, canonical Bazel tests, parity, parity-progress, redaction,
  reference cleanliness, real ESP firmware build, and diff checks pass. The
  selector reports only this API-009 plan and the active task binds it once.
- Outcome: The software-only deadline plan is ready to commit and push.
- Blocker or next safe action: Push this checkpoint before editing code; no
  hardware-capable action is authorized.

## 2026-08-13T18:02:00Z | deadline contract verified

- Actions: Replaced the fixed parent/fixture timeouts with one checked typed
  transaction budget. Bound it to the Rust child source limits, routed both
  process guards through it, and reused the private recovery-fact reader on
  outer timeout without changing primary precedence.
- Verification: Unit arithmetic and overflow checks, a scaled real-child
  cleanup-before-parent regression, cross-language source-contract assertions,
  and closed/missing/malformed timeout-recovery cases pass. Focused automation
  and fixture targets pass. Formatting, strict Clippy, all-target build,
  all-feature tests, Bright Builds, canonical Bazel tests, parity,
  parity-progress, redaction, reference, real ESP build, and diff checks pass.
- Evidence: Public source/tests and redaction-safe pass/fail facts only. No
  credential, protected evidence, detector, USB, device, or network access.
- Outcome: `software_fix_complete`. API-009 remains `implemented`; no parity
  transition or hardware evidence is claimed.
- Blocker or next safe action: Close and push this software plan. Any future
  attempt-013 requires a separate clean selector and immutable contract.
