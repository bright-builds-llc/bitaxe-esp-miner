# Parity work log

## 2026-08-15T04:11:03Z | immutable-plan draft

- Source commit: `f39892d0619f882010155357de38e1aacc642cc4`.
- Actions: Selected API-009 again after the pushed paused-dismissal repair and
  drafted one fresh attempt-026 contract with the corrected dismissal-before-
  IDENTIFY order.
- Verification: Clean synchronized HEAD, no open plan, API-009 first, pushed
  repair and closure, and the new attempt ordinal were confirmed. Plan-only
  focused, mandatory, privacy, reference, package, selector, digest, and diff
  gates remain pending.
- Evidence: Public source, tests, and prior categorical closure facts only. No
  credential, detector, USB, device/network, display, mining, hardware-control,
  protected attempt, or raw trace was accessed.
- Outcome: Hardware remains ineligible until this immutable plan and its task
  checkpoint pass, commit, and push.
- Blocker or next safe action: Gate, commit, and push this plan before any
  package or device-facing work.

## 2026-08-15T04:15:26Z | plan-only gates passed

- Plan SHA-256:
  `0608272dae94a34b60c7d5f092da1321479dae9aa737f4f29cfbb86fa8e5d379`.
- Actions: Kept the contract to one attempt-026 and bound its request order to
  the pushed dismissal-before-IDENTIFY repair. Confirmed the credential input
  is non-empty without reading it and that detector, attempt, and projection
  paths are fresh.
- Verification: Focused paused-dismissal, reactivation, active-budget,
  recovery, campaign, firmware-owner, loopback HTTP, and real-process tests;
  ordered Cargo format, strict lint, build, and tests; Bright Builds; all 44
  Bazel tests; parity/progress; redaction; reference cleanliness; firmware
  build; selector; unique task; immutable plan digest; sensitive-output; and
  diff checks pass.
- Evidence: Public source, task, plan, and tests only. No credential contents,
  protected attempts, detector, USB, device/network, display, mining,
  hardware-control, UART, or pin interface was accessed.
- Outcome: The plan is eligible to commit and push as the immutable attempt-
  026 authorization boundary. API-009 remains `implemented`.
- Blocker or next safe action: Commit and push this checkpoint, then require
  clean synchronized HEAD before package creation or detector admission.

## 2026-08-15T04:25:23Z | exact-package attempt and terminal closure

- Source commit: `5621ea0fe981efbc568279af9a35923c65e9bbed`.
- Actions: Built and validated the exact package, ran the fresh detector once,
  and invoked the sole attempt-026 campaign once. No operator checkpoint
  opened, so no ready, rendered, replayed, or cleared response was issued.
- Verification: The sealed result proves trusted package/runtime identity, a
  ready protocol gate, genuine positive notification, one pause, confirmed
  stopped hardware, one dismissal request, and notification clearing. Exact
  count preservation failed before IDENTIFY readiness. The protected command
  record proves recovery API pause, serial safe stop, and correlated recovery
  completion. Public output proves cleanup complete with no secondary recovery
  failure. Result and network seals, private modes, fixture stop, process
  cleanup, USB-holder absence, and public-projection withholding pass.
- Evidence: Approved categorical fields, booleans, counts, hashes, and modes
  only. No origin, hostname, port, USB/network identity, credential, worker,
  address, password, token, sensor value, or raw trace was exposed.
- Outcome: API-009 remains `implemented`; public evidence is withheld,
  attempt-026 is consumed, and this plan authorizes no retry.
- Blocker or next safe action: Close this attempt, then reproduce the stale
  count baseline in software and compare dismissal readback with the paused
  pre-request count rather than the earlier active-notification count.
