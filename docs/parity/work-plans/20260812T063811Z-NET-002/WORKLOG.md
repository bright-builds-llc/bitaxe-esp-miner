# Parity work log

## 2026-08-12T06:38:11Z | selection and immutable plan

- Source commit: `41a88e98a997c8bb81e12821bd3c206b7c25dc24`.
- Actions: Selected the first canonical candidate, linked the continuation to
  the typed attempt-003 closure, and designed exact-device private candidate
  derivation plus a recurring redaction-safe AP readiness marker.
- Evidence: Repository source, the protected detector transaction, pinned
  ESP-IDF 5.5.4 MAC allocation documentation, and the generated four-universal-
  address SDK configuration. No new detector, USB, host-network, credential,
  NVS, DNS/HTTP, or device effect occurred.
- Outcome: Plan drafted; software gates pending.
- Blocker or next safe action: Run the complete plan-only gate, commit and push
  the immutable plan, then implement without editing `PLAN.md`.

## 2026-08-12T06:41:00Z | plan-only gate complete

- Source commit: `41a88e98a997c8bb81e12821bd3c206b7c25dc24`.
- Verification: Ordered Cargo format, strict Clippy, all-target build,
  all-feature tests, Bright Builds, all 37 Bazel tests, parity progress,
  redaction, reference, selector, task uniqueness, immutable-plan digest,
  reference cleanliness, fresh-path, and diff checks pass.
- Evidence: Plan SHA-256 is
  `48796a1c9bdbbce5fbe3b8f07ae7c34ac6f2a6069396d081321b135e6e569877`.
  No detector, credential, NVS, USB, host-network, DNS/HTTP, or device effect
  occurred.
- Outcome: The immutable plan is eligible for commit and push.
- Blocker or next safe action: Commit and push, then implement the detector
  binding and recurring readiness marker without editing `PLAN.md`.

## 2026-08-12T06:50:57Z | implementation gate complete

- Source commit: plan commit `5279ea6c`.
- Actions: The detector now captures one base MAC inside its owned board-info
  transaction, derives the pinned ESP-IDF SoftAP candidate with checked
  arithmetic, and emits it only in the protected handoff. Firmware publishes
  and replays a closed AP/DHCP/DNS readiness marker after both netif and captive
  DNS startup. The client joins that exact candidate even when macOS inventory
  omits it, while any different Bitaxe candidate fails closed.
- Verification: Focused API, firmware ownership, flash, automation, real-child,
  and real ESP32-S3 firmware-package targets pass. Ordered Cargo format, strict
  Clippy, all-target build, all-feature tests, Bright Builds, all 37 Bazel
  tests, parity, progress, redaction, reference, generated-contract, selector,
  immutable-plan, task, fresh-path, reference-cleanliness, sensitive-output,
  and diff checks pass. One parity process immediately after the full Bazel run
  hit transient macOS resource exhaustion; the isolated full tail then passed
  without a code change. Plan SHA-256 remains
  `48796a1c9bdbbce5fbe3b8f07ae7c34ac6f2a6069396d081321b135e6e569877`.
- Evidence: Software fixtures and protected categorical facts only. Candidate,
  device, interface, address, credential, DNS/HTTP, and serial values remain
  absent from public output. No detector or hardware effect occurred.
- Outcome: The exact attempt-004 implementation is eligible for commit and
  push after final diff review.
- Blocker or next safe action: Commit and push, build the exact package, run
  one detector, and consume attempt-004 at most once if admitted.

## 2026-08-12T06:56:43Z | simplification and final software gate

- Source commit: plan commit `5279ea6c`.
- Actions: The final diff review removed the remaining stale-evidence edge by
  clearing readiness replay whenever the configuration AP is disabled or the
  station reaches connected publication. This keeps the recurring marker a
  live readiness fact instead of a historical latch.
- Verification: The focused API, firmware ownership, flash, automation, and
  real package targets pass after the refinement. The complete ordered Cargo,
  Bright Builds, all 37 Bazel tests, parity/progress, redaction, reference,
  generated-contract, selector, immutable-plan, task, fresh-path,
  reference-cleanliness, and diff gates also pass. Plan SHA-256 remains
  `48796a1c9bdbbce5fbe3b8f07ae7c34ac6f2a6069396d081321b135e6e569877`.
- Evidence: Software fixtures and closed categorical facts only. No detector,
  host-network, credential, USB, DNS/HTTP, serial, or device effect occurred.
- Outcome: The implementation is eligible for commit and push.
- Blocker or next safe action: Commit and push, rebuild the exact package, then
  run the single detector-gated attempt-004 transaction.

## 2026-08-12T07:07:53Z | hardware attempt-004 closed

- Source commit: `fb9623d6c2f877a716324642311480bdd707a391`.
- Actions: Built the exact package, ran one detector, and consumed the one
  conditional attempt-004 transaction. The detector admitted one Ultra 205,
  one private exact-device candidate, protected modes, and no holder. The
  capture performed the authorized AP-only flash, bounded host association
  attempt, host cleanup, and exact-package owner-Wi-Fi recovery flash.
- Verification: Exact-package passive safety and eleven recurring AP/DHCP/DNS
  readiness samples passed. The typed terminal result is `hardware_blocked` at
  `association`. Host restoration and device recovery are true, the recovery
  flash was used, secondary recovery failure is false, private modes are 0700
  and 0600, and the public projection is absent.
- Evidence: Only the closed terminal category and safe recovery booleans are
  public. Detector identity, candidate, port, serial, Wi-Fi, credential,
  network, DNS/HTTP, command, and process values remain ignored and private.
- Outcome: Attempt-004 is consumed and closes without verification;
  `NET-002` remains `implemented`.
- Blocker or next safe action: Type the raw association sub-boundary privately
  and qualify a supported exact-SSID macOS association transaction in a new
  continuation. Do not retry this ordinal.
