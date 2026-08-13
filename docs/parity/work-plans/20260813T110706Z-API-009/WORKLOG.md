# Parity work log

## 2026-08-13T11:07:06Z | attempt-010 contract checkpoint

- Source commit: `ecb19811feaae5494af38a6fdd8cf3a17ba10f4e`.
- Actions: Selected API-009 first from the clean synchronized selector and
  bound exactly one attempt-010 to the regression-backed resumable-pause fix.
- Verification: The prior software closure proves the exact attempt-009
  timeout was removed from operator pause without weakening terminal/fault
  shutdown. The complete five-command quorum and live physical-observation
  boundary remain unchanged.
- Evidence: Current source, immutable prior closure, active task, and this
  public redaction-safe contract only. No protected attempt content,
  credential, detector, package, or hardware interface was accessed.
- Outcome: Attempt-010 can become effect-eligible only after this plan/task
  checkpoint is verified, committed, pushed, clean, and synchronized and all
  named post-push gates pass.
- Blocker or next safe action: Run the plan-only gates and push this immutable
  checkpoint before any hardware-capable command.

## 2026-08-13T11:20:00Z | immutable plan verification

- Actions: Bound the fresh attempt-010 contract to the active API-009 task and
  reviewed its single-attempt, effect, physical-observation, privacy, recovery,
  cleanup, timeout, and stop boundaries.
- Verification: Cargo format, clippy with warnings denied, all-target build,
  all-feature tests, Bright Builds checks, all 42 Bazel tests, parity
  validation/progress, redaction, pinned-reference cleanliness, the real
  firmware build, focused actuation/campaign/sensor/flash/real-process tests,
  diff checks, unique task binding, and sensitive-output review passed. One
  initial Cargo doc-test process encountered transient macOS uninterruptible
  I/O and was cancelled; the unchanged exact command then passed completely.
- Evidence: Immutable plan SHA-256
  `466b878f67b5664cec18071f5ce94fb47d70b9692bf54fd2baec64be6fe2e936`.
- Outcome: The plan/task checkpoint is ready to commit and push. It remains
  software-only until the pushed commit is clean and synchronized.
- Blocker or next safe action: Commit and push the checkpoint, then run only
  its named exact-package, private-root, credential-presence, and detector
  gates before the sole campaign.

## 2026-08-13T11:42:00Z | attempt-010 terminal checkpoint

- Source commit: `8e89891f445f49493d3909fb2e1b4c30795c5dce`.
- Actions: Re-ran the focused pushed-commit tests and firmware build, built the
  exact package, admitted exactly one Ultra 205 through the sole protected
  detector, and ran the sole attempt-010. Relayed the live rendered checkpoint
  without issuing a confirmation.
- Verification: The package matches the pushed source and pinned reference.
  The protected v8 aggregate proves trusted identity, protocol readiness, a
  genuine positive block, five accepted shares, confirmed pause, confirmed
  resumable safe stop, confirmed resume, and active mining after resume. The
  user's live physical report at the rendered checkpoint described the normal
  statistics/block-notification page, not the required blank / `BITAXE
  IDENTIFY` / `Hello!` / blank frame. No confirmation command ran.
- Evidence: Protected attempt-010 aggregate artifacts and seal only. Every
  private directory is mode `0700`, every private file is mode `0600`, no
  symlink exists, and the result digest matches. No public projection exists.
- Outcome: `stop_hardware_blocker`. The public wrapper classified
  `hardware_blocked`; safe stop and USB cleanup are confirmed, recovery was not
  required, and public evidence was withheld. API-009 remains `implemented`.
- Blocker or next safe action: Close this attempt without retry. A future clean
  selector may diagnose the production identify-state-to-display-render path,
  but attempt-011 requires a regression-backed fix and a separate immutable
  contract.

## 2026-08-13T14:41:48Z | closure verification

- Actions: Added the redaction-safe closure and updated only the active API-009
  task block; left the checklist row unchanged and emitted no public evidence.
- Verification: Cargo format, clippy with warnings denied, all-target build,
  all-feature tests, Bright Builds checks, all 42 Bazel tests, parity
  validation/progress, redaction, pinned-reference cleanliness, real firmware
  build, immutable-plan digest, private modes, seal, no-symlink, projection-
  absence, diff, and sensitive-output checks pass.
- Outcome: The closed attempt-010 result is ready to commit and push. API-009
  remains `implemented`; attempt-011 remains prohibited.
- Blocker or next safe action: A later selector run must diagnose and fix the
  production identify-to-display boundary before considering new hardware.

## 2026-08-13T15:31:00Z | late-observation interpretation correction

- Correction: The 11:42 terminal checkpoint incorrectly called the user's
  normal statistics-screen description a live rendered-checkpoint observation.
  The report arrived after multiple waiting turns, beyond the firmware's fixed
  30-second IDENTIFY duration, and the public signal had explained neither the
  expected frame nor that deadline.
- Preserved facts: Attempt-010 remains consumed; its exact package identity,
  genuine block, accepted shares, pause, resumable safe stop, resume,
  active-after-resume, safe terminal stop, cleanup, seal, private modes,
  redaction, and public withholding remain unchanged.
- Corrected outcome: `stop_authority_boundary`. The late report is neither
  positive nor negative IDENTIFY render evidence. API-009 remains
  `implemented`, and attempt-011 remains unauthorized by this historical plan.
- Next safe action: Complete the separately committed pre-armed v2 checkpoint
  plan before any future hardware contract is considered.
