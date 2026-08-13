# Parity work log

## 2026-08-13T01:12:07Z | Selection and discriminating-cause checkpoint

- Source commit: `70329dabef817d63eb5590a24b12a3a7be80e113`
- Actions: Re-ran the clean synchronized selector, skipped API-009 at its
  sealed repeated boundary, and selected THR-001. Bound this fresh plan to the
  exact attempt-001 closure diagnosis rather than retrying the prior command.
- Verification: Protected boolean-only diagnosis from attempt-001 proved all
  live device, exact-package, fresh thermal-correlation, safe-state, cleanup,
  and privacy members passed. The sole failure is reproducible in clean source:
  the orchestration requires a textual intermediate statement absent from the
  simplified production reducer.
- Evidence: Planning and software diagnosis only. Attempt-001 remains sealed;
  no raw temperature, acquisition stamp, boot session, origin, hostname, port,
  network or USB identity, credential, log, command, PID, or trace entered this
  public record.
- Outcome: THR-001 is actionable through a checked-in-source regression and
  exact host admission fix, followed by at most one fresh attempt-002.
- Blocker or next safe action: Freeze, verify, commit, and push this immutable
  plan/task checkpoint before editing implementation files.

## 2026-08-13T01:24:00Z | Host admission fix checkpoint

- Source commit: `f33b700207ff16d3d3bec91b557cd704bc81ae4e`
- Actions: Replaced the stale intermediate-statement requirement with the
  actual production reducer boundary, made token-order admission insensitive
  to whitespace-only formatting, exposed the narrow source-semantics test
  seam, and added the seven Rust/reference source owners as real Bazel test
  runfiles. Advanced the closed Rust/TypeScript contract, protected paths,
  task/plan binding, and produced projection to attempt ordinal 2.
- Verification: The focused Rust contract and complete automation suites pass.
  Coverage proves the current checked-in reducer is admitted, the exact stale
  attempt-001 intermediate shape is rejected, ordinal 1 is rejected, ordinal 2
  is accepted, generated contracts are synchronized, and the existing
  real-child, typed-failure, protected-mode, redaction, correlation, and atomic
  withholding boundaries remain green.
- Evidence: Software-only tests and synthetic device inputs. No hardware,
  credentials, serial, network, private attempt-001 artifact, raw thermal
  value, acquisition stamp, boot session, origin, hostname, port, USB/network
  identifier, log, command, PID, or trace was accessed or published.
- Outcome: The discriminating host defect is fixed without changing firmware
  runtime behavior or weakening the evidence quorum.
- Blocker or next safe action: Run the full mandatory sequence, inspect the
  complete diff, commit and push the software fix, then build and admit the
  exact package before detector admission for attempt-002.

## 2026-08-13T01:42:00Z | Implementation verification and cache isolation

- Source commit: `f33b700207ff16d3d3bec91b557cd704bc81ae4e`
- Actions: Completed the explicit simplification pass and the mandatory
  verification sequence. Two initial all-feature Cargo runs stalled before
  Rust startup in the same cached `bitaxe-core` test executable. Process
  sampling showed `_dyld_start` with zero CPU; macOS `codesign`, another
  generated executable read, and package-scoped `cargo clean` then blocked on
  the same default target cache. Terminated only the owned processes and moved
  verification to a fresh temporary `CARGO_TARGET_DIR`; all 85 focused core
  tests and the full sequence then passed. Restarted Bazel under that same
  environment so its workspace-status Cargo process used the healthy cache.
- Verification: Cargo format, strict Clippy, all-target build, and all-feature
  tests passed from the fresh target. Bright Builds reported zero findings;
  all 41 Bazel tests passed; parity reported no validation errors; progress
  remained 64/94 (68.1%); redaction checked 17 public artifacts; reference
  cleanliness passed; and plan hash, unique task, generated-contract byte
  equality, absent attempt/projection/candidate, mode boundary, and diff checks
  passed.
- Evidence: Software verification only. The unhealthy generated default Cargo
  cache remains local and untracked; no repository source or hardware state
  was changed to work around it. No credential, serial, network, attempt-001,
  or raw private value entered output or Git.
- Outcome: The host fix is fully verified and ready for its required clean
  commit and push. The cache anomaly is isolated from repository correctness.
- Blocker or next safe action: Review, commit, and push the implementation;
  build and admit an exact package from that pushed commit; then execute the
  immutable detector/capture attempt-002 sequence once.

## 2026-08-13T01:52:40Z | Sole attempt-002 and wider-integer diagnosis

- Source commit: `0fa223e795ca7c2fcd4f4507f999bb3c61b71bae`
- Actions: Built and independently admitted the exact clean package, passed
  detector admission, and executed the immutable attempt-002 capture exactly
  once. The command ended `evidence_invalid`; no retry or second hardware
  process was started. Diagnosed the protected result through fixed safe error
  summaries and boolean-only invariant checks.
- Verification: Projection and candidate are absent. The detector, exact
  source/reference/package and app identity, stable safe boot, protected modes
  and file set, source semantics, plan/task binding, source system-info
  validator, and fresh finite below-throttle HTTP/WebSocket value, state,
  package, and boot correlation all pass. The safe failure summary identifies
  the sole rejected member: the HTTP acquisition-stamp boot-session integer is
  valid JSON but wider than JavaScript's safe-integer range.
- Evidence: Raw temperatures, stamp integers, boot sessions, origins,
  hostnames, ports, USB/network identities, credentials, logs, commands, PIDs,
  traces, and HTTP/WebSocket documents remain in the ignored mode-0700 root.
  No private value entered terminal output or Git.
- Outcome: `stop_impossible_contract`. THR-001 remains `implemented`; no
  evidence or checklist transition is claimed. Attempt-002 is consumed.
- Blocker or next safe action: Under a fresh immutable plan, implement lossless
  acquisition-stamp integer-token validation and exact cross-document
  correlation, verify it against wide/malformed/mismatched regressions, then
  authorize at most one fresh attempt-003. Never retry or reuse attempt-002.
