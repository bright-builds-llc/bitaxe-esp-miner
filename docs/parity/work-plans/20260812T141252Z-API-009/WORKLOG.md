# API-009 worklog

## Attempt 1 | 2026-08-12T14:12:52Z

- Starting status: `implemented`
- Actions: Repaired the preceding closure's schema heading, completed the
  clean synchronized preflight, selected API-009 first with no skips, and
  traced the missing producer from current-generation nonce correlation to
  the sole firmware command-visible snapshot owner.
- Verification: The closure parser regression, mandatory repository gate,
  isolated parity rerun, reference guard, and selector pass. Source and pinned
  reference commits are recorded in the immutable plan.
- Outcome: The actionable seam is the existing production correlation receipt:
  it already owns nonce difficulty and the admitted compact target but emits
  only submit and scoreboard effects. No hardware action was performed.
- Next action: Commit and push the immutable plan/task checkpoint, then add the
  pure network-target decision, redacted effect, firmware state transition,
  and focused regressions.

## Attempt 2 | 2026-08-12T14:31:00Z

- Starting status: `implemented`
- Actions: Reused one compact-target difficulty calculation in the pure
  Stratum core, carried its closed-boundary qualification through the
  current-generation correlation receipt, emitted the redaction-safe
  `RecordBlockFound` production effect before submit and scoreboard effects,
  and executed it through the sole retained runtime-snapshot owner. Added API
  state, Stratum correlation/effect, and firmware source-ownership regressions.
- Verification: Focused Cargo and Bazel tests passed. The mandatory ordered
  Cargo, Bright Builds, Bazel, parity, parity-progress, redaction, and reference
  gates passed; `just parity` required one isolated rerun after the first run
  completed its report but hit transient macOS `os error 35`. The initial
  `just test` exposed one missing explicit Bazel source entry for the new nested
  test; adding that entry made the target and complete gate pass.
- Outcome: Production now raises the command-visible notification for every
  valid current-generation nonce at or above the admitted network target,
  including upstream-compatible duplicate counting. Below-target, stale, and
  malformed-target results fail closed. No hardware action was performed.
- Next action: Commit and push the verified source, then update API-009's
  ownership target, synchronize parity progress, and close this bounded plan
  below verified with the remaining genuine physical-effect requirements.
