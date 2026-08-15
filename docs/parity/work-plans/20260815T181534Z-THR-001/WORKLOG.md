# Parity work log

## 2026-08-15T18:15:34Z | selection and software-only diagnosis contract

- Selection: After API-009 was verified and pushed, the clean repository
  selector ranked THR-001 first. Its attempt-004 closure requires a production-
  order software reproduction before any fresh hardware ordinal.
- Failure signal: The exact-package run emitted `fault_observed`, then aborted
  as `fault_projection_missing`; ordinary restoration passed and no public
  fault evidence was published.
- Action: Freeze a software-only diagnosis contract around the actual stimulus,
  reducer, producer stale-processing, and next-sweep boundary. No hypothesis is
  accepted until a fast deterministic loop reproduces that exact category.
- Safety: This plan authorizes no detector or device interaction and cannot
  promote THR-001. Attempt-005 requires a later separately committed contract.

## 2026-08-15T18:20:00Z | immutable-plan verification

- Plan SHA-256:
  `2351951778835e6b27f4b61a0706128650a05d4b1ac9ea087cb98f9d014eb98c`.
- Verification: Ordered Cargo format, strict Clippy, all-target build, and
  all-feature tests passed. Bright Builds, parity/progress, semantic redaction,
  pinned-reference cleanliness, and diff checks also passed.
- Outcome: The software-only reproduction and correction boundary is ready to
  commit and push before implementation. THR-001 remains `implemented`; no
  hardware effect or attempt-005 authority exists.

## 2026-08-15T18:35:00Z | exact production-order reproduction and correction

- Red loop: `cargo test -p bitaxe-safety
  thermal_fault_stimulus::tests::production_order_fault_sequence_survives_stale_processing
  -- --exact` failed deterministically in 0.00 seconds with
  `FaultProjectionMissing`. The loop performs a successful real temperature
  reduction, each stimulus step, `reduce_sensor_sweep`, the production stale
  pass, and the next sweep.
- Hypotheses: Ordinary fault-to-stale aging ranked first; reducer loss,
  stimulus off-by-one, and sequence/timestamp rejection were independently
  falsifiable alternatives. The existing sustained-failure regression proved
  the one-second fault-to-stale transition is intentional, the standalone
  exact-five test disproved the off-by-one, and every reducer call installed
  successfully without sequence failure.
- Root cause: The stimulus correctly proved the first reducer-published
  `thermal_reading_invalid` state, then redundantly required that state on
  every later one-second overlay. The normal producer must age the retained
  last-good sample to stale, so the third overlay encountered stale truth and
  aborted even though real reads and reduction continued.
- Correction: Latch the first proven fault at the existing `fault_observed`
  boundary, require a successful real read for every remaining overlay, count
  exactly five invalid outcomes, and retain the existing later fresh-recovery
  proof. Ordinary producer stale semantics are unchanged.
- Focused proof: The original production-order loop is green; all stimulus
  tests pass; missing first fault projection and real-read failure remain
  fail-closed; and the ordinary sustained-failure stale transition still
  passes.

## 2026-08-15T18:50:00Z | implementation verification

- Simplification: The correction removes the redundant final-fault phase and
  reuses the existing `fault_observed` transition as the single authoritative
  latch. It adds no new state, timeout, public surface, evidence exception, or
  test-only production branch.
- Verification: The focused red/green loop, complete stimulus tests, and
  ordinary sustained-failure stale regression pass. Ordered Cargo format,
  strict Clippy, all-target build, and all-feature tests pass. The real
  ESP32-S3 firmware build, Bright Builds, all 45 Bazel tests, parity/progress,
  semantic redaction, pinned-reference cleanliness, no-debug-instrumentation,
  and diff checks pass.
- Outcome: The root-cause correction is ready to commit and push with THR-001
  still `implemented`. This software-only plan emits no hardware evidence and
  does not authorize attempt-005.
