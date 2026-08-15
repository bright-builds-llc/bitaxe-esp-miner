# Parity work log

## 2026-08-14T23:49:17Z | immutable-plan draft

- Source commit: `168e599ec6a76224232c089095a693e68b1cce8d`.
- Actions: Confirmed clean synchronized HEAD, no open plan, API-009 first, and
  the completed latency-tolerant software boundary; drafted one exact-package,
  detector-gated attempt-023 contract.
- Verification: Plan-only focused, mandatory, privacy, reference, firmware,
  selector, task, digest, and diff gates are pending before commit.
- Evidence: Public source, selector, task, plan, and categorical closure facts
  only. No credential, protected attempt artifact, detector, USB,
  device/network, display, mining, hardware-control, UART, or pin interface was
  accessed.
- Outcome: Attempt-023 remains ineligible until this immutable contract and its
  complete plan-only gate sequence are committed and pushed.
- Blocker or next safe action: Verify, commit, and push the plan before package,
  credential, detector, USB, device/network, mining, display, or restart access.

## 2026-08-15T00:01:00Z | immutable-plan verified

- Source commit: `168e599ec6a76224232c089095a693e68b1cce8d`.
- Actions: Kept the attempt-023 plan immutable and reviewed its exact effect,
  observation, recovery, privacy, retry, and stop boundaries.
- Verification: Focused delayed-attestation, natural-expiry, replay, campaign,
  and real-process targets pass. Cargo format, strict Clippy, all-target build,
  all-feature tests, Bright Builds, all 44 Bazel test targets, parity,
  parity-progress, redaction, reference cleanliness, real firmware build,
  selector, unique task, plan digest, and diff checks pass. Plan SHA-256 is
  `ab36b8e974eac2ce6be40c6cf7c569d68ae52780af6da7f1e7548f71af75eaa9`.
- Evidence: Public software, task, plan, and categorical gate results only. No
  credential, protected attempt artifact, detector, USB, device/network,
  display, mining, hardware-control, UART, or pin interface was accessed.
- Outcome: The immutable attempt-023 contract is ready to commit and push.
- Blocker or next safe action: Publish this plan checkpoint before building the
  exact package or accessing credentials, detector, USB, or device/network.

## 2026-08-15T00:18:00Z | exact-package attempt and terminal closure

- Source commit: `5f137f6f7b4dd168d81ba059ce67cdd872064d38`.
- Actions: Built and validated the exact package, ran the fresh detector once,
  and invoked the sole attempt-023 campaign once. Consumed ready, rendered,
  and cleared only after their matching live user inputs. No replay was needed.
- Verification: One board-205 session, trusted runtime identity, genuine
  notification, positive block count, pause, stopped hardware, one IDENTIFY
  request, live rendered observation, natural expiry, and live cleared
  observation passed. The resume request was issued, but active mining did not
  return before the fixed 15-second automated phase deadline. The earliest
  typed result is `network_correlation_failed`; resume confirmation, dismissal,
  restart, and public evidence are absent. USB cleanup is ready, all private
  modes pass, the result seal is valid, attempt processes are absent, recovery
  reports a secondary failure, and terminal safe stop is unconfirmed.
- Evidence: Only closed categorical fields, booleans, counts, bounded elapsed
  durations, modes, and source identities were inspected. Credential, raw
  port/USB/network identity, origin, hostname, sensor values, and traces remain
  protected.
- Outcome: Attempt-023 is consumed. API-009 remains `implemented`; the
  latency-tolerant IDENTIFY transaction passed, but the complete command and
  restart quorum did not.
- Blocker or next safe action: Close this immutable plan without attempt-024.
  A fresh software-only plan should diagnose the resume-to-active boundary and
  recovery-safe-stop failure before any later hardware plan.
