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

## 2026-08-16T19:35:00Z | Immutable plan synchronized

- Plan commit: `508ab40f`
- Plan SHA-256:
  `dbe601a0a1d54999d4d73438bb6f00156d0813ec8c59f8525c2842674e47c3e6`
- Verification: The mandatory Cargo sequence, managed Bright Builds checks,
  Bazel-backed `just test`, parity report, parity progress, redaction, and
  reference guard passed. The first combined run encountered a transient
  host `Resource temporarily unavailable` while emitting `just parity`; an
  immediate isolated retry passed with `validation_errors: none`, followed by
  successful remaining gates.
- Outcome: The immutable plan and active-task checkpoint are pushed to
  `origin/main`; implementation may now begin without editing `PLAN.md`.

## 2026-08-16T20:00:00Z | Compiled watchdog policy correction

- Actions: Replaced the pure evaluator's unrelated 2,000-ms freshness
  constant with a named `RuntimeHealthTiming` policy. The firmware adapter now
  converts the compiled `CONFIG_ESP_TASK_WDT_TIMEOUT_S` binding to
  milliseconds with checked arithmetic and supplies it to the pure evaluator.
  The passive-source guard permits that read-only symbol while continuing to
  reject the three effectful ESP task-watchdog calls.
- Regression proof: Focused tests prove a feed aged 2,001 ms remains fresh
  under the five-second policy, 5,000 ms is accepted, 5,001 ms is stale, and
  existing closed subscription/feed/unsubscription/sequence failures remain
  unchanged. API serialization and correlated projections remain unchanged.
- Firmware/package proof: `just build` and `just package` succeeded; the exact
  generated firmware sdkconfig records `CONFIG_ESP_TASK_WDT_TIMEOUT_S=5`.
- Full gate: `cargo fmt --all`, strict all-target/all-feature clippy, build,
  and tests; managed Bright Builds checks; `just test`; `just parity` with
  `validation_errors: none`; unchanged `just parity-progress`; redaction and
  reference guards; `git diff --check`; and immutable plan hashing all passed.
- Safety/privacy: No credentials, protected attempt artifact, detector,
  device, USB/network runtime, private value, or hardware effect was accessed.
- Outcome: The source-owned false-stale boundary is corrected. STAT-001 stays
  `implemented`; live accuracy and device behavior still require separately
  authorized hardware evidence.
- Blocker or next safe action: Commit and push this verified implementation,
  then write the non-promotion closure without changing the checklist or
  progress history.
