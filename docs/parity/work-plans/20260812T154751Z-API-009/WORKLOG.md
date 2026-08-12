# API-009 worklog

## 2026-08-12T15:47:51Z | Plan checkpoint

- Source commit: `4fb85ff5df13a40424117a75bc3b01db78e25b0f`.
- Actions: Selected API-009 first, reproduced the attempt-001 failure through
  the actual deployed launcher with an inert campaign child, and isolated the
  unset `JS_BINARY__NODE_BINARY` wrapper dependency as the exact cause.
- Verification: Clean synchronized selector, direct fixture, pinned-Node
  fixture, unpatched production process adapter, and deployed-launcher inert
  diagnostics were compared without USB effects. Plan-only repository gates
  remain pending.
- Evidence: No parity or hardware evidence claimed. Diagnostics contain only
  closed categories and mode-protected ignored files.
- Outcome: A targeted repo-owned executable plus child/readiness race can fix
  the real boundary without weakening the sanitized environment.
- Blocker or next safe action: Complete the plan-only gates, commit and push
  this immutable plan, then implement without editing `PLAN.md`.

## 2026-08-12T15:51:35Z | Plan gate complete

- Source commit: `4fb85ff5df13a40424117a75bc3b01db78e25b0f`.
- Actions: Bound the exact launcher diagnosis, repo-owned executable fix,
  child/readiness race, protected diagnostics, real-process regression, and
  single attempt-002 contract to the unique active task.
- Verification: Ordered Cargo, Bright Builds, all 39 Bazel tests, parity,
  progress, redaction, pinned reference, generated contracts, selector, unique
  task, immutable digest, reference cleanliness, and diff checks pass. PLAN.md
  SHA-256 is `b9d055764d046233159226a12e9e44444f52a66d44ce2c83375ce692fd04e52b`.
- Evidence: No hardware or parity evidence claimed.
- Outcome: The plan-only checkpoint is ready to commit and push.
- Blocker or next safe action: Push this immutable plan, then implement and
  prove the exact deployed-launcher boundary before any hardware effect.
