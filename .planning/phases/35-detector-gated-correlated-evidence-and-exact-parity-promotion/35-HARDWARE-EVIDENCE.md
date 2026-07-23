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

## Continuation Attempt 8 Checkpoint

The user completed the requested USB-only re-enumeration while barrel/DC power
remained connected and unchanged. This was not a cold start or barrel-power
cycle. The eighth fresh continuation then ran the full Phase 35 command exactly
once from clean source `564153c57cea64da26f380e793c542a18bfa7c7a`.

The corrected protected-root contract passed immediately before launch. The
supervisor created the nonexistent child, passed exact-package Gate 1, made its
sole detector invocation, selected exactly one candidate, and completed its
single board-info invocation successfully. The USB re-enumeration therefore
resolved the transport connection boundary that blocked attempts 5 and 7. The
post-detector opaque input gate passed, the direct flash command completed, and
Boot A monitor capture was non-empty.

The strict Boot A classifier rejected the capture with
`baseline_multiple_sessions`. The capture contained 59 distinct boot sessions
and 59 distinct boot ordinals, with every observed transition advancing the
ordinal by one. Its reset-category distribution was one `other`, 53 `panic`,
and five `watchdog`; 52 sessions explicitly reported a stack overflow in the
firmware `main` task. The first stack overflow preceded the second boot identity.
This is a current-firmware restart loop, not expected flash, NVS-seed, or monitor
boundary noise. The classifier correctly refused to select an arbitrary session
or weaken the one-coherent-session admission rule.

The attempt stopped before target admission, HTTP settings reads, PATCH, the
approved normal reboot, or any settings mutation. Finalization recorded cleanup
once, confirmed zero unexpected serial holders and zero remaining Phase 35
processes, and sealed the protected root non-promotable and non-reusable.

| Field | Recorded value |
| --- | --- |
| Completion | `2026-07-19T05:19:35Z` |
| Attempt ordinal | `8` |
| Source commit | `564153c57cea64da26f380e793c542a18bfa7c7a` |
| Board category | `205` |
| Full command invocations | `1` |
| Corrected protected-root contract | `true` |
| Exact `local-root` child absent before launch | `true` |
| Sibling wrapper mode | `0600` |
| Exact-package Gate 1 passed | `true` |
| Detector invocations | `1` |
| Selected candidate count | `1` |
| Board-info invocations | `1` |
| Board-info verified | `true` |
| USB re-enumeration resolved prior transport blocker | `true` |
| Opaque input gate passed | `true` |
| Flash command completed | `true` |
| Boot A monitor capture non-empty | `true` |
| Boot A distinct session count | `59` |
| Boot A distinct ordinal count | `59` |
| Boot ordinal transition pattern | `increment_one` |
| Reset-category counts | `other:1, panic:53, watchdog:5` |
| Main-task stack-overflow count | `52` |
| Boot A classification status | `failed` |
| Failure category | `baseline_multiple_sessions` |
| Failure boundary | `boot_a_baseline_qualification` |
| Runtime restart loop proven | `true` |
| Expected flash/NVS/monitor boundary noise | `false` |
| Current-session origin admitted | `false` |
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
| Task 3 authorized | `false` |
| Plan summary created | `false` |
| Retry in this continuation | `false` |

The protected root is sealed non-promotable and cannot be reused, retried, or
spliced. Task 2 and Phase 35 remain incomplete, and Task 3 is not authorized.

## Software Repair After Attempt 8

Commit `9fb0a488d95a40303e8db6773af0ffb132d0b044` repairs the deterministic
runtime defect without touching hardware. The ESP-IDF `main` task stack is now
16 KiB instead of 8 KiB, and the Phase 35 hermetic regression requires exactly
one numeric stack assignment at or above that minimum. The strict classifier,
detector order, opaque input boundary, flash behavior, one-reboot rule,
restoration, cleanup, redaction, and admission contracts are unchanged.

| Software verification | Result |
| --- | --- |
| Shell syntax, formatting, and lint checks | passed |
| Ordered Rust format, lint, build, and test gates | passed |
| Phase 35 correlated-evidence regression suite | passed |
| Phase 35 promotion and Phase 30 non-promotion contracts | passed |
| Parity tests and checklist validation | passed |
| Canonical firmware image build and package | passed |
| Reference cleanliness | passed |
| Phase 35 lifecycle verification | passed |
| Diff review | passed |

This software repair is not hardware evidence. It does not reopen or qualify the
sealed attempt-8 root, admit an evidence generation, update a checklist row,
complete Task 2, authorize Task 3, or authorize a retry in this continuation.
The next allowed action is a separately authorized fresh continuation from the
clean repair commit. Any such continuation must rebuild and lock the exact
current package, use a new protected root, and make its own explicit one-shot
hardware decision. No further physical action is requested at this checkpoint.

## Continuation Attempt 9 Checkpoint

The ninth fresh continuation ran the full Phase 35 command exactly once from
clean source `bc35f9579200450ca03d78bc545cf2691a2cec87`. The repo-owned
entrypoint rebuilt and locked the exact current package containing the 16 KiB
main-task stack repair before the supervisor ran.

The corrected protected-root contract passed immediately before launch. The
supervisor created the nonexistent child, passed exact-package Gate 1, made its
sole detector invocation, selected exactly one candidate, and completed its
single board-info invocation successfully. The post-detector opaque input gate
passed, direct flash completed, and the strict Boot A classifier admitted one
coherent session. No restart loop or main-task stack overflow recurred, so this
attempt confirms the attempt-8 stack-capacity repair at the exact-current-package
hardware boundary.

The fresh current-session target was derived and passed the closed target-shape
validation. The first original-settings request then timed out before producing
any response body. Because no body existed, JSON parsing and hostname-field
schema/value validation did not begin. Private software-only diagnosis found no
deterministic supervisor or API-path defect from this outcome. The supervisor
sealed typed category `original_setting_unavailable` and stopped before any
PATCH, approved normal reboot, or settings mutation.

| Field | Recorded value |
| --- | --- |
| Completion | `2026-07-19T13:18:23Z` |
| Attempt ordinal | `9` |
| Source commit | `bc35f9579200450ca03d78bc545cf2691a2cec87` |
| Board category | `205` |
| Full command invocations | `1` |
| Exact current package rebuilt and locked | `true` |
| Main-task stack capacity | `16384` |
| Exact-package Gate 1 passed | `true` |
| Detector invocations | `1` |
| Selected candidate count | `1` |
| Board-info invocations | `1` |
| Board-info verified | `true` |
| Opaque input gate passed | `true` |
| Flash command completed | `true` |
| Boot A monitor capture non-empty | `true` |
| Boot A classification status | `passed` |
| Boot A coherent session count | `1` |
| Runtime restart loop observed | `false` |
| Main-task stack overflow observed | `false` |
| Target derived | `true` |
| Target shape validated | `true` |
| Original settings request outcome | `timeout` |
| Original settings response body | `missing` |
| Hostname schema validation reached | `false` |
| Failure category | `original_setting_unavailable` |
| Failure boundary | `original_settings_http_read` |
| Deterministic repository defect proven | `false` |
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
| Task 3 authorized | `false` |
| Plan summary created | `false` |
| Retry in this continuation | `false` |

The protected root is sealed non-promotable and cannot be reused, retried, or
spliced. Task 2 and Phase 35 remain incomplete, and Task 3 is not authorized.
No hardware retry occurred. Any later hardware attempt requires a separately
authorized fresh continuation and a new protected root; no further hardware
action is authorized by this checkpoint.

## Continuation Attempt 10 Checkpoint

The tenth fresh continuation first refreshed remote state and proved the clean
current commit was an exact descendant of the attempt-9 checkpoint with no newer
upstream commit. All nine ordered Rust, Bazel, reference, parity, lifecycle, and
preflight-only gates passed. The canonical package manifest source equaled the
current commit before the full command began.

The caller created one fresh ignored mode-`0700` private parent, left the
supervisor-owned child nonexistent through the immediate pre-launch assertion,
and placed mode-`0600` wrapper output in a sibling file. The full Phase 35 command
then ran exactly once with the literal opaque credential-path argument, a
360-second capture budget, and detector ownership inside the command.

The supervisor created the child, passed exact-package Gate 1, made its sole
detector invocation, selected exactly one candidate, and completed board-info
successfully. The post-detector opaque input gate passed, direct flash completed,
and the strict Boot A classifier admitted one coherent session. The fresh target
was present and passed the closed target-shape validation.

The first original-settings request then failed while receiving network data
without producing a response body. JSON parsing and hostname schema/value
validation therefore did not begin. The supervisor sealed typed category
`original_setting_unavailable` and stopped before PATCH, approved reboot, or any
settings mutation.

| Field | Recorded value |
| --- | --- |
| Completion | `2026-07-19T14:03:22Z` |
| Attempt ordinal | `10` |
| Source commit | `fbb667c282be0e55d4b644c42e86f659f939aec9` |
| Board category | `205` |
| Full command invocations | `1` |
| Full command duration seconds | `480` |
| Exact current package rebuilt and locked | `true` |
| Exact-package Gate 1 passed | `true` |
| Package capability digest | `ef6f947bede18a040b146f68d5251bd42b8754eb41282947a806dc38358d94fc` |
| Manifest digest | `8a632ce5192472e0f6e2b29137fbc5715f6ca432569ad129202ead6cd6884a41` |
| Executable-image digest | `81291cb80ab16492ab9b796d7010de0bb0b0e6bfe7d49f1fe183ebf60bf500b8` |
| Factory-image digest | `a31686e73c36e6911ba61f5de618289372a12e5a45b26d1f835996bca6fbbefd` |
| Package digest | `4c8aac79e8dbe669ba9a19e81e14bcff7beb93715c0fc9e4326765f72e19ad30` |
| Runtime-identity digest | `45f002bd3d73e0e84bd10eaf73f2d7fcd71ed96857877d6e5d6ba097ad43cff9` |
| Detector invocations | `1` |
| Selected candidate count | `1` |
| Board-info verified | `true` |
| Opaque input gate passed | `true` |
| Flash command completed | `true` |
| Boot A classification status | `passed` |
| Boot A coherent session count | `1` |
| Target present and shape validated | `true` |
| Original settings request outcome | `receive_failure` |
| Original settings response body | `missing` |
| Hostname schema validation reached | `false` |
| Failure category | `original_setting_unavailable` |
| Failure boundary | `original_settings_http_read` |
| Deterministic repository defect proven | `false` |
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
| Task 3 authorized | `false` |
| Plan summary created | `false` |
| Retry in this continuation | `false` |

### Software-Only Diagnosis

Private category-only comparison with attempt 9 found the same sealed
`original_setting_unavailable` boundary but different host-side outcomes:
attempt 9 timed out, while attempt 10 failed while receiving network data. Both
captures recorded one Wi-Fi-connected runtime, one started HTTP route shell,
continued boot-lifetime heartbeats after HTTP startup, and no panic, restart,
HTTP-server startup failure, snapshot-publication failure, or Wi-Fi disconnect
marker. Neither request produced a body.

The sealed traces therefore do not prove a deterministic supervisor, firmware
HTTP-service, readiness, or route-path defect, and they cannot distinguish
device/host transport loss from an unobserved HTTP responsiveness failure. The
single request failed closed exactly as required. No speculative retry or
software repair was made.

The protected root is sealed non-promotable and cannot be reused, retried, or
spliced. Task 2 and Phase 35 remain incomplete, and Task 3 and
`35-04-SUMMARY.md` remain unauthorized. Any later work requires a separately
authorized fresh continuation and should first own a redacted, pre-mutation HTTP
transport/readiness diagnostic that can distinguish connection, request,
response, and valid-body boundaries without weakening the exact original-setting
readback, restoration, cleanup, redaction, or admission gates.

## Continuation Attempt 11 Checkpoint

The eleventh fresh continuation ran only after all nine ordered software gates
passed against clean exact source `812aba9a429cccdbe252245dc593cb7c419d7b39`.
Before launch, the caller created one ignored mode-`0700` private parent, left
the supervisor-owned child nonexistent, and created mode-`0600` sibling wrapper
logs. The full Phase 35 command then ran exactly once with its internal detector,
the literal opaque credential-path argument, a 360-second capture budget, and
commit-redacted flash evidence.

The supervisor created the child, passed exact-package Gate 1, made its sole
detector invocation, selected exactly one board-205 candidate, completed
board-info, passed the post-detector opaque-input gate, and completed the direct
flash/monitor command. The strict Boot A classifier then stopped with typed
category `runtime_origin_corrupt`. The supervisor never derived or validated an
HTTP target, never invoked the instrumented original-settings GET, and never
started PATCH or any settings mutation. Cleanup completed with no secondary
failure, and the protected root was sealed non-promotable and non-reusable.

| Field | Recorded value |
| --- | --- |
| Completion | `2026-07-20T15:00:02Z` |
| Attempt ordinal | `11` |
| Source commit | `812aba9a429cccdbe252245dc593cb7c419d7b39` |
| Reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| Board category | `205` |
| Full command invocations | `1` |
| Supervisor event span seconds | `471` |
| Exact current package rebuilt and locked | `true` |
| Exact-package Gate 1 passed | `true` |
| Package capability digest | `f2cb29844bbdf23f8a01ccf6212d1f669927633a71c45d8f92c9000746467a3c` |
| Manifest digest | `9a0802ac4b9696aa4b6e30a24e80662d4904997513c9edc7251e6a4b8d263c30` |
| Executable-image digest | `8c02621411aeb9f815f5a016b5546be767fbfa4b8284a7bb4f49fc22ef749069` |
| Factory-image digest | `137f51c6025396e5718e7ccbc91e2b4dd2424242593094fda8ec65fa2f5310d8` |
| Package digest | `8a7f62463bbc8cdbf27d76eddd28ce309921e6d0ecadb88c4cf1ec89bd7ebecb` |
| Runtime-identity digest | `c22589c01a7f78ffa8d3bc2260d2f5f57e1737734693058774b72fe438f5dbac` |
| Detector invocations | `1` |
| Selected candidate count | `1` |
| Board-info verified | `true` |
| Opaque input gate passed | `true` |
| Flash evidence redaction mode | `commit_redacted` |
| Flash command completed | `true` |
| Boot A monitor capture non-empty | `true` |
| Boot A classification status | `failed` |
| Failure category | `runtime_origin_corrupt` |
| Failure boundary | `boot_a_runtime_origin_classification` |
| HTTP target derived | `false` |
| Instrumented original-settings GET reached | `false` |
| HTTP boundary projection count | `0` |
| PATCH mutation started | `false` |
| Approved reboot started | `false` |
| Restoration | `not_needed` |
| Process-tree cleanup | `true` |
| Unexpected serial-holder count | `0` |
| Protected root mode | `0700` |
| Private file modes | `0600` |
| Protected root reusable | `false` |
| Deterministic repository defect proven | `true` |
| Admission invoked | `false` |
| Evidence generation changed | `false` |
| Checklist changed | `false` |
| Task 3 authorized | `false` |
| Plan summary created | `false` |
| Retry in this continuation | `false` |

### Redaction-Boundary Diagnosis

The production adapter correctly forwarded the required commit-redaction flag.
That mode redacted the runtime origin inside the monitor evidence before the
Phase 33 Boot A classifier consumed the file. The classifier intentionally
requires an origin-only URL so it can validate the current-session origin and
privately derive the HTTP target; the already-redacted value therefore failed
closed as `runtime_origin_corrupt`.

This is a deterministic ordering defect between private classification and
shareable redaction, not HTTP evidence. A future software change must preserve a
mode-`0600` private raw classifier input while producing a distinct redacted
evidence projection, or equivalently complete private classification before
redacting the shareable copy. That boundary must be planned and regression-tested
before another separately authorized hardware attempt. No retry, diagnostic
probe, admission, checklist promotion, Task 3 audit, or
`35-04-SUMMARY.md` creation occurred in this continuation.

The attempt-11 root is sealed non-promotable and cannot be reused, retried, or
spliced. Task 2 and Phase 35 remain incomplete.

## Continuation Attempt 12 Checkpoint

The twelfth fresh continuation ran only after all nine ordered software gates
passed against clean exact source
`7fcad7090b94c04bee40a13552c857a3ac3ad2f1`. The caller created one fresh
mode-`0700` private parent, left the supervisor-owned child nonexistent through
the immediate pre-launch assertion, and created distinct mode-`0600` sibling
wrapper logs. The full Phase 35 command then ran exactly once with its internal
detector, literal opaque credential-path argument, dual private-first evidence,
a 360-second capture budget, and a caller budget of at least 420 seconds.

Gate 1 revalidated the exact-current package, the sole detector gate admitted
one board-205 candidate with successful board-info, and the post-detector opaque
input gate passed. Direct flash/monitor completed. The immutable private Boot A
classifier input retained its recorded digest through classification and the
digest-bound finalizer created the separate admitted flash projection only after
Boot A classification succeeded.

The instrumented original-settings GET then reached its private HTTP diagnostic
classification boundary. That boundary failed closed with primary typed category
`http_diagnostic_invalid`. The supervisor preserved that earliest category
through finalization, stopped before PATCH or any settings mutation, recorded no
restoration or cleanup secondary failure, confirmed cleanup exactly once, and
sealed the fresh root non-promotable and non-reusable.

| Field | Recorded value |
| --- | --- |
| Completion | `2026-07-20T22:30:14Z` |
| Attempt ordinal | `12` |
| Source commit | `7fcad7090b94c04bee40a13552c857a3ac3ad2f1` |
| Reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| Board category | `205` |
| Full command invocations | `1` |
| Ordered software gates passed | `true` |
| Exact current package rebuilt and locked | `true` |
| Exact-package Gate 1 passed | `true` |
| Package capability digest | `2339efb0238f696abfffb4afcb9ffdb0452b65fe15fe02c3e7d8faee64b6be3d` |
| Manifest digest | `41bc1f095039b1646f62eb4b1fb55f6c97dc4e30b0e22929c1dee3c03e9a224e` |
| Executable-image digest | `3dd47c456cd2f4c9614415942c919f911be94141d46e48bd478588202da9dc3a` |
| Factory-image digest | `e7261aa4b7f6481480c1a733908e1bf888085f36190efc64b01e6b3c5ab300dd` |
| Package digest | `897bdfc56745439054ea0fd033bd7ef5a2d214eb4d57b86256c1012e7dd5414f` |
| Runtime-identity digest | `00a6fa9b8706ea0c4ffac810139baf987d12114af4f761d3efccf15bf804e666` |
| Detector invocations | `1` |
| Selected candidate count | `1` |
| Board-info verified | `true` |
| Opaque input gate passed | `true` |
| Evidence mode | `dual_private_first` |
| Flash command completed | `true` |
| Boot A classification status | `passed` |
| Private classifier digest stable | `true` |
| Admitted flash projection created after classification | `true` |
| Instrumented original-settings GET reached | `true` |
| HTTP diagnostic classification status | `failed` |
| Failure category | `http_diagnostic_invalid` |
| Failure boundary | `original_settings_http_diagnostic_classification` |
| PATCH mutation started | `false` |
| Approved reboot started | `false` |
| Restoration | `not_needed` |
| Restoration secondary category | `none` |
| Process-tree cleanup | `true` |
| Cleanup secondary category | `none` |
| Unexpected serial-holder count | `0` |
| Remaining Phase 35 process count | `0` |
| Protected root mode | `0700` |
| Private file modes | `0600` |
| Protected root reusable | `false` |
| Admission invoked | `false` |
| Evidence generation changed | `false` |
| Checklist changed | `false` |
| Task 3 authorized | `false` |
| Plan summary created | `false` |
| Retry in this continuation | `false` |

The attempt-12 root is sealed non-promotable and cannot be reused, retried, or
spliced. Attempts beyond 12 are not authorized. Task 2 and Phase 35 remain
incomplete; Task 3, checklist promotion, evidence admission, and
`35-04-SUMMARY.md` remain prohibited from this outcome.

## Attempt 13 Exact-Head Software Gate

The progress-gated hardware policy and Phase 35 attempt authority were committed
before hardware preparation. At clean exact source
`c93c34cc2d0956a6df3a598a1654f29b689e141d`, the policy contract, Phase 35
HTTP/correlated/promotion contracts, Phase 30 non-promotion contract, reference,
parity, lifecycle, redaction, diff, and mandatory Rust gates passed. The
repo-owned preflight-only command then rebuilt and admitted the exact package in
software, reported `status=preflight_passed`, and proved current-HEAD equality.

| Field | Recorded value |
| --- | --- |
| Completion | `2026-07-21T05:20:00Z` |
| Next authorized attempt ordinal | `13` |
| Attempt 13 command invoked | `false` |
| Source commit | `c93c34cc2d0956a6df3a598a1654f29b689e141d` |
| Reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| Board category | `205` |
| Ordered software gates passed | `true` |
| Hardware policy contract passed | `true` |
| Exact package capability digest | `de19eee758a1eaa08a369bb7d1dea8e43a6be73cf595e8aea0626e786e999e24` |
| Preflight status | `preflight_passed` |
| Current HEAD equal | `true` |
| Preflight effects permitted | `false` |
| Detector invoked | `false` |
| Credential accessed | `false` |
| Device or network request made | `false` |
| Evidence admitted or promoted | `false` |

This software checkpoint does not reuse or alter attempts 1 through 12 and does
not itself consume attempt ordinal 13. The next action is to commit this redacted
checkpoint, rerun preflight at that resulting exact head, and then invoke the
full attempt-13 command exactly once with a fresh protected root.

## Attempt 13 Stop Checkpoint

Attempt 13 ran the full Phase 35 command exactly once from clean exact source
`02f128db56b332e50e11f57935f29e22e3830f66`. The exact-current-package gate,
sole internal detector, opaque credential gate, dual private-first flash and
Boot A classification all passed. The instrumented original-settings read then
failed closed with primary category `http_diagnostic_invalid` before PATCH,
mutation, or reboot.

The same primary category recurred after the targeted sub-millisecond timing
fix from `58b7e33a`. The canonical progress decision is therefore
`stop_repeated_boundary`, not another fresh ordinal. The private HTTP projection
remained category-safe, restoration was not needed, cleanup completed, and the
fresh root was sealed non-promotable and non-reusable.

| Field | Recorded value |
| --- | --- |
| Completion | `2026-07-21T05:34:36Z` |
| Attempt ordinal | `13` |
| Source commit | `02f128db56b332e50e11f57935f29e22e3830f66` |
| Reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| Board category | `205` |
| Full command invocations | `1` |
| Exact package capability digest | `a999925e633af27ac8777dea8fab202caecdec4b3c6d0b86c109253d988826e2` |
| Package digest | `c07c0a6110895519b76f5d2442d5af8998d93a7820149bf64edc1f4e5c898fe5` |
| Current HEAD verified | `true` |
| Detector invocations | `1` |
| Selected candidate count | `1` |
| Board-info verified | `true` |
| Opaque input gate passed | `true` |
| Evidence mode | `dual_private_first` |
| Private classifier input present | `true` |
| Commit-redacted derivative present | `true` |
| Boot A classification status | `passed` |
| Boot A ordinal | `10967` |
| Instrumented original-settings GET reached | `true` |
| HTTP diagnostic classification status | `failed` |
| Failure category | `http_diagnostic_invalid` |
| Failure boundary | `original_settings_http_diagnostic_classification` |
| Same category after targeted fix | `true` |
| Progress decision | `stop_repeated_boundary` |
| PATCH mutation started | `false` |
| Approved reboot started | `false` |
| Restoration | `not_needed` |
| Restoration secondary category | `none` |
| Process-tree cleanup | `true` |
| Cleanup secondary category | `none` |
| Protected parent mode | `0700` |
| Supervisor root mode | `0700` |
| Private file modes | `0600` |
| Protected root reusable | `false` |
| Admission invoked | `false` |
| Evidence generation changed | `false` |
| Checklist changed | `false` |
| Task 3 authorized | `false` |
| Plan summary created | `false` |
| Later ordinal authorized | `false` |

Attempt 13 is immutable non-promotion history. The progress-gated loop stops at
this repeated post-fix boundary. Phase 35 Task 2 remains incomplete; Task 3,
evidence admission, checklist promotion, and `35-04-SUMMARY.md` remain blocked.

## Attempt 13 Diagnostic Repair and Attempt 14 Authority

The sealed attempt-13 HTTP shape was replayed through the built adapter and
runfiles classifier without hardware, network, credentials, or raw protected
output. The replay deterministically reproduced the all-zero
`http_diagnostic_invalid` fallback twice. It then proved two independent adapter
defects: curl's case-insensitive scheme token was not canonicalized, and a
configured 10-second request deadline was also enforced as an exact maximum
observed duration even though the real timeout observation finished within five
milliseconds after that deadline.

Commit `53d8bcee` canonicalizes scheme case and separates the configured
10-second request timeout from a bounded 11-second observation ceiling in the
shell and Rust classifier. The unchanged sealed shape now reaches the precise
earliest category `request_transmission_incomplete`; observations above the
11-second ceiling remain invalid. The original root and its conclusion remain
immutable.

| Field | Recorded value |
| --- | --- |
| Completion | `2026-07-21T21:40:53Z` |
| Repair commit | `53d8bcee` |
| Hardware command invoked | `false` |
| Detector invoked | `false` |
| Credential accessed | `false` |
| Device or network request made | `false` |
| Sealed root changed | `false` |
| Pre-fix replay category | `http_diagnostic_invalid` |
| Post-fix replay category | `request_transmission_incomplete` |
| Scheme case normalized | `true` |
| Configured request timeout seconds | `10` |
| Maximum observed-duration seconds | `11` |
| Above-bound rejection retained | `true` |
| Focused Rust tests passed | `13` |
| Bazel adapter/runfiles tests passed | `true` |
| Redaction verification passed | `true` |
| Progress decision | `continue_after_verified_fix` |
| Next authorized attempt ordinal | `14` |

The user's 2026-07-21 standing authority permits attempt 14 and later fresh
ordinals after distinct verified fixes or confirmed non-invasive remediation,
without a fixed total cap. Every attempt still requires a new ordinal, fresh
protected root, exact committed package, complete software/preflight gate, and
one full hardware invocation. Blind retries remain prohibited.

## Attempt 14 Checkpoint

Attempt 14 ran the full Phase 35 command exactly once from clean exact source
`8afbed3248fb00e02d1a09f726b48ec241b552da`. Exact-package admission, the sole
internal detector, opaque input gate, dual private-first flash, and Boot A
classification passed. The instrumented original-settings read then produced
the newly discriminating category `request_transmission_incomplete`: TCP
connected, but zero request bytes were recorded before curl exited with its
typed transport status. The command stopped before PATCH, mutation, or reboot.

| Field | Recorded value |
| --- | --- |
| Completion | `2026-07-21T21:54:06Z` |
| Attempt ordinal | `14` |
| Source commit | `8afbed3248fb00e02d1a09f726b48ec241b552da` |
| Reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| Board category | `205` |
| Full command invocations | `1` |
| Exact package capability digest | `4d48514480b7b1d325dbd88e16ad86ce3a2917b20291b8f39815a29e16e0b116` |
| Package digest | `a7fcab90daa1036148b84236d754feef11e9e60e54353c94f20f65137449a54c` |
| Current HEAD verified | `true` |
| Detector invocations | `1` |
| Selected candidate count | `1` |
| Board-info verified | `true` |
| Opaque input gate passed | `true` |
| Evidence mode | `dual_private_first` |
| Private classifier input present | `true` |
| Commit-redacted derivative present | `true` |
| Boot A classification status | `passed` |
| Boot A ordinal | `10970` |
| Instrumented original-settings GET reached | `true` |
| HTTP terminal category | `request_transmission_incomplete` |
| TCP connected | `true` |
| Curl exit code | `56` |
| Request bytes | `0` |
| TCP connect duration milliseconds | `261` |
| Total duration milliseconds | `6539` |
| Response status received | `false` |
| Response headers received | `false` |
| Response body received | `false` |
| PATCH mutation started | `false` |
| Approved reboot started | `false` |
| Restoration | `not_needed` |
| Restoration secondary category | `none` |
| Process-tree cleanup | `true` |
| Cleanup secondary category | `none` |
| Protected root reusable | `false` |
| Admission invoked | `false` |
| Checklist changed | `false` |
| Task 3 authorized | `false` |
| Plan summary created | `false` |

The attempt-14 root is sealed non-promotable and non-reusable. The new typed
boundary authorizes software diagnosis, not an unchanged retry. Attempt 15
requires a deterministic reproduction, regression-backed fix, clean commit, and
fresh exact-current-HEAD gate.

## Attempt 14 Diagnostic Repair and Attempt 15 Authority

The attempt-14 category was reproduced through the built adapter/classifier
seam. An isolated local loopback peer observed a complete 93-byte bodyless GET
before forcing curl's typed receive-side failure; curl retained a raw request
byte count of zero. This demonstrates that the counter is not authoritative
negative proof once curl has entered its receive-failure boundary.

Commit `0dd2134e` preserves the raw zero counter and derives request
transmission completion from either a positive count or the closed receive-error
category. The send-failure category remains `request_transmission_incomplete`.
A receive failure without response facts advances to `response_status_missing`,
and a receive failure after a partial response advances to
`response_body_incomplete_or_over_limit`.

| Field | Recorded value |
| --- | --- |
| Repair completion | `2026-07-21T22:21:54Z` |
| Hardware invoked during diagnosis | `false` |
| Device request issued during diagnosis | `false` |
| Credential access during diagnosis | `false` |
| Deterministic adapter regression | `passed` |
| Isolated loopback request bytes observed | `93` |
| Raw curl request-byte metric preserved | `0` |
| Curl receive category | `56` |
| Curl send category guard retained | `55` |
| Pre-fix sealed replay category | `request_transmission_incomplete` |
| Post-fix sealed replay category | `response_status_missing` |
| Sealed input digests unchanged | `true` |
| Repair commit | `0dd2134e` |
| Progress decision | `continue_after_verified_fix` |
| Next authorized attempt ordinal | `15` |

Attempt 15 requires a clean committed head, complete exact-current-HEAD
software gate, passing Phase 35 preflight, fresh protected parent, nonexistent
supervisor child, mode-0600 sibling output, and exactly one full invocation.
The standing authority has no fixed ordinal cap, but every later attempt must
follow a distinct verified fix or confirmed non-invasive remediation.

## Attempt 15 Checkpoint

Attempt 15 ran the full Phase 35 command exactly once from clean exact source
`1c4979f67c0b12daee356ae5df1c1c5468ba1013` after the complete software gate and
preflight passed. The command stopped before mutation or reboot with the same
primary category as attempt 14, but the redacted transport facts are distinct:
curl reached TCP connection and then its configured timeout boundary rather
than returning the previously repaired receive-error category.

| Field | Recorded value |
| --- | --- |
| Completion | `2026-07-21T22:38:30Z` |
| Attempt ordinal | `15` |
| Source commit | `1c4979f67c0b12daee356ae5df1c1c5468ba1013` |
| Board category | `205` |
| Full command invocations | `1` |
| Exact package capability digest | `9e3376818f4e1a302e8c5c057fadcb9b87fe53d3dfd1407700938e6f7f255650` |
| Current HEAD verified | `true` |
| Protected parent mode | `0700` |
| Wrapper output mode | `0600` |
| HTTP terminal category | `request_transmission_incomplete` |
| TCP connected | `true` |
| Curl exit category | `28` |
| Request bytes | `0` |
| TCP connect duration milliseconds | `434` |
| Total duration milliseconds | `10005` |
| Response status received | `false` |
| Response headers received | `false` |
| Response body received | `false` |
| PATCH mutation started | `false` |
| Approved reboot started | `false` |
| Restoration secondary category | `none` |
| Cleanup secondary category | `none` |
| Protected root reusable | `false` |
| Admission invoked | `false` |
| Checklist changed | `false` |
| Task 3 authorized | `false` |
| Plan summary created | `false` |
| Progress decision | `stop_repeated_boundary` |

The attempt-15 root is sealed non-promotable and non-reusable. The user's
standing authority removes a fixed ordinal cap only for qualifying progress;
it does not erase the repository's repeated-primary-category stop. Software
diagnosis of the timeout semantics remains authorized. No attempt 16 is
authorized unless a later explicit policy decision validly reopens the loop
after a distinct regression-backed fix.

## Attempt 15 Diagnostic Repair and Attempt 16 Authority

Deterministic local peers proved that the host curl build reports a zero request
size after receiving confirmation that the complete bodyless GET arrived,
including on a successful response. Attempt 15 therefore did not prove an
incomplete send; its coarse terminal category concealed a distinct response-
timeout transport signature.

Commit `d097bbbf` replaces the production curl request with a schema-v2
repo-owned Rust probe. It records a positive request-send completion timestamp
only after every request byte is accepted and the transport flush succeeds.
Partial bytes remain bounded diagnostic facts, TLS uses certificate and hostname
verification, and raw request material never reaches disk or terminal output.

| Field | Recorded value |
| --- | --- |
| Repair completion | `2026-07-21T23:20:46Z` |
| Hardware invoked during diagnosis or repair | `false` |
| Device request issued during diagnosis or repair | `false` |
| Credential access during diagnosis or repair | `false` |
| HTTP boundary schema | `phase35-http-boundary-v2` |
| Positive send boundary | `full_write_and_transport_flush` |
| Valid-response real adapter regression | `passed` |
| Silent-response real adapter regression | `response_status_missing` |
| Short-write regression | `request_transmission_incomplete` |
| TLS failure regression | `tls_handshake_failure` |
| Correlated supervisor regression | `passed` |
| Direct and Bazel/runfiles coverage | `passed` |
| Raw request persisted | `false` |
| Repair commit | `d097bbbf` |
| Progress decision | `continue_after_verified_fix` |
| Next authorized attempt ordinal | `16` |

The clarified hardware policy compares a phase-declared redacted authoritative
boundary signature: the terminal category plus its minimum closed discriminator
fields. It still stops the same post-fix signature and explicitly forbids
renaming unchanged conditions. The user's latest 2026-07-21 post-fix authority
permits attempt 16 only after the clean exact-current-HEAD software gate and
preflight. The fresh-root, one-invocation, safety, privacy, restoration, cleanup,
and no-promotion rules remain unchanged.

## Attempt 16 Checkpoint

Attempt 16 ran the full Phase 35 command exactly once from clean exact source
`823309599209cde451435c85bb882fe8a456f80d` after the complete software gate and
exact-head preflight passed. It stopped before PATCH or mutation with the new
authoritative boundary signature `response_status_missing` plus transport
outcome `response_timeout`: TCP connected, the repository-owned probe completed
and flushed the full request, and no response status, headers, or body arrived
before the bounded deadline.

| Field | Recorded value |
| --- | --- |
| Attempt ordinal | `16` |
| Invocation count for root | `1` |
| Source commit | `823309599209cde451435c85bb882fe8a456f80d` |
| Software gate | `passed` |
| Exact-head preflight | `passed` |
| Supervisor result | `non_promotion` |
| HTTP boundary schema | `phase35-http-boundary-v2` |
| HTTP terminal category | `response_status_missing` |
| Transport outcome | `response_timeout` |
| TCP connected | `true` |
| Request transmission complete | `true` |
| Response status received | `false` |
| Response headers received | `false` |
| Response body received | `false` |
| Mutation started | `false` |
| Root reusable | `false` |
| Progress decision | `diagnose_new_boundary` |

The attempt-16 root is sealed non-promotable and non-reusable. This signature
is distinct from the post-fix request-boundary signatures in attempts 14 and
15, so the progress-gated policy authorizes software diagnosis but not an
unchanged attempt 17. A later attempt requires a deterministic reproduction,
regression-backed verified fix, clean committed head, and passing preflight, or
a confirmed permitted non-invasive remediation. Task 3, evidence admission,
checklist promotion, and `35-04-SUMMARY.md` remain blocked.

## Attempt 16 Diagnostic Repair and Attempt 17 Authority

Exact release-ELF analysis isolated attempt 16's response timeout to the
firmware request task. The Phase 34 ordered snapshot publisher's system-info
instantiation reserves 6,080 bytes and candidate collection reserves 1,456
bytes before the HTTP framework, retained-record construction, and JSON writer.
The server task still had the historical 8 KiB allocation. This leaves no safe
request-execution margin and matches the connection-specific timeout while the
listener and firmware remain alive.

The repair raises only the ESP-IDF HTTP server task to an explicit 16 KiB and
adds a source regression that rejects the previous literal 8 KiB configuration.
It does not weaken Phase 34's completion-ordered retention and external issuance
contract or change any Phase 35 classifier, mutation, restoration, cleanup,
redaction, or admission rule.

| Field | Recorded value |
| --- | --- |
| Repair completion | `2026-07-21T23:47:16Z` |
| Hardware invoked during diagnosis or repair | `false` |
| Device request issued during diagnosis or repair | `false` |
| Credential access during diagnosis or repair | `false` |
| Pre-fix HTTP server task stack | `8192` |
| Post-fix HTTP server task stack | `16384` |
| Ordered publisher frame observed | `6080` |
| Candidate collector frame observed | `1456` |
| Phase 34 publication ordering changed | `false` |
| Source regression | `passed` |
| Affected uncached Bazel suites | `passed` |
| Canonical firmware build | `passed` |
| Progress decision | `continue_after_verified_fix` |
| Next authorized attempt ordinal | `17` |

The user's standing authority and this distinct regression-backed fix select
`continue_after_verified_fix`. Fresh attempt 17 is authorized only after the
complete clean exact-current-HEAD software gate and preflight pass. It must use
a fresh protected parent, nonexistent supervisor child, mode-0600 sibling
output, and exactly one full command invocation. An unchanged retry or reuse of
attempt 16 remains prohibited.

## Attempt 17 Checkpoint

Attempt 17 ran the full Phase 35 command exactly once from clean exact source
`98463e8a735233b4e283b6535d3c9f375a984523` after the complete software gate
and exact-head preflight passed. The 16 KiB HTTP task repair succeeded at the
hardware boundary: the schema-v2 original-settings read connected, completed
request transmission, and received status, headers, and body in 396
milliseconds. The supervisor then stopped before PATCH or mutation with the new
category `pre_patch_mismatch`.

| Field | Recorded value |
| --- | --- |
| Attempt ordinal | `17` |
| Invocation count for root | `1` |
| Source commit | `98463e8a735233b4e283b6535d3c9f375a984523` |
| Software gate | `passed` |
| Exact-head preflight | `passed` |
| Supervisor result | `non_promotion` |
| HTTP terminal category | `ready` |
| Transport outcome | `complete` |
| Request transmission complete | `true` |
| Response status received | `true` |
| Response headers received | `true` |
| Response body received | `true` |
| HTTP total duration milliseconds | `396` |
| Primary category | `pre_patch_mismatch` |
| Mutation started | `false` |
| Restoration secondary | `none` |
| Cleanup secondary | `none` |
| Root reusable | `false` |
| Progress decision | `diagnose_new_boundary` |

The outer zsh-only status summary used a reserved read-only variable after the
repo command returned. That reporting error did not invoke the command again or
change the supervisor's authoritative seal. The attempt-17 root remains sealed
non-promotable and non-reusable. The new pre-mutation mismatch authorizes
software diagnosis, but not an unchanged attempt 18. Task 3, admission,
checklist promotion, and `35-04-SUMMARY.md` remain blocked.

## Attempt 17 Diagnostic Repair and Attempt 18 Authority

Protected structural inspection proved that the failure was in the evidence
adapter, not the ready HTTP response or the device setting. Fixture mode emitted
a complete epoch, while production omitted the setting digest, expected a boot
ordinal the API does not expose, stored the WebSocket envelope under an
incorrect identity, fabricated a retained marker and interval, and treated the
later WebSocket revision as a mismatch.

The production adapter now binds boot identity to the serial classifier, hashes
the validated private hostname, stores the actual API document and WebSocket
data object, downloads the actual retained log, requires both exact retained
markers, and records real monotonic bounds. The pure validator requires one
session with a WebSocket revision strictly later than the API storage revision.
The reboot adapter also proves service loss before its Boot B trace boundary and
forwards the baseline identity to the post-restart classifier. All raw command
errors and operational artifacts remain mode-0600 below the protected root.

| Field | Recorded value |
| --- | --- |
| Repair completion | `2026-07-22T00:22:22Z` |
| Hardware invoked during diagnosis or repair | `false` |
| Device request issued during diagnosis or repair | `false` |
| Credential access during diagnosis or repair | `false` |
| Serial classifier owns boot ordinal | `true` |
| Actual retained-log download required | `true` |
| Same-session later WebSocket revision required | `true` |
| Private setting represented only by digest | `true` |
| Post-loss Boot B trace boundary required | `true` |
| Hermetic production adapter regression | `passed` |
| Full software gate | `passed` |
| Progress decision | `continue_after_verified_fix` |
| Next authorized attempt ordinal | `18` |

Fresh attempt 18 is authorized only after the full clean exact-current-HEAD
software gate and preflight pass. It must use a fresh protected parent,
nonexistent supervisor child, mode-0600 sibling output, and exactly one full
command invocation. Attempt 17 remains immutable, non-promotable,
non-reusable, and ineligible for evidence splicing.

## Attempt 18 Checkpoint

Attempt 18 ran the full Phase 35 command exactly once from clean exact source
`065240279c4657945ffce70d2baa501b4da7ceae` after the complete software gate
and exact-head preflight passed. Boot A pre-capture and PATCH completed. The
post-PATCH API and WebSocket artifacts were coherent, then the actual
retained-log response failed at a malformed chunk-framing boundary. The
supervisor preserved `boot_a_capture_failed`, confirmed restoration and
cleanup, and sealed the root non-promotable and non-reusable.

| Field | Recorded value |
| --- | --- |
| Attempt ordinal | `18` |
| Invocation count for root | `1` |
| Source commit | `065240279c4657945ffce70d2baa501b4da7ceae` |
| Software gate | `passed` |
| Exact-head preflight | `passed` |
| Supervisor result | `non_promotion` |
| Boot A pre-capture | `passed` |
| PATCH response | `passed` |
| Post-PATCH API capture | `passed` |
| Post-PATCH WebSocket capture | `passed` |
| Retained-log transport | `malformed_chunk_framing` |
| Primary category | `boot_a_capture_failed` |
| Mutation started | `true` |
| Restoration secondary | `none` |
| Cleanup secondary | `none` |
| Root reusable | `false` |
| Progress decision | `diagnose_new_boundary` |

Attempts 1 through 18 remain immutable, non-promotable, non-reusable, and
ineligible for evidence splicing. Task 3, admission, checklist promotion, and
`35-04-SUMMARY.md` remain blocked.

## Attempt 18 Diagnostic Repair and Attempt 19 Authority

Protected structural inspection isolated the failure to the handoff between a
successful WebSocket capture and the following retained-log GET. The helper
called close after its terminal frame but resolved before receiving the close
event. The supervisor therefore had no proof that the upgraded connection had
completed its lifecycle before opening the next HTTP request.

The helper now waits for the close event under a strict timeout and emits an
exact closed marker only after that boundary. Phase 35 requires the marker
before retained-log capture. A real loopback peer deliberately delays its close
response and proves the helper cannot return early; the existing Phase 17 and
Phase 35 suites preserve compatibility and request ordering.

| Field | Recorded value |
| --- | --- |
| Repair completion | `2026-07-22T00:49:49Z` |
| Hardware invoked during diagnosis or repair | `false` |
| Device request issued during diagnosis or repair | `false` |
| Credential access during diagnosis or repair | `false` |
| WebSocket peer-close proof required | `true` |
| Close-handshake timeout bounded | `true` |
| Following HTTP request gated by close marker | `true` |
| Delayed-close real-process regression | `passed` |
| Focused Phase 17 and Phase 35 suites | `passed` |
| Full software gate | `passed` |
| Progress decision | `continue_after_verified_fix` |
| Next authorized attempt ordinal | `19` |

Fresh attempt 19 is authorized only after the full clean exact-current-HEAD
software gate and preflight pass. It must use a fresh protected parent,
nonexistent supervisor child, mode-0600 sibling output, and exactly one full
command invocation. Attempt 18 remains immutable, non-promotable,
non-reusable, and ineligible for evidence splicing.

## Attempt 19 Checkpoint and Manual-Remediation Gate

Attempt 19 ran the full Phase 35 command exactly once from clean exact source
`6a88300f84d0db1907455974372fe0468f4957e3` after the complete software gate
and exact-head preflight passed. The internal detector admitted one board-205
target. The flash process then failed to establish its target connection before
Boot A capture or mutation. Cleanup passed, and the root is sealed
non-promotable and non-reusable.

| Field | Recorded value |
| --- | --- |
| Attempt ordinal | `19` |
| Invocation count for root | `1` |
| Source commit | `6a88300f84d0db1907455974372fe0468f4957e3` |
| Software gate | `passed` |
| Exact-head preflight | `passed` |
| Detector admission | `passed` |
| Supervisor result | `non_promotion` |
| Coarse category | `flash_or_boot_a_failed` |
| Authoritative discriminator | `target_connection_failed` |
| Boot A capture started | `false` |
| Mutation started | `false` |
| Restoration | `not_needed` |
| Cleanup secondary | `none` |
| Root reusable | `false` |
| Progress decision | `continue_after_manual_remediation` |

Attempts 1 through 19 remain immutable, non-promotable, non-reusable, and
ineligible for evidence splicing. One exact non-invasive remediation is now
required: disconnect USB and barrel power, reconnect barrel power, reconnect
USB, and confirm completion after the target has had time to re-enumerate. The
agent must wait for that confirmation. Fresh attempt 20 then requires another
clean exact-current-HEAD preflight, fresh protected parent, nonexistent
supervisor child, mode-0600 sibling output, and one invocation. If the same
`flash_or_boot_a_failed` plus `target_connection_failed` signature recurs after
that remediation, policy selects `stop_hardware_blocker`. Task 3, admission,
checklist promotion, and `35-04-SUMMARY.md` remain blocked.

## Attempt 20 Checkpoint and Hardware-Blocker Stop

The user confirmed completion of the exact USB and barrel-power remediation
required after attempt 19. Attempt 20 then ran the full Phase 35 command exactly
once from clean exact source
`b06bf416cf65283c53aa0f69c15ed216a9858eaa` after a fresh exact-head
preflight passed. The internal detector again admitted one board-205 target.
The flash process then reproduced the same target-connection failure before Boot
A capture or mutation. Cleanup passed, and the root is sealed non-promotable
and non-reusable.

| Field | Recorded value |
| --- | --- |
| Attempt ordinal | `20` |
| Invocation count for root | `1` |
| Source commit | `b06bf416cf65283c53aa0f69c15ed216a9858eaa` |
| Manual remediation confirmed | `true` |
| Exact-head preflight | `passed` |
| Detector admission | `passed` |
| Supervisor result | `non_promotion` |
| Coarse category | `flash_or_boot_a_failed` |
| Authoritative discriminator | `target_connection_failed` |
| Same as pre-remediation signature | `true` |
| Boot A capture started | `false` |
| Mutation started | `false` |
| Restoration | `not_needed` |
| Cleanup secondary | `none` |
| Root reusable | `false` |
| Progress decision | `stop_hardware_blocker` |

The same authoritative signature recurred after its one applicable remediation.
The progress-gated policy therefore prohibits attempt 21 and any unchanged
retry. Direct UART, pins, probes, and other stronger electrical interfaces
remain outside current authority. Attempts 1 through 20 remain immutable,
non-promotable, non-reusable, and ineligible for evidence splicing. Task 2,
Task 3, Phase 35 admission, checklist promotion, and `35-04-SUMMARY.md` remain
incomplete and blocked.

## Attempts 19–20 Offline Boundary Classification and Attempt 21 Gate

The sealed roots and their historical fields remain unchanged. Applying the
new closed classifier to the already-recorded safe counters classifies both
attempts at `stage=factory` with device information complete, transfer not
started, and `terminal_boundary=post_info_pre_transfer_failed`. This is an
offline interpretation of existing shareable facts, not new evidence and not a
promotion claim.

The earlier `stop_hardware_blocker` remains authoritative for an unchanged
retry. The user separately authorized a materially different repair that pins
espflash 4.5.0, supplies explicit native-USB reset controls, captures private
typed stage evidence, and requires stable three-sample USB readiness around
reset-capable stages. Fresh attempt 21 is gated on the complete clean software
suite and exact-current-HEAD preflight. Its first hardware boundary is one
read-only checksum of exactly 4 KiB at address `0x0` inside the same phase-owned
invocation and root. Probe failure stops before credential access or writes;
probe success permits the existing factory, NVS, and monitor sequence.

Attempts 1 through 20 remain non-promotable, non-reusable, and ineligible for
splicing. If attempt 21 reproduces
`flash_or_boot_a_failed/factory/post_info_pre_transfer_failed`, policy selects
`stop_repeated_boundary`. Task 3, admission, checklist promotion, and
`35-04-SUMMARY.md` remain blocked until `complete`.

## Attempt 21 Checkpoint and Pre-Probe Connection Stop

Attempt 21 executed once after the complete software gate, the exact espflash
4.5.0 doctor check, and exact-current-HEAD preflight. The protected parent,
nonexistent supervisor child, sibling wrapper log, and resulting sealed child
met the required `0700`/`0600` ownership contract.

| Field | Redacted value |
| --- | --- |
| Attempt ordinal | `21` |
| Source commit | `e007c06a5350b197a7f2a1af1bb6a41472be651d` |
| espflash version | `4.5.0` |
| Software gate | `passed` |
| Exact-head preflight | `passed` |
| Exact-head equality | `true` |
| Supervisor result | `non_promotion` |
| Primary category | `connection_failure` |
| Typed flash stage | `none` |
| Typed flash boundary | `none` |
| Checksum probe started | `false` |
| Credential access | `false` |
| Factory/NVS write started | `false` |
| Restoration secondary | `none` |
| Cleanup secondary | `none` |
| Root reusable | `false` |
| Progress decision | `continue_after_manual_remediation` |

The sole detector invocation reached its reset-capable board-info command and
failed to connect before the new checksum probe. This differs from Attempts 19
and 20, which completed device information in the factory stage. Review of the
official espflash source history shows that 4.5.0 contains a reset-order change
made for Windows compatibility, while its review explicitly notes the absence
of USB-JTAG-Serial validation. That is a strong compatibility hypothesis, not a
hardware conclusion.

One exact non-invasive remediation is required before a discriminating fresh
Attempt 22: disconnect USB and barrel power, reconnect barrel power, reconnect
USB, allow the target to re-enumerate, and confirm completion. The agent must
wait for confirmation, rerun exact-head preflight, and use a fresh protected
root. If `connection_failure` recurs at the detector before the probe, policy
selects `stop_hardware_blocker`. Attempts 1 through 21 remain immutable,
non-promotable, non-reusable, and ineligible for evidence splicing. Task 3,
admission, checklist promotion, and `35-04-SUMMARY.md` remain blocked.

## Attempt 22 Checkpoint and Late-Monitor Trust Diagnosis

After the user confirmed the exact non-invasive USB and barrel-power
remediation, Attempt 22 executed once after another exact-current-HEAD
preflight. The protected parent, nonexistent supervisor child, sibling wrapper
log, and resulting sealed child met the required `0700`/`0600` ownership
contract.

| Field | Redacted value |
| --- | --- |
| Attempt ordinal | `22` |
| Source commit | `55a8f31ac9be6a2c056cd04f8cc226b923782b22` |
| espflash version | `4.5.0` |
| Exact-head preflight | `passed` |
| Exact-head equality | `true` |
| Supervisor result | `non_promotion` |
| Primary category | `flash_or_boot_a_failed` |
| Typed flash stage | `monitor` |
| Typed flash boundary | `ready` |
| Checksum probe | `ready` |
| Factory stage | `ready` |
| NVS stage | `ready` |
| Monitor stage | `ready` |
| Capture status | `timed_out_without_trusted_output` |
| Offline Phase 33 baseline classification | `passed` |
| Offline classifier category | `none` |
| Mutation started | `false` |
| Restoration secondary | `none` |
| Cleanup secondary | `none` |
| Root reusable | `false` |
| Progress decision | `continue_after_verified_fix` |

The remediation advanced the target through every detector and flash/reset
boundary, so Attempt 21's pre-probe connection category did not recur. The
private monitor capture omitted early one-shot legacy boot markers but retained
the later replayed identity and origin evidence. The built Phase 33 classifier
accepted that immutable private input offline. No raw device, network,
credential, process, or local-path value is recorded here.

The software repair makes this ordering explicit: only a dual-mode timeout may
return `timed_out_pending_private_classification`; default evidence mode and
spawn/child failures remain terminal. No admitted derivative exists until the
supervisor verifies the private digest, the authoritative classifier passes,
and the software-only finalizer runs. Attempt 22 remains immutable,
non-promotable, non-reusable, and ineligible for splicing. Fresh Attempt 23 is
allowed only after the complete clean software gate, atomic commits, and
exact-current-HEAD preflight. Task 3, admission, checklist promotion, and
`35-04-SUMMARY.md` remain blocked until `complete`.

## Attempt 23 Checkpoint and Retained-HTTP Corruption Diagnosis

Attempt 23 executed once after the complete software gate and
exact-current-HEAD preflight. The protected parent, nonexistent supervisor
child, sibling wrapper log, and resulting sealed child met the required
`0700`/`0600` ownership contract.

| Field | Redacted value |
| --- | --- |
| Attempt ordinal | `23` |
| Source commit | `ead2347d32ed0dbb8be43c74a3fb3a85a32734a1` |
| Exact-head preflight | `passed` |
| Exact-head equality | `true` |
| Supervisor result | `non_promotion` |
| Primary category | `boot_a_pre_capture_failed` |
| Typed flash stage | `monitor` |
| Typed flash boundary | `ready` |
| Private Phase 33 baseline classification | `passed` |
| Private classifier category | `none` |
| Pre-capture API schema | `valid` |
| WebSocket observations | `1` |
| WebSocket close status | `closed` |
| Retained HTTP chunk framing | `invalid` |
| Mutation started | `false` |
| Restoration secondary | `none` |
| Cleanup secondary | `none` |
| Root reusable | `false` |
| Progress decision | `continue_after_verified_fix` |

Attempt 23 proves the Attempt 22 repair: the immutable private capture reached
the authoritative Phase 33 classifier and dual finalizer. The next failure was
a distinct retained-response boundary after a closed WebSocket exchange.
Private inspection found an invalid chunk-length signature without exposing raw
response, device, network, credential, process, or path values.

Firmware cadence tasks previously called `httpd_ws_send_frame_async` outside
the HTTPD work queue. A stale send could therefore target a numeric descriptor
after ESP-IDF had reused it for the retained HTTP request. The targeted repair
assigns every registration a generation lease with disconnect cleanup, copies
each frame and lease into owned queued work, rechecks the exact current lease
and WebSocket protocol state inside HTTPD context, and sends only after both
checks pass. Attempt 23 remains immutable, non-promotable, non-reusable, and
ineligible for splicing. Fresh Attempt 24 is allowed only after the complete
clean software gate, atomic commits, and exact-current-HEAD preflight. The same
retained-chunk signature recurring after this repair selects
`stop_repeated_boundary`. Task 3, admission, checklist promotion, and
`35-04-SUMMARY.md` remain blocked until `complete`.

## Attempt 24 Checkpoint and Probe Checksum Diagnosis

Attempt 24 executed once after doctor and exact-current-HEAD preflight passed.
The protected parent, nonexistent supervisor child, sibling wrapper log, and
resulting sealed child met the required `0700`/`0600` ownership contract.

| Field | Redacted value |
| --- | --- |
| Attempt ordinal | `24` |
| Source commit | `dec8b8a6bef8f504ec83a7eebe03b69a08be5064` |
| Doctor | `passed` |
| Exact-head preflight | `passed` |
| Exact-head equality | `true` |
| Supervisor result | `non_promotion` |
| Primary category | `flash_boundary_invalid` |
| Probe connected | `true` |
| Probe device information complete | `true` |
| Protected checksum candidate count | `1` |
| Credential access | `false` |
| Factory/NVS write started | `false` |
| Mutation started | `false` |
| Restoration secondary | `none` |
| Cleanup secondary | `none` |
| Root reusable | `false` |
| Progress decision | `continue_after_verified_fix` |

The protected output shape and installed espflash 4.5.0 source prove the child
printed one valid leading-zero-elided checksum: espflash converts MD5 to `u128`
and uses unpadded lowercase hexadecimal formatting. The fixed-width parser
therefore marked the probe incomplete. The non-ready Rust projection then used
serde's default `failure` spelling while the shell requires the canonical
`failed` category, collapsing the typed boundary into
`flash_boundary_invalid`.

The repair accepts exactly one official lowercase `0x`-prefixed checksum with
1 through 32 hexadecimal digits and rejects malformed, uppercase, embedded,
overlong, or multiple candidates. It also explicitly serializes
`post_info_pre_transfer_failed`. Offline classification of Attempt 24's
immutable inputs now emits that canonical typed boundary without altering the
sealed root. Attempt 24 remains immutable, non-promotable, non-reusable, and
ineligible for splicing. Fresh Attempt 25 is allowed only after the complete
clean software gate, atomic checkpoint commit, and exact-current-HEAD
preflight. Task 3, admission, checklist promotion, and `35-04-SUMMARY.md`
remain blocked until `complete`.

## Attempt 25 Checkpoint and Passive-Monitor Runfiles Diagnosis

Attempt 25 executed once after doctor and exact-current-HEAD preflight passed.
The protected parent, nonexistent supervisor child, sibling wrapper log, and
resulting sealed child met the required `0700`/`0600` ownership contract.

| Field | Redacted value |
| --- | --- |
| Attempt ordinal | `25` |
| Source commit | `f3a4d350492f5cc1073c0f62bd1a20f8af4355e2` |
| Doctor | `passed` |
| Exact-head preflight | `passed` |
| Exact-head equality | `true` |
| Supervisor result | `non_promotion` |
| Primary category | `approved_reboot_failed` |
| Probe stage | `ready` |
| Factory stage | `ready` |
| NVS stage | `ready` |
| Monitor stage | `ready` |
| Original HTTP category | `ready` |
| Immediate HTTP category | `ready` |
| Mutation started | `true` |
| Reboot request issued | `false` |
| Restoration HTTP category | `ready` |
| Restoration secondary | `none` |
| Cleanup secondary | `cleanup_passive_monitor_failed` |
| Root reusable | `false` |
| Progress decision | `continue_after_verified_fix` |

The run validates the Attempt 24 checksum repair and the Attempt 23 firmware
repair through the private Boot A classifier and dual finalizer. PATCH and the
storage-confirmed immediate readback then succeeded. Before issuing the
approved reboot POST, the passive-monitor child exited while sourcing its
helper closure: the built supervisor contained `phase13-monitor-capture.sh`
and `serial-session-trace.sh` but omitted the required adjacent
`process-group.sh`. The supervisor preserved `approved_reboot_failed`, restored
the original setting successfully, recorded the already-exited monitor as a
secondary cleanup outcome, and sealed the root.

The repair adds the missing helper to the production Bazel runfiles and a
built-target regression that loads the passive-monitor script from that
runfiles closure. Attempt 25 remains immutable, non-promotable, non-reusable,
and ineligible for splicing. Fresh Attempt 26 is allowed only after the complete
clean software gate, atomic checkpoint commit, and exact-current-HEAD
preflight. Task 3, admission, checklist promotion, and `35-04-SUMMARY.md`
remain blocked until `complete`.

## Attempt 26 Checkpoint and Repeated Reboot Boundary

Attempt 26 executed once after doctor and exact-current-HEAD preflight passed.
The protected parent, nonexistent supervisor child, sibling wrapper log, and
resulting sealed child met the required `0700`/`0600` ownership contract.

| Field | Redacted value |
| --- | --- |
| Attempt ordinal | `26` |
| Source commit | `a4de3c3a480bb29075c1c17df5c7cb8fe9d69f7c` |
| Doctor | `passed` |
| Exact-head preflight | `passed` |
| Exact-head equality | `true` |
| Supervisor result | `non_promotion` |
| Primary category | `approved_reboot_failed` |
| Probe stage | `ready` |
| Factory stage | `ready` |
| NVS stage | `ready` |
| Monitor stage | `ready` |
| Original HTTP category | `ready` |
| Immediate HTTP category | `ready` |
| Mutation started | `true` |
| Passive pre-readiness | `ready` |
| Passive active owner | `verified` |
| Reboot request issued | `true` |
| Service loss observed | `true` |
| Passive capture disposition | `timed_out_after_capture` |
| Passive serial bytes | `0` |
| Passive post-readiness | `ready` |
| Boot B classifier category | `post_restart_identity_missing` |
| Restoration HTTP category | `ready` |
| Restoration secondary | `none` |
| Cleanup secondary | `none` |
| Root reusable | `false` |
| Progress decision | `stop_repeated_boundary` |

The run proves the Attempt 25 runfiles repair: the passive monitor loaded its
complete helper closure, established exclusive serial ownership, issued the
approved reboot POST, observed service loss, and recovered post-cleanup
readiness. The complete bounded passive interval nevertheless captured zero
serial bytes, leaving no post-restart identity marker for Boot B. The private
classifier therefore recorded `post_restart_identity_missing` while the
supervisor preserved its existing public primary category
`approved_reboot_failed`.

Restoration and cleanup completed without secondary failures. Because the same
primary category recurred immediately after its targeted verified fix, the
repository hardware policy selects `stop_repeated_boundary`. Attempt 26 remains
immutable, non-promotable, non-reusable, and ineligible for splicing. At that
checkpoint, Attempt 27 was prohibited pending a separately authorized contract
change. Task 3, admission, checklist promotion, and `35-04-SUMMARY.md` remain
blocked until eligible evidence exists.

## Attempt 26 Offline Device-Session Diagnosis and Attempt 27 Contract

The separately authorized diagnosis does not modify Attempt 26's sealed root,
recorded categories, or non-promotion disposition. It classifies the runtime
observer itself as unqualified: the fixed-path espflash passive monitor proved
attachment ownership but did not model USB disappearance, enumeration change,
same-device reacquisition, or an independent HTTP postcondition. Existing
repository hardware evidence had already distinguished that backend from an
OS-native receive-only reader by observing application bytes only through the
latter.

The regression-backed repair is therefore a backend and evidence-contract
change, not a category rename or an unchanged retry. It introduces a built
device-session command with these required facts:

| Boundary | Required fact |
| --- | --- |
| Pre-effect observer | OS-native receive-only application bytes are delivered before restart |
| Application effect | Exactly one restart POST is fully transmitted; an ambiguous response is never resent |
| Device continuity | A stable node or one unique reacquired node has the original physical identity |
| Recovery target | Only the previously trusted origin is polled; discovery and scans remain prohibited |
| Boot transition | HTTP reports a changed boot session and exact RTC ordinal `N → N+1` |
| Reset attribution | HTTP reports the closed `software_cpu` category |
| Build identity | Source, reference, and application ELF identity match the exact admitted package |
| Postcondition | The persisted hostname digest matches the storage-confirmed mutation |
| Corroboration | USB re-enumeration, sampled service loss, and post-reboot serial bytes are recorded but are not individually mandatory |

Fresh Attempt 27 is authorized only after these regressions, the full software
gate, atomic commits, and exact-current-HEAD preflight pass. It must use one
fresh protected parent and nonexistent supervisor child, one full Phase 35
invocation, one restart POST, dual private-first evidence, a 360-second
device-session bound, and at least 420 seconds of caller budget. Attempts 1
through 26 remain immutable, non-promotable, non-reusable, and ineligible for
splicing. Task 3, promotion, and `35-04-SUMMARY.md` remain blocked until Attempt
27 or a later qualifying attempt genuinely admits eligible evidence.

## Attempt 27 Checkpoint and Nested-ioreg Parser Boundary

Attempt 27 executed exactly once from clean source
`120e09dd117faaaa3bfdc056ebe6ea640e9b99c7` after the complete software gate and
exact-current-HEAD preflight passed. The protected parent, nonexistent
supervisor child, mode-`0600` sibling wrapper, dual evidence, internal detector,
360-second device-session bound, and caller margin met the approved contract.

| Field | Redacted value |
| --- | --- |
| Attempt ordinal | `27` |
| Source commit | `120e09dd117faaaa3bfdc056ebe6ea640e9b99c7` |
| Exact-head preflight | `passed` |
| Exact-head equality | `true` |
| Supervisor result | `non_promotion` |
| Primary category | `observer_unqualified` |
| Flash boundary | `ready` |
| Device-session schema | `esp-device-session-v1` |
| Initial device samples | `33` |
| Same physical device established | `false` |
| Reader armed | `false` |
| Pre-restart serial delivery | `false` |
| Restart request attempts | `0` |
| Device-session duration milliseconds | `10092` |
| Restoration secondary | `none` |
| Cleanup secondary | `none` |
| Cleanup complete | `true` |
| Root reusable | `false` |
| Progress decision | `continue_after_verified_fix` |

The fresh boundary occurred before receive-only open/read work and before the
restart POST. A separately protected read-only comparison found exactly one
admitted callout and proved that the canonical shell parser reproduces the
detector-bound physical identity. The Rust parser nevertheless produced no
candidate because it required a quoted property key to begin the trimmed line;
real nested ioreg properties carry tree branch prefixes before that key.

This is a newly actionable deterministic software defect, so the repository
progress policy permits fresh Attempt 28 only after a sanitized nested-tree
regression, verified parser fix, clean commit, complete software gate, and exact
head preflight. Attempt 27 remains immutable, non-promotable, non-reusable, and
ineligible for splicing. Task 3, promotion, and `35-04-SUMMARY.md` remain
blocked.

## Attempt 28 Checkpoint and macOS Serial-Alias Boundary

Attempt 28 executed exactly once from clean source
`7e9be48adcdb64a072f08b41dc4849b073c5ab15` after the complete software gate and
exact-current-HEAD preflight passed. It preserved the fresh protected-root,
internal-detector, dual-evidence, single-request, and bounded caller contract.

| Field | Redacted value |
| --- | --- |
| Attempt ordinal | `28` |
| Source commit | `7e9be48adcdb64a072f08b41dc4849b073c5ab15` |
| Exact-head preflight | `passed` |
| Supervisor result | `non_promotion` |
| Primary category | `usb_identity_drift` |
| Flash boundary | `ready` |
| Initial same-device samples | `3` |
| Reader armed | `true` |
| Pre-restart serial delivery | `true` |
| Restart request attempts | `1` |
| Restart request fully transmitted | `true` |
| Restart response received | `true` |
| Post-restart serial delivery | `true` |
| Recovery physical match | `multiple` |
| HTTP Boot B observations | `0` |
| Restoration secondary | `restoration_action_failed` |
| Fresh protected original-setting confirmation | `true` |
| Cleanup secondary | `none` |
| Cleanup complete | `true` |
| Root reusable | `false` |
| Progress decision | `continue_after_verified_fix` |

Attempt 28 proves the nested-property repair and advances the evidence boundary
through receive-only qualification and exactly-once restart transmission. The
recovery scan then treated the callout and dial-in aliases of one macOS serial
service as two candidates sharing one physical identity. No Boot B HTTP fact
was admitted. Although the immutable seal conservatively records a restoration
action failure, a fresh separately protected typed GET classified ready and its
private digest matched the original rather than the mutated setting, so no
additional recovery mutation was required.

Fresh Attempt 29 is permitted only after a sanitized paired-alias regression,
canonical callout candidate fix, clean commit, complete software gate, and exact
head preflight. Attempt 28 remains immutable, non-promotable, non-reusable, and
ineligible for splicing. Task 3, promotion, and `35-04-SUMMARY.md` remain
blocked.
