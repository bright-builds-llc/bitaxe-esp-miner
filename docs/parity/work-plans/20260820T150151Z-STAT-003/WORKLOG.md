# Parity work log

## 2026-08-20T16:05:48Z | protected readiness implementation

- Source commit: `2eb620c530f612f7097e1b53d35c1e18b39ced07`.
- Actions: added the repo-owned pool-readiness Cargo/Bazel/Just surface with
  exclusive private-root creation, three exact bounded Stratum V1 sessions,
  closed classifications, source/reference binding, atomic private evidence,
  and no-submit/secret-safe output; rotated only the scoreboard plan, task,
  paths, and attempt ordinal from 4 to 5.
- Verification: seven focused library tests and one real CLI/subprocess test
  passed, including configure/subscribe/authorize ordering, authorize rejection,
  malformed and oversized input, timeout, modes, exact contract bounds, and
  secret absence. Scoreboard Rust and TypeScript tests, generated-contract
  verification, all workspace Rust gates, Bright Builds, all 48 Bazel tests,
  redaction, reference, parity/progress, and package passed.
- Evidence: immutable plan SHA-256
  `43d13ec599e9f46988f0ebb44607dc000eff95db78c37fdc340fe52e14365684`;
  implementation commit `2eb620c530f612f7097e1b53d35c1e18b39ced07`.
- Outcome: the software and attempt-005 rotation are ready for a clean pushed
  source-bound readiness effect. No credential, external network, detector,
  device, flash, mining, or share effect has run yet.
- Blocker or next safe action: commit and push this source-bound record, rerun
  the pre-effect clean-source checks, then execute the sole readiness command.
  Continue to package/detector/attempt-005 only if its private result is exactly
  `ready`.

## 2026-08-20T16:24:00Z | readiness and attempt-005

- Source commit: `a31af2873e6b2d41fe47aa18a57626f33aaf099b`.
- Actions: ran the sole source-bound three-session pool-readiness command; after
  its exact private `ready` result, built the clean package, ran the sole
  detector, and ran the sole 600-active-second scoreboard campaign with mining
  and share submission. No raw protected value was read or published.
- Verification: readiness passed 3/3 configure/subscribe/authorize sessions
  without submit. The detector, package, result seal, mode checks, redaction,
  runtime identity, 20/20 windows, safety, watchdog, safe stop, cleanup, live
  SPA, software restart, and stable immediate reads passed. The campaign
  recorded 19 qualified accepted shares and zero rejects.
- Evidence: protected attempt-005 result and seal match. Public projection is
  absent. Closed comparison found 20 entries before and after restart, stable
  repeats in each epoch, exact equality for job ID, extranonce2, ntime, nonce,
  and version bits, and difficulty-only mismatch for all 20 entries.
- Outcome: `hardware_blocked` at `scoreboard restart persistence is invalid`.
  Pinned source proves the verifier compared full in-memory difficulty against
  the expected one-decimal NVS reload representation.
- Blocker or next safe action: close without promotion or rerun. A fresh
  software-only plan must validate the pinned durable difficulty projection and
  explicitly govern any later protected attempt-005 re-evaluation.
