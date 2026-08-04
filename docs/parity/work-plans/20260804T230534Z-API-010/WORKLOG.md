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

## 2026-08-04T23:25:36Z | bounded physical-screen stack path complete

- Source commit: `2fbee117` plan checkpoint.
- Actions: Replaced the physical screen's full API snapshot call path with a
  narrow atomic command projection plus direct read-only health, safety,
  Wi-Fi, catalog, build, and pool projections. Restored the unrelated boot
  observer to its original 8 KiB stack and integrated a fail-closed demangled
  ELF entry-frame audit into the canonical firmware build.
- Verification: The focused screen ownership regression failed before the
  implementation and passed afterward. The real packaged ELF reports 2,048
  bytes for `operator_sensor_runtime::run`, 960 bytes for
  `collect_screen_snapshot`, and 3,008 bytes combined against 3 KiB individual
  and 4 KiB combined limits. Formatting, strict Clippy, all-target/all-feature
  build and tests, Bright Builds, all 35 Bazel tests, canonical build/package,
  parity validation/progress, redaction, pinned-reference cleanliness,
  selector stability, immutable-plan, and diff checks passed.
- Evidence: Public source, deterministic tests, canonical build outcomes, and
  bounded stack-frame sizes only. No credential, device, network, or protected
  hardware value was read or published.
- Outcome: The root-cause fix is software-complete and ready for its clean
  pushed exact-package checkpoint.
- Blocker or next safe action: Commit and push this implementation, confirm a
  clean synchronized tree, then run the single contracted detector and
  conditional attempt-008 capture without retry.

## 2026-08-04T23:28:41Z | attempt-008 stopped at detector admission

- Source commit: `50287f62f7d04e09cad088798566edccb0663f2b`.
- Actions: Confirmed a clean synchronized tree, built the exact package, and
  ran the one protected attempt-008 detector. The conditional durability
  capture was not launched because detector admission failed.
- Verification: Packaging passed. Protected output reduced to the allowlisted
  terminal category `bootloader_connect_failed`; the private wrapper root is
  mode 0700, its detector output is mode 0600, and neither the private capture
  root nor public projection exists. The complete final software gate passed;
  one transient host resource error during the first parity invocation cleared
  on the bounded read-only rerun.
- Evidence: Only the exact source commit, package success, typed-safe terminal
  category, file-mode facts, and absence booleans are public. Port, physical
  identity, USB details, tool trace, and any device output remain private.
- Outcome: No theme mutation, software restart, recovery action, public
  evidence, `RESULT.md`, or checklist promotion occurred. `API-010` remains
  `implemented`.
- Blocker or next safe action: Stop without retry under the immutable plan.
  A later attempt requires a new task and immutable contract after the USB
  bootloader-connect condition is externally resolved.
