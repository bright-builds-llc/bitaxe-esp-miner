# Parity work log

## 2026-08-15T22:34:00Z | implementation checkpoint

- Source checkpoint: plan commit `68634ac4` plus the current implementation
  diff.
- Actions: Rebound only the ADC evidence workflow, its public contract, and
  behavior fixtures from the consumed attempt-001 task, plan, paths, digest,
  and ordinal to the immutable attempt-002 contract. Production firmware ADC,
  API, safety, mining, and control behavior remain unchanged.
- Verification: Ten focused Rust ADC evidence/input tests passed, including
  fresh zero, fractional, negative, above-`u16`, stale, regressed, and unsafe-
  state cases. The complete real-child automation test passed on the new
  bindings. A combined build command then named a nonexistent convenience
  target after those tests; this was an invocation error, not a product or test
  failure, and the canonical `contracts_verified` target remains to run.
- Simplification review: The consumed attempt is a closed one-shot contract, so
  explicit replacement of its small fixed binding set is clearer and safer
  than introducing a reusable ordinal parameter that could authorize unplanned
  retries.
- Outcome: The minimal attempt-002 software binding is implemented; mandatory
  verification, commit/push, package admission, and hardware remain pending.
- Next safe action: Run the canonical generated-contract target and the full
  ordered pre-hardware verification matrix, then commit and push before any
  detector or device access.

## 2026-08-15T22:40:00Z | software verification checkpoint

- Source checkpoint: plan commit `68634ac4` plus the completed implementation
  diff.
- Verification: The canonical generated TypeScript contract check, ADC capture
  and both independent validator builds, ordered Cargo format/Clippy/build/test
  sequence, Bright Builds, real ESP32-S3 package, all 45 Bazel tests, parity
  progress, redaction, pinned-reference, and diff checks passed. The verbose
  parity report twice exhausted the host output pipe while rendering; the same
  validation command passed with stdout suppressed, confirming no checklist
  validation error.
- Evidence: The built package remains local and ineligible for hardware until
  the implementation commit is clean, synchronized, and pushed. No device,
  credentials, protected evidence, or attempt ordinal has been consumed.
- Outcome: Software and package gates are green for the minimal immutable
  attempt-002 rebinding.
- Next safe action: Commit the implementation, fetch and resolve any upstream
  movement without amending the immutable plan, push, rebuild/admit the exact
  pushed package, then run only the plan's detector and one-shot capture.
