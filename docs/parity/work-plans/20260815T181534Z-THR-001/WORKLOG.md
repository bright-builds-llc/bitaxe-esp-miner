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
