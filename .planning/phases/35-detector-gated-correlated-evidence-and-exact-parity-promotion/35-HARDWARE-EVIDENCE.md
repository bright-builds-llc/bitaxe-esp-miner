---
phase: 35
lifecycle_id: 35-2026-07-17T17-00-37
board_category: "205"
evidence_state: hardware_attempt_non_promotion
redaction_mode: typed_redacted_projection
---

# Phase 35 Hardware Evidence

## Software Gate Record

The nine ordered software gates passed before any detector or hardware action. The
canonical preflight sealed an exact current-package capability and exited with
effects disabled.

| Field | Recorded value |
| --- | --- |
| Gate start | `2026-07-17T22:45:11Z` |
| Preflight completion | `2026-07-17T22:55:12Z` |
| Lifecycle ID | `35-2026-07-17T17-00-37` |
| Board category | `205` |
| Manifest schema | `manifest-v3` |
| Source commit | `37a83c4c47dd60bf37312ee6e4aa4590a9e77d28` |
| Reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| Manifest digest | `60bb7569b5e54b88660e7815c6d453088af1e76ae5ce71f0aeb16da2e651f048` |
| Executable-image digest | `d2bb6161f34caebe2fc98b7c22c86907e80dcb649f3b3b7302ce720e74ce659a` |
| Factory-image digest | `f6f33279814b1c1db9055bf97444099fa1796a97e005d35eeb5632c72a238f4b` |
| Package digest | `2113efa31cd7b7045ef238e1a1193586ef51409e7f1b73d9405a547257141922` |
| Runtime-identity digest | `fc98aed702c010bd4c463f29ec9b9cabe09df4e48271d2b8ef38f0fd835164f0` |
| Exact-package capability digest | `55b8eb75b3f35724096c233580ceea1dddc70ea29c47b88a733ef065ec8849ef` |
| Current source equality | `true` |
| Reference cleanliness | `true` |
| Effects permitted | `false` |
| Redaction mode | `typed_redacted_projection` |

## Ordered Gate Results

| Gate | Command category | Result |
| --- | --- | --- |
| 1 | `cargo fmt --all` | passed |
| 2 | `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| 3 | `cargo build --all-targets --all-features` | passed |
| 4 | `cargo test --all-features` | passed |
| 5 | Scoped parity and Phase 35 contract tests | passed |
| 6 | `just verify-reference` | passed |
| 7 | `just parity` | passed with zero validation errors |
| 8 | Phase 35 lifecycle verification with required plans | passed |
| 9 | `just phase35-evidence preflight-only=true` | passed |

## Pre-Hardware Boundary

- Detector invocations: `0`
- Credential accesses: `0`
- Serial sessions: `0`
- Flash operations: `0`
- Reset operations: `0`
- HTTP operations: `0`
- Hardware effects: `0`

The software preflight is evidence of package identity and gate readiness only. It
is not hardware parity evidence and does not authorize or imply any checklist
promotion.

## Hardware Attempt Conclusion

| Field | Recorded value |
| --- | --- |
| Completion | `2026-07-17T22:59:25Z` |
| Conclusion | `non_promotion` |
| Failure category | `wifi_credentials_path_missing` |
| Restoration | `not_needed` |
| Process-tree cleanup | `true` |
| Unexpected serial-holder count | `0` |
| Protected root reusable | `false` |
| Admission invoked | `false` |
| Checklist changed | `false` |

The protected root was sealed non-promotable. Admission was not invoked, no retry
was attempted, and Phase 35 remains incomplete.
