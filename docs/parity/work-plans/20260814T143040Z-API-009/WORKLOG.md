# Parity work log

## 2026-08-14T14:30:40Z | immutable-plan draft

- Source commit: `eacaea0fa9595a2bca60211af88370b9b655a69b`.
- Actions: Selected the still-active API-009 task and drafted one bounded
  attempt-018 contract for the verified startup-order repair.
- Verification: Plan-only mandatory, privacy, reference, firmware, selector,
  task, immutable-digest, and diff gates are pending before commit.
- Evidence: Public source and prior closure facts only. No credential,
  protected attempt content, detector, USB, device, network, display, mining,
  or hardware interface was accessed.
- Outcome: Attempt-018 remains ineligible until this immutable contract and all
  named plan gates are committed and pushed at clean synchronized HEAD.
- Blocker or next safe action: Run the complete plan gate sequence, review the
  diff, commit and push, then perform exact-package admission before the sole
  detector run.

## 2026-08-14T14:35:04Z | immutable-plan verification

- Plan SHA-256:
  `59153655eec37e959493f4fa96d661bf5ba5db8215363f83b1230f418b59229c`.
- Actions: Ran the complete plan-only gate sequence and confirmed the selector
  resumes this unique API-009 attempt-018 plan.
- Verification: Formatting, strict Clippy, all-target build, all-feature tests,
  Bright Builds, canonical Bazel tests, focused startup/Wi-Fi/campaign tests,
  parity, parity-progress, redaction, reference cleanliness, real ESP firmware
  build, task uniqueness, open-plan selection, immutable digest, fresh paths,
  and diff checks pass.
- Evidence: Public plan/task/source and category-only gate outcomes. No
  credential, protected attempt content, detector, USB, device, network,
  display, mining, or hardware interface was accessed.
- Outcome: The exact-package detector-gated attempt-018 contract is ready to
  commit and push without changing API-009 from `implemented`.
- Blocker or next safe action: Push this checkpoint, confirm clean synchronized
  HEAD and the same open plan, then perform exact-package admission before the
  sole detector run.

## 2026-08-14T14:48:57Z | exact-package attempt and terminal closure

- Source commit: `dc72ad075a374b54c29f158bd684f16a99d21770`.
- Actions: Confirmed clean synchronized HEAD, built the exact Ultra 205
  package, checked only that the ignored Wi-Fi input was non-empty, ran the
  fresh detector once, and invoked the sole attempt-018 campaign once. Sent
  ready, rendered, and cleared signals only after their matching live typed
  checkpoints and physical reports.
- Verification: One board-205 session was admitted. Factory and NVS transfers
  each completed once as `ready`; runtime attestation became `trusted`; the
  genuine notification, pause, safe stop, resume, active recovery, and rendered
  observation passed. The typed terminal result is
  `network_target_unavailable` / `safety_prerequisites_stale`: after 40,707 ms
  active, all five Ultra 205-required observations were fresh and
  `vr_temp_celsius` was correctly not required, while the aggregate mining
  safety sample itself was stale. The cleared signal was not consumed before
  terminal closure.
- Evidence: Only categorical fields, booleans, counts, and bounded durations
  were inspected. Credential, port, USB/network identity, origin, hostname,
  sensor values, and raw traces remain protected.
- Outcome: Attempt-018 is consumed. Safe stop is confirmed, recovery was
  attempted without a secondary failure, USB cleanup is ready, protected modes
  pass, the campaign process is absent, and the public projection is withheld.
  API-009 remains `implemented`.
- Blocker or next safe action: Close this immutable plan without attempt-019.
  Investigate why the command flow leaves mining active throughout the
  physical IDENTIFY windows before considering any later hardware contract.

## 2026-08-14T15:19:10Z | post-closure root-cause fix

- Actions: Built a deterministic red diagnostic at the production command-
  effect seam, reproduced it twice, minimized the transition sequence, and
  changed checkpoint ordering without modifying the immutable plan.
- Verification: The diagnostic proved that consuming ready released the
  initial pause before the two unbounded physical observation windows. The
  attempt-018 typed facts independently show all five required observations
  were fresh and VR temperature was correctly not required, falsifying the
  missing-sensor and parser hypotheses. The fixed sequence keeps the pause
  through ready, rendered, and cleared, resumes exactly once afterward, then
  confirms active recovery before dismissal. Focused flash and real-child
  automation tests plus the complete Cargo, Bright Builds, Bazel, parity,
  privacy, reference, firmware, and diff gates pass.
- Evidence: Public source, deterministic tests, and attempt-018 categorical,
  boolean, count, and bounded-duration facts only. No protected value, raw
  trace, credential, detector, USB, device, network, or hardware interface was
  accessed.
- Outcome: The active-window stale-readiness root cause is fixed in software.
  API-009 remains `implemented`; attempt-018 remains closed and unchanged.
- Blocker or next safe action: Commit and push the verified software repair.
  Any future hardware attempt requires a separate immutable contract and new
  objectively verified progress; this work authorizes no attempt-019.
