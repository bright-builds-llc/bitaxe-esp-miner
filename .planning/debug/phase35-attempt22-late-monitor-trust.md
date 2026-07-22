---
status: verifying
trigger: "Attempt 22 reached ready detector, probe, factory, NVS, and monitor boundaries but the flash wrapper rejected the private capture before the Phase 33 baseline classifier ran."
created: 2026-07-22T14:05:06Z
updated: 2026-07-22T14:05:06Z
---

## Current Focus

hypothesis: espflash 4.5.0's native-USB monitor reset and reopen delay can miss one-shot early boot markers even when the later replayed Phase 33 identity and origin evidence is complete, so the dual-evidence wrapper is applying its legacy trust gate before the authoritative private classifier.
test: Classify the sealed Attempt 22 private monitor input offline, then reproduce the late-attach shape hermetically with a dual-mode timeout that lacks legacy one-shot markers but contains valid replayed Phase 33 identity and origin blocks.
expecting: The authoritative Phase 33 classifier passes the immutable private input, ordinary evidence mode remains fail-closed, and dual mode defers only timeout captures to classification before finalization.
next_action: Complete the full software gate, commit the verified repair and redacted checkpoint, then run exact-head preflight before fresh Attempt 23.

## Symptoms

expected: A dual private capture is classified by the Phase 33 functional core before any admitted derivative is created.
actual: All typed flash stages were `ready`, but the flash process returned nonzero because early legacy boot markers were absent; the supervisor preserved `flash_or_boot_a_failed` before invoking Phase 33 classification.
errors: Shareable signature is `flash_or_boot_a_failed` with `flash_stage=monitor`, `flash_boundary=ready`, `capture_status=timed_out_without_trusted_output`, and no mutation.
reproduction: Attempt 22 is sealed and must not be reused. A hermetic late-attach log reproduces the wrapper rejection while passing `classify_phase33_baseline`.
started: Attempt 22 at exact source `55a8f31ac9be6a2c056cd04f8cc226b923782b22` after the user-confirmed remediation and exact-head preflight.

## Eliminated

- Persistent detector/reset incompatibility: detector admission and the in-invocation checksum probe passed.
- Factory, NVS, or monitor flash-stage failure: every typed stage classified `ready`.
- Missing authoritative boot evidence: offline Phase 33 baseline classification of the immutable private input returned `status=passed` and `category=none`.
- Credential, PATCH, restoration, or cleanup failure: mutation never started and both secondary outcomes were `none`.

## Evidence

- timestamp: 2026-07-22T14:05:06Z
  checked: Attempt 22's sealed projection and private evidence metadata without printing protected identifiers or raw monitor material.
  found: Detector, checksum probe, factory, NVS, and monitor all reached `ready`; the capture timed out after later runtime evidence but without the legacy one-shot boot markers.
  implication: The hardware and flash/reset repair worked; the remaining defect is evidence-gate ordering.
- timestamp: 2026-07-22T14:05:06Z
  checked: The built Phase 33 parity classifier against the immutable private monitor input.
  found: `status=passed` and `category=none`.
  implication: The supervisor would have accepted Boot A if the dual wrapper had returned control instead of terminating at the legacy marker gate.
- timestamp: 2026-07-22T14:05:06Z
  checked: Focused Cargo and Bazel regressions after the dual-mode deferred-classification repair.
  found: The flash crate's 151 tests and the focused flash, Phase 35 HTTP/supervisor/promotion, and Phase 30 non-promotion targets passed.
  implication: The repair is narrow, preserves default fail-closed behavior, and reaches authoritative private classification before admitted derivation.

## Resolution

root_cause: Dual evidence reused the legacy wrapper trust decision as a terminal exit condition even though Phase 35 intentionally delegates admission to the stronger Phase 33 classifier over an immutable private artifact. espflash's delayed native-USB monitor reopen made the ordering defect observable by missing one-shot early markers while replayed classifier evidence remained complete.
fix: Add explicit pending and post-classification timeout states. Only dual-mode timeouts may return a private artifact for authoritative classification; ordinary mode and child failures remain terminal. Finalization accepts only legacy-trusted or explicitly pending captures and marks deferred evidence complete only after the caller's digest-bound classification gate.
verification: Attempt 22 offline classification passed; focused Cargo and Bazel suites passed; full repository verification is pending before commit and Attempt 23.
files_changed:
  - tools/flash/src/main.rs
  - .planning/debug/phase35-attempt21-detector-connection.md
  - .planning/debug/phase35-attempt22-late-monitor-trust.md
