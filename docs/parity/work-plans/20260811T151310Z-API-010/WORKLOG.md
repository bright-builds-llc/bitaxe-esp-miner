# Ultra 205 canonical observation recovery work log

## 2026-08-11T15:13:10Z | plan checkpoint

- Source baseline: `0c25b17ea50ec3f922a19b15d1e5917b89dfe13a`.
- Reference baseline: `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- Confirmed cause: assignment-style campaign tokens were passed literally to
  Clap and rejected before process admission or hardware effects.
- Targeted change: prove the exact observation flag shape in the flash CLI
  unit suite and use canonical `--flag value` arguments in attempt 011.
- Hardware status: untouched by the attempt-010 campaign invocation. The prior
  detector passed, but attempt 011 will perform its own detector.
- Outcome: Plan verification and push pending.

## 2026-08-11T15:31:00Z | software checkpoint

- Immutable plan SHA-256:
  `ed79db9ec36f9c7426db50926bc7dc5c55183267bae569425007376c591317c3`.
- Plan commit `5d647eb7` was pushed before further implementation.
- Added focused parser coverage for the exact canonical observation command,
  including `profile=None`, `pool_credentials=None`, board 205, duration 360,
  and redaction, plus rejection of assignment-style stage syntax.
- Focused Cargo and Bazel flash tests passed. The complete ordered Cargo gate,
  Bright Builds checks, all Bazel tests, package build, parity/progress,
  redaction, reference, selector, and diff checks passed.
- The first parity process after the all-Bazel suite encountered transient
  macOS `os error 35`. Host process and VM pressure checks were healthy; after
  a Bazel shutdown, the complete remaining sequence passed in a fresh server.
- Hardware status: untouched. Regression commit and push, clean exact package,
  detector, and conditional campaign remain pending.
