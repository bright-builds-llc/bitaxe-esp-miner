# CPU0 cadence continuation — host build blocker

The recorder/log-path corrections are published as
`4c53747068d7a8fc006dc2f7e8b39b7368bd1242`, with Gate
`ad16c23d2dd1ec6c48f659ef6ebd4a3f7015666a`. Software checks and the native
software-check package passed, but the subsequent clean published-head package
did not complete. Preparation 4 was not admitted and no hardware effect or
mining reservation followed this publication.

## Observed boundary

The clean package build reached Cargo's ESP-IDF dependency build-script stage.
An observed `esp-idf-svc` build-script child remained at `_dyld_start`; its sample
contained no Rust frames. The parent Cargo sample showed it waiting for child
output. The child's static `codesign --verify` check passed. Kernel AMFI messages
reported a missing CMS blob and CT-signature issue at launch, but those messages
alone do not establish the cause of the stall or a definitive execution denial.
No signing, quarantine or host security settings were changed.

The stalled child was stopped with identity-checked SIGTERM. The package command
then failed, and the observed Cargo, child and wrapper processes were absent.
Its 747.5-second command duration includes investigation and the deliberate
cancellation; it is not a firmware runtime measurement. An unrelated project's
Cargo process was identified and left untouched.

A fresh Terminal launch was attempted as an environment comparison, but Computer
Use rejected access to Terminal for safety reasons. No Terminal command ran.
The subsequent canonical closure invocation also stalled in Bazel workspace
status; only its identified client was interrupted. That failure did not justify
using the available dirty or wrong-commit package for hardware.

## Completed closure and retained state

The already-published, effect-free Node backend was invoked directly to close and
review preparation 3 after the canonical launcher was stopped. This bypassed
only the unavailable build launcher, not any validator: production defaults,
exact accepted seal, ancestry, accounting and cleanup checks remained active.
The v2 receipt is a protected sibling and remains unverified with no continuation
authority. Its original failure seal remains
`1936ad2be4fffa59dbc650fd833bc335e280dd31394ab3378e5d2760aa1ab580`.
The previous failure records and sibling closure are unchanged.

The last actual device review confirmed the safe baseline, inactive lease, next
ordinal 16, last completed 15, charged 1,200,000 ms and no pending reservation.
The browser, observer, supervisor and USB resources were released before these
software build attempts. No fresh physical-state claim is inferred from elapsed
time or host cleanup.

## Resume requirement

Resolve the host executable-launch stall, then run the normal `just package`
from the clean published checkout. Verify manifest source identity, clean status
and artifact hashes before creating preparation 4 with the existing guarded
supersession and a fresh authenticated ledger check. Repeat all four continuity
cycles and all three captures under the original limits. Do not reuse a dirty
package, reset accounting, restart a failed observer or promote old captures.

Cadence, USB qualification and migration remain active. The remaining hardware
proof is a passing complete cadence campaign, including actual mining and
independent shutdown. Parity remains 90/95. Private host diagnostics are retained
under `scratch/cpu0-usb-correction-20260913`; they are not device timing evidence.

## Final record verification

The final ordered host Cargo format, Clippy, build and test sequence passed under
`diagnose-host-stalls`, with 2,216 tests and one existing ignored. All four
recorders report success and complete owned-process cleanup. This does not clear
the separate native dependency executable-launch boundary.

The already-built repository redaction verifier passed with its normal launcher
environment, invoked directly because Bazel workspace status was unavailable;
its semantic checks were not disabled or changed. Standards and Markdown checks
passed. Prior hardware reports, failure seals, archive and checklist remain
unchanged. No workspace-status processes remained after cancellation. The host
blocker record changes no parity or hardware qualification status.
