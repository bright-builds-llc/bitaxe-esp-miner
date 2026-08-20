# Parity work log

## 2026-08-20T22:55:00Z | truthful v2 evaluation

- Evaluator source: `cbc5fa7f`.
- Added strict v1/v2 Rust validation, retained capture-package identity
  commitment, capture/evaluation dual-source and plan binding, protected input
  commitment, and closed failure-stage diagnostics without a manifest-byte
  claim.
- Focused/full pre-effect gates passed and the evaluator was committed/pushed
  cleanly.
- The sole protected v2 command exited zero and published projection SHA-256
  `e8054e9176154f154a82b4c9f5301f9d87f64ca558e2ad117be7c37fc4efe920`.
- Independent Rust validation passed. The redaction scanner initially skipped
  the new schema (`checked=0`); v2 was added to its explicit allowlist with a
  regression, after which the projection passed with `checked=1`.
- No hardware or external effect ran. Attempt-005 remained immutable; only the
  caller-owned v2 wrapper streams and public projection were created.
- Outcome: complete evidence quorum supports `STAT-003` promotion.
