---
status: resolved
trigger: "Investigate and fix Phase 35 Attempts 19–20 flash-access boundary per the user's approved plan. Establish the red software reproduction, then implement the exact espflash 4.5.0 pin, typed phase35-flash-boundary-v1 classification/projection, private stage capture, explicit USB reset flags, detector/stage stability gates, and the read-only checksum probe integrated into Attempt 21's phase-owned command. Do not run real hardware yet; stop after code/tests and report exact-head readiness."
created: 2026-07-22T03:39:51Z
updated: 2026-07-22T04:37:18Z
---

## Current Focus

hypothesis: Confirmed and repaired. The factory boundary lacked an exact supported-tool/reset contract and collapsed the private espflash process result into one coarse supervisor failure.
test: Hermetic real-process and supervisor regressions exercise the probe, factory, NVS, and monitor boundaries with exact tool identity, private capture, stable USB readiness, and typed failure projection.
expecting: Probe is the first post-detector hardware boundary, credentials follow it, every stage is classified in order, and no raw child output or unstable device identity is admitted.
next_action: Commit the verified repair, install espflash 4.5.0, rerun exact-head preflight, and execute the authorized Attempt 21 once.

## Symptoms

expected: Detector passes, then Phase 35 factory flash, NVS, Boot A capture, HTTP evidence, mutation/reboot/restoration and admission complete.
actual: Attempts 19 and 20 pass detector, then fail in the first factory espflash process after device information and before transfer progress.
errors: Committed safe signature is flash_or_boot_a_failed + target_connection_failed; private logs must remain protected. Offline safe counters classify the process as factory/post-info/pre-transfer.
reproduction: Use hermetic real-process fake espflash tests; do not invoke hardware.
started: Earlier 4.0.1 flash runs succeeded; Attempts 19 and 20 repeated after one exact USB/barrel remediation. User approved upgrading to espflash 4.5.0 and Attempt 21 after verified software repair.

## Eliminated

## Evidence

- timestamp: 2026-07-22T03:39:51Z
  checked: Active lesson inputs and repository guidance.
  found: Private-first classification, protected evidence ownership, exact OS-process boundary tests, explicit espflash reset controls, and progress-gated attempt rules apply; hardware execution is explicitly out of scope.
  implication: The repair must be proven with hermetic real-process tests and must preserve private artifacts while exposing only a closed redacted projection.
- timestamp: 2026-07-22T04:02:00Z
  checked: Hermetic isolated-supervisor execution with a real fake flash process that writes strict factory metrics and a mode-0600 child log, then exits after device information and before transfer.
  found: The regression fails because the current non-promotion seal lacks category=post_info_pre_transfer_failed and the phase35-flash-boundary-v1 fields; the supervisor still collapses the process to its coarse flash failure.
  implication: The production path needs a typed classifier over immutable private stage artifacts and must preserve that classification as the earliest failure.
- timestamp: 2026-07-22T04:25:00Z
  checked: Phase 35 stage execution and the existing tools/flash evidence capture primitive.
  found: Factory/NVS currently use Command::output and concatenate raw stdout/stderr before writing, whereas monitor capture uses the streaming InterleavedSanitizer. The primitive accepts the command program without checking it against LocalFlashEnvironment's canonical espflash path.
  implication: The stage path must reuse the sanitizer and the primitive must explicitly bind the requested program to the trusted canonical executable before spawning.
- timestamp: 2026-07-22T04:30:00Z
  checked: Focused tools/flash capture repair with cargo test -p bitaxe-flash, git diff --check, and shell syntax checks.
  found: All 144 bitaxe-flash tests pass. New real-process tests prove trusted-program binding, streaming secret redaction, and rejection before output creation for an arbitrary executable. Factory/NVS command-vector tests now prove the explicit non-interactive USB reset, hard reset, and skip-update flags.
  implication: The capture layer is ready for the parent to finish the probe/readiness wiring; no hardware was invoked.
- timestamp: 2026-07-22T04:37:18Z
  checked: Complete Phase 35 integration, exact stage classifier, local toolchain contract, shell quality gates, focused Bazel suites, repository reference/parity/lifecycle/redaction checks, and the mandatory Rust sequence.
  found: The checksum probe runs before credential validation; factory, NVS, and monitor stage artifacts are classified privately; the non-promotion seal preserves the earliest typed boundary; all focused and repository-wide software checks pass.
  implication: The repair is software-complete and ready for an exact-head preflight followed by the single authorized Attempt 21 invocation.

## Resolution

root_cause: Phase 35 admitted detector success but did not enforce one exact espflash version/reset contract for subsequent effects, did not preserve sanitized per-stage child evidence, and collapsed the post-device-info/pre-transfer boundary into flash_or_boot_a_failed.
fix: Pinned and resolved espflash 4.5.0, added the strict phase35-flash-boundary-v1 classifier and parity CLI, captured every stage through the fail-closed streaming sanitizer, added explicit native-USB reset arguments and three-sample identity/readiness gates, and inserted a read-only checksum probe before credential access.
verification: The mandatory Rust format, Clippy, build, and all-feature test sequence passes. The flash/parity, detector, doctor, readiness, Phase 35 HTTP/supervisor/promotion, and Phase 30 Bazel suites pass. Shell syntax, shfmt, ShellCheck warning-level, reference, parity, lifecycle, redaction, and diff checks pass. No hardware was invoked during diagnosis.
files_changed: [scripts/espflash-tool.sh, scripts/bootstrap-esp.sh, scripts/esp-doctor.sh, scripts/detect-ultra205.sh, scripts/phase35-stage-readiness.sh, scripts/phase35-correlated-evidence.sh, scripts/phase35-correlated-evidence-effects.sh, tools/flash/src/evidence.rs, tools/flash/src/main.rs, tools/parity/src/phase35_flash.rs, tools/parity/src/main.rs]
