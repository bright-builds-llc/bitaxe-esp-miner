# Parity work log

## 2026-08-11T23:49:07Z | Attempt-001 plan checkpoint

- Source commit: `cd87164dc17db9977dbe7c3dd7707723f3582fa7`.
- Actions: Selected only `CFG-001` and traced the live default path from the
  pinned CSV through the pure config model, flash NVS generation, firmware
  settings load, system-info projection, and typed evidence boundary.
- Verification: The branch and reference are clean, upstream is synchronized,
  the selector reports no open plan, and `CFG-001` is the first candidate.
  Source inspection confirms the ordinary Wi-Fi image currently omits the
  exact Ultra 205 configured-default seed.
- Evidence: Source inspection and existing unit/golden fixtures only. No new
  detector, credential read, hardware effect, or public evidence exists.
- Outcome: A minimal root-cause plan now binds one safety-paused default seed,
  one closed firmware attestation, and one reused system-info transaction.
- Blocker or next safe action: Run the complete plan-only software gate, commit
  and push the immutable plan/task, then implement without editing `PLAN.md`.

## 2026-08-12T00:02:31Z | Plan-only gate passed

- Source commit: `cd87164dc17db9977dbe7c3dd7707723f3582fa7`.
- Actions: Ran the repository's complete software-only pre-commit gate and
  selected `CFG-001` through the canonical parity selector.
- Verification: Ordered Cargo format, lint, build, and tests passed; Bright
  Builds and all Bazel tests passed. The first `just parity` launch met a
  transient host `Resource temporarily unavailable` condition after the build
  quiesced; a single boundary retry then passed with progress at 46/94, as did
  redaction and pinned-reference verification. No code or policy failure was
  observed.
- Evidence: Software command results only. No detector, credentials, hardware
  effect, or public evidence was used.
- Outcome: The immutable plan and complete active task contract are eligible
  to be committed and pushed before implementation.
- Blocker or next safe action: Commit and push this plan/task checkpoint, then
  implement the default seed and typed evidence path without editing `PLAN.md`.

## 2026-08-12T00:06:45Z | Implementation checkpoint

- Source commit: `e0d029cb`.
- Actions: Added the exact typed 27-field Ultra 205 seed to the ordinary Wi-Fi
  NVS image with `mineonboot=false`, a closed firmware attestation, and a typed
  evidence workflow that reuses the existing exact-package system-info
  transaction and independently validates its public projection.
- Verification: Focused config, flash, Rust contract, firmware source-ownership,
  automation, and real-child-process tests pass. Generated TypeScript contracts
  are byte-identical, the diff is whitespace-clean, and the new source files
  remain below the managed file-length threshold.
- Evidence: Software-only regression results. No detector, credential read,
  hardware effect, or public hardware evidence exists yet.
- Outcome: The root implementation and closed privacy boundary are ready for
  the complete ordered software gate.
- Blocker or next safe action: Run and pass every mandatory software gate,
  review the final diff, then commit and push before the one bounded detector-
  gated attempt.

## 2026-08-12T00:10:36Z | Firmware compile correction

- Source commit: `e0d029cb`.
- Actions: The first full `just test` reached the ESP firmware build and
  exposed one borrow-check error in the new strict settings reader. Reordered
  the local read so the borrowed key is no longer live when its owned schema
  key moves into the snapshot.
- Verification: Reproduced the exact ESP-IDF Cargo command with its generated
  provenance inputs, observed `E0505`, applied the minimal ownership fix, and
  reran the same ESP32-S3 release build successfully.
- Evidence: Compiler diagnostics and the successful local firmware artifact
  build only. The earlier Bazel run's 37 test targets all passed; no detector,
  credential read, or device effect occurred.
- Outcome: The root implementation now compiles for the real firmware target.
- Blocker or next safe action: Restart the complete ordered software gate from
  format because implementation changed, and do not proceed to hardware until
  every command is green and the exact commit is pushed.

## 2026-08-12T00:14:57Z | Complete software gate passed

- Source commit: `e0d029cb` plus the uncommitted CFG-001 implementation.
- Actions: Completed the mandatory ordered Cargo, Bright Builds, Bazel,
  parity, progress, redaction, reference, selector, immutable-plan, task,
  freshness, reference-cleanliness, and diff checks after the firmware fix and
  final typed-contract review.
- Verification: Format, warnings-denied clippy, all-target build, all-feature
  tests, Bright Builds, all 37 Bazel tests, both ESP firmware variants, parity,
  progress, redaction, and reference checks pass. The first post-build parity
  process met the already documented host resource-spawn limit; its single
  quiescent boundary retry passed with `validation_errors: none` and progress
  `46/94`. The immutable plan digest remains
  `5420cdb7d911e1f98492060a2d30223244aaecf740283df38c6174a12353694c`.
- Evidence: Complete software verification only. Fresh attempt paths remain
  absent and no USB detector, credentials, or hardware effect has run.
- Outcome: The implementation is eligible for its exact commit and push.
- Blocker or next safe action: Commit and push without changing the checklist,
  then build that exact commit and run the one detector-gated attempt-001.
