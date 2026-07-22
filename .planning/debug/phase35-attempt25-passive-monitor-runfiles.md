---
status: verifying
trigger: "Attempt 25 stopped at approved_reboot_failed after every earlier probe, flash, Boot A, PATCH, and readback boundary passed."
created: 2026-07-22T15:35:23Z
updated: 2026-07-22T15:35:23Z
---

## Current Focus

hypothesis: The approved-reboot path failed before its POST because the Bazel-built supervisor distributed the passive-monitor script without its required process-group helper.
test: Inspect only protected artifact presence and safe failure-shape predicates, compare the production target's runfiles declaration with the passive monitor's source closure, and load that script from the rebuilt target runfiles without hardware.
expecting: The rebuilt supervisor contains the passive monitor, process-group helper, and serial-session helper adjacently, and the passive-monitor help path loads successfully from Bazel runfiles.
next_action: Commit this redacted checkpoint, run the complete clean software gate and exact-current-HEAD preflight, then use fresh Attempt 26 under `continue_after_verified_fix`.

## Symptoms

expected: After storage-confirmed readback, the supervisor starts a passive no-reset monitor, waits for exclusive ownership readiness, issues one approved reboot POST, and classifies only the post-loss boot bytes.
actual: The helper contract was recorded, but no passive ready, capture, response, or Boot B artifact was created. The private wrapper recorded a missing-file shape for the process-group helper. Restoration subsequently classified `ready`.
errors: Shareable signature is `approved_reboot_failed`, with probe/factory/NVS/monitor and original/immediate HTTP boundaries all `ready`; restoration has no secondary failure, cleanup records `cleanup_passive_monitor_failed`, and the root is non-reusable.
reproduction: Attempt 25 is sealed and must not be reused. The production Bazel target declared the passive monitor and serial-session helper as data but omitted the process-group helper sourced unconditionally at script startup.
started: Attempt 25 at exact source `f3a4d350492f5cc1073c0f62bd1a20f8af4355e2` after doctor and exact-head preflight passed.

## Eliminated

- Probe, flash, and Boot A failure: every typed flash stage reached `ready`, and private baseline classification/finalization passed.
- HTTP or storage failure: original and immediate reads classified `ready`, PATCH succeeded, and storage-confirmed readback preceded the reboot checkpoint.
- Reboot endpoint failure: the passive-monitor child exited before readiness, so the POST was never issued.
- Restoration failure: the restoration HTTP projection classified `ready`, and the primary category remained unchanged.
- Hardware retry as diagnosis: the missing Bazel runfile deterministically reproduces during software-only script loading.

## Evidence

- timestamp: 2026-07-22T15:35:23Z
  checked: Attempt 25's sealed typed metadata and protected artifact-presence facts without printing device, network, credential, command, process, or path values.
  found: Earlier typed boundaries passed, mutation and immediate readback completed, the passive contract existed, and every passive capture/reboot response artifact was absent.
  implication: Failure occurred while starting the passive monitor, before the reboot request.
- timestamp: 2026-07-22T15:35:23Z
  checked: The protected wrapper only for closed known error predicates.
  found: The process-group helper missing-file predicate matched; command-not-found, permission, argument, and readiness predicates did not.
  implication: The failure is a Bazel runfiles closure defect rather than a target or serial readiness boundary.
- timestamp: 2026-07-22T15:35:23Z
  checked: Production `phase35_correlated_evidence` data and the passive monitor's unconditional source statements.
  found: The target included the passive monitor and serial-session helper but omitted `process-group.sh`.
  implication: Direct invocation of the built supervisor could not load the reboot monitor even though workspace-relative tests could.
- timestamp: 2026-07-22T15:35:23Z
  checked: Rebuilt production runfiles and the Phase 35 hermetic supervisor suite.
  found: All three passive-monitor closure files are present, the help path loads from runfiles, and the focused suite passes.
  implication: The regression covers the observed production boundary without another hardware request.

## Resolution

root_cause: The Phase 35 Bazel target treated `phase13-monitor-capture.sh` as a standalone data file even though it unconditionally sources `process-group.sh`; the missing sibling caused the child to exit before readiness and before the approved reboot POST.
fix: Add `process-group.sh` to the production supervisor runfiles and verify the built target's passive-monitor closure loads successfully from Bazel runfiles.
verification: Code commit `f1f01aa1` passes the built-runfiles check, Phase 35 supervisor/stage/promotion suites, hardware-attempt policy, Phase 30 non-promotion, reference/parity/redaction checks, scoped ShellCheck/shfmt, and the mandatory Rust gate. The checkpoint commit and exact-head preflight remain pending before Attempt 26.
files_changed:

- scripts/BUILD.bazel
- scripts/phase35-correlated-evidence-test.sh
- .planning/debug/phase35-attempt25-passive-monitor-runfiles.md
