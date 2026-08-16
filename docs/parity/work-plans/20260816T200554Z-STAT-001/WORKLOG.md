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

## 2026-08-16 attempt-008 workflow rebind

- Plan commit: `0c9b1cb602630b9cee13cdc2b5b818bcb283e1b9`
  (pushed and synchronized before implementation).
- Action: Rebound only the protected wrapper/attempt roots, immutable plan and
  SHA-256 admission, Rust validator ordinal, canonical and generated
  TypeScript contracts, Bazel input, current-task guard, and fresh-process
  fixtures from consumed attempt-007 to attempt-008.
- Verification: Four focused Rust contract tests, generated-contract
  verification, and the canonical Bazel-isolated fresh-process automation
  suite pass. The unsupported direct Bun shortcut was discarded because it
  recursively loaded generated copies and cannot host these `node:test`
  runfile suites; it did not identify a repository defect.
- Verification: The ordered Cargo format, strict Clippy, all-target build,
  all-feature test, Bright Builds, all 45 Bazel tests, package,
  parity/progress, redaction, reference, selector, diff, and immutable-plan
  gates pass. The plan SHA-256 remains
  `6fdb431561daeae1b43e2856330c63df629917f5f3e9f596548731923530915a`.
- Simplification review: The complete implementation is an 18-line-for-18-line
  ordinal and admission rebind. No campaign, watchdog, safety, privacy,
  effect, recovery, or projection logic changed.
- Outcome: The implementation is eligible to become the clean pushed package
  source for the one authorized detector and conditional attempt-008 capture.
- Next safe action: Commit and push the implementation, rebuild and validate
  its exact clean package, then run only the two frozen hardware commands.
