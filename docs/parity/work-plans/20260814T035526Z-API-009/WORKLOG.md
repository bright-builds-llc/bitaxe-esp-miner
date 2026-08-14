# Parity work log

## 2026-08-14T03:55:26Z | immutable-plan draft

- Source commit: `13f75265bb5439594bef95d69bebc0974705b5d9`.
- Actions: Selected API-009 first from the clean synchronized selector and
  drafted one bounded attempt-015 contract for the newly verified
  startup-intent fix.
- Verification: Plan-only software, privacy, reference, firmware, selector,
  task, immutable-digest, and diff gates are pending before commit.
- Evidence: Public source and task/plan metadata only. No credential,
  detector, protected attempt artifact, USB, device, network, display, mining,
  or hardware interface was accessed.
- Outcome: Attempt-015 remains ineligible until this immutable contract and all
  named plan gates are committed and pushed at clean synchronized HEAD.
- Blocker or next safe action: Run the complete plan gate sequence, review the
  diff, commit and push, then perform exact-package admission before the sole
  detector run.

## 2026-08-14T03:59:28Z | immutable-plan verification

- Plan SHA-256:
  `6d07b66930f3af731392c1499802a06495e5fdf0af687dcc75f787b344c6922d`.
- Actions: Ran the complete plan-only gate sequence and queried the selector
  against the unique API-009 task binding.
- Verification: Formatting, strict Clippy, all-target build, all-feature tests,
  Bright Builds, all 44 canonical Bazel test targets, parity, parity-progress,
  focused startup-intent/campaign/Stratum/flash/automation/real-process tests,
  redaction, reference cleanliness, the real ESP firmware build, task
  uniqueness, selector ownership, immutable digest, and diff checks pass.
- Evidence: Public plan/task/source and category-only gate outcomes. No
  credential, protected attempt artifact, detector, USB, device, network,
  display, mining, or hardware interface was accessed.
- Outcome: The exact-package, detector-gated attempt-015 contract is ready to
  commit and push without changing API-009 from `implemented`.
- Blocker or next safe action: Push this checkpoint, confirm clean synchronized
  HEAD and the same open plan, then perform exact-package admission before the
  sole detector run.

## 2026-08-14T04:13:04Z | attempt-015 terminal checkpoint

- Source commit: `843ac0c9271c051d365deb23bfb107d13b51a1a1`.
- Package app ELF SHA-256:
  `cd5ad1429f77774611a05b624afb5338611d1891715cf0fc2470e54020d69894`.
- Actions: Built and independently validated the exact clean package, required
  the ignored Wi-Fi credential input without reading it, ran the detector once,
  and consumed the single authorized attempt-015. No physical checkpoint was
  confirmed or inferred.
- Verification: The detector admitted one holder-free board-205 device.
  Factory and NVS flashes completed, runtime identity was trusted, the protocol
  gate was ready, safe stop and USB cleanup were confirmed, private modes
  passed, and no related process or port holder remained. The public projection
  is absent. The serial diagnostics contain 145 candidates: the first event is
  one 1,490-byte `marker_json_invalid` line at the ingress boundary, followed by
  144 accepted 2,516–2,578-byte markers with no invalid encoding or trailing
  partial line.
- Evidence: Exact package identity and sealed category/count/length/boolean
  diagnostics only. No origin, hostname, USB/network identity, credential,
  worker, address, password, token, sensor value, or raw trace was published.
- Outcome: `blocked`. The public wrapper preserved `hardware_blocked`; evidence
  is withheld and API-009 remains `implemented`.
- Blocker or next safe action: The live receive path does not establish a
  newline boundary before feeding its first post-open bytes to the campaign and
  network analyzers. Close this attempt and start a software-only plan for one
  shared line-admission boundary. Attempt-016 is not authorized.
