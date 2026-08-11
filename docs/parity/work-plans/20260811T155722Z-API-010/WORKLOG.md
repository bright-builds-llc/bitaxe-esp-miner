# Parity work log

## 2026-08-11T15:57:22Z | selection checkpoint

- Source commit: `e9b775166e7f93c2933fef8694204aaaaabde02f`.
- Actions: Selected `API-010` as the first actionable row after recording the
  concrete safety, hardware, network-environment, mining, and evidence blockers
  for preceding candidates. Linked the stable attempt-011 runtime result as
  objective proof that the prior bootloader/panic-loop boundary changed.
- Verification: Clean synchronized `main`; selector reports no open plan; new
  wrapper, attempt, and public projection paths are absent.
- Evidence: Plan and task preflight only; hardware untouched.
- Outcome: Immutable plan verification and push pending.
- Blocker or next safe action: Complete and push the plan gate, then run focused
  regressions and the single detector-gated capture.

## 2026-08-11T16:00:42Z | plan gate checkpoint

- Source commit: `e9b775166e7f93c2933fef8694204aaaaabde02f`.
- Actions: Corrected the uncommitted continuation link to the latest
  selector-participating non-verified closure while retaining attempt 011 as
  the objective progress basis.
- Verification: Ordered Cargo format, strict Clippy, all-target build,
  all-feature tests, Bright Builds, all Bazel tests, parity/progress, redaction,
  reference, selector, and diff checks passed. Plan SHA-256 is
  `00f99503aae8695761ddb1ca0ec96a2e8d8cd0d5827ca758ea9f4f08bbcdcb50`.
- Evidence: Plan and task only; hardware untouched.
- Outcome: Plan gate complete.
- Blocker or next safe action: Commit and push the immutable plan before
  focused regression or hardware work.

## 2026-08-11T16:14:13Z | attempt-012 completion checkpoint

- Source commit: `f2520d1e7c12f49dc376d528ddf78b6f9c5391c5`.
- Actions: Ran the focused theme, device-session, CLI, and redaction suite with
  no production change required; completed the full software gate; built and
  admitted the exact clean package; then ran one detector and one conditional
  theme-durability capture.
- Verification: Detector admission passed. The v1 projection binds one restart
  request and response, the same physical device, trusted exact-build recovery,
  changed boot session, ordinal `N+1`, software reset, immediate and
  post-restart theme equality, confirmed original-theme restoration, disabled
  mining and hardware control, and complete cleanup.
- Evidence: Redacted projection SHA-256
  `fbf93cd115e1c99cd1c727b2e4536c49f571b69172fc94228d4942181d005288`;
  both private roots are mode 0700, all 16 private attempt files and wrapper
  streams are mode 0600, no USB holder remains, and sensitive-value scans pass.
- Outcome: Complete and eligible to promote only `API-010` to `verified`.
- Blocker or next safe action: Commit the evidence/result without changing the
  checklist, save that commit as `SOURCE_COMMIT`, then perform the audited
  checklist transition and progress synchronization.
