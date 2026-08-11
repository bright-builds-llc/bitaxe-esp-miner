# Parity work log

## 2026-08-11T18:29:00Z | corrected selection checkpoint

- Source commit: `52db06a39a7301f77d5611eec8498d5681310a75`.
- Actions: Re-ran the deterministic selector after the truthful predecessor
  closure and selected `API-003` with hostname plus rotation, both real benign
  fields of the system-settings schema.
- Verification: Clean synchronized `main`, no open plan, pinned clean
  reference, absent corrected plan/wrapper/attempt/projection paths, and no
  predecessor hardware attempt consumed.
- Evidence: Planning and source-schema evidence only; hardware and credentials
  untouched.
- Outcome: Corrected bounded multi-field system PATCH contract prepared.
- Blocker or next safe action: Run the complete plan gate, then commit and push
  this immutable plan before implementation or hardware use.

## 2026-08-11T18:31:26Z | corrected plan gate checkpoint

- Source commit: `52db06a39a7301f77d5611eec8498d5681310a75`.
- Actions: Ran the complete ordered repository gate and reviewed the corrected
  plan/task-only change.
- Verification: Cargo format, strict Clippy, all-target build, all-feature
  tests, Bright Builds, all Bazel tests, parity/progress, redaction, reference,
  selector, task uniqueness, and diff checks passed. The selector resumes only
  this corrected `API-003` plan. Immutable plan SHA-256 is
  `4d2929003f930753f8c27056c3fc5989264c0a413d817947aa2ec38a3382f36b`.
- Evidence: Plan, task, and worklog only; hardware and credentials untouched.
- Outcome: Corrected plan gate complete.
- Blocker or next safe action: Commit and push the immutable plan before any
  implementation or hardware work.

## 2026-08-11T18:40:25Z | implementation checkpoint

- Source commit: `6eb3ca95`.
- Actions: Added the Rust-owned v1 evidence schema and validator, synchronized
  TypeScript command contract, aggregate-only hostname/rotation capture,
  CLI/Just/Bazel wiring, semantic redaction admission, typed failure mapping,
  and focused tests.
- Verification: Contract tests, TypeScript build, automation suite, invocation
  surface, success/restoration, no-clobber, all four terminal categories,
  recovery PATCH, exact-package fallback, primary-failure precedence, public
  privacy, private modes, and a real child-process boundary pass.
- Evidence: Implementation and synthetic/private test artifacts only; no
  detector, credential access, hardware effect, or public projection.
- Outcome: Corrected software workflow is implemented and focused tests pass.
- Blocker or next safe action: Run the complete mandatory gate, review and push
  the implementation commit, then build and admit its exact package before the
  sole hardware attempt.

## 2026-08-11T18:44:33Z | software gate checkpoint

- Source commit: `6eb3ca95` plus the reviewed implementation diff.
- Actions: Ran the complete ordered repository gate, redaction and reference
  checks, selector and immutable-plan admission, generated-contract comparison,
  task uniqueness, tracked-scratch, and diff checks.
- Verification: Cargo format, strict Clippy, all-target build, all-feature
  tests, Bright Builds, all Bazel tests, parity/progress, redaction, reference,
  focused contract/orchestration tests, and the real child-process boundary
  pass. The selector resumes only this corrected `API-003` plan and the plan
  hash remains unchanged.
- Evidence: Software and synthetic/private test evidence only; no detector,
  credential access, hardware effect, or public projection.
- Outcome: The corrected implementation is ready for its immutable commit and
  push.
- Blocker or next safe action: Commit and push the reviewed implementation,
  then build and admit the package whose source commit is exactly that pushed
  implementation before the single hardware attempt.

## 2026-08-11T18:56:09Z | exact-package hardware evidence

- Source commit: `3dea210228722634360daeda1327f2676e78db3a`.
- Actions: Re-ran the complete clean-head gate, built and admitted the exact
  schema-v3 package, ran the sole detector, and ran the sole conditional
  settings PATCH capture. The detector admitted one Ultra 205. The capture
  performed one exact-package flash, one atomic hostname/rotation mutation and
  combined readback, then one atomic restoration and combined readback.
- Verification: The Rust validator accepts the projection; exact package,
  workflow, board, safety, two-field atomicity, immediate readback, restoration,
  private-mode, cleanup, and no-holder assertions pass. The initial semantic
  redaction scan rejected two digest field names because `original` contains
  the denylisted substring `origin`; no raw value was present. Renaming those
  public contract fields to `baseline` and adding a production-shaped
  regression makes the focused automation, contract, validator, and redaction
  checks pass without another hardware attempt.
- Evidence: Public aggregate-only projection at
  `docs/parity/evidence/api003-settings-patch/settings-patch-projection.json`;
  all detector, credential, port, hostname, rotation, origin, network, serial,
  HTTP, and process material remains in ignored mode-protected roots.
- Outcome: The single attempt passed and closed evidence supports promoting
  only `API-003` to `verified`.
- Blocker or next safe action: Run the full final evidence gate, commit the
  evidence and redaction-contract correction as the source commit, then
  transition only `API-003`, synchronize progress, archive the completed task,
  and push after the final gate.
