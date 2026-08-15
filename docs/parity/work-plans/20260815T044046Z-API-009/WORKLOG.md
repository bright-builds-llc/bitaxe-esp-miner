# Parity work log

## 2026-08-15T04:40:46Z | immutable-plan draft

- Source commit: `606e84a1061dc5ff77ba3da8a3f1803fa50257fa`.
- Actions: Selected API-009 after the pushed paused-count repair and drafted
  one fresh attempt-027 contract bound to the immediate pre-dismissal count.
- Verification: Clean synchronized HEAD, no open plan, API-009 first, pushed
  repair and closure, non-empty ignored credential input, and fresh detector,
  attempt, and projection paths were confirmed. Plan-only focused, mandatory,
  privacy, reference, package, selector, digest, and diff gates remain pending.
- Evidence: Public source, tests, and prior categorical closure facts only. No
  credential contents, detector, USB, device/network, display, mining,
  hardware-control, protected attempt, or raw trace was accessed.
- Outcome: Hardware remains ineligible until this immutable plan and its task
  checkpoint pass, commit, and push.
- Blocker or next safe action: Gate, commit, and push this plan before any
  package or device-facing work.

## 2026-08-15T04:43:52Z | plan-only gates passed

- Plan SHA-256:
  `a7175490d833b4ebffd399b9c9db16ac4097058e16b888a3d3467ad396b0e504`.
- Actions: Kept the contract to one attempt-027 and bound count preservation
  to the pushed immediate paused pre-dismissal baseline. Confirmed the ignored
  credential input is non-empty without reading it and that detector, attempt,
  and projection paths are fresh.
- Verification: Focused command-effects, network, campaign, CLI, loopback, and
  recovery tests; ordered Cargo format, strict lint, build, and tests; Bright
  Builds; all 44 Bazel tests; parity/progress; redaction; reference cleanliness;
  firmware build; selector; unique task; immutable plan digest; sensitive-
  output; and diff checks pass.
- Evidence: Public source, task, plan, and tests only. No credential contents,
  protected attempts, detector, USB, device/network, display, mining, hardware-
  control, UART, or pin interface was accessed.
- Outcome: The plan is eligible to commit and push as the immutable attempt-
  027 authorization boundary. API-009 remains `implemented`.
- Blocker or next safe action: Commit and push this checkpoint, then require
  clean synchronized HEAD before package creation or detector admission.

## 2026-08-15T05:28:00Z | attempt-027 intentionally declined and closed

- Source commit: `4cc6b808050c98f763fb661a6f039ea31e745498`.
- Actions: Consumed the live `identify-ready` checkpoint through the repo-owned
  typed `declined` signal after the programmatic-verification redesign made the
  coupled physical checkpoint obsolete. No protected attempt content was read.
- Verification: The response was consumed, the campaign, fixture, and wrapper
  process group exited, no process retained the admitted USB node, and the
  public command-effects projection remained absent.
- Evidence: Safe process, holder, mode, checkpoint-consumption, and projection-
  withholding facts only. Protected traces, credentials, origins, device and
  network identifiers, and private result contents remain unread and private.
- Outcome: Attempt-027 is consumed and closed without operator attribution.
  API-009 remains `implemented`; its physical display UAT is intentionally
  deferred and no attempt-028 is authorized by this plan.
- Blocker or next safe action: Build and software-verify the shared autonomous
  device-transaction platform before planning any fresh hardware attempt.

## 2026-08-15T06:04:08Z | programmatic verification platform software gates passed

- Source baseline: `4cc6b808050c98f763fb661a6f039ea31e745498`.
- Actions: Added the access-gated command-status extension, framebuffer-flush
  receipts, retained command markers, read-only live inspection, one typed
  transaction interface, compatibility adapters, autonomous API command-effect
  proof, migrated durability and OTA callers, and the independently replayable
  durable display UAT.
- Verification: The complete simulated no-human command-effects campaign,
  focused failure regressions, real-child transaction test, ordered Cargo
  format/strict-lint/build/test gates, all 44 Bazel tests, firmware build,
  Bright Builds, parity/progress, semantic redaction, pinned-reference
  cleanliness, and diff checks pass.
- Evidence: Public source and deterministic test results only. No fresh USB,
  device, network, display, mining, hardware-control, protected attempt, or raw
  trace was accessed.
- Outcome: The software platform is ready to commit and push. API-009 remains
  `implemented`; hardware and visual evidence remain intentionally pending.
- Blocker or next safe action: Commit and push this clean implementation, then
  create a separate exact-package pilot contract before detector admission.

## 2026-08-15T06:08:00Z | software checkpoint pushed and attempt-028 contracted

- Pushed source: `c9faaaa0`.
- Actions: Published the programmatic verification platform to `origin/main`
  and added a separate one-run attempt-028 contract in `TASKS.md` without
  changing the immutable attempt-027 plan.
- Verification: Remote-default detection, fetch, fast-forward ancestry, push,
  clean package-path prerequisites, fresh attempt/detector/projection paths,
  and non-empty ignored Wi-Fi input checks pass. Credential contents were not
  read.
- Evidence: Public source/task facts only; no detector or hardware command has
  run at this checkpoint.
- Outcome: The software publication gate is complete. Attempt-028 becomes
  effect-eligible only after this task checkpoint is committed and pushed.
- Blocker or next safe action: Commit and push the attempt contract, rebuild
  the exact package from that clean synchronized HEAD, then run its detector.

## 2026-08-15T06:22:20Z | attempt-028 blocked at over-constrained pause quorum

- Exact package source: `1510e8cb76379bf6d7c4b43e4b5ac1543608a9bd`.
- Actions: Built and validated the exact package, admitted one Ultra 205 with
  protected detector modes, and consumed the sole programmatic attempt-028.
- Verification: Trusted package/runtime identity, safety, local fixture,
  genuine notification, accepted activity, one pause request, HTTP paused
  state, stopped hardware, serial safe-stop, and USB cleanup passed. The public
  projection is absent. Primary category is `network_correlation_failed`;
  recovery was attempted and its HTTP request failed secondarily.
- Diagnosis: The host required both USB and WebSocket transition generations
  in addition to the authoritative HTTP generation and runtime safe-stop. That
  recreated the generic quorum explicitly rejected by the platform design.
- Fix proof: The exact production-seam regression
  `pause_join_uses_claim_specific_http_generation_and_safe_stop_without_log_quorum`
  failed deterministically twice before the fix and passes afterward. Full
  flash and automation focused suites also pass. The command-effects schema is
  now v8; pause/resume/dismiss use their claim-specific proof, while IDENTIFY
  requires its HTTP render receipts and one retained-marker channel.
- Outcome: Attempt-028 is consumed without promotion or retry. API-009 remains
  `implemented`; no user action contributed to the failure.
- Full verification: Ordered Cargo format, strict lint, all-target build and
  all-feature tests; Bright Builds; firmware build; all 44 Bazel tests;
  parity/progress; semantic redaction; pinned-reference cleanliness; and diff
  checks pass.
- Blocker or next safe action: Commit and push the fix, then decide whether the
  material contract correction justifies a separately bounded attempt-029
  task.

## 2026-08-15T06:31:00Z | claim-specific fix pushed and attempt-029 contracted

- Pushed fix: `26cf68f7`.
- Actions: Published the v8 claim-specific command proof and added one separate
  attempt-029 task contract. The immutable attempt-027 plan remains unchanged.
- Verification: Full repository gates passed before publication; remote fetch,
  ancestry, and push completed without rewrite. Attempt-029 paths are fresh.
- Evidence: Public source/task facts only; no new package, detector, or hardware
  operation has run under attempt-029.
- Outcome: The corrected boundary is materially different from attempt-028.
  Attempt-029 becomes eligible only after this contract is committed and pushed.
- Blocker or next safe action: Publish this contract, then rebuild and validate
  the exact package before the one detector run.

## 2026-08-15T06:38:12Z | attempt-029 blocked by stopped-state freshness coupling

- Exact package source: `410bc830`.
- Actions: Published the attempt contract, rebuilt and validated its exact
  package, admitted one holder-free Ultra 205, and consumed the sole
  programmatic attempt-029.
- Verification: Trusted package/runtime identity, the local fixture, genuine
  notification, accepted activity, one pause request, HTTP paused state,
  stopped hardware, serial safe-stop, recovery HTTP and serial safe-stop, and
  process/USB cleanup passed. Public evidence remains absent.
- Diagnosis: The earliest category was `safety_stale`. Both the marker parser
  and HTTP phase loop still treated transient sensor staleness as fatal after
  the pause had stopped the device, even though the stopped-state safe-stop
  witness was authoritative. This was orchestration logic, not a user action.
- Regression: A production-shaped stopped-pause marker failed deterministically
  before the fix and now passes. Paired HTTP tests admit identity with stale
  sensors only for the stopped pause while proving active phases continue to
  reject stale safety observations.
- Outcome: Attempt-029 is consumed, API-009 remains `implemented`, and no
  attempt-030 is authorized by this contract.
- Blocker or next safe action: Run focused and full repository gates, commit and
  push the material stopped-state fix, then create a separate exact-package
  contract before any new detector or hardware attempt.

## 2026-08-15T06:49:44Z | stopped-state fix passed full software verification

- Actions: Kept active samples under strict fresh-safety validation, admitted
  only exact stopped command states by identity plus their safe-stop witness,
  and split the regressions into focused test modules with explicit Bazel
  source ownership.
- Verification: The focused regressions, ordered Cargo format/strict-lint/
  all-target build/all-feature test sequence, Bright Builds, firmware build,
  all 44 Bazel tests, parity and progress, semantic redaction, pinned-reference
  cleanliness, and diff checks pass.
- Outcome: The attempt-029 root cause is fixed and ready to publish. No hardware
  or protected evidence was accessed during the fix and verification cycle.
- Blocker or next safe action: Commit and push this clean material correction;
  any attempt-030 still requires its own exact-package task contract.

## 2026-08-15T06:52:00Z | stopped-state fix pushed and attempt-030 contracted

- Pushed fix: `b2a8a066`.
- Actions: Published the regression-backed stopped-state admission correction
  and added a separate one-run attempt-030 task contract without changing the
  immutable attempt-027 plan.
- Verification: Full repository gates passed before publication; remote fetch,
  fast-forward ancestry, and push completed without rewrite.
- Outcome: Attempt-030 is materially different from attempt-029 and becomes
  effect-eligible only after this contract is committed and pushed.
- Blocker or next safe action: Publish the contract, rebuild its exact package,
  then perform its one detector-gated programmatic campaign.

## 2026-08-15T06:57:36Z | attempt-030 stopped before detector admission

- Exact package source: `3a5aa94c`.
- Actions: Built the exact package and checked private-input presence. A manual
  preflight then guessed a manifest `board` field that the v3 package does not
  define. Because its nonzero exit was not surfaced, the campaign command was
  invoked once with no detector artifact and returned `process_failed`.
- Verification: No detector directory, attempt root, public projection, device
  process, USB admission, fixture, mining, or recovery effect exists. This was
  host orchestration error and not a user or device failure.
- Regression: Detector handoff now owns a typed privacy-safe
  `evidence_invalid` failure with `detector_admitted=false` for unavailable,
  malformed, wrongly permissioned, or ambiguous output. Focused Bun and real
  Bazel automation tests pass.
- Outcome: Attempt-030 is consumed by its campaign-start rule, API-009 remains
  `implemented`, and no attempt-031 is authorized by this contract.
- Blocker or next safe action: Run full repository gates, publish the typed
  preflight correction, then create a separate attempt contract using only
  repo-owned package validation and explicit command exit checks.

## 2026-08-15T07:01:00Z | typed detector preflight passed full verification

- Verification: Focused Bun tests, the real Bazel automation suite, ordered
  Cargo format/strict-lint/all-target build/all-feature tests, Bright Builds,
  firmware build, all 44 Bazel tests, parity and progress, semantic redaction,
  pinned-reference cleanliness, and diff checks pass.
- Outcome: Missing or invalid detector evidence can no longer collapse into a
  generic launch failure. The fix and its durable lesson are ready to publish.
- Blocker or next safe action: Commit and push, then separately contract any
  attempt-031 with repo-owned package validation and explicit exit gating.

## 2026-08-15T07:04:00Z | typed preflight fix pushed and attempt-031 contracted

- Pushed fix: `35bc2280`.
- Actions: Published typed detector-evidence failures and added a separate
  attempt-031 contract that delegates package admission to `just package` and
  requires explicit zero exits plus artifact checks at every preflight step.
- Outcome: Attempt-031 becomes effect-eligible only after this contract is
  committed and pushed. No hardware ran at this checkpoint.
- Blocker or next safe action: Publish the contract, then execute its package,
  detector, and one campaign command in order with explicit exit gating.

## 2026-08-15T07:10:33Z | attempt-031 reached a genuine active-safety blocker

- Exact package source: `7320cf86`.
- Actions: Passed clean-source, repo-owned package, and one-device detector
  gates, then consumed the sole no-human programmatic campaign.
- Verification: Exact runtime/package identity, the local fixture, genuine
  notification, accepted work, terminal safe stop, child cleanup, USB cleanup,
  and evidence withholding passed. Recovery was not required.
- Diagnosis: The earliest category is `safety_stale`. The closed readiness
  transition records active campaign state, ready hardware, a stale safety
  sample, and an unchanged observation epoch; every required observation was
  stale at that boundary. The later terminal marker records fresh observations,
  proving transient recovery after the firmware correctly consumed the active
  campaign. This is not the stopped-state host false positive fixed after
  attempt-029 and is not attributable to user timing or action.
- Outcome: Attempt-031 is consumed, no public projection exists, API-009 remains
  `implemented`, and no attempt-032 is authorized.
- Terminal blocker: Further evidence requires a separate redaction-safe
  sensor-producer/I2C latency diagnostic and a verified root-cause change.
  Weakening active safety or rerunning the unchanged campaign is prohibited.

## 2026-08-15T07:31:00Z | sensor-sweep latency continuation opened

- Trigger: Attempt-031 proved a transient active-mining freshness loss with an
  unchanged observation epoch and later recovery.
- Scope: A new active task owns a deterministic production-shaped reproduction,
  redaction-safe stage timing, ranked hypothesis tests, and a root-cause fix.
- Safety: The 1,000 ms active-safety freshness boundary remains immutable. This
  continuation authorizes no hardware or protected-attempt access and cannot
  authorize attempt-032.
- Next action: Reproduce the exact freshness loss from the current shared-I2C
  retry envelope before changing runtime behavior.

## 2026-08-15T07:42:44Z | sensor-sweep root cause fixed in software

- Failure proof: The production retry envelope reproduced deterministically:
  a sensor read starting at 500 ms could remain in three 500 ms attempts plus
  retry delays until 2,030 ms, beyond the immutable 1,000 ms freshness limit.
  Three uncached runs failed identically before the change.
- Ranked hypotheses: Shared sensor retry was sufficient and confirmed. Display
  flush and safety actuation remained credible co-triggers because they use the
  same owner; producer scheduling and consumer alignment could amplify but not
  independently explain the reproduced bound. No hardware bus-fault trigger
  can be classified until a materially changed live attempt emits the new
  closed stage diagnostic.
- Root fix: Startup retains the upstream retry contract. Every runtime sensor,
  display, and actuation transfer now shares the sensor producer's absolute
  publication deadline with 100 ms headroom. Lower-priority display work is
  deferred, not permanently disabled, when that budget is exhausted.
- Diagnostic: Firmware retains only boot session, monotonic revision, closed
  stage/outcome labels, and a coarse duration bucket. The public automation
  failure projection deliberately omits boot session and all values, origins,
  ports, USB/network identifiers, and raw traces.
- Focused proof: The original regression passed three uncached runs; retry,
  diagnostic, source-ownership, campaign-marker, host parsing, failure
  precedence, redaction, and API command-effects tests passed. The complete
  firmware target also built successfully.
- Next action: Run the mandatory repository gates, review the exact diff, then
  commit and push before defining any attempt-032 hardware contract.

## 2026-08-15T07:54:56Z | sensor-sweep software boundary closed

- Verification: Ordered Cargo format, clippy, all-target build, and all-feature
  tests passed. The firmware build, all 45 Bazel test targets, Bright Builds,
  parity and progress, redaction, reference cleanliness, source ownership,
  sensitive-output, and diff checks passed.
- Review findings fixed before closure: Explicit Serde wire names prevent
  duration-bucket drift; display budget exhaustion defers instead of disabling
  the panel; host display and ownership fixtures exercise the budget boundary;
  optional diagnostic corruption cannot erase independently closed recovery
  facts or replace the campaign's primary failure.
- Boundary: Commit `c2fb0c93` is pushed to `origin/main`. The software-only task
  is complete and archived. No detector, credentials, USB, network, device,
  mining, control, reset, flash, OTA, or recovery effect occurred.
- Next action: Define a separate exact attempt-032 contract bound to this clean
  commit, then run at most one detector-gated programmatic campaign.

## 2026-08-15T07:55:00Z | attempt-032 contract opened

- Objective: Prove the materially changed bounded runtime-I2C behavior with
  one no-human programmatic API-009 campaign.
- Admission: Clean pushed source descended from `c2fb0c93`, a repo-built exact
  package, non-empty ignored Wi-Fi input, and exactly one successful protected
  detector run are required before the single campaign invocation.
- Effects: One exact-package flash/reset, private Wi-Fi and local-fixture seed,
  conservative mining for at most 600 active seconds, one each command effect,
  same-device software restart, safe recovery, process termination, and holder
  cleanup. All automated effect deadlines remain finite.
- Privacy: Only a ready redacted projection may publish. A failed run may expose
  closed recovery booleans and stage/outcome/duration/revision diagnostics, but
  never boot session, sensor values, identities, origins, ports, credentials,
  frame text, or raw traces.
- Retry/stop: Campaign start consumes attempt-032. There is no same-contract
  retry or attempt-033 authority. Any non-ready boundary withholds evidence and
  stops for diagnosis. The physical-display UAT remains a separate later task.
