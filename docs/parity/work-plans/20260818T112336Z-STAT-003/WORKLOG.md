# Parity work log

## 2026-08-18T11:36:07Z | disabled boot-mining state correction

- Source commit: `251205a57b42a1f0a1e4a59a0d90dd5b5837f5af`.
- Actions: added one pure `bootMiningDisabled` predicate requiring false boot
  intent plus `paused` or `safe_blocked`; applied it to restart admission and
  final evidence; extended the real-child server and workflow coverage for a
  paused post-restart device.
- Verification: pure table coverage accepts both closed non-active states and
  rejects active, unknown, and enabled shapes. Full real-child paused restart
  reaches post-restart scoreboard persistence and public projection; existing
  safe-blocked success and restart-drift withholding remain green. Focused
  automation, ordered Cargo gates, Bright Builds, all 47 Bazel targets, parity,
  progress, firmware build/package, redaction, reference, source inventory,
  file-size, sensitive-value, and diff checks passed.
- Evidence: the single predicate is used by both admission and projection, and
  its source plus consumers remain bound by the 31-path evaluator identity.
- Outcome: software correction complete, committed, and pushed. No detector,
  credentials, protected attempt evidence, device, USB, network runtime, flash,
  monitor, mining, restart, projection, recovery, or attempt-004 was used.
- Blocker or next safe action: close without parity transition. A future
  immutable hardware plan may rotate to attempt-004, package this correction,
  and run one detector-gated verification. Repetition of the attempt-003 stopped
  state after this targeted fix must stop without another retry.
