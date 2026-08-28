# Parity work log

## 2026-08-28T18:59:00Z | Plan checkpoint

- Source commit: `478fed4d25a38d1d87cf70edf9a5c40b0f183614`
- Actions: selected STR-005 after skipping hardware-blocked ASIC-009 and
  ASIC-010; wrote the immutable TCP-payload plan and complete task contract.
- Verification: ordered Rust gates, isolated full Cargo tests, Bright Builds,
  all 56 Bazel tests, parity, progress, redaction, reference, whitespace, and
  diff review passed. The first post-suite parity launch hit transient host
  `EAGAIN`; its isolated rerun passed with `validation_errors: none`.
- Evidence: plan/task commit `5c7d6f46a3f1414fdba968322e0688365879b519`
  was pushed to `origin/main` before implementation edits.
- Outcome: immutable planning checkpoint accepted.
- Blocker or next safe action: implement and software-verify the exact
  diagnostic contract before any detector, network, credential, or device
  effect.

## 2026-08-28T19:20:00Z | Implementation checkpoint

- Source commit: `5c7d6f46a3f1414fdba968322e0688365879b519`
- Actions: added consume-once TCP diagnostic admission, fixed 64-byte firmware
  writer, exact-payload fixture mode, plan-bound flash command, protected outer
  supervisor, closed marker/projection validator, and CLI/Bazel/Just wiring.
- Verification: focused Cargo compile/tests and Bazel automation/flash tests
  are in progress; macOS `_dyld_start` stalls were rerouted to isolated Cargo
  targets and executed successfully.
- Evidence: implementation remains uncommitted; no hardware or network effect
  has occurred.
- Outcome: software implementation in progress.
- Blocker or next safe action: finish focused regressions, evaluator/privacy
  closure, canonical package, and every mandatory gate before source commit.

## 2026-08-28T19:47:00Z | Software verification complete

- Source commit: `5c7d6f46a3f1414fdba968322e0688365879b519`
- Actions: completed the fixed-payload firmware, fixture, flash, protected
  supervisor, independent validator, transitive evaluator inventory, and
  command/build wiring. Split the fixture TCP reader after the managed
  file-length gate identified the module boundary.
- Verification: formatting, strict Clippy, all-target/all-feature build,
  isolated full Cargo tests, focused real-loopback TCP coverage, Bright Builds,
  all 57 Bazel tests, canonical six-artifact ESP32-S3 package, parity with no
  validation errors, parity progress, redaction, reference cleanliness,
  whitespace, and diff checks passed. Two immediate post-suite parity launches
  encountered transient host `EAGAIN`; isolated reruns passed.
- Evidence: software behavior only; no detector, credential, network, USB,
  flash, monitor, or device effect occurred.
- Outcome: implementation and evidence contract are ready for a distinct source
  commit and clean push.
- Blocker or next safe action: commit/push this source state, rebuild the exact
  clean package, then run detector admission and at most diagnostic-001.

## 2026-08-28T20:02:00Z | Diagnostic-001

- Source commit: `f2e559391f78e4f073b5e911fce999730c60cdd1`
- Actions: built the exact clean package, privately captured successful
  one-board detector admission, and invoked the sole diagnostic-001 workflow.
- Verification: protected artifacts confirm the fixture rejected its
  360-second session timeout before creating the listener root; no diagnostic
  child or public projection exists.
- Evidence: private root `scratch/str005-tcp-payload/diagnostic-001`; earliest
  signature `timeout:fixture_ready`; `effect_started=false` and
  `projection_published=false`.
- Outcome: `continue_after_verified_fix`.
- Blocker or next safe action: bind fixture session timeout to 120 seconds in a
  production argument-constructor regression, move to fresh diagnostic-002,
  run every gate, commit/push, repackage, redetect, and invoke once. A recurrence
  of this signature stops the task.

## 2026-08-28T20:22:00Z | Diagnostic-002 and terminal closure

- Source commit: `35ae9cb33458ad6c76f6eedef5d0538720d80367`
- Actions: rebuilt the exact clean package, captured fresh successful detector
  admission, and invoked diagnostic-002 once after the verified fixture-bound
  fix.
- Verification: the protected fixture terminal records `payload_read`, zero of
  64 bytes, no payload match, and zero extra bytes. The outer workflow then
  stopped at `hardware_blocked:restoration`; no public projection exists.
- Evidence: private roots `scratch/str005-tcp-payload/diagnostic-002` and
  `scratch/str005-tcp-payload/detector-002`; no protected content is promoted.
- Outcome: `stop_hardware_blocker`. TCP delivery and exact restoration are both
  unverified; STR-005 remains `implemented | unit,golden,workflow`.
- Blocker or next safe action: a fresh recovery-only task and immutable plan
  must add current-root restore admission and prove recovery-006 identity,
  settings, inactive zero-work runtime, and cleanup before any later TCP
  diagnostic plan. Diagnostic-003 is not authorized.

## 2026-08-28T20:08:00Z | Iterative recovery debugging resumed

- Source commit: `a3a490c24e7fdad71cd107d2c0ec244fe99faf86`
- Actions: resumed the active task under the user's at-will hardware-fix
  authorization; built a focused restore-admission feedback loop before
  changing production code.
- Verification: `cargo test -p bitaxe-flash
  tcp_payload_recovery_authority_is_current_and_narrow --all-features` failed
  red with `restore_installed=blocked reason=identity_contract`, then passed
  after adding only the current plan/action/ordinal and recovery root. A second
  focused test proves the root maps only to the current remediation plan.
- Evidence: software-only; no additional hardware effect occurred.
- Outcome: hypotheses 1 and 2 confirmed. The restore allowlist and root selector
  omitted the decomposed task; old authorization documents also cannot be
  reused because source identity must rotate.
- Blocker or next safe action: finish the recovery-only supervisor that creates
  fresh authorization, restores package/settings, and confirms inactive
  zero-work runtime; run all gates, commit/push, then execute recovery-001 once.

## 2026-08-28T20:25:00Z | Recovery-001

- Source commit: `bdb35427782d1b89c261460895456cf0d2f5355c`
- Actions: built the exact clean package, captured fresh detector admission,
  and invoked recovery-001 without a fixture or payload effect.
- Verification: the protected restore stderr reports only
  `restore_installed=blocked reason=authorization_action`; no restore command
  receipt or recovery result exists.
- Evidence: private root `scratch/str005-tcp-payload/recovery-001`; no protected
  values or artifacts are promoted.
- Outcome: `continue_after_verified_fix` before flash writes.
- Blocker or next safe action: the second inline action/root allowlist omitted
  the new recovery action. The focused tuple test reproduced red, then passed
  after the exact current-root arm. Rotate to recovery-002, run all gates,
  commit/push/package, redetect, and invoke once.

## 2026-08-28T20:38:00Z | Recovery-002 and diagnostic completeness

- Source commit: `41d19ed6150e189ff213848470761454ec1dbfe9`
- Actions: ran recovery-002 from a fresh detector admission, then built a
  red-capable real-child loop for the missing diagnostic output.
- Verification: recovery-002 returned `accepted` with exact recovery-006
  identity/settings, `mineonboot=false`, inactive zero-work/share state, final
  detector admission, and cleanup. The timeout child test then failed because
  partial stdout/stderr were discarded and passed after the managed error began
  retaining the bounded streams.
- Evidence: private root `scratch/str005-tcp-payload/recovery-002`; no protected
  values are promoted.
- Outcome: baseline restoration complete; diagnostic completeness fix verified
  at the real child-process boundary.
- Blocker or next safe action: bind diagnostic-003 and its exact restoration
  root, run every gate, commit/push/package, redetect, and invoke once to obtain
  closed firmware stage/timing/terminal discriminators for the zero-byte TCP
  boundary.

## 2026-08-28T20:56:00Z | Diagnostic-003

- Source commit: `1cd7ff0926f8d5e7db7a930c1997d2ee2756809d`
- Actions: built the exact package, captured fresh detector admission, and ran
  diagnostic-003 with timeout stream retention and current-task restoration.
- Verification: the fixture never accepted a peer and retained firmware stages
  were all false. Partial child output proved command rendering and USB session
  ownership but no monitor command. Sanitized stderr classified the supervised
  factory flash as `flash_failed_before_transfer`. Exact restoration fields all
  passed; the private projection candidate was rejected only at final evidence
  admission and no public projection was emitted.
- Evidence: private root `scratch/str005-tcp-payload/diagnostic-003`; no
  protected values are promoted.
- Outcome: distinct pre-TCP `continue_after_verified_fix`; device restored.
- Blocker or next safe action: persist the closed USB diagnostic, stop the
  fixture immediately on pre-monitor failure, rotate to diagnostic-004, run all
  gates, commit/push/package, redetect, and invoke once. A repeated
  flash-before-transfer signature stops.

## 2026-08-28T21:12:45Z | Diagnostic-004 and receipt-bound fix

- Source commit: `e6e22f29671394c8286c07860becfb38d7c20b8a`
- Actions: ran diagnostic-004 from fresh one-board admission, retained the
  complete closed TCP boundary, and built a receipt-bound discriminator for the
  zero-byte result.
- Verification: firmware reached monitor armed, resolve, exact-peer connect,
  and payload-buffer write; connect was 158 ms and the local write reported 0
  ms. The exact-peer fixture accepted the connection but timed out with zero of
  64 bytes and no extras. Exact recovery-006 identity/settings, inactive
  zero-work state, and cleanup all passed. Focused Cargo and Bazel tests now
  prove a fixed `0xa5` receipt round-trip after exact payload validation, exact
  diagnostic-005 restoration authority, source ownership, and acceptance of a
  complete evidence chain when only bounded monitor capture times out.
- Evidence: private root `scratch/str005-tcp-payload/diagnostic-004`; the
  rejected private candidate retained closed categories and no protected value
  is promoted.
- Outcome: distinct post-write `continue_after_verified_fix`. The strongest
  current hypothesis is immediate socket destruction before lwIP transmitted
  its queued buffer; the receipt keeps ownership alive until fixture receipt.
- Blocker or next safe action: finish every mandatory gate, commit/push and
  package the exact diagnostic-005 contract, redetect, then invoke
  diagnostic-005 once. The fixed receipt does not enter Noise, V2 protocol,
  ASIC control, or mining.

## 2026-08-28T21:18:00Z | Diagnostic-005 software verification

- Source commit: `e6e22f29671394c8286c07860becfb38d7c20b8a`
- Actions: completed the receipt-gated firmware/fixture path, closed projection
  fields, exact fifth-ordinal command/restoration authority, and bounded
  monitor-timeout acceptance predicate.
- Verification: formatting, strict Clippy, all-target/all-feature build, full
  Cargo tests, Bright Builds, all 57 Bazel tests, canonical six-artifact
  ESP32-S3 package, parity with no validation errors, parity progress,
  redaction, reference cleanliness, focused TCP/restore/source-ownership tests,
  whitespace, and staged diff review passed.
- Evidence: software and loopback behavior only at this checkpoint; no new
  detector, credential, network, USB, flash, monitor, or device effect.
- Outcome: exact diagnostic-005 source and evidence contract are ready for a
  distinct commit and clean push.
- Blocker or next safe action: commit/push, rebuild the exact clean package,
  capture fresh one-board detector admission, then run diagnostic-005 once.

## 2026-08-28T21:33:49Z | Diagnostic-005 and TCP half-close discriminator

- Source commit: `b52a5261b85a6c983f6361699460dfbf5ac589af`
- Actions: built the exact clean package, captured fresh one-board admission,
  ran diagnostic-005 once, allowed exact restoration to finish, and returned to
  a red production-seam regression before changing the transport sequence.
- Verification: firmware reached monitor armed, resolve, exact-peer connect,
  payload-buffer write, and then the new receipt failure. The exact-peer fixture
  timed out with zero of 64 bytes, no extras, and no receipt. Recovery-006
  identity/settings, `mineonboot=false`, inactive zero-work runtime, USB
  cleanup, and zero owned processes all passed. No public projection was
  published. The source-ownership test then failed red because neither
  `TCP_NODELAY` nor write half-close preceded the receipt read, and passed after
  that exact sequence was implemented. Real loopback proves payload, write
  half-close, EOF, and fixed receipt on the remaining read direction.
- Evidence: protected roots `scratch/str005-tcp-payload/diagnostic-005` and
  `scratch/str005-tcp-payload/detector-diagnostic-005`; no protected values are
  promoted.
- Outcome: receipt-only hypothesis stopped at a repeated zero-byte boundary.
  The next distinct hypothesis is that this ESP-IDF socket needs an explicit
  one-way completion boundary so queued bytes and FIN precede the receipt wait.
- Blocker or next safe action: finish every gate, commit/push/package the exact
  diagnostic-006 contract, redetect, and invoke once. No Noise, V2 message,
  ASIC, mining, fan, voltage, or other hardware effect is admitted.

## 2026-08-28T21:38:43Z | Diagnostic-006 software verification

- Source commit: `b52a5261b85a6c983f6361699460dfbf5ac589af`
- Actions: completed the exact sixth-ordinal command/restoration contract,
  half-close stage/projection validation, and production/fixture regressions.
- Verification: formatting, strict Clippy, all-target/all-feature build, full
  Cargo tests, Bright Builds, all 57 Bazel tests, canonical six-artifact
  ESP32-S3 package, parity with no validation errors, parity progress,
  redaction, reference cleanliness, whitespace, and diff review passed. The
  first full Bazel run exposed a 50 ms child-start test race; widening only that
  test deadline to 500 ms passed three repeated focused runs and the full suite.
- Evidence: software and loopback behavior only at this checkpoint; no new
  detector, credential, network, USB, flash, monitor, or device effect.
- Outcome: diagnostic-006 is ready for a distinct clean source checkpoint.
- Blocker or next safe action: commit/push, rebuild the exact clean package,
  capture fresh one-board detector admission, then invoke diagnostic-006 once.
