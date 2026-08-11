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
