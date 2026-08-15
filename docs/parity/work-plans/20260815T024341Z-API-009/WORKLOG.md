# Parity work log

## 2026-08-15T02:43:41Z | immutable-plan draft

- Source commit: `9c82096fe106ceed9c1dafd0633e66b70a261fde`.
- Actions: Confirmed clean synchronized HEAD, no open plan, API-009 first, the
  closed reactivation-safety repair, and a non-empty ignored Wi-Fi credential
  input without reading it; drafted one exact-package attempt-025 contract.
- Verification: Plan-only focused, mandatory, privacy, reference, firmware,
  selector, task, digest, and diff gates are pending before commit.
- Evidence: Public source, selector, task, plan, and categorical prior-closure
  facts only. No credential contents, protected attempt artifact, detector,
  USB, device/network, display, mining, hardware-control, UART, or pin
  interface was accessed.
- Outcome: Attempt-025 remains ineligible until this immutable plan and its
  complete plan-only gate sequence are committed and pushed.
- Blocker or next safe action: Verify, commit, and push the plan before package,
  detector, credential, USB, network, mining, display, or restart effects.

## 2026-08-15T03:00:12Z | plan-only gates passed

- Plan SHA-256:
  `35382a8c693ead5b04cbfb3e7011c0319b0618612a01de11dbad1cb0e7d34ab6`.
- Actions: Kept the plan and active task within the single attempt-025 scope;
  no implementation or device-facing source changed.
- Verification: Focused reactivation-safety, active-budget, command-effects,
  recovery, and firmware-owner tests passed. The complete ordered Cargo format,
  strict lint, all-target build, and all-feature test sequence; Bright Builds;
  all 44 Bazel tests; parity and progress; redaction; reference cleanliness;
  firmware build; selector; unique-task; immutable-plan digest; and diff checks
  passed. Twice, only when chained after the long Bazel suite, the high-volume
  parity report reached transient host `os error 35`; the exact isolated
  `just parity` command passed from a fresh process.
- Evidence: Public source, task, plan, tests, and categorical prior-closure
  facts only. No credential contents, protected attempt artifact, detector,
  USB, device/network, display, mining, hardware-control, UART, or pin
  interface was accessed.
- Outcome: The plan is eligible to be committed and pushed as the immutable
  attempt-025 authorization boundary. API-009 remains `implemented`.
- Blocker or next safe action: Commit and push this checkpoint, then require
  clean synchronized HEAD before package creation or detector admission.

## 2026-08-15T03:31:53Z | exact-package attempt and terminal closure

- Source commit: `bcb46964a48f5e815c9e39bb3b06607bf2370a4b`.
- Actions: Built and validated the exact package, ran the fresh detector once,
  invoked the sole attempt-025 campaign once, and consumed ready, rendered,
  and cleared only after their matching live user inputs. No replay was used.
- Verification: Trusted identity, genuine notification, positive block count,
  pause, stopped hardware, one IDENTIFY request, rendered and cleared
  observations, one resume request, recovery from stale to five fresh safety
  observations, active reactivation, and one accepted share passed. The sealed
  result then closed as `network_correlation_failed`: none of twenty required
  network windows were covered, so watchdog, work-renewal, terminal HTTP, and
  terminal WebSocket checks were false. Terminal safe stop, USB cleanup,
  private modes, fixture cleanup, result seal, and process cleanup pass.
- Evidence: Closed categorical fields, booleans, counts, and bounded active
  duration only. No origin, hostname, port, USB/network identity, credential,
  worker, address, password, token, sensor value, or raw trace was exposed or
  committed.
- Outcome: API-009 remains `implemented`; public evidence is withheld,
  attempt-025 is consumed, and no attempt-026 is authorized by this plan.
- Blocker or next safe action: Close this plan, then use a fresh software-only
  continuation to reproduce why the campaign network collector covered zero
  required windows despite a healthy resumed marker stream.

## 2026-08-15T03:39:07Z | terminal-boundary correction

- Actions: Validated the sealed command-effects member after source review
  showed that generic continuity windows do not govern command-effects
  completion.
- Verification: The command-specific record proves one dismiss request but no
  dismiss confirmation or block-count preservation. Its terminal HTTP check is
  consequently false. All prior command effects through active reactivation
  remain true. The generic zero-of-twenty counters are inherited non-applicable
  fields and are not the attempt's causal blocker.
- Outcome: The exact terminal boundary is the post-reactivation dismissal
  join, still reported as `network_correlation_failed`; cleanup, withholding,
  status, and non-promotion are unchanged.
- Blocker or next safe action: Reproduce the dismissal race at the command
  state-machine seam and move the single dismissal into a stable safe-stopped
  interval without weakening the genuine-notification or count-preservation
  quorum.
