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
