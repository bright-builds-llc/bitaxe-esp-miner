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

## Continuation Attempt 2 Checkpoint

The second fresh attempt stopped at the post-detector opaque-input boundary. A
software-only diagnosis corrected the emitted category: the input was available,
but the Bazel/runfiles process resolved the caller-relative argument against the
wrong working directory. No hardware command was retried after diagnosis.

| Field | Recorded value |
| --- | --- |
| Attempt ordinal | `2` |
| Corrected category | `path_resolution_failure` |
| Pre-mutation | `true` |
| Cleanup confirmed | `true` |
| Unexpected serial-holder count | `0` |
| Protected root reusable | `false` |
| Admission invoked | `false` |
| Tracked diff count at root close | `0` |

The protected root remains sealed non-promotable. The software repair does not
admit evidence, update a checklist row, complete Task 2, or authorize an automatic
retry. A fresh continuation must own any later one-shot attempt.

## Continuation Attempt 3 Checkpoint

The third fresh attempt ran the full Phase 35 command exactly once. Gate 1
revalidated the exact current package, the sole detector gate admitted one
board-205 candidate with successful board-info, and the post-detector opaque input
gate passed. The attempt then failed during the flash/Boot A boundary before any
PATCH or settings mutation. The protected log contains no emitted flash command,
NVS-seed command, monitor command, capture outcome, or monitor log. The exact
sub-boundary is therefore a pre-capture wrapper failure: it does not prove a device
flash hard error, and Boot A capture or qualification did not begin.

| Field | Recorded value |
| --- | --- |
| Completion | `2026-07-18T16:15:57Z` |
| Attempt ordinal | `3` |
| Source commit | `cd468b9197637be7b994ef97b38320e96bc66e54` |
| Board category | `205` |
| Full command invocations | `1` |
| Failure category | `flash_or_boot_a_failed` |
| Failure boundary | `pre_capture_wrapper_failure` |
| Device flash hard error proven | `false` |
| Boot A capture started | `false` |
| Boot A qualification ran | `false` |
| Pre-mutation | `true` |
| Restoration | `not_needed` |
| Process-tree cleanup | `true` |
| Unexpected serial-holder count | `0` |
| Protected root mode | `0700` |
| Private file modes | `0600` |
| Protected root reusable | `false` |
| Admission invoked | `false` |
| Checklist changed | `false` |

The protected root is sealed non-promotable and cannot be reused or spliced.
Task 2 and Phase 35 remain incomplete. No admission, checklist promotion, Task 3
audit, or plan summary is authorized from this attempt.

## Software Repair After Attempt 3

Commit `46fe7f0b2837255749ef63a6f6f7aa4f3ad605d1` repairs the diagnosed
pre-capture wrapper boundary without touching hardware. After detector and opaque
input validation, the supervisor now resolves the already-built flash executable
from the workspace `bazel-bin` tree or its Bazel runfiles and invokes
`flash-monitor` directly. It no longer starts a nested `just flash-monitor` or
Bazel process.

The hermetic regression test makes nested `just` and Bazel executables fail on
invocation, then proves exactly one direct `flash-monitor` call after detector and
credential validation. It checks the exact admitted manifest path, detector-derived
port category, opaque workspace credential path, protected evidence directory, and
360-second capture timeout without real hardware or secret material.

| Software verification | Result |
| --- | --- |
| Shell syntax, formatting, and lint checks | passed |
| Phase 35 correlated-evidence regression suite | passed |
| Phase 35 promotion and Phase 30 non-promotion contracts | passed |
| Parity tests and checklist validation | passed |
| Reference cleanliness | passed |
| Phase 35 lifecycle verification | passed |
| Ordered Rust format, lint, build, and test gates | passed |
| Diff and redaction review | passed |

This software repair is not hardware evidence. It does not reopen or qualify the
sealed attempt-3 root, admit evidence, change a checklist row, complete Task 2, or
authorize a retry. Any later hardware action requires a separately owned explicit
continuation decision and a fresh protected root.

## Continuation Attempt 4 Checkpoint

The fourth fresh attempt ran the full Phase 35 command exactly once from clean
source `28b68dcccd3b8547a7781db0212cab774ad97ab7`. Gate 1 revalidated the
exact-current package, the sole detector gate admitted one board-205 candidate
with successful board-info, and the post-detector opaque input gate passed. The
direct flash command completed and produced a non-empty Boot A monitor capture,
but the typed baseline classifier rejected that capture because it contained
multiple boot-session identities. No current-session origin was admitted.

The supervisor emitted `target_missing` after reading the rejected classifier
projection. Private software-only diagnosis established that this category masked
the earlier typed `baseline_multiple_sessions` rejection. The attempt stopped
before any HTTP settings read, PATCH, reboot, or settings mutation.

| Field | Recorded value |
| --- | --- |
| Completion | `2026-07-18T18:34:10Z` |
| Attempt ordinal | `4` |
| Source commit | `28b68dcccd3b8547a7781db0212cab774ad97ab7` |
| Board category | `205` |
| Full command invocations | `1` |
| Detector invocations | `1` |
| Single candidate verified | `true` |
| Board-info verified | `true` |
| Opaque input gate passed | `true` |
| Flash command completed | `true` |
| Boot A monitor capture non-empty | `true` |
| Boot A classification status | `failed` |
| Boot A classification category | `baseline_multiple_sessions` |
| Supervisor-emitted category | `target_missing` |
| Failure boundary | `boot_a_baseline_qualification` |
| Current-session origin admitted | `false` |
| HTTP settings read started | `false` |
| PATCH mutation started | `false` |
| Restoration | `not_needed` |
| Process-tree cleanup | `true` |
| Unexpected serial-holder count | `0` |
| Remaining Phase 35 process count | `0` |
| Protected root mode | `0700` |
| Private file modes | `0600` |
| Protected root reusable | `false` |
| Admission invoked | `false` |
| Checklist changed | `false` |

The protected root is sealed non-promotable and cannot be reused, retried, or
spliced. Attempt 4 does not complete Task 2, authorize Task 3, or support a plan
summary.

## Software Repair After Attempt 4

Commit `572da63864fd73efefb7672dbe8c2908d4885d13` fixes the deterministic
supervisor/classifier contract defect diagnosed from the sealed attempt. The
supervisor now checks the classifier document's typed status before target
derivation, rejects invalid classifier output, preserves a safe classifier rejection
category, and explicitly propagates that category through Boot A failure handling.

The hermetic direct-flash regression supplies a rejected Boot A classifier
projection and proves that the supervisor:

- preserves `baseline_multiple_sessions` in stderr and the non-promotion seal;
- stops before settings reads, capture epochs, PATCH, reboot, restoration, or
  validation;
- performs cleanup after exactly one detector, opaque-input, and direct-flash
  sequence.

| Software verification | Result |
| --- | --- |
| Shell syntax and format checks | passed |
| Shell lint for changed paths | passed |
| Ordered Rust format, lint, build, and test gates | passed |
| Phase 35 correlated-evidence regression suite | passed |
| Phase 35 promotion and Phase 30 non-promotion contracts | passed |
| Parity tests and checklist validation | passed |
| Reference cleanliness | passed |
| Phase 35 lifecycle verification | passed |
| Diff and redaction review | passed |

This repair is software-only. It does not change the attempt-4 result, admit an
evidence generation, update a checklist row, complete Task 2, or authorize a
hardware retry.

## Continuation Attempt 5 Checkpoint

The fifth fresh attempt ran the full Phase 35 command exactly once from clean
source `8265520c4888bcb8eeca3363c11b4716e33d7385`. Gate 1 revalidated the
exact-current package. The supervisor then made its sole detector invocation,
selected exactly one candidate, and invoked board-info exactly once. Board-info
failed at the transport connection boundary, so detector admission did not
complete.

Private category-level diagnosis distinguished this outcome from no candidate,
multiple candidates, an open or ownership failure, and a deterministic
detector/supervisor defect. No deterministic software defect was proven. The
attempt stopped before the opaque credential gate, flash, Boot A capture, target
derivation, HTTP settings reads, PATCH, reboot, or any settings mutation.

| Field | Recorded value |
| --- | --- |
| Completion | `2026-07-18T23:46:44Z` |
| Attempt ordinal | `5` |
| Source commit | `8265520c4888bcb8eeca3363c11b4716e33d7385` |
| Board category | `205` |
| Full command invocations | `1` |
| Exact-package Gate 1 passed | `true` |
| Detector invocations | `1` |
| Selected candidate count | `1` |
| Board-info invocations | `1` |
| Board-info verified | `false` |
| Failure category | `connection_failure` |
| Failure boundary | `board_info_transport_connection` |
| Deterministic software defect proven | `false` |
| Opaque credential gate reached | `false` |
| Flash started | `false` |
| Boot A capture started | `false` |
| HTTP settings read started | `false` |
| PATCH mutation started | `false` |
| Approved reboot started | `false` |
| Restoration | `not_needed` |
| Process-tree cleanup | `true` |
| Unexpected serial-holder count | `0` |
| Remaining Phase 35 process count | `0` |
| Protected root mode | `0700` |
| Private file modes | `0600` |
| Protected root reusable | `false` |
| Admission invoked | `false` |
| Evidence generation changed | `false` |
| Checklist changed | `false` |

The protected root is sealed non-promotable and cannot be reused, retried, or
spliced. No hardware retry occurred in this continuation. Attempt 5 does not
complete Task 2, authorize Task 3, or support a plan summary. Any later hardware
action requires a separately owned explicit continuation decision and a fresh
protected root.

## Continuation Attempt 6 Checkpoint

The sixth fresh continuation invoked the full Phase 35 command exactly once from
clean source `fa6dbf9e8d12b34b3238eae4b4541d4cc5b805fa`. The caller created the
exact path passed as `local-root` before launch so it could place the wrapper log
inside that directory. The fail-closed supervisor correctly rejected the existing
path with typed category `evidence_root_already_exists` before exact-package Gate
1 and before its sole detector boundary.

This was a caller setup error, not a deterministic repository defect. The
supervisor contract must continue to require that the exact `local-root` child
does not exist at launch. No detector, opaque credential gate, serial session,
flash, monitor, reset, target derivation, HTTP operation, PATCH, reboot,
restoration mutation, admission, evidence publication, checklist update, or
generation change occurred.

| Field | Recorded value |
| --- | --- |
| Completion | `2026-07-19T00:08:03Z` |
| Attempt ordinal | `6` |
| Source commit | `fa6dbf9e8d12b34b3238eae4b4541d4cc5b805fa` |
| Board category | `205` |
| Full command invocations | `1` |
| Exact-package Gate 1 reached | `false` |
| Detector invocations | `0` |
| Failure category | `evidence_root_already_exists` |
| Failure boundary | `protected_root_initialization` |
| Deterministic repository defect proven | `false` |
| Opaque credential gate reached | `false` |
| Serial sessions started | `0` |
| Flash or monitor started | `false` |
| HTTP settings read started | `false` |
| PATCH mutation started | `false` |
| Approved reboot started | `false` |
| Restoration | `not_needed` |
| Process-tree cleanup | `true` |
| Unexpected serial-holder count | `0` |
| Remaining Phase 35 process count | `0` |
| Protected root mode | `0700` |
| Private file modes | `0600` |
| Protected root reusable | `false` |
| Admission invoked | `false` |
| Evidence generation changed | `false` |
| Checklist changed | `false` |
| Plan summary created | `false` |
| Retry in this continuation | `false` |

The protected root is sealed non-promotable and cannot be reused, retried, or
spliced. Task 2 and Phase 35 remain incomplete, and Task 3 is not authorized.

For any separately authorized later attempt, create a private mode-`0700` parent,
keep the exact child passed as `local-root` nonexistent, and redirect the
mode-`0600` wrapper output to a sibling file in the private parent. Never place
wrapper output inside the nonexistent child before the supervisor creates it.

## Continuation Attempt 7 Checkpoint

The seventh fresh continuation corrected the caller root contract before invoking
the full Phase 35 command exactly once from clean source
`a53831d47f38a92443b78eab743fb85104f9caf5`. The caller created one ignored
mode-`0700` private parent, left the supervisor-owned `local-root` child
nonexistent through the immediate pre-launch assertion, and placed mode-`0600`
wrapper output in a sibling file. The supervisor created the child successfully,
passed exact-package Gate 1, and made its sole detector invocation.

The detector selected exactly one candidate and invoked board-info exactly once.
Board-info failed at the transport connection boundary with typed category
`connection_failure`, matching the category-level outcome from attempt 5. This
was not a missing or ambiguous candidate, open or ownership failure, leaked-holder
failure, or deterministic repository defect. The attempt stopped before the
opaque credential gate, flash, Boot A capture, target derivation, HTTP settings
reads, PATCH, reboot, or any settings mutation.

| Field | Recorded value |
| --- | --- |
| Completion | `2026-07-19T00:21:09Z` |
| Attempt ordinal | `7` |
| Source commit | `a53831d47f38a92443b78eab743fb85104f9caf5` |
| Board category | `205` |
| Full command invocations | `1` |
| Corrected protected-root contract | `true` |
| Exact `local-root` child absent before launch | `true` |
| Sibling wrapper mode | `0600` |
| Supervisor root initialization passed | `true` |
| Exact-package Gate 1 passed | `true` |
| Detector invocations | `1` |
| Selected candidate count | `1` |
| Board-info invocations | `1` |
| Board-info verified | `false` |
| Failure category | `connection_failure` |
| Failure boundary | `board_info_transport_connection` |
| Matches attempt-5 typed category | `true` |
| Deterministic repository defect proven | `false` |
| Opaque credential gate reached | `false` |
| Flash started | `false` |
| Boot A capture started | `false` |
| HTTP settings read started | `false` |
| PATCH mutation started | `false` |
| Approved reboot started | `false` |
| Restoration | `not_needed` |
| Process-tree cleanup | `true` |
| Unexpected serial-holder count | `0` |
| Remaining Phase 35 process count | `0` |
| Protected root mode | `0700` |
| Private file modes | `0600` |
| Protected root reusable | `false` |
| Admission invoked | `false` |
| Evidence generation changed | `false` |
| Checklist changed | `false` |
| Plan summary created | `false` |
| Retry in this continuation | `false` |

The protected root is sealed non-promotable and cannot be reused, retried, or
spliced. Task 2 and Phase 35 remain incomplete, and Task 3 is not authorized.
After the repeated board-info transport connection failure, the next checkpoint
is a human-action USB re-enumeration: leave barrel/DC power connected and
unchanged; unplug only the provided USB cable from the computer or device; wait
for the USB node to disappear; reconnect the same USB cable; and wait for USB
re-enumeration. This is USB re-enumeration, not a cold start or barrel-power
cycle. It does not authorize direct UART, pins, pads, probes, or other electrical
manipulation.
