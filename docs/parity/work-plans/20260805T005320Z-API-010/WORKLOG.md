# Parity work log

## 2026-08-05T00:53:20Z | bootloader diagnostic contract

- Source commit: `db0bef8fbdbf7cbffe64a4c015de1c7ea7b080fc`.
- Actions: Resumed only `API-010`, reduced attempt-008 through protected
  classification, and inspected the pinned espflash 4.5.0 reset/connect source.
- Verification: The exact current signature is generic espflash
  `connection_failed`, bootloader synchronization failed, enumeration remained
  unchanged, the same physical device stayed accessible and holder-free, and
  cleanup completed. Espflash source proves the underlying reset-strategy error
  is available only at debug level before the CLI collapses it.
- Evidence: Closed categories, booleans, safe source provenance, and the pushed
  attempt-008 record only. Protected child output and all device, port,
  USB/process identity, command, and path values remain private.
- Outcome: Private child-local debug capture plus a closed classifier can
  distinguish the remaining hypotheses without changing detector effects.
- Blocker or next safe action: Run the complete plan gate, commit and push this
  immutable plan/task checkpoint, then make the focused classifier loop red.

## 2026-08-05T01:05:00Z | immutable plan gate

- Actions: Ran the complete pre-plan software gate and confirmed the
  deterministic selector points only to this plan.
- Verification: Cargo format, clippy, build, and all-feature tests passed;
  Bright Builds passed; all 35 Bazel tests passed; parity, progress, semantic
  redaction, pinned-reference cleanliness, immutable-output, selector, and
  diff checks passed. The first parity report encountered the known macOS
  `Resource temporarily unavailable (os error 35)` transient; its single
  allowed read-only rerun passed.
- Outcome: The immutable task/plan checkpoint is eligible to commit and push
  before source changes.
- Blocker or next safe action: Commit and push this checkpoint, then make the
  focused bootloader diagnostic classifier test red.
