# Parity work log

## 2026-08-18T02:22:12Z | audited plan checkpoint

- Source commit: `221430b1c6ac91993f9bc4e425600455746732e2`
- Actions: selected `STAT-001` after deterministic `SELF-001` and `BAP-002`
  skips; created immutable attempt-018 plan and matching active task contract.
- Verification: parity `next-item` admitted this plan as the sole open plan;
  ordered Rust, Bright Builds, Bazel, parity, progress, and diff gates passed.
- Evidence: plan SHA-256
  `cd641d862cd1246b7905ec2389c2fad17a61b45090cb85496f82ed32bf7989d3`.
- Outcome: plan/task checkpoint committed and pushed at `8d474a9d`.
- Blocker or next safe action: rebind the closed workflow before device access.

## 2026-08-18T03:45:56Z | attempt-018

- Source commit: `e14b98d53751867588870da293a9110b8c2f1d84`
- Actions: rebound ordinal, immutable plan, protected roots, task admission,
  Rust validator, TypeScript contracts, fixtures, and Bazel runfiles; passed
  focused/full gates; built the exact package; ran only the frozen detector and
  sole conditional capture.
- Verification: detector admitted exactly one Ultra 205; package source,
  pushed source, pinned reference, plan digest, protected roots, input
  presence, and pre-effect absence checks passed. The sealed campaign result
  and private diagnostics were inspected only through closed allowlisted
  fields after the wrapper exited nonzero.
- Evidence: ignored mode-0700 `wrapper-018` and `attempt-018` roots contain
  mode-0600 private artifacts. The sealed result reports 273,286 active ms,
  9/20 windows, stable/valid watchdog, fresh safety, ready USB cleanup, mixed
  session/ordinal identity, and first mixed reset category `panic`. The public
  projection is absent.
- Outcome: `hardware_blocked`; `STAT-001` remains `implemented` and no checklist
  or progress transition is authorized.
- Blocker or next safe action: create a separate software-only plan that adds a
  value-free panic discriminator at the retained boot/crash boundary and
  produces a regression-backed root-cause fix. Do not run attempt-019 until
  that new plan proves materially changed source and a complete authority
  contract.
