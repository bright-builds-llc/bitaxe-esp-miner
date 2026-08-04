# Parity work log

## 2026-08-04T12:24:08Z | selection and plan

- Source commit: `dd6437e7d5d639507443eaf0d291265578669d08`
- Actions: Loaded the parity workflow, bounded active lessons, repository
  guidance, relevant standards, active tracker, checklist, and deterministic
  candidate selector; reconciled the resolved predecessor hostname task; chose
  the first actionable row and defined its task-gated hardware contract.
- Verification: The worktree began clean and `HEAD...origin/main` was `0 0`;
  `next-item --format json` reported no open plan and ranked the selected row
  after the documented ineligible candidates.
- Evidence: Current checklist correction `snapshot_substance_insufficient` and
  Phase 36 `snapshot_join: null`; verified typed device-session reboot evidence
  from `V12-HOSTNAME-205` supplies the bounded restart primitive only.
- Outcome: Plan ready for pre-implementation verification and commit.
- Blocker or next safe action: Run the repository pre-commit sequence, commit
  the plan/task and tracker reconciliation, then implement without changing
  this immutable plan.

## 2026-08-04 | typed capture implementation

- Source commit: implementation based on planning commit `3209134e`.
- Actions: Added the detector-gated `capture-operator-snapshot-evidence`
  command, Rust-owned evidence contract and validator, private same-origin
  HTTP/WebSocket/retained-log epoch joins, a callable existing snapshot
  coherence validator, the authoritative `device-session reboot-live`
  transaction, and bounded exact-package recovery with primary-failure
  precedence.
- Privacy and safety: Candidate evidence is validated beneath the mode-`0700`
  private root before publication; private inputs are mode `0600`; the public
  projection contains only digests, revisions, booleans, counts, categories,
  source identities, and schema names. No settings mutation, mining, hardware
  control, discovery, direct UART, or pin path was added.
- Verification: Focused Bazel builds for the automation, parity, and Rust
  validator binaries passed. Focused contract, automation, and parity tests
  passed, including every non-ready restart category, malformed and missing
  projections, epoch session/revision/substance mismatch, recovery precedence,
  prepublication validation, sensitive-value denial, and a real child-process
  transaction.
- Outcome: Software implementation is ready for the mandatory full gate and
  an explicit simplification/diff review.
- Blocker or next safe action: Run the exact pre-commit sequence and repository
  verification; commit and push only if every gate passes.

## 2026-08-04 | implementation gate

- Actions: Ran the mandatory repository sequence in order after the final
  simplification and prepublication-validation changes.
- Verification: `cargo fmt --all`, `cargo clippy --all-targets --all-features
  -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test
  --all-features`, `bun scripts/bright-builds-check.ts all`, `just test`,
  `just parity`, `just parity-progress`, `just verify-redaction`, `just
  verify-reference`, and `git diff --check` all passed. Bright Builds reported
  zero findings; Bazel reported 28 passing test targets; parity reported no
  validation errors and 35 of 94 active rows verified before this row's
  hardware capture.
- Outcome: Implementation is software-qualified for commit and push.
- Blocker or next safe action: Commit and push the exact implementation, prove
  branch cleanliness and remote synchronization, package that exact source,
  then run the single task-authorized detector-gated hardware attempt.

## 2026-08-04 | detector-gated hardware capture

- Source commit: `409864d0eb053c335225a99999bac61d7a6b3d1b` was clean,
  pushed, synchronized with `origin/main`, and matched the exact package.
- Actions: Ran `just package`, one private `just detect-ultra205` admission,
  and the single task-authorized `attempt-001` capture command. The workflow
  flashed the exact package, captured both substantive boot epochs, and issued
  exactly one typed normal software restart.
- Evidence: The closed v1 projection records two HTTP/WebSocket/retained-log
  joins, monotonic boot-local revisions, matching substantive projections,
  distinct boot-session digests, same physical device, exact build recovery,
  ordinal `N+1`, one restart request, cleanup, disabled mining and hardware
  control, and passed redaction. Projection SHA-256 is
  `f2ea9b8ef5e566d32a42605bbd819bb19575406dd3ae2a63853b7149a5597bd5`.
- Verification: The Rust evidence validator, mode checks, package-source
  identity check, semantic redaction verifier, and diff check passed after the
  capture. The private root is mode `0700` and all private files are mode
  `0600`.
- Outcome: `complete`; the one attempt passed and no recovery flash or retry
  ran. `V12-OPERATOR-SNAPSHOT-205` is eligible for its sole planned transition
  from `implemented` to `verified`.
- Blocker or next safe action: Create `RESULT.md`, apply the typed one-row
  transition, synchronize progress, run final gates, archive the completed
  active task, commit, and push.

## 2026-08-04 | promotion and closure

- Actions: Created `RESULT.md`, applied typed transition
  `20260804T124921Z-V12-OPERATOR-SNAPSHOT-205`, and synchronized parity
  progress against exact implementation/package source `409864d0`.
- Verification: The transition changed only
  `V12-OPERATOR-SNAPSHOT-205` from `implemented` to `verified`; synchronized
  progress is 36 of 94 active rows, or 38.3%.
- Outcome: The planned row is complete. The active tracker record is ready for
  append-only archival in the same final commit.
- Residual risks: Runtime health and every broader configuration, network,
  mining, ASIC, safety-control, update, recovery, other-board, and release
  claim remain outside this result.
- Blocker or next safe action: Archive this completed task, run the final
  repository gates, commit, push, and return to deterministic parity selection.

## 2026-08-04 | final ledger-format correction

- Failure signal: The first post-promotion `just parity` run rejected the new
  row because its Rust-owned target cell lacked required Markdown code spans.
- Root cause and fix: The transition command received plain target text. The
  active row and its uncommitted transition receipt were corrected to the
  canonical code-span form, the receipt result digest was recomputed, and a
  new append-only progress record synchronized the corrected checklist hash.
- Verification: `just parity` then passed with no validation errors;
  `just parity-progress` still reported 36 of 94 active rows verified; semantic
  redaction, reference cleanliness, and diff checks passed. No hardware action
  or retry occurred.
- Outcome: Checklist, transition ledger, progress ledger, result, and evidence
  are internally consistent and ready for the final commit gate.
