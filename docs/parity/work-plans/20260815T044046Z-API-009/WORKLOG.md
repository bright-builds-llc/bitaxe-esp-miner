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

## 2026-08-15T08:06:26Z | attempt-032 safely closed with actionable diagnostic defects

- Admission: Clean synchronized source `a92196e4`, the exact package/reference
  identity, private-input presence, fresh paths, and exactly one protected
  detector run passed.
- Result: The single programmatic campaign stopped `hardware_blocked` on active
  safety staleness with an unchanged observation epoch and all five required
  observations stale. Safe stop and cleanup were confirmed, no recovery request
  was required, and the public projection was withheld.
- Diagnostic defect 1: The terminal closed fact was revision 8,
  `display / ready / under_250_ms`. A later lower-severity pressure event had
  overwritten the earlier actionable failure, defeating stage diagnosis.
- Diagnostic defect 2: The Rust-generated private boot session is a valid
  `u64` but exceeded JavaScript's safe-integer range. The wrapper therefore
  discarded the whole diagnostic even though it never publishes boot identity.
- Disposition: No user action contributed. Attempt-032 is consumed and no
  attempt-033 is authorized. A software-only task now owns severity-preserving
  retention and the private `u64` handoff before any further hardware work.

## 2026-08-15T08:10:00Z | diagnostic precedence fixed in software

- Red proofs: A later `display / ready / under_250_ms` event replaced an earlier
  budget exhaustion before the fix. A valid private boot session above
  JavaScript's safe-integer limit also caused the public closed diagnostic to be
  omitted. Both focused tests failed at the production seams before the change.
- Retention: Every pressure event still receives a boot-scoped revision and
  emits its marker. The retained campaign projection now keeps the earliest
  event at the highest observed severity; only a strictly higher-severity event
  replaces it. Deterministic tests cover lower, higher, and equal severity.
- Handoff: TypeScript accepts a positive finite integer from the Rust-validated
  private campaign result without requiring lossless JavaScript arithmetic.
  Boot session remains excluded from public success and failure projections.
- Focused proof: The firmware diagnostic target and the command-effects host
  integration suite pass, including the above-safe-integer regression,
  malformed optional evidence, recovery preservation, and redaction.
- Next action: Run the full repository gates and publish the software boundary;
  no hardware ordinal is authorized by this task.

## 2026-08-15T08:18:00Z | diagnostic correction verified

- Full proof: Ordered Cargo format, clippy, all-target build, and all-feature
  tests passed. Bright Builds, firmware build, all 45 Bazel test targets,
  parity validation and progress, redaction, reference cleanliness, sensitive-
  output review, and diff checks also passed.
- Simplification review: The firmware retains one ranked event rather than a
  second failure channel, and the host accepts the already Rust-validated
  private integer without converting or publishing it.
- Disposition: The software-only diagnostic task is complete. Hardware remains
  untouched; a separately committed task contract is required before one new
  detector-gated attempt may use the corrected boundary.

## 2026-08-15T08:22:00Z | attempt-033 diagnostic contract drafted

- Objective: Run one no-human programmatic campaign against pushed diagnostic
  fix `bee8c1c9` so any recurring active-safety lapse reports the earliest event
  at the highest observed severity rather than a later successful display.
- Admission: A clean synchronized source tree, exact repo package and reference
  identity, opaque ignored Wi-Fi input, fresh protected paths, and exactly one
  successful detector run are required.
- Effects: The existing bounded campaign effects and cleanup are permitted;
  active-safety freshness remains unchanged. No visual claim or user checkpoint
  is included.
- Privacy and stop: Only ready redacted evidence may publish. A failed run may
  expose the closed diagnostic and safe recovery booleans, never private
  identity or trace data. Campaign start consumes attempt-033, and no unchanged
  retry or attempt-034 is authorized.

## 2026-08-15T08:30:00Z | attempt-033 rejected before effects

- Admission: Clean pushed source `e22b17fd`, exact package/reference identity,
  opaque ignored input, fresh paths, and the one protected detector run passed.
- Result: The single campaign invocation returned `evidence_invalid`; no public
  projection was created. Exact-signature inspection proved stdout/stderr shell
  redirection had pre-created the private attempt root before automation
  freshness admission.
- Safety: The root contained no child campaign directory or artifact, and no
  detector holder remained. Therefore no flash, reset, network, mining,
  command, restart, or recovery effect launched. The false cleanup booleans
  report a lifecycle that was never entered.
- Disposition: No user action contributed. Attempt-033 is consumed and no
  attempt-034 is authorized. A software-only task owns a production-seam
  regression and explicit sibling-wrapper contract before another attempt.

## 2026-08-15T08:45:00Z | sibling-wrapper contract verified

- Regression: A production-seam test pre-creates the private attempt root with
  a wrapper output file and proves typed rejection before any process launch.
- Contract: The freshness guard now states that wrapper stdout/stderr must be
  captured under a protected sibling root, never inside the absent attempt
  root. Non-empty attempt roots remain fail-closed.
- Verification: Focused tests and all mandatory Cargo, Bright Builds, firmware,
  Bazel, parity, redaction, reference, and diff gates pass. The new regression
  was split into a focused module to preserve file-length budgets.
- Disposition: The software boundary is ready to publish. A fresh attempt must
  use separate `wrapper-034` and `attempt-034` paths under a new exact contract.

## 2026-08-15T08:50:00Z | attempt-034 sibling-wrapper contract drafted

- Admission: Require pushed `3ae3c85e`, exact HEAD/reference package, opaque
  ignored input, fresh detector/public paths, absent `attempt-034`, and a
  distinct protected `wrapper-034`.
- Execution: Run one detector and one campaign. Shell redirection targets only
  the sibling wrapper root so the automation command exclusively creates and
  owns the attempt root.
- Effects/privacy: Existing bounded effects, recovery, safe stop, cleanup, and
  redaction rules are unchanged. No user checkpoint or visual claim is part of
  the attempt.
- Stop: Campaign start consumes attempt-034. Any non-ready result withholds
  evidence and stops without attempt-035 for typed diagnosis.

## 2026-08-15T09:00:00Z | attempt-034 exposes downstream budget consequence

- Admission/execution: Clean pushed source `dae58db9`, exact package/reference,
  opaque input, distinct protected detector/wrapper paths, absent command-owned
  attempt root, one detector, and one campaign invocation all passed.
- Result: The campaign stopped `hardware_blocked` with safe stop and cleanup
  confirmed and no recovery required. The projection was withheld. The public
  closed diagnostic is revision 4, `asic_temperature / budget_exhausted /
  under_100_ms`; no holder remains and private modes pass.
- Root-cause refinement: Power acquisition precedes ASIC temperature under the
  same absolute deadline. A concrete driver failure currently ranks below a
  later budget exhaustion, so the downstream consequence can replace the
  causal failure. This is a diagnostic-ordering defect, not a user failure.
- Disposition: Attempt-034 is consumed and no attempt-035 is authorized. A
  software-only task owns causal driver-failure precedence before another live
  observation.

## 2026-08-15T09:12:00Z | causal diagnostic precedence verified

- Red proof: A production-shaped sequence of power driver failure followed by
  ASIC-temperature budget exhaustion initially retained the downstream event.
- Fix: Driver failure now ranks above budget exhaustion. Both remain above
  invalid/unavailable/recovered/slow-success facts; stable ties, revisions, and
  retained markers remain unchanged.
- Verification: Focused diagnostic tests and all mandatory Cargo, Bright
  Builds, firmware, Bazel, parity, redaction, reference, and diff gates pass.
- Disposition: The software boundary is ready to publish. Any next live ordinal
  requires a separate exact-package contract and must stop after one result.

## 2026-08-15T09:18:00Z | attempt-035 causal-stage contract drafted

- Objective: One no-human campaign against pushed causal-precedence fix
  `86987839`, either completing the programmatic quorum or surfacing the actual
  driver-failed stage that precedes downstream budget exhaustion.
- Admission/layout: Exact clean package, one protected detector, separate
  protected wrapper, and absent command-owned attempt/public paths.
- Effects/privacy: Existing bounded campaign/recovery/cleanup effects and
  redaction rules are unchanged; no physical display claim is included.
- Stop: Campaign start consumes attempt-035. Any non-ready result withholds
  evidence and stops without attempt-036 for root-cause diagnosis.

## 2026-08-15T09:35:00Z | attempt-035 proves owner scheduling miss

- Preflight: A first read-only shell stopped because zsh reserves lowercase
  `path`; using `candidate_path` passed without detector or campaign effect.
- Result: Exact package, one detector, separate wrapper, and one campaign ran.
  It stopped `hardware_blocked` with safe stop, cleanup, and recovery confirmed,
  no secondary failure or holder, and evidence withheld. The diagnostic is
  revision 11, `power / budget_exhausted / under_100_ms`.
- Root cause: Power is the first sweep stage, so the owner woke after its
  publication deadline. The Rust pthread default is priority 5; upstream runs
  its power-management sensor task at priority 10.
- Disposition: Attempt-035 is consumed. A software-only task will align only
  this owner with upstream priority before any attempt-036.

## 2026-08-15T09:52:00Z | sensor owner priority aligned

- Fix: The combined sensor/display/I2C owner raises only its newly spawned
  current FreeRTOS task from the Rust pthread default to upstream priority 10
  before its first runtime action.
- Scope: Global pthread defaults, mining-worker priority, the one-second active
  safety rule, sensor freshness, retry behavior, I2C deadlines, and campaign
  timing remain unchanged.
- Verification: Source ownership proves one local current-task priority call
  and no mining-worker call. The focused test, real firmware build, mandatory
  Cargo sequence, Bright Builds, Bazel, parity, redaction, reference, and diff
  gates pass.
- Disposition: The software fix is complete and may be published. A live
  attempt requires a separate exact-package attempt-036 contract.

## 2026-08-15T10:05:00Z | attempt-036 priority-fix contract drafted

- Objective: Run one no-human programmatic campaign against clean pushed
  sensor-owner priority fix `7917de87`, either completing the full machine
  quorum or producing one typed post-fix failure for diagnosis.
- Admission/layout: Require exact HEAD/reference package identity, opaque
  ignored input, one protected detector, a separate protected wrapper, and
  absent command-owned attempt/public paths.
- Effects/privacy: Existing bounded campaign, recovery, safe-stop, cleanup,
  and redaction contracts remain unchanged. No human checkpoint or physical
  display claim is included.
- Stop: Campaign start consumes attempt-036. Any non-ready result withholds
  evidence and stops without attempt-037 or an unchanged retry.

## 2026-08-15T10:20:00Z | attempt-036 clears sensor blocker and exposes witness loss

- Result: Exact package, one detector, separate wrapper, and one campaign ran.
  It stopped `hardware_blocked / command_effects`; safe stop, cleanup, and
  recovery passed, no secondary failure remained, and evidence was withheld.
- Sensor proof: Revision 1 closed as `display / ready / under_500_ms`, proving
  the upstream-aligned owner priority removed the earlier publication miss.
- Command boundary: Pause completed through HTTP generation plus receive-only
  USB safe-stop. One dismiss request followed, but the host stopped before
  confirming it. Identity and safety remained valid, and the failure occurred
  before the automated phase deadline.
- Root cause: The observer currently makes transient WebSocket close/read loss
  and transient HTTP status-read loss immediately terminal after commands
  begin, despite independent USB evidence and existing bounded phase deadlines.
- Disposition: Attempt-036 is consumed. A software-only continuity task will
  preserve request-once and fail-closed validation while allowing observation
  transport recovery before any attempt-037.

## 2026-08-15T10:40:00Z | independent witness continuity verified

- Fix: WebSocket connect, peer-close, and I/O loss now reconnect without
  invalidating independent receive-only USB facts. Transient HTTP status reads
  wait within the existing phase deadline.
- Fail-closed boundary: Command requests remain request-once. Malformed HTTP or
  WebSocket data, identity/safety drift, protocol/capacity failure, stale or
  duplicate generations, missing required witnesses, and deadline expiry
  remain terminal.
- Verification: Forty-eight focused command-effects tests plus mandatory Cargo,
  Bright Builds, real firmware, full Bazel, parity, redaction, reference, and
  diff gates pass. The source and tests were split into bounded modules.
- Disposition: The software boundary is ready to publish. Any live attempt-037
  requires a separate exact-package contract and one-result stop rule.

## 2026-08-15T10:48:00Z | attempt-037 continuity-fix contract drafted

- Objective: Run one no-human programmatic campaign against clean pushed
  independent-witness continuity fix `223b7990`, either completing the full
  machine quorum or producing one typed post-fix failure for diagnosis.
- Admission/layout: Require exact HEAD/reference package identity, opaque
  ignored input, one protected detector, a separate protected wrapper, and
  absent command-owned attempt/public paths.
- Effects/privacy: Existing bounded campaign, recovery, safe-stop, cleanup,
  request-once, deadline, and redaction contracts remain unchanged. No human
  checkpoint or physical display claim is included.
- Stop: Campaign start consumes attempt-037. Any non-ready result withholds
  evidence and stops without attempt-038 or an unchanged retry.

## 2026-08-15T11:08:00Z | attempt-037 requires causal command diagnostic

- Admission: The detector succeeded once; a local post-check expected obsolete
  `port=` syntax instead of current `port: ` output and was corrected without a
  second detector or any campaign-root collision.
- Result: One campaign stopped `hardware_blocked / command_effects`; safe stop,
  cleanup, and recovery passed, no secondary failure remained, and evidence
  was withheld. Identity, safety, package, and the ready sensor diagnostic held.
- Command boundary: One pause request was issued, but pause confirmation did
  not complete before the early `network_correlation_failed` result.
- Blocker: The coarse category cannot distinguish serial, phase deadline,
  WebSocket, HTTP parsing, identity/safety, state-machine, terminal, or quorum
  causes. Another live retry would not be actionable.
- Disposition: Attempt-037 is consumed. A software-only first-failure diagnostic
  is required before any attempt-038.

## 2026-08-15T11:30:00Z | causal command diagnostic verified

- Contract: Private campaign evidence and public failure output now carry only
  `mining-command-failure-diagnostic-v1`, a closed command phase, and a closed
  cause. No identity, origin, port, address, value, or trace is included.
- Precedence: The first failure wins and recovery cannot replace it. Missing
  final command proof closes explicitly as `terminal / quorum_incomplete`.
- Coverage: Rust types enumerate every phase/cause; focused Rust and host tests
  cover serialization, malformed input, transport boundaries, redaction, and
  primary-failure precedence. Full Cargo, Bright Builds, firmware, Bazel,
  parity, redaction, reference, and diff gates pass.
- Disposition: The diagnostic boundary is ready to publish. A separately
  committed attempt-038 contract may run exactly one live campaign.

## 2026-08-15T11:42:00Z | attempt-038 causal-result contract drafted

- Objective: One no-human campaign against pushed diagnostic `6602383b`,
  either sealing the full machine quorum or publishing one closed phase/cause.
- Admission/layout: Exact package, current `port: ` detector contract, one
  detector, separate wrapper, and absent command-owned attempt/public paths.
- Effects/privacy: Existing bounded effects, request-once, recovery, cleanup,
  deadlines, and redaction remain unchanged; no physical display claim.
- Stop: Campaign start consumes attempt-038. Any non-ready result withholds
  evidence and stops without attempt-039 or an unchanged retry.

## 2026-08-15T12:05:00Z | attempt-038 isolated terminal deadline mismatch

- Result: One detector and one campaign ran. The public result was
  `hardware_blocked / command_effects`, with `terminal / phase_deadline` as the
  first failure. Safe stop, recovery, cleanup, private modes, and evidence
  withholding all passed; no user action contributed.
- Proven boundary: Every requested command and command-specific machine
  postcondition completed exactly once before the failure. Same-package,
  safety, serial-transition, and display-render receipts passed.
- Root cause: Entering the host `Terminal` phase starts a generic 15-second
  phase deadline, but firmware still needs to finish the remainder of its
  admitted 600-active-second resumable lease before publishing `consumed`.
- Disposition: Attempt-038 is consumed. Remove only the contradictory host
  terminal-phase deadline while retaining the lease, outer process,
  post-consumption HTTP, recovery, and cleanup bounds. No attempt-039 is
  authorized until that software change passes complete gates and is pushed.

## 2026-08-15T12:35:00Z | terminal timing aligned and verified

- Fix: Completed commands now wait for the firmware's admitted resumable lease
  to publish `consumed`; the generic 15-second command-phase deadline no longer
  applies after entry to `Terminal`.
- Bounds: Serial capture is finite at activation + active duration + terminal
  grace (`600 + 600 + 180` seconds). The host uses the existing complete
  3,850-second child budget. Post-consumption HTTP confirmation remains exactly
  15 seconds, and recovery/cleanup bounds are unchanged.
- Regression: Deterministic tests cover waiting past 600 elapsed seconds before
  consumption, the exact post-consumption deadline, the Rust capture budget,
  and the host child lifetime.
- Verification: Focused tests, mandatory Cargo, Bright Builds, real firmware,
  full Bazel, parity with no validation errors, redaction, reference, and diff
  gates pass. No hardware effect occurred.

## 2026-08-15T12:42:00Z | attempt-039 lease-terminal contract drafted

- Objective: One no-human exact-package campaign against pushed fix
  `57fafecf`, proving the same completed command quorum can remain open until
  firmware consumes its admitted active-duration lease.
- Bounds: Firmware activation and active-duration leases, 1,380-second serial
  capture, complete host child budget, phase deadlines, post-consumption HTTP,
  recovery, process termination, and cleanup all remain finite.
- Effects/privacy: Existing request-once, detector, same-device, redaction, and
  protected-artifact contracts are unchanged; no physical display claim.
- Stop: Campaign start consumes attempt-039. Any non-ready result withholds
  evidence and stops without attempt-040 or an unchanged retry.

## 2026-08-15T13:05:00Z | attempt-039 exposed request and recovery ambiguity

- Result: One detector and one campaign ran. The public result was
  `hardware_blocked / command_effects`, with `pause / command_state_machine` as
  the first failure. Cleanup and private modes passed; safe-stop publication
  was false because the redundant recovery request failed. Evidence remained
  withheld and no user action contributed.
- Proven boundary: Notification and pause completed exactly once, including
  the HTTP pause generation and receive-only USB safe-stop witness. The single
  dismiss request was not confirmed. IDENTIFY, resume, and restart were never
  requested. Same-package, safety, and the sensor diagnostic passed.
- Root cause: Command POST handling keeps only an exact 200 response and drops
  typed request-write progress, so a complete write with a missing response
  cannot converge through the authoritative status generation. The public
  cause is consequently generic. Recovery also ignores the already-proved
  paused safe stop and issues an unnecessary second pause request.
- Disposition: Attempt-039 is consumed. Preserve complete-write ambiguity
  without retry, require generation/postcondition proof, add a command-request
  diagnostic cause, and reuse only an already HTTP- and serial-confirmed safe
  stop. No attempt-040 is authorized until the fix passes full gates and is
  pushed.

## 2026-08-15T10:28:46Z | request ambiguity and safe recovery fixed

- Delivery: The engine now admits a request for postcondition waiting after an
  exact 200 response or a typed fully written/flushed request with no parsed
  response. Explicit non-200, incomplete write, and pre-delivery failure still
  stop as request failures; no command is retried.
- Recovery: A command failure reuses the existing safe stop only when HTTP
  pause convergence, receive-only USB safe-stop, no resume request, and current
  stopped state are all proved. Other states retain bounded recovery.
- Diagnostic: The closed public cause vocabulary now includes
  `command_request`, while primary-failure precedence and redaction remain
  unchanged.
- Verification: Real TCP delivery-loss regression, focused Rust and Bun tests,
  mandatory Cargo, Bright Builds, real firmware, full Bazel, parity/progress,
  redaction, reference, and diff checks pass. No hardware effect occurred.

## 2026-08-15T10:31:00Z | attempt-040 ambiguous-delivery contract drafted

- Objective: One no-human exact-package campaign against pushed fix
  `333674f3`, proving every command through its generation and machine
  postcondition even if the response boundary is unavailable.
- Bounds: Firmware activation and active-duration leases, 1,380-second serial
  capture, complete host child budget, phase deadlines, post-consumption HTTP,
  recovery, process termination, and cleanup all remain finite.
- Effects/privacy: Existing request-once, detector, same-device, redaction, and
  protected-artifact contracts are unchanged; no physical-display claim.
- Stop: Campaign start consumes attempt-040. Any non-ready result withholds
  evidence and stops without attempt-041 or an unchanged retry.

## 2026-08-15T11:02:38Z | attempt-040 exposed terminal capture handoff race

- Result: One detector and one campaign ran. Every command effect and the
  600-active-second lease completed once, then the public result stopped
  `hardware_blocked / command_effects` with `terminal / serial_ended`.
  Cleanup passed, no recovery request or secondary failure occurred, and
  evidence remained withheld. No user action contributed.
- Boundary: The receive-only owner returns immediately when its serial analyzer
  accepts the consumed terminal marker. Its coordinator then marks serial input
  finished while the concurrent network worker still awaits the post-terminal
  HTTP sample, so the terminal join can fail at an ownership race.
- Disposition: Attempt-040 is consumed. Reconcile the analyzer's authoritative
  terminal facts into the network coordinator before input closure and retain
  the exact 15-second HTTP confirmation deadline. No attempt-041 is authorized
  until the fix passes full gates and is pushed.

## 2026-08-15T11:08:00Z | terminal capture handoff fixed and verified

- Fix: The completed serial capture now hands its closed consumed/persistence
  fact to the network coordinator before setting `serial_finished`. The worker
  continues only its existing post-terminal HTTP join after USB closes.
- Fail-closed behavior: Missing terminal consumption still fails immediately;
  contradictory terminal persistence is a network-correlation failure, and an
  earlier failure remains primary. The HTTP deadline remains 15 seconds.
- Verification: Ordering, input-closure, contradiction, precedence, and exact
  deadline regressions pass with focused and full Rust suites, Bright Builds,
  real firmware, full Bazel, parity/progress, redaction, reference, and diff
  checks. No hardware effect occurred.

## 2026-08-15T11:12:00Z | attempt-041 terminal-handoff contract drafted

- Objective: One no-human exact-package campaign against pushed fix
  `60457bf1`, proving serial capture closure preserves the bounded terminal
  HTTP join before restart evidence is sealed.
- Bounds: Firmware activation and active-duration leases, 1,380-second serial
  capture, complete host child budget, phase deadlines, 15-second post-terminal
  HTTP, recovery, process termination, and cleanup all remain finite.
- Effects/privacy: Existing request-once, detector, same-device, redaction, and
  protected-artifact contracts are unchanged; no physical-display claim.
- Stop: Campaign start consumes attempt-041. Any non-ready result withholds
  evidence and stops without attempt-042 or an unchanged retry.

## 2026-08-15T11:42:00Z | attempt-041 exposed serial resynchronization mismatch

- Result: The one detector-gated campaign stopped `hardware_blocked` with
  `terminal / serial_ended`; cleanup passed, recovery was not attempted, and
  the public projection remained absent. No user action contributed.
- Proven boundary: All command-specific machine postconditions, the active
  lease, and an accepted consumed terminal marker completed. Private counters
  retained transient UTF-8/JSON framing damage alongside thousands of later
  accepted markers.
- Root cause: The analyzer permanently classifies recoverable mid-stream JSON
  damage as `marker_invalid`, and terminal handoff then suppresses even the
  independently accepted consumed marker whenever any serial failure exists.
  The network worker consequently reports the secondary `serial_ended` symptom.
- Disposition: Attempt-041 is consumed. Recover only framing damage followed by
  a fully valid marker, keep schema/semantic and final corruption fail-closed,
  and hand accepted terminal facts to the network join independently. No
  attempt-042 is authorized until that software correction passes full gates.

## 2026-08-15T12:01:26Z | serial resynchronization fix verified

- Fix: UTF-8/JSON framing damage remains pending across native USB receive
  chunks and becomes terminal only if capture ends before a fully valid marker
  resynchronizes the parser. Private diagnostics retain every corruption count.
- Fail-closed boundary: Schema and semantic failures remain immediate;
  unrecovered framing damage remains `marker_invalid`; accepted terminal facts
  are handed to the network join independently so they cannot replace or hide
  an earlier serial failure.
- Verification: Production-shaped split-chunk recovery, unrecovered JSON and
  UTF-8 damage, schema rejection, and independent terminal-handoff regressions
  pass. Ordered Cargo format/strict-lint/build/test, Bright Builds, real
  firmware, all 45 Bazel tests, parity/progress, redaction, pinned-reference,
  and diff checks pass.
- Disposition: The software correction is complete and ready to publish. A
  fresh hardware campaign still requires a separately committed exact-package
  attempt contract; attempt-041 remains consumed.

## 2026-08-15T12:08:00Z | attempt-042 serial-resynchronization contract drafted

- Objective: Run one no-human programmatic campaign against clean pushed fix
  `2a97230c`, either sealing the complete machine quorum or producing one typed
  post-fix result for diagnosis.
- Admission/layout: Require exact HEAD/reference package identity, opaque
  ignored input, one protected detector, a separate protected wrapper, and
  absent command-owned attempt/public paths.
- Effects/privacy: Existing bounded campaign, recovery, safe-stop, cleanup,
  request-once, deadline, same-device, and redaction contracts are unchanged.
  No physical-display claim is included.
- Stop: Campaign start consumes attempt-042. Any non-ready result withholds
  evidence and stops without attempt-043 or an unchanged retry.

## 2026-08-15T12:38:21Z | attempt-042 isolated terminal state contradiction

- Result: One detector-gated exact-package campaign completed all command
  effects once and stopped non-ready. Public evidence remained withheld,
  recovery was not attempted, cleanup passed, and no user action contributed.
- Serial boundary: The receive-only stream was clean apart from one trailing
  fragment; thousands of markers were accepted with no UTF-8, JSON, or schema
  failure. The prior framing defect is therefore no longer explanatory.
- Primary diagnosis: The terminal marker carried reason
  `campaign_lease_consumed` while campaign state remained `armed`. The analyzer
  correctly classified this as `terminal_state_unconfirmed`; the network
  worker's `terminal / serial_ended` result is a later secondary symptom.
- Disposition: Attempt-042 is consumed. Reproduce and fix the state machine so
  terminal causes cancel resumability and an already-stopped lease becomes
  consumed without a duplicate effect. No attempt-043 is authorized until the
  correction passes complete gates and is pushed.

## 2026-08-15T12:51:28Z | terminal lease consumption fixed and verified

- Reproduction: Deterministic production-session tests captured both failure
  shapes: expiry after a confirmed resumable stop left the lease `armed`, and
  expiry while that stop was pending allowed its later confirmation to re-arm
  the terminal lease.
- Fix: Entering any terminal safe stop now clears resumability. An already-
  stopped admitted lease is consumed immediately using its prior hardware-stop
  confirmation; an in-flight stop consumes when its confirmation arrives. No
  duplicate hardware-stop effect is emitted.
- Diagnostic: The serial-to-network handoff recognizes the old contradictory
  consumed-reason/non-consumed-state marker and preserves
  `terminal_state_unconfirmed / serial_witness` before input closure, so the
  secondary `serial_ended` symptom cannot replace the primary failure.
- Verification: Focused suites, ordered Cargo gates, Bright Builds, real
  firmware, all 45 Bazel tests, parity/progress, redaction, pinned reference,
  sensitive-output, and diff checks pass. No hardware effect occurred.

## 2026-08-15T12:54:20Z | attempt-043 terminal-consumption contract drafted

- Objective: Run one no-human exact-package campaign against clean pushed fix
  `cf41ecaf`, proving the terminal lease remains consumed after the complete
  command sequence or producing one new typed boundary for diagnosis.
- Admission/layout: Require exact HEAD/reference package identity, opaque
  ignored input, one protected detector, a separate protected wrapper, and
  absent command-owned attempt/public paths.
- Effects/privacy: Existing finite campaign, recovery, safe-stop, cleanup,
  request-once, deadline, same-device, and redaction contracts are unchanged.
  No physical-display claim is included.
- Stop: Campaign start consumes attempt-043. Any non-ready result withholds
  evidence and stops without attempt-044 or an unchanged retry.

## 2026-08-15T13:20:00Z | attempt-043 isolated recovery-poll starvation

- Result: The one detector-gated exact-package command-effects campaign passed
  every machine command postcondition, consumed its terminal lease, confirmed
  safe stop, and released USB. The later common restart transaction stopped
  `service_recovery_timeout`; the public projection remained absent and no
  user action contributed.
- Restart evidence: Exactly one restart request was written and acknowledged.
  Service loss, post-restart receive-only serial delivery, the same stable
  physical USB device, and final device-session cleanup were all observed.
- Root cause: The first recovery GET connected during service shutdown and
  wrote its request but received no response. That exchange inherited the full
  360-second transaction deadline as its socket read timeout, preventing every
  later recovery poll even after the device returned.
- Disposition: Attempt-043 is consumed. Bound each recovery exchange within
  the overall deadline and prove that a stalled first request yields to a later
  successful poll. No attempt-044 is authorized until the software correction
  passes complete gates and is pushed.

## 2026-08-15T13:32:00Z | recovery-poll starvation fixed and verified

- Fix: Each post-restart system-info observation now uses the shared
  10-second HTTP exchange budget capped by the remaining transaction deadline.
  A connection accepted during shutdown can no longer monopolize the complete
  recovery window.
- Regression: A production-shaped loopback server accepts the first request
  and withholds its response; that poll times out independently and a second
  request succeeds. A separate boundary test proves the exchange cap never
  extends the overall deadline. Existing real-child transaction/file evidence
  integration remains green.
- Verification: All device-session tests, ordered Cargo gates, Bright Builds,
  real firmware, all 45 Bazel tests, parity/progress, redaction, pinned
  reference, sensitive-output, and diff checks pass. No hardware effect
  occurred.
- Disposition: The software correction is ready to publish. Attempt-043 remains
  consumed; a fresh hardware campaign requires a separately committed
  attempt-044 contract.

## 2026-08-15T13:38:00Z | attempt-044 recovery-poll contract drafted

- Objective: Run one no-human exact-package campaign against clean pushed fix
  `5ba7c192`, proving bounded post-restart observations can recover and seal the
  complete programmatic projection or producing one new typed boundary.
- Admission/layout: Require exact current HEAD/reference package identity,
  opaque ignored input, one protected detector, a separate protected wrapper,
  and absent command-owned attempt/public paths.
- Effects/privacy: Existing finite campaign, recovery, safe-stop, cleanup,
  request-once restart, same-device, and redaction contracts are unchanged. No
  physical-display claim is included.
- Stop: Campaign start consumes attempt-044. Any non-ready result withholds
  evidence and stops without attempt-045 or an unchanged retry.

## 2026-08-15T13:50:47Z | attempt-044 programmatic quorum passed

- Result: One detector-gated exact-package campaign completed successfully and
  emitted the independently validated redacted v1 projection. No user action
  or physical-display claim contributed.
- Command proof: Pause, resume, IDENTIFY, and dismissal each executed exactly
  once with their claim-specific machine postconditions. IDENTIFY obtained
  successful render and later non-IDENTIFY clear receipts plus retained and
  receive-only USB transition witnesses. Terminal safe stop, pool persistence,
  disabled mining/control, and cleanup all passed.
- Restart proof: One acknowledged software-restart request observed service
  loss, correlated pre/post serial, the same stable physical device, trusted
  origin, exact build recovery, a changed boot session, ordinal N+1, matching
  reset reason/postcondition, and cleanup. The session closed `ready` after two
  HTTP observations, proving the bounded-poll correction.
- Disposition: Attempt-044 is consumed and no programmatic retry is authorized.
  Preserve its sealed projection independently. API-009 remains `implemented`
  only until the separate unbounded-readiness physical-display UAT confirms
  the IDENTIFY frame and its clearing.

## 2026-08-15T13:50:47Z | physical-display UAT contract drafted

- Readiness: Wait indefinitely for the user to say they are watching; no chat
  or observation deadline applies. Then use one fresh detector and one bounded
  display-UAT machine pass bound to the sealed attempt-044 evidence.
- Proof: Software must independently prove same package/boot, one IDENTIFY
  request, successful framebuffer render, and later clear. The user supplies
  one durable response confirming both visible observations; software receipts
  never stand in for illuminated pixels.
- Replay: A missed human observation may replay only the isolated display UAT
  with a fresh private ordinal. It never reruns or invalidates the completed
  mining/command/restart campaign.
- Promotion: Finalize an aggregate-only redacted UAT projection and promote
  API-009 only when both projections, restoration, cleanup, parity, reference,
  redaction, and diff gates pass.

## 2026-08-15T14:15:00Z | display UAT attempt-001 failed before effect admission

- Result: After one fresh detector passed, the bounded UAT process stopped
  before its initial command-status admission. The command-owned root remained
  empty, so no USB inspection, IDENTIFY request, human checkpoint, or public
  evidence occurred. The user's readiness action did not cause the failure.
- Diagnosis: The original CLI exposed only `host_error`, so attempt-001 cannot
  distinguish an unavailable response from malformed command status after the
  fact. Independently, the UAT contract contained a deterministic mismatch: it
  permits an indefinite human delay, retained the earlier campaign origin, and
  replayed the current connected origin on receive-only USB for only six
  minutes. After that lease expires, a delayed UAT cannot refresh its target
  through any policy-compliant mechanism.
- Fix contract: Keep the current connected origin privately observable on USB,
  require one fresh same-session origin observation before HTTP/USB inspection,
  and preserve a closed typed admission failure before any IDENTIFY. Because
  this changes firmware, rerun the programmatic campaign against the new exact
  package before a fresh isolated physical UAT.
- Disposition: Attempt-001 is consumed without a display effect. No unchanged
  retry is authorized; attempt-002 follows only after the correction passes all
  gates, is committed and pushed, and new exact-package programmatic evidence
  is sealed.

## 2026-08-15T16:54:36Z | delayed display UAT admission fixed and verified

- Firmware: The current connected origin now remains observable at the existing
  ten-second private USB cadence for the connection lifetime and is cleared
  immediately on station disconnect. A pure regression proves observation
  remains due after a one-day human delay.
- Host: Display UAT intent v2 contains no origin. The live command admits one
  fresh private runtime-origin marker, binds its boot session to command status,
  then performs the existing same-physical-device and exact-build inspection
  before the single IDENTIFY request. Missing/conflicting markers, unavailable
  HTTP, malformed status, boot drift, and later postcondition failures retain
  closed terminal categories plus a private zero-or-one request receipt.
- Regression: Focused parser, loopback transport, malformed-status, privacy,
  and real-child CLI tests pass. The production-shaped API campaign emits the
  origin-free intent and its existing no-human simulation remains green.
- Verification: Ordered Cargo format/clippy/build/test gates, Bright Builds,
  the real ESP32-S3 firmware build, all 45 Bazel test targets, parity/progress,
  redaction, pinned-reference cleanliness, sensitive-output review, and diff
  checks pass. No hardware effect occurred.
- Disposition: The correction is ready to commit and push. A separately
  committed attempt-045 contract is still required before rebuilding/flashing
  and replacing the exact-package programmatic evidence.

## 2026-08-15T17:02:00Z | attempt-045 exact-package refresh contract drafted

- Objective: Refresh the no-human API-009 programmatic projection against the
  durable delayed-UAT firmware fix pushed as `3e6d88f6`, without making or
  depending on a physical-display claim.
- Admission/layout: Require a clean pushed contract HEAD, exact HEAD/reference
  package, opaque ignored Wi-Fi input, one protected detector, separate
  protected wrapper, absent command-owned attempt root, and a new projection
  path that preserves the truthful attempt-044 artifact.
- Effects/privacy: Retain the established single exact-package flash, bounded
  conservative mining, pause/dismiss/IDENTIFY/resume/restart, same-device
  recovery, safe-stop, cleanup, finite deadlines, and aggregate-only redaction
  contracts. No additional hardware or control surface is authorized.
- Stop: Campaign start consumes attempt-045. Any non-ready result withholds the
  new projection and stops without attempt-046 or an unchanged retry.

## 2026-08-15T17:45:00Z | attempt-045 consumed safely at asynchronous pause join

- Admission: Clean synchronized pushed contract, exact package, one protected
  detector, one admitted board-205 device, private modes, and ignored opaque
  Wi-Fi input all passed before the single campaign invocation.
- Result: The campaign stopped with typed `hardware_blocked` in the pause phase
  after one pause request. The new public projection was withheld. No restart
  occurred, recovery confirmed pause and safe stop, and process/USB cleanup
  succeeded without a secondary recovery failure.
- Root cause: Serial campaign markers expose a current safe-stop fact that may
  be replaced by later markers. `PauseJoinState` latched the independent HTTP
  pause fact but required serial safe-stop to still be true in that same poll.
  Its serial-first unit path repeated the serial fact and therefore missed the
  disjoint-cycle failure observed on hardware.
- Correction: Latch both boot-scoped witnesses within the bounded pause join
  and add a production-seam regression where serial confirmation arrives first
  and is false by the time the HTTP pause generation arrives. Attempt-045 is
  archived and may not be retried; attempt-046 requires verified pushed code
  plus a separate committed contract.

## 2026-08-15T18:10:00Z | asynchronous pause join correction verified

- Implementation: `PauseJoinState` now latches both independent boot-scoped
  facts for its bounded lifetime. The serial observer remains current-state
  truth for other consumers. Request-once behavior and exact-deadline
  fail-closed semantics are unchanged.
- Regression: The pure join test now uses genuinely disjoint serial-first
  samples, and the production command-effects seam proves a one-shot serial
  safe-stop followed by later HTTP pause generation issues exactly one dismiss
  request without correlation failure.
- Verification: Focused tests, ordered Cargo format/clippy/build/all-feature
  tests, Bright Builds, firmware build, all 45 Bazel tests, parity/progress,
  redaction, pinned-reference cleanliness, and diff review pass. A transient
  host resource error during one parity report disappeared on an unchanged
  isolated retry and did not correspond to a parity assertion failure.
- Next gate: Publish the correction before creating a separate attempt-046
  contract. No hardware effect is authorized by the software-fix task.

## 2026-08-15T18:20:00Z | attempt-046 exact-package refresh contract drafted

- Published prerequisite: The asynchronous pause-join correction and complete
  software verification were pushed as `dcb01c58` before this contract was
  created.
- Admission/layout: Require a clean pushed contract HEAD, exact HEAD/reference
  package, opaque ignored Wi-Fi input, one protected detector, separate
  protected wrapper, absent command-owned attempt root, and a new projection
  path preserving attempts 044 and the withheld 045 path.
- Effects/privacy: Retain the established single exact-package flash, bounded
  conservative mining, pause/dismiss/IDENTIFY/resume/restart, same-device
  recovery, safe-stop, cleanup, finite deadlines, and aggregate-only redaction
  contracts. No additional hardware or control surface is authorized.
- Stop: Campaign start consumes attempt-046. Any non-ready result withholds the
  new projection and stops without attempt-047 or an unchanged retry.

## 2026-08-15T18:45:00Z | attempt-046 exact-package programmatic proof accepted

- Admission: Clean synchronized pushed contract `522d5abd`, exact package,
  opaque ignored Wi-Fi input, absent fresh roots, and one protected detector
  admitting exactly one board-205 device passed. The post-validator initially
  expected a JSON envelope while the current detector CLI emits colon-delimited
  labels; the successful capture was validated in place without rerunning the
  hardware command.
- Result: The single campaign invocation succeeded and published the redacted
  attempt-046 projection. One pause, dismiss, IDENTIFY, resume, and restart each
  met their claim-specific postconditions; the restart proved reader-first
  request-once behavior, same physical device, exact build, changed boot
  session, ordinal N+1, service loss, software reset, recovery, and cleanup.
- Safety/privacy: Safe stop, disabled mining and hardware control, no recovery
  requirement, no secondary failure, protected mode-0700/mode-0600 artifacts,
  no symlinks, and redaction independently pass. No private identity, origin,
  port, credential, boot session, value, body, or trace was published.
- Disposition: Attempt-046 is archived as accepted. Its exact private display
  intent and public projection replace the pre-fix programmatic binding for the
  outstanding independently replayable physical display UAT.

## 2026-08-15T19:05:00Z | display UAT attempt-002 stopped before effect

- Admission: The pushed attempt-046 programmatic evidence, private intent,
  fresh detector, and bounded receive-only runtime-origin capture were ready.
- Result: `display-uat-live` returned generic `host_error` before creating its
  private root or admission receipt. No IDENTIFY request was sent, no display
  observation was requested, and no public UAT projection was produced.
- Root cause: The committed command contract requires an absent fresh attempt
  root, but the CLI delegated directly to a library function that requires an
  already existing empty mode-0700 root. The CLI owned neither creation nor a
  typed path across that boundary.
- Correction: `display-uat-live` now atomically creates the absent root at mode
  0700 before entering typed admission. A real-child regression begins with an
  absent root and proves a typed pre-effect failure creates the root, one
  mode-0600 admission receipt, and zero identify requests.
- Stop: Attempt-002 is consumed. A fresh attempt-003 requires the verified
  correction to be committed and pushed; it may reuse the accepted
  programmatic campaign without reflashing or rerunning mining.

## 2026-08-15T19:20:00Z | display UAT attempt-003 stopped before effect

- Result: The fresh detector and bounded runtime-origin capture passed, but the
  machine command again returned `host_error` before creating its root or
  receipt. No IDENTIFY request was sent and no visual observation was requested.
- Root cause: The root-creation correction was present, but Bazel launched the
  direct device-session CLI from a non-workspace directory. All committed UAT
  arguments were repository-relative, while the CLI resolved them against its
  process directory and failed while loading the first private input.
- Correction: Display-live and display-finalize now resolve relative inputs and
  outputs against `BUILD_WORKSPACE_DIRECTORY`, retaining absolute paths as-is
  and using the current directory only outside Bazel. The real-child regression
  launches from a different directory with relative arguments.
- Reproduction: A production-shaped Bazel launch using the exact private inputs,
  an intentionally mismatched public evidence digest, and a synthetic port now
  creates the absent mode-0700 root and returns a typed `evidence_invalid`
  receipt with zero IDENTIFY requests before any transport or hardware access.
- Stop: Attempt-003 is consumed. Attempt-004 requires full gates plus a pushed
  correction and may reuse the accepted programmatic proof without reflashing.
