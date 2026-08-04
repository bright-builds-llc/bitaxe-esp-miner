# Parity work log

## 2026-08-04T23:05:34Z | attempt-008 root-cause remediation contract

- Source commit: `bfcd42343666cf8f347df7888bc7085859a93980`.
- Actions: Resumed only `API-010` after the pushed attempt-007 stop and reduced
  the protected boot loop to an exact source and ELF call-path violation.
- Verification: Attempt-007 proves the exact flash completed, every observed
  reset was a stack-overflow panic, startup reached the rendered operator
  display, and the 8 KiB operator-sensor task's 2 KiB frame called a 7,872-byte
  full API-snapshot frame before Wi-Fi startup.
- Evidence: Closed categories, safe booleans, bounded counts, stack-frame
  sizes, public source provenance, and the pushed prior checkpoint only.
  Protected trace material and local hardware/network values remain private.
- Outcome: A narrow screen-only projection plus a canonical binary stack audit
  is actionable; another stack increase is explicitly rejected.
- Blocker or next safe action: Verify, commit, and push this immutable
  plan/task checkpoint before editing firmware, automation, or build files.

## 2026-08-04T23:12:00Z | pre-implementation plan gate complete

- Source commit: `bfcd42343666cf8f347df7888bc7085859a93980`.
- Actions: Ran the complete repository gate against the new immutable
  attempt-008 plan and active task without editing implementation files.
- Verification: Formatting, strict Clippy, all-target/all-feature build, all
  Cargo tests, Bright Builds, all 35 Bazel tests, parity validation/progress,
  semantic redaction, pinned-reference cleanliness, immutable-plan, selector,
  and diff checks passed. One transient host resource error during the first
  parity invocation cleared on the bounded read-only rerun.
- Evidence: Public software outcomes only; no new package, detector, device,
  credential, or hardware action occurred.
- Outcome: The attempt-008 remediation contract is ready to commit and push
  without amendment.
- Blocker or next safe action: Commit and push this checkpoint, then turn the
  existing screen source regression red before implementation.
