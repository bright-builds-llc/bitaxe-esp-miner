# Parity work log

## 2026-08-12T07:12:23Z | selection, diagnosis, and immutable plan

- Source commit: `62f633674ca9e1ef66d356231f3b1feaa9e3db17`.
- Actions: Selected the first canonical candidate, linked attempt-004's typed
  closure, and inspected the installed Apple CoreWLAN SDK without invoking a
  scan or association. Designed a protected directed exact-SSID transaction
  with private sub-boundaries and the existing closed public boundary.
- Evidence: Repository source, the closed attempt-004 result, installed Apple
  SDK headers, and a no-effect Swift typecheck. No detector, USB, host-network,
  credential, DNS/HTTP, serial, or device effect occurred.
- Outcome: Plan drafted; software gates pending.
- Blocker or next safe action: Run the complete plan-only gate, commit and push
  the immutable plan, then implement without editing `PLAN.md`.

## 2026-08-12T07:15:15Z | plan-only gate complete

- Source commit: `62f633674ca9e1ef66d356231f3b1feaa9e3db17`.
- Verification: No-effect Swift CoreWLAN typecheck, ordered Cargo format,
  strict Clippy, all-target build, all-feature tests, Bright Builds, all 37
  Bazel tests, parity/progress, redaction, reference, generated contracts,
  selector, task uniqueness, immutable-plan digest, reference cleanliness,
  fresh paths, and diff checks pass.
- Evidence: Plan SHA-256 is
  `2d705f1a10befc1e235d383b9b33e5fe620e6522fcba4f6b0ba814d92bc99028`.
  No detector, credential, NVS, USB, host-network, DNS/HTTP, or device effect
  occurred.
- Outcome: The immutable plan is eligible for commit and push.
- Blocker or next safe action: Commit and push, then implement the protected
  CoreWLAN association transaction without editing `PLAN.md`.

## 2026-08-12T07:23:27Z | implementation gate complete

- Source commit: plan commit `b02f56b1`.
- Actions: Added the protected CoreWLAN helper, exact-SSID directed scan and
  association, private intent/result/child records, cleanup arming before the
  effect, and a 45-second per-child timeout. A real-child sandbox regression
  caught and fixed workspace-root resolution before hardware use.
- Verification: Swift typecheck and no-effect real-child fixture pass. All
  automation tests, focused production build, exact firmware package build,
  ordered Cargo format/Clippy/build/tests, Bright Builds, all 37 Bazel tests,
  parity/progress, redaction, reference, generated contracts, selector,
  immutable-plan, task, fresh-path, reference-cleanliness, sensitive-output,
  and diff gates pass. Plan SHA-256 remains
  `2d705f1a10befc1e235d383b9b33e5fe620e6522fcba4f6b0ba814d92bc99028`.
- Evidence: Software fixtures and protected closed categories only. No
  detector, host-network, credential, USB, DNS/HTTP, serial, or device effect
  occurred.
- Outcome: The exact attempt-005 implementation is eligible for commit and
  push after final diff review.
- Blocker or next safe action: Commit and push, rebuild the exact package, run
  one detector, and consume attempt-005 at most once if admitted.

## 2026-08-12T07:37:14Z | hardware attempt-005 closed

- Source commit: `2af62ad54600dd6c7bffcf3481cc8cd94b98ab3d`.
- Actions: Built the exact package, ran one detector, and consumed the sole
  attempt-005 transaction with directed CoreWLAN association and recovery.
- Verification: CoreWLAN returned ready without a private error. DHCP, DNS,
  captive redirect, system-info, AP state, exact build identity, host cleanup,
  and device recovery passed. The typed terminal category is
  `service_recovery_failed` because `startMiningOnBoot` was true rather than the
  checker-required false; authoritative passive runtime safety remained
  disabled. Projection absence and private modes pass.
- Evidence: Only closed categorical facts and safe booleans are public. Raw
  local identities, Wi-Fi, network, HTTP, command, process, and serial values
  remain private.
- Outcome: Attempt-005 is consumed; `NET-002` remains `implemented`.
- Blocker or next safe action: Remove the invalid preference-value postcondition
  in a fresh continuation while retaining runtime safety as authoritative.
