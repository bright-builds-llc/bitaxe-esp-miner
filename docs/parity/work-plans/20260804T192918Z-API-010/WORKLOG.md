# Parity work log

## 2026-08-04T19:29:18Z | selection and root-cause checkpoint

- Source commit: `6d42e35271d973ba1425521c152b799d24575519`.
- Actions: Resumed the open `API-010` lineage, preserved its immutable plan,
  inspected only committed flash, USB-session, firmware boot-evidence,
  classifier, and orchestration source, and derived the production multi-epoch
  transcript shape without reading the private attempt trace.
- Verification: The exact-package flash-monitor path performs a factory write
  and a credential-seed write before opening its final receive-only reader.
  Firmware emits a fresh boot identity for each reset and replays current
  identity/origin evidence, while queued intermediate bytes can precede the
  final epoch in the reader's transcript. The existing strict whole-trace
  classifier therefore returns `baseline_multiple_sessions` by design.
- Evidence: Source-derived root cause only; no private trace, credential,
  device, network, or hardware evidence was accessed or reproduced.
- Outcome: A closed terminal-epoch rule is specified in the immutable follow-up
  plan. The generic whole-trace classifier remains strict and unchanged.
- Blocker or next safe action: Run planning verification, commit and push this
  plan, then implement the synthetic classifier/orchestration regression with
  no hardware interaction.

## 2026-08-04 | terminal baseline implementation checkpoint

- Source commit: implementation based on planning commit `3d6fe7be`.
- Actions: Added a distinct terminal-baseline classifier that validates every
  identity and origin marker, requires an unambiguous sequential boot-ordinal
  chain, selects the final session at its first identity marker, and applies
  the existing strict baseline plus passive-safe-state rules only to that
  terminal slice. Routed settings, theme, and operator-snapshot exact-package
  captures through the new mode and its admitted origin.
- Verification: Seventeen focused Phase 33 tests pass for one ready epoch, an
  ordered stale prefix, session reappearance, ordinal gaps, malformed identity
  and origin markers, unknown origins, stale-only safety, mixed terminal
  origins, and incomplete terminal epochs. Bazel parity and automation tests
  pass. The API-010 real-child regression now delegates to the actual parity
  executable against a production-shaped two-epoch trace rather than emitting
  a manufactured passing classifier result.
- Evidence: Synthetic software evidence only. No private trace, hardware,
  credential, origin, hostname, port, USB identity, or network identifier was
  read or published.
- Outcome: The root-cause fix and focused regression are complete; the original
  whole-trace baseline interface remains strict and unchanged.
- Blocker or next safe action: Run the complete mandatory verification and
  privacy/diff review, then commit and push the software fix. Do not run a
  hardware retry under this task.

## 2026-08-04 | mandatory gate and conservative closure

- Source commit: implementation based on planning commit `3d6fe7be`.
- Actions: Ran the complete ordered Rust, managed-standards, Bazel, parity,
  privacy, reference, immutable-plan, sensitive-output, and diff gates; reviewed
  the final functional core, orchestration callers, test subprocess, and build
  wiring; and archived the completed software task.
- Verification: Format, strict Clippy, all-target/all-feature Cargo build and
  tests, Bright Builds with zero findings, all 34 Bazel tests including the
  ESP32-S3 firmware build, parity with no validation errors, progress, semantic
  redaction, pinned-reference cleanliness, immutable-plan comparison, and diff
  checks pass. The production-shaped classifier regression executes the actual
  parity binary across a real child-process and file boundary.
- Evidence: Synthetic and repository verification only. Public task/worklog
  output contains no operational origin, hostname, port, USB identity, network
  identifier, credential, or raw trace.
- Outcome: The software blocker is fixed and the active follow-up task is
  complete. `API-010` remains `implemented`; no evidence or parity status was
  promoted.
- Blocker or next safe action: Commit and push the exact software fix. A live
  retry is outside this task and requires a new active hardware contract.

## 2026-08-04T19:53:46Z | open-plan lineage diagnosis

- Source commit: `67974ccc1cd2d16455f01e9ca71da4c608c00f06`.
- Actions: Reproduced the deterministic selector failure after the completed
  terminal-baseline fix and inspected its open-plan scan against the two
  immutable `API-010` plans.
- Verification: Both plans remain open because `API-010` correctly remains at
  their `implemented` initial status and neither software-only plan may create
  a hardware `RESULT.md`. The newer plan directly names the older plan as the
  lineage it resumed, but the selector currently treats every result-less plan
  as unrelated and fails whenever more than one survives status filtering.
- Evidence: Repository metadata and a real `next-item --format json` process
  only. No private trace, hardware, credential, origin, hostname, port, USB
  identity, or network identifier was accessed or published.
- Outcome: The root cause is an absent immutable-plan lineage reconciliation
  rule. The closed design accepts only a same-row chronological chain where
  each newer plan directly references the immediately preceding plan path;
  unlinked duplicates and cross-row plans remain ambiguous and fail closed.
- Blocker or next safe action: Commit and push this task checkpoint, then add
  the pure reconciliation helper and focused filesystem regressions without
  modifying either `PLAN.md`.

## 2026-08-04 | open-plan lineage implementation checkpoint

- Source commit: implementation based on task checkpoint `61aeb829`.
- Actions: Added a pure open-plan reconciliation step after existing status
  admission. It sorts surviving plans by their repository path, requires one
  row ID, verifies that every newer plan directly names the immediately older
  backticked `PLAN.md` path, and returns only the newest linked continuation.
- Verification: Ten focused Cargo selector tests and the Bazel parity test
  target pass. New filesystem regressions cover a linked same-row continuation,
  unlinked same-row duplicates, and cross-row ambiguity. The real
  `next-item --format json` process now resumes the newer immutable `API-010`
  plan with an empty candidate list.
- Evidence: Synthetic repository files and public plan metadata only. No
  parity evidence, private trace, hardware, credential, origin, hostname,
  port, USB identity, or network identifier was accessed or published.
- Outcome: Explicit continuation lineage is admitted without weakening the
  existing status-regression or unrelated-plan guards. Neither immutable plan
  changed, and `API-010` remains `implemented`.
- Blocker or next safe action: Run the complete mandatory verification,
  privacy, reference, immutable-plan, sensitive-output, and diff checks before
  conservative software-only closure.

## 2026-08-04 | open-plan lineage verification and closure

- Source commit: implementation based on task checkpoint `61aeb829`.
- Actions: Ran the complete ordered Rust, managed-standards, Bazel, parity,
  privacy, reference, immutable-plan, sensitive-output, and diff gates; reviewed
  the final reconciliation core and filesystem regressions; and archived only
  the completed selector-lineage task.
- Verification: Format, strict Clippy, all-target/all-feature Cargo build and
  tests, Bright Builds with zero findings, all 34 Bazel tests including the
  ESP32-S3 firmware build, parity with no validation errors, progress at 39 of
  94 active rows verified, semantic redaction, pinned-reference cleanliness,
  immutable-plan comparison, sensitive-output review, and diff checks pass.
  The real selector resumes
  `docs/parity/work-plans/20260804T192918Z-API-010/PLAN.md`.
- Evidence: Synthetic repository verification only. No `RESULT.md`, parity
  evidence, private trace, hardware, credential, origin, hostname, port, USB
  identity, or network identifier was created, accessed, or published.
- Outcome: The selector blocker is closed while both immutable plans and the
  `API-010` `implemented` status remain unchanged. Unlinked and cross-row
  duplicates retain closed terminal errors.
- Blocker or next safe action: Commit and push the exact implementation. A live
  retry remains outside this task and requires a new active hardware contract.
