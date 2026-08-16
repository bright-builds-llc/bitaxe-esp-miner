# STAT-001 worklog

## 2026-08-16 selection and immutable plan

- Action: Re-ran the clean synchronized deterministic selector, classified the
  earlier rows' concrete blockers, and selected STAT-001 as the first row with
  verified source progress after consumed attempt-007.
- Verification: Source and pinned reference are clean and synchronized;
  wrapper-008, attempt-008, and the public projection are absent. The current
  workflow and reference breadcrumbs preserve separate volts for input safety
  and millivolts for the ASIC core-voltage command.
- Outcome: Froze one detector-gated attempt-008 with exact effects, safety,
  privacy, recovery, cleanup, retry, stop, and promotion boundaries. No device,
  credentials, protected attempt, or network runtime was accessed.
- Verification: Ordered Cargo format, strict Clippy, all-target build,
  all-feature tests, Bright Builds, all 45 Bazel tests, package,
  parity/progress, redaction, reference, selector, immutable-plan hash, fresh
  path, and diff gates pass. The admitted immutable plan SHA-256 is
  `6fdb431561daeae1b43e2856330c63df629917f5f3e9f596548731923530915a`.
- Next safe action: Commit and push this immutable plan/task checkpoint before
  editing any implementation file.
