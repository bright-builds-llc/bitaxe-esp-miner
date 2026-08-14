# Parity work log

## 2026-08-14T18:52:21Z | immutable-plan draft

- Source commit: `a4a48db77d33809e77315ba31bef553f32cf1651`.
- Actions: Ran the clean synchronized selector, selected API-009 first, and
  drafted one attempt-020 contract for the verified operator-gated lifetime
  repair.
- Verification: Plan-only mandatory, privacy, reference, firmware, selector,
  task, immutable-digest, and diff gates are pending before commit.
- Evidence: Public source and prior categorical closure facts only. No
  credential, protected attempt content, detector, USB, device, network,
  display, mining, or hardware interface was accessed.
- Outcome: Attempt-020 remains ineligible until this immutable contract and all
  named plan gates are committed and pushed at clean synchronized HEAD.
- Blocker or next safe action: Run the complete plan gate sequence, review the
  diff, commit and push, then perform exact-package admission before the sole
  detector run.

## 2026-08-14T19:00:14Z | immutable-plan verification

- Plan SHA-256:
  `2ca40f616dfadd517a7af45acb59f28180e2d8ada669a073bae6392e639a5082`.
- Actions: Ran the complete plan-only gate sequence and confirmed the selector
  resumes this unique API-009 attempt-020 plan.
- Verification: Formatting, strict Clippy, all-target build, all-feature tests,
  Bright Builds, canonical Bazel tests, focused operator-lifetime, command,
  campaign, and real-child tests, parity, parity-progress, redaction, reference
  cleanliness, real ESP firmware build, task uniqueness, open-plan selection,
  immutable digest, ignored credential/root admission, fresh attempt/detector/
  projection paths, and diff checks pass.
- Evidence: Public plan/task/source and category-only gate outcomes. The ignored
  Wi-Fi input was checked only for non-empty and ignored status. No credential,
  protected attempt content, detector, USB, device, network, display, mining,
  or hardware interface was accessed.
- Outcome: The exact-package detector-gated attempt-020 contract is ready to
  commit and push without changing API-009 from `implemented`.
- Blocker or next safe action: Push this checkpoint, confirm clean synchronized
  HEAD and the same open plan, then perform exact-package admission before the
  sole detector run.

## 2026-08-14T19:13:54Z | exact-package attempt and terminal closure

- Source commit: `6aec858230141521ca3079847061554de3f0f917`.
- Actions: Confirmed clean synchronized HEAD and the unique open plan, built
  and validated the exact package, ran the fresh detector once, invoked the
  sole attempt-020 campaign once, and consumed ready, rendered, and cleared
  only after their matching live physical reports.
- Verification: One board-205 session was admitted. Factory and NVS transfers
  each completed once as `ready`; runtime attestation was `trusted`; the
  genuine notification, positive block count, pause, paused safe stop, both
  IDENTIFY requests, and both physical IDENTIFY observations passed. The one
  resume request was issued but active recovery was not confirmed before the
  earliest `safety_stale` terminal marker. USB cleanup is `ready`, result digest
  and private modes pass, attempt processes are absent, and public evidence is
  withheld.
- Evidence: Only categorical fields, booleans, counts, and freshness flags were
  inspected. Credential, port, USB/network identity, origin, hostname, sensor
  values, and raw traces remain protected.
- Outcome: Attempt-020 is consumed. API-009 remains `implemented`; no dismissal,
  restart, terminal safe-stop, public evidence, or promotion claim is made.
- Blocker or next safe action: Close this immutable plan without attempt-021.
  Reproduce and fix the resume-time safety freshness mismatch in software
  before considering any later hardware attempt.

## 2026-08-14T19:34:09Z | resume-freshness root-cause repair

- Source commit: `6aec858230141521ca3079847061554de3f0f917` plus the uncommitted
  software repair described here.
- Actions: Built a production-shaped red regression for a stopped, armed
  command-effects resume; ranked acquisition, publication, recovery, and host
  terminal-policy hypotheses; and narrowed the host exception to the exact
  non-actuating resume-readiness state.
- Verification: The regression failed deterministically as `safety_stale`
  before the fix and passes after it. Active command-effects stale telemetry and
  observation-stage stale telemetry remain terminal negative controls. Focused
  command-effects, flash-tool, mandatory Cargo, Bright Builds, Bazel, parity,
  privacy, reference, firmware, and diff gates pass.
- Evidence: Repository source, deterministic fixtures, and the already closed
  categorical attempt facts only. No credential, protected trace, USB, device,
  network, HTTP, display, mining, or hardware interface was accessed.
- Outcome: The host no longer converts a transient resume-readiness sample into
  an irreversible campaign failure. Attempt-020's closure is unchanged and
  API-009 remains `implemented`.
- Blocker or next safe action: Commit and push this software repair. Any later
  hardware attempt requires its own immutable contract; attempt-021 was not
  created or run here.
