# Parity work log

## 2026-08-13T10:04:28Z | selection and diagnosis checkpoint

- Source commit: `66f9cd78169696b17dcce5538e31eb5fd5c818a0`.
- Actions: Selected API-009 first from the clean synchronized selector and
  traced attempt-009's pause deadline through the production session,
  actuation adapter, sensor publication/wakeup owner, campaign marker, host
  pause join, and pinned reference pause path.
- Verification: The Rust resumable pause currently invokes the terminal
  eight-step safe shutdown synchronously. Its 120-second 45 C cooling wait
  blocks the production owner inside a 130-second host join. The reference
  ordinary-pause path has no such cooling loop; that loop is overheat-only.
- Evidence: Current production and pinned-reference source plus the prior
  redacted closure. No protected attempt contents or hardware interfaces were
  accessed.
- Outcome: API-009 remains `implemented`; software root cause is identified
  and no hardware retry is eligible.
- Blocker or next safe action: Commit and push this immutable plan/task
  checkpoint, then add the typed stop-purpose split and real-boundary
  regressions without editing `PLAN.md`.

## 2026-08-13T10:26:00Z | immutable plan verification

- Actions: Bound the diagnosis to the active API-009 task and the immutable
  software-only plan. Reviewed its authorization, retry, evidence, redaction,
  and no-promotion boundaries.
- Verification: `cargo fmt --all`, clippy with warnings denied, all-target Cargo
  build, all-feature Cargo tests, Bright Builds checks, `just test`, parity
  validation/progress, redaction verification, reference verification, diff
  checks, task uniqueness, and sensitive-output review passed.
- Evidence: Plan SHA-256
  `4e54992c4466ddf01ef079066d97f4183da7e59d27e502908afbeca044419096`.
- Outcome: The software implementation may begin after this checkpoint is
  committed and pushed. No hardware effect or retry is authorized here.

## 2026-08-13T11:08:00Z | implementation and closure checkpoint

- Actions: Added the closed resumable-versus-terminal hardware-stop purpose,
  split the pure prompt-pause and terminal actuation plans, bound the firmware
  adapter to the typed selector, and added a production-session-to-actuation-
  to-same-lease-confirmation regression plus focused ownership tests.
- Verification: The prompt plan is exactly six immediate fail-closed steps,
  retains 100-percent fan duty, and excludes the 120-second cooling wait and
  30-percent fan settlement. Terminal and preparation-rollback behavior retain
  the original eight-step plan. Earliest failures remain primary and all steps
  in the selected plan are attempted.
- Gates: Focused Stratum, actuation, campaign-status, sensor-ownership, and
  flash pause-join tests passed. Cargo format, clippy with warnings denied,
  all-target build, all-feature tests, Bright Builds checks, `just test`,
  parity validation/progress, redaction, reference verification, and canonical
  `just build` firmware compilation passed. The immutable plan's `just
  firmware` spelling does not exist; the repo-defined `just build` equivalent
  passed. One initial full Bazel run timed out in an unrelated real-process
  automation case during machine-wide I/O delay; the isolated suite passed in
  180.9 seconds with a 900-second diagnostic bound and unchanged `just test`
  then passed all 42 targets.
- Outcome: The attempt-009 software blocker is fixed. API-009 remains
  `implemented`; no checklist field changed, no hardware was contacted, and no
  public parity evidence was emitted.
- Next safe action: Close this software-only plan. A later selector run may
  create a fresh immutable attempt-010 hardware contract using this fix.
