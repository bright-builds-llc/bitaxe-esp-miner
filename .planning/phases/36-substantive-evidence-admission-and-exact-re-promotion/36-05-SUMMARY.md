---
phase: 36-substantive-evidence-admission-and-exact-re-promotion
plan: "05"
subsystem: evidence
tags: [rust, broker, capability, append-only-ledger, unix-socket, privacy, bazel]
requires:
  - phase: 36-03
    provides: fail-closed substantive, runtime-identity, independent-effect, and claim-specific admission
provides:
  - exclusive single-use Phase 36 effect capabilities
  - hash-chained append-only typed effect ledger
  - exact substantive private-capture and commit-redacted candidate contracts
  - five-mode supervisor with fresh-process and deployed-runfiles verification
  - exact-current-package hardware transaction with typed eligible/non-promotion sealing
affects: [36-06, 36-07, phase36-hardware-preflight, evidence-promotion]
tech-stack:
  added: []
  patterns:
    - broker-exclusive effect capability with typed ledger transitions
    - immutable private capture followed by distinct commit-redacted derivation
    - length-prefixed sequenced cross-process frames with fail-closed parsing
key-files:
  created:
    - tools/parity/src/phase36_broker.rs
    - tools/parity/src/phase36_broker/contract.rs
    - tools/parity/src/phase36_broker/hardware.rs
    - tools/parity/src/phase36_broker/hardware_process.rs
    - tools/parity/src/phase36_broker/ipc.rs
    - tools/parity/src/phase36_broker/ledger.rs
    - tools/parity/src/phase36_evidence/capture.rs
    - tools/parity/src/phase36_evidence/capture/hardware.rs
    - scripts/phase36-hardware-effect.sh
    - scripts/phase36-substantive-evidence.sh
    - scripts/phase36-substantive-evidence-test.sh
  modified:
    - tools/parity/src/main.rs
    - tools/parity/BUILD.bazel
    - scripts/BUILD.bazel
    - Justfile
    - scripts/phase36-evidence-test.sh
key-decisions:
  - "Bind each private single-use capability to the exact attempt, evaluator, package, peer, protected root, and expiry before admitting any closed operation."
  - "Keep private capture bytes immutable and derive the commit-redacted candidate only after exact bundle, runtime-identity, package, and ledger joins pass."
  - "Implement the complete hardware transaction now, while leaving its single authorized real execution to Plan 36-06."
  - "Keep the exact detector program and argument broker-owned; the supervisor carries only an opaque credential path and never receives detector output, port, DEVICE_URL, or an effect adapter."
patterns-established:
  - "Effect ownership: the supervisor may pass an opaque capability but never receives effect adapters, device targets, or a writable ledger."
  - "Offline boundaries: inspect is read-only; classify writes only one explicit mode-0600 output without changing private or candidate bytes."
requirements-completed: [SYS-02, EVD-11, EVD-12, EVD-14]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 36-2026-07-23T15-20-53
generated_at: 2026-07-24T21:29:57Z
duration: 55min
completed: 2026-07-24
---

# Phase 36 Plan 05: Substantive Evidence Broker and Capture Harness Summary

**Exact-package passive hardware transactions with a hash-chained private ledger, same-session capture admission, and typed eligible/non-promotion sealing**

## Performance

- **Duration:** 55 min
- **Started:** 2026-07-24T20:35:34Z
- **Completed:** 2026-07-24T21:29:57Z
- **Tasks:** 3
- **Files modified:** 21

## Accomplishments

- Added a closed broker capability contract with replay, expiry, peer, evaluator, attempt, package, and protected-root checks plus typed failure categories.
- Added an append-only, hash-chained, mode-0600 effect ledger that preserves the earliest failure and cleanup result and converts only sealed intervals into evidence admission.
- Added exact private substantive capture and commit-redacted candidate derivation covering sensor truth, runtime health, runtime identity, package identity, broker ledger, and same boot/revision/device joins.
- Added a five-mode supervisor and deployed Bazel test exercising preflight, synthetic, hardware fail-closed routing, inspection, and classification without detector, USB, serial, credentials, target discovery, device network, or hardware access.
- Added the hardware broker's exact-once `just detect-ultra205` pre-capture gate, with detector output retained inside the broker and credential validation unreachable after detector failure.
- Added the complete broker-owned production transaction: exact package admission, detector facts, exact flash, passive serial/API/WebSocket/retained capture, same-image recovery, cleanup, private classification, and distinct commit-redacted derivation.
- Bound preflight capabilities to clean current HEAD, reference, target, board, ASIC, evaluator, manifest, ELF, application/executable, factory, and package identities, with single-use child/replay rejection.
- Added typed `SealedEligible` and `SealedNonPromotion` outcomes that retain the earliest failure, plus a qualified fake capture proving the production assembler can derive an eligible candidate.
- Proved cross-process frame handling, permissions, descriptor isolation, immutable inputs, source/runfiles bypass restrictions, cleanup, and actual Phase 36 incomplete-plan ordering.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define the exclusive broker capability and append-only typed ledger** - `9a5f35cf` (feat)
2. **Task 2: Build exact substantive capture and admission harness** - `b3e001a0` (feat)
3. **Task 3: Prove broker and harness OS boundaries** - `697d59e4` (test)
4. **Plan-wide compatibility fix: Accept the rotated evaluator rejection boundary** - `f270aea1` (fix)
5. **Cross-plan correctness fix: Wire the broker-owned detector gate** - `f9bfba39` (fix)
6. **Plan 36-06 preflight repair: Complete the hardware broker transaction** - `b0f688f3` (fix)

## Files Created/Modified

- `tools/parity/src/phase36_broker.rs` and `tools/parity/src/phase36_broker/` - Closed capability, ledger, and IPC contracts with focused tests.
- `tools/parity/src/phase36_broker/hardware.rs` and `hardware_process.rs` - Exact-once detector, typed transaction reducer, private child/ledger ownership, bounded adapters, recovery, cleanup, and sealing.
- `tools/parity/src/phase36_evidence/capture.rs` and `tools/parity/src/phase36_evidence/capture/` - Exact private bundle validation, synthetic and qualified hardware assembly, candidate derivation, and filesystem boundaries.
- `tools/parity/src/main.rs` - Fresh-process capture, evaluator identity, hardware assembly, transaction, inspection, and classification command wiring.
- `scripts/phase36-substantive-evidence.sh` and `scripts/phase36-hardware-effect.sh` - Five-mode protected-root supervisor and closed production effect adapter.
- `scripts/phase36-substantive-evidence-test.sh` - Deployed process, runfiles, privacy, cleanup, and graph-order regression suite.
- `tools/parity/fixtures/phase36-broker/ipc-cases.json` - Fragmented, coalesced, short, oversized, duplicate, reordered, and post-close frame cases.
- `tools/parity/BUILD.bazel`, `scripts/BUILD.bazel`, and `Justfile` - Exact test targets, deployed runfiles, and command surface.
- `scripts/phase36-evidence-test.sh` - Compatibility assertion for authoritative fail-closed envelope rejection after evaluator identity rotation.

## Decisions Made

- Capabilities remain private, single-use values rather than serializable public authority; only their digest crosses into the fresh synthetic capture process.
- The broker is the sole ledger writer and opens the ledger with append-only and close-on-exec semantics; children cannot inherit writable authority.
- Private capture and public candidate are separate files with independent digests. Inspection cannot write, and classification cannot modify either source.
- The software-only repair implements the full hardware path but tests it only with fake and synthetic boundaries. It did not run detector, credentials, USB, serial, target discovery, flash, monitor, device network, or hardware.
- The literal detector command is owned only by the Rust broker. Its captured output is discarded on failure and never crosses to supervisor stdout/stderr; credential metadata is examined only after detector success.
- The same-session origin exists only inside the broker process; the supervisor never receives a port, origin, credential value, physical identity, or effect adapter.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Removed an ineffective append-open flag**

- **Found during:** Task 1 (exclusive broker capability and typed ledger)
- **Issue:** Clippy rejected the redundant `write(true)` option when the ledger was already opened with append semantics.
- **Fix:** Removed the ineffective option while retaining append-only, mode-0600, and close-on-exec behavior.
- **Files modified:** `tools/parity/src/phase36_broker/ledger.rs`
- **Verification:** The ordered Rust fmt, clippy, build, and test gate passed.
- **Committed in:** `9a5f35cf`

**2. [Rule 1 - Bug] Accepted evaluator-identity drift as an earlier authoritative rejection**

- **Found during:** Plan-wide `just test` after Tasks 2 and 3
- **Issue:** Adding evaluator-reachable commands rotated the Phase 36 evaluator identity, so the older envelope-only regression failed at `evaluator_identity_mismatch` before reaching `protected_input_missing`.
- **Fix:** Narrowed the assertion to accept exactly those two authoritative fail-closed categories while preserving empty-public-output and canary checks.
- **Files modified:** `scripts/phase36-evidence-test.sh`
- **Verification:** `//scripts:phase36_evidence_test`, all 75 `just test` targets, and the full ordered Rust gate passed.
- **Committed in:** `f270aea1`

**3. [Rule 1 - Bug] Added the missing broker-owned detector invocation**

- **Found during:** Wave 6 cross-plan key-link verification after Plan 36-05 completion
- **Issue:** Hardware mode delegated to an absent broker command, so the required prior-wave link to exactly one `just detect-ultra205` invocation did not exist.
- **Fix:** Added a broker-owned pre-capture gate that invokes the detector exactly once, captures its output privately, stops on failure before credential metadata or later effects, and validates the opaque credential file only after detector admission. The supervisor receives no detector output, port, DEVICE_URL, or effect adapter.
- **Files modified:** `tools/parity/src/phase36_broker/hardware.rs`, `tools/parity/src/phase36_broker.rs`, `tools/parity/src/main.rs`, `tools/parity/BUILD.bazel`, `scripts/phase36-substantive-evidence.sh`, `scripts/phase36-substantive-evidence-test.sh`, `scripts/BUILD.bazel`
- **Verification:** Three focused Rust ordering tests pass; the deployed fake-adapter test proves one detector invocation and zero credential/effect access after detector failure; all exact Phase 36 Bazel targets and the full Rust gate pass; the first Plan 36-06 key link verifies while its two Wave 6 output links remain absent.
- **Committed in:** `f9bfba39`

**4. [Rule 1 - Bug] Replaced the terminal capture stub with the complete transaction**

- **Found during:** Plan 36-06 software preflight
- **Issue:** The production command stopped at `phase36_broker_capture_not_started`; it had no exact-package binding, child/ledger lifecycle, passive capture adapters, typed failure seal, recovery, cleanup, or eligible candidate path.
- **Fix:** Added preflight v2 identity closure, a single-use broker transaction, qualified effect adapters, same-session private assembly, typed recovery and cleanup, eligible/non-promotion seals, and evaluator/source drift binding.
- **Files modified:** `scripts/phase36-hardware-effect.sh`, `scripts/phase36-substantive-evidence.sh`, `tools/parity/src/phase36_broker/hardware.rs`, `tools/parity/src/phase36_broker/hardware_process.rs`, `tools/parity/src/phase36_evidence/capture/hardware.rs`, supporting capture/runtime/main/Bazel/test files.
- **Verification:** The qualified fake assembly produced an eligible candidate; the deployed detector-failure path produced a 16-record private ledger and typed non-promotion seal; the exact current package and preflight v2 passed all software gates.
- **Committed in:** `b0f688f3`

**Total deviations:** 4 auto-fixed (1 blocking issue, 3 compatibility/correctness bugs)

**Impact on plan:** The fixes preserve the intended fail-closed and append-only contracts while making the Plan 36-06 production path constructible without widening effect authority or claim scope.

## Issues Encountered

- Bazel serialized two concurrent commands on its output-base lock during final parallel verification; both commands completed successfully without changing the acceptance result.

## User Setup Required

None - this plan was fully software-only and required no external configuration.

## Verification

- Ordered Rust gate passed: `cargo fmt --all`, Clippy with warnings denied, all-target/all-feature build, and all-feature tests.
- `just test` passed all 75 targets.
- Exact targets passed fresh: `//tools/parity:phase36_broker_tests`, `//tools/parity:phase36_evidence_tests`, and `//scripts:phase36_substantive_evidence_test`.
- The exact current firmware image rebuilt successfully, and preflight v2 verified every required manifest, package, firmware, target, board, ASIC, source, and evaluator identity before detector or credential access.
- The deployed detector-failure path sealed ordered detector and recovery failures, completed portless cleanup, emitted no candidate, and rejected replay without a second detector invocation.
- Plan 36-06 key-link verification now passes the prior-wave supervisor-to-detector link; its two later Wave 6 output links remain correctly absent.
- `just parity`, `just verify-redaction`, `just verify-reference`, and `git diff --check` passed.
- Phase 35 remained byte-identical and retained root digest `0401e7b485df2d1ccfc67e63845f98b6217816a184901bf0595d03af3219757d`.
- The paused Plan 36-04 tree remained identical to its start-of-plan baseline.
- No hardware, detector, credentials, USB, serial, target discovery, flash, monitor, direct UART/pin, archived Phase 28.1.1, or canonical reconciliation action occurred.

## Known Stubs

None. The production hardware transaction is implemented and software-preflighted; its single authorized real execution remains exclusively owned by Plan 36-06.

## Next Phase Readiness

- Plan 36-06 may evaluate the exact preflight against the now-green software broker and capture boundaries.
- Hardware success, eligible substantive evidence, and any re-promotion remain unclaimed until their owning later plans execute and satisfy the existing evidence gates.

## Self-Check: PASSED

All key created files exist, and commits `9a5f35cf`, `b3e001a0`, `697d59e4`, `f270aea1`, `f9bfba39`, and `b0f688f3` are present in repository history.
