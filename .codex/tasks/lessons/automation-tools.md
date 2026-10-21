# Lessons: Automation tools

Scope: host automation and evidence tools (`tools/`, `scripts/`), parity tooling, Gate and browser harness pages, sealed runs, and host test fixtures.

## lesson-opaque-handoff-before-fallible-validation | 2026-07-11 14:55

1. Date: 2026-07-11
2. What went wrong: A fresh exact-head attempt created a one-time opaque resume handle, then a fallible handoff assertion rejected the otherwise valid checkpoint before the handle reached the operator. The live private attempt could no longer be addressed through the normal handle-only cleanup path.
3. Preventive rule: Emit or durably escrow a one-time public locator before running fallible post-construction assertions. Provide a narrowly guarded, effect-free cleanup path for a uniquely identifiable pristine orphan without reconstructing or exposing the clear locator.
4. Trigger signal to catch it earlier: A command creates a private active record or capability and then performs validation, formatting, or output transformation before returning its only public locator.

## lesson-evaluator-identity-binds-transitive-validators | 2026-07-24

1. Date: 2026-07-24
2. What went wrong: The Phase 36 evidence evaluator identity omitted a materially reachable runtime-identity state reducer, so validator behavior could drift without rotating the evaluator or successor-contract identities.
3. Preventive rule: Bind every materially reachable repository-owned validator, including transitive reducers and models, through a versioned inventory of relative path and source bytes; declare every source in the build/runfiles graph and regression-test source, path, addition, removal, and replacement drift.
4. Trigger signal to catch it earlier: An evaluator inventory lists entrypoint validators but omits a reducer or model they call, accepts caller-authored digests, or lacks a test that membership drift rotates every derived identity.

## lesson-stop-on-first-success-cannot-prove-later-events | 2026-10-06 18:00

1. Date: 2026-10-06
2. What went wrong: The accepted-share probe stopped as soon as the share was acknowledged, which on the candidate came before the Gate's first 20-second renewal, so the Renew path went unexercised. The renewal probe's 45-second window, with about 2.5 expected qualifying shares, also left about a 6% chance of no share; renew-current-001 hit it and sealed unverified.
3. Preventive rule: When a run must prove several events, make its stop rule require every claimed event, not the first success. Size each bounded window from measured event rates (probability of no event) and keep it inside the lease or authority expiry those events extend.
4. Trigger signal to catch it earlier: A claim needs two events from a run that stops on the first, an event scheduled after the stop condition is listed as covered, or a window's expected event count is below about 4.

## lesson-sealed-run-bindings-must-accept-every-run-shape | 2026-10-06 18:00

1. Date: 2026-10-06
2. What went wrong: The lineage record-key check and the restart owner's generation lookup assumed one sealed-run shape (a heartbeat dispatch record). They refused a share run, which keeps a proof instead, and then a share run without a share, which keeps neither; each needed a separate fix while hardware work waited.
3. Preventive rule: When a new owner seals runs that later owners consume, enumerate every run shape it can produce, including failed and partial ones, and test each in the shared binding helpers (`deviceRecordAttemptId`, `startGeneration`) before the first hardware run.
4. Trigger signal to catch it earlier: `start_record_attempt_mismatch`, `preparation_parent_binding` or a refused previous Start right after a new owner or a new failure mode produced its first sealed run.

## lesson-serve-a-page-s-whole-module-graph | 2026-10-06 18:00

1. Date: 2026-10-06
2. What went wrong: The recovery client gained an import (`recovery-error-row.mjs`) that the shared current-recovery server never served. Every restart and preparation recovery page then failed to load before connecting, which surfaced only on hardware (restart005).
3. Preventive rule: A server that serves a browser module must own that module's whole import graph, derived in one place, with a test that walks the real imports and requires each to load. Never let individual callers list a page's modules.
4. Trigger signal to catch it earlier: A served page shows only the base Gate controls, the network log shows a non-200 for an `.mjs`, or a browser module gains an import while several servers serve it.

## lesson-join-random-ids-to-their-cli-flags | 2026-10-06 18:00

1. Date: 2026-10-06
2. What went wrong: The noise-serial owner passed a random base64url attempt id as a separate argument. One id began with `-`, the fixture's clap parser read it as a flag and exited with code 2 before listening, and the install attempt failed after five good installs.
3. Preventive rule: Pass generated or user-derived values to CLIs as `--flag=value`, never as a separate argument, and cover a leading `-` value in a regression on the real parser.
4. Trigger signal to catch it earlier: A child exits with code 2 and a short stderr before doing work, or an argv builder places a random token after its flag as its own element.

## lesson-update-validator-scope-lists-with-scope-changes | 2026-10-06 18:00

1. Date: 2026-10-06
2. What went wrong: A June parity guard hard-coded STR-005 as deferred scope, from when only Stratum v1 was planned. After STR-005 became an active Ultra 205 row, the guard blocked its own promotion at the final `just parity` check.
3. Preventive rule: When a row's scope or deferral changes, search the parity tool for hard-coded row lists and update them in the same change, replacing a blanket exclusion with the precise rule it protected and keeping its original test case.
4. Trigger signal to catch it earlier: A validation error cites deferred or non-205 scope for an active row, or a checklist row's status leaves `deferred` without a parity-tool diff.

## lesson-register-new-evidence-errors-in-typed-failure | 2026-10-06 21:30

1. Date: 2026-10-06
2. What went wrong: The new OTAWWW supervisor's error class was not added to `tools/automation/src/typed-failure.ts`, so a typed `hardware_blocked` stop was reported as `process_failed`, the wrong category for the attempt policy.
3. Preventive rule: When adding an automation command with its own typed error class, register that class in the typed-failure mapping in the same change and test that a typed failure surfaces with its own category through the CLI result.
4. Trigger signal to catch it earlier: A new `*EvidenceError` class exists without an `instanceof` entry in `typed-failure.ts`, or a supervisor failure prints `process_failed` while its stderr names a specific stage.

## lesson-macos-holds-fresh-executables-at-launch | 2026-10-09 17:14

1. Date: 2026-10-09
2. What went wrong: On this macOS host the first exec of a freshly written script or binary stalled 5–93 s at `_dyld_start` with 0% CPU while `syspolicyd` was busy; repeats were instant. Tests that exec freshly written fakes under 10 s bounds failed, and in BWG-007 a 309 s Bazel launch hold let the agent's tool timeout kill `owner-finish` and left a rerun's detector stale.
3. Preventive rule: For time-bounded steps such as detector freshness windows and seals, run already-built binaries directly instead of a path that first builds or writes a new executable. In tests, spawn an existing interpreter on a script (for example `node script.mjs`) rather than exec a freshly written executable, or size the bound for the first-exec hold.
4. Trigger signal: A test writes a fake executable and execs it under a bound of seconds, a freshness-windowed step runs through `just` or Bazel right after a rebuild, or a process sits at `_dyld_start` with 0% CPU.
