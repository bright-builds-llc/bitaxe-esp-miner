# STAT-001 worklog

## 2026-08-16T19:20:25Z | Watchdog-timeout diagnosis and plan checkpoint

- Source commit: `e52832220f58638fe028fd14eeab98e3b9e73cf0`
- Actions: Ran the clean synchronized selector, skipped two concrete
  dependency/safety blockers, traced attempt-007's closed
  `watchdog_feed_stale` result through the pure evaluator, firmware producer,
  compiled sdkconfig, and pinned reference hashrate task.
- Verification: The Rust health evaluator uses an unrelated 2,000-ms constant;
  the exact firmware binding and generated package sdkconfig use a five-second
  ESP task-watchdog timeout; and upstream schedules hashrate work every second
  without the extra two-second failure boundary. No protected evidence,
  credentials, detector, device, or private values were accessed.
- Evidence: Repository source, exact generated configuration, and the active
  task's already-public closed discriminator identify a source-owned false
  stale boundary without requiring another hardware run.
- Outcome: STAT-001 is first actionable for a software-only correction. This
  plan authorizes no hardware ordinal and cannot promote the checklist row.
- Blocker or next safe action: Commit and push this immutable plan/task
  checkpoint, then parameterize the pure evaluator from the compiled ESP-IDF
  timeout and add focused regression coverage without editing `PLAN.md`.
