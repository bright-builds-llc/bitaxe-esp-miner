# Parity work log

## 2026-08-16T03:45:56Z | immutable plan checkpoint

- Source commit: `34768147feea166354dc97044ea1d5e12dce939a`
- Actions: Selected STAT-001 after higher-ranked rows required unavailable
  physical observation or unauthorized accessory wiring, then froze the exact
  attempt-004 software, detector, hardware, privacy, recovery, retry, and
  promotion contract.
- Verification: Plan-only Cargo format, lint, build, and tests; Bright Builds;
  all 45 Bazel tests; parity/progress; redaction; reference integrity; diff;
  and immutable-plan hash checks passed.
- Evidence: Plan SHA-256
  `703f4b8ed726f6ec8fffe7d4a152982d1674ca60cef2b8f7e8c0acd1193602b5`
  is committed and pushed in `bcc710f29d04a0432715cb35931f62afcd090c48`.
- Outcome: The attempt-004 contract is immutable and implementation work may
  proceed without changing `PLAN.md`.
- Blocker or next safe action: Rebind the closed evidence workflow to
  attempt-004 and campaign-result v10, add the sealed non-ready diagnostic
  projection, then run the full pre-device gate set.

## 2026-08-16T04:18:30Z | implementation checkpoint

- Source commit: `bcc710f29d04a0432715cb35931f62afcd090c48`
- Actions: Rebound the capture shell, Rust validator, generated TypeScript
  contracts, task/plan admission, Bazel runfiles, fixtures, and protected paths
  to attempt-004 and sealed campaign-result v10. Added a value-free public
  parse discriminator for sealed failed campaigns while retaining all raw
  child output and arbitrary result fields privately.
- Verification: Focused Rust contract and real-process automation tests passed.
  The real child proves a sensitive decoy cannot cross the public boundary and
  a tampered result seal withholds the diagnostic. Ordered Cargo format, clippy,
  all-target build, all-feature tests and doctests; Bright Builds; the canonical
  firmware package; all 45 Bazel tests; generated-contract verification;
  redaction; reference integrity; parity; progress; and diff checks passed.
- Evidence: Campaign acceptance now requires result schema v10, a `none` parse
  discriminator, and seven zero parse-failure counts. Nonzero campaigns expose
  at most one allowlisted parse category after private mode, schema, failed
  status, and SHA-256 seal validation.
- Outcome: The exact attempt-004 implementation is ready to commit and push.
- Blocker or next safe action: Push this implementation, rebuild and validate
  its exact clean package, then run only detector command 1 from `PLAN.md`.
