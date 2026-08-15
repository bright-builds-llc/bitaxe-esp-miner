# Parity work log

## 2026-08-15T18:57:00Z | software-only marker observation contract

- Selection: THR-001 remains first after the truthful attempt-005 closure.
- Failure signals: Canonical production marker payloads begin after the ESP log
  prefix, the host accepts only byte-zero fixtures, and the early baseline
  marker was absent from the post-flash capture.
- Action: Freeze a software-only plan that requires two real-process red loops
  before choosing a strict parsing and late-observer design.
- Safety: No hardware or attempt-006 authority exists. THR-001 remains
  `implemented` and the consumed attempt-005 evidence remains withheld.

## 2026-08-15T19:05:00Z | immutable-plan verification

- Plan SHA-256:
  `df13633088f5471dc84d31439add7c4144732c5c4f153e9202af371c4d324187`.
- Verification: Ordered Cargo gates, Bright Builds, parity/progress, redaction,
  pinned-reference cleanliness, and diff checks passed without device access.
- Outcome: The two-loop software diagnosis contract is ready to commit and
  push before implementation. No attempt-006 effect is authorized.

## 2026-08-15T19:20:00Z | red production-shape regressions

- Red seam: The canonical automation target rejected three production-shaped
  cases with `evidence_invalid`: an ESP-prefixed complete triplet, a late
  attachment followed by a replayed triplet, and the real child-process case.
- Ranked causes: byte-zero host matching was confirmed; device marker loss was
  falsified because all three states are already retained; ordinary thermal
  freshness and the fault reducer are outside this failure path.
- Design: Parse only canonical INFO payloads from the `bitaxe_firmware` tag,
  accept one exact contiguous triplet, and request bounded replay only for an
  admitted diagnostic package. Bare fixture lines remain inadmissible.

## 2026-08-15T19:42:00Z | green implementation and verification

- Host boundary: The validator extracts only canonical INFO payloads with a
  numeric uptime and exact `bitaxe_firmware` tag, then requires one complete
  contiguous baseline/fault/recovery triplet. Arbitrary prefixes, malformed
  timestamps, wrong levels/tags, missing states, and wrong order fail closed.
- Firmware boundary: An admitted thermal stimulus requests the existing
  bounded retained-diagnostic replay. The allowlist now includes only complete
  `thermal_fault_stimulus` records, so a late monitor receives the already-
  retained triplet without a sleep, public setter, or ordinary-package replay.
- Green evidence: The production-shaped fake and real-child cases, explicit
  late attachment, duplicate prefix, timeout precedence, non-replay recovery,
  and evidence withholding pass. The ESP32-S3 firmware build also passes.
- Gates: Ordered Cargo format/Clippy/build/test, Bright Builds, `just build`,
  all 45 Bazel tests, parity/progress, redaction, reference cleanliness, and
  diff checks passed. THR-001 remains `implemented`; no hardware ran.
