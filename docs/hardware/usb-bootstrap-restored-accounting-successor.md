# USB bootstrap restored-accounting successor

Contract ID: `usb-bootstrap-restored-accounting-successor-v1`.
Owner: `task-usb-bootstrap-drain-observability`.

This prospective amendment permits exactly one fresh no-mining measurement,
attempt 004, after a verified collector correction, publication and clean packaging.
It supplements the immutable [measurement](usb-bootstrap-drain-measurement.md),
[preflight successor](usb-bootstrap-preflight-successor.md) and
[early-reader correction](usb-bootstrap-reader-correction.md) contracts. Freeze
these decisions before implementation; publish the verified implementation and
exact clean package before effects. Existing standing authorization covers this
bounded continuation. No failed context becomes reusable.

## Exact failed predecessor

Preserve attempt 003 permanently as `unverified`, with correction acceptance false:

- Root: `scratch/usb-bootstrap-measure/attempt-003`.
- Context: `9f4375e1dd8602f47b3a6f5713395b701b09caf9a8fd758e74a9339b2bbd1cfe`.
- Result: `bb097f4b85e9ef92b433700ec2c83b15b701bf1de7ab7636180da39500ac4f5b`.
- Seal: `b53c0db5d163f17cedc33dc039274e2171339882a8d9a15ca157ae1c3cfec2b7`.
- Failure: `c630a2da0002e0e1b8be336a5db32141f8e4d47082407018adc17c4f94e9aa10`.

The earliest failure remains browser / browser / `bootstrap_client_failed`, with
null observation hash. Do not replace it with a more specific invented browser or
device event. Parent UI observation and review of the frozen source explain the
incompatible ordering: Stop/Restore produced `baseline_confirmed`, then the
accounting collector required `ready`. This is a source-based diagnosis of the
collector, separate from the retained generic failure.

The ordinary installation/capture passed. Its host reader opened 818.544 ms after
reset-call start, and the first 92-byte bootstrap record completed in one device
millisecond with zero observed TX failures. The first five correction checks are
true. Final accounting and its derived restoration receipt are absent; the final
two checks and overall acceptance remain false. Do not fill those missing records.

Final journal state 15 records closed serial, explicit device restoration, inactive
leases, preserved Device Identity/settings/authorization high-water and
mine-on-boot false. The native window was observed closed. Parent cleanup facts
are complete, the supervisor exited zero within five milliseconds, and recorded
kernel absence covers serial holders, owned groups, listener and operator socket.
These actual resource facts do not turn the formal cleanup result into complete:
its accounting/restoration joins are missing.

The installed before-source for the successor is firmware
`5bda5b91304f18d74bd64160abb807cfafbd0087`, ELF
`c16658a3472e5e508e85c6186d04882ee0eb968cb67bbc4c73c23ead718f1af0`.
Gate remains `e20c0fd52d2216596f904992ffa54fda33be9025` unless a separately
verified change is needed. No Gate or firmware protocol change is planned.

## Narrow collector correction

For version 4 only, define a measurement-local, phase-aware accounting predicate.
Keep shared V2 judges and historical version-1/2/3 interpretation unchanged.

- Initial accounting still requires fresh connected `ready` state.
- After accounting requires explicit `deviceRestorationConfirmed === true`.
  Accept actual connected `baseline_confirmed` or `ready` state only with that
  confirmation. Keep the original status in every journal and receipt.
- Both phases retain all existing identity/settings/high-water, baseline-ID,
  inactive-lease, mine-on-boot-false, no-failure, no-running and no-renewal checks.
- Keep both fresh accounting reviews, unchanged counters and ledger equality.
  Gate's existing accounting helper obtains fresh possession after Restore;
  do not bypass that helper or add authority, signing or mining routes.
- Apply the same rule to the page collector and server save/read-only review.
  Refresh does not change the page status back to `ready`; do not depend on it.
- Present the after-accounting action only when the required restored state is
  available. An incomplete restoration cannot be presented as ready to export.

A shared browser-safe predicate may use one fixed, source-bound local JavaScript
asset route. No arbitrary file serving, device endpoint or new control route is
permitted. Its bytes and tests belong to the validator/source inventory.

The native workflow is explicit: connect/authenticate the candidate, Stop/Restore,
wait for a confirmed safe baseline, record both fresh accounting reviews, then
close and flush serial. The final closed state must still confirm restoration.
No historical error is cleared and attempt 003 is never served again.

## Guarded preparation and evidence

Use `usb-bootstrap-measure-context-v4`, retaining the version-3 fields and adding
only `accountingAmendment` with this document's path and digest. Keep the original
`correctionAmendment` binding. Use the existing `preflight --predecessor-receipt`
interface, pointed at attempt 003's exact final result. Reject supersession flags;
no new closure command, generic retry input or recursive lineage is introduced.

Admit only `scratch/usb-bootstrap-measure/attempt-004`, a fresh attempt identity
and exclusive sibling `attempt-ordinal-4.json`. Preserve every older assignment,
preparation, source snapshot, result and seal byte-for-byte.

Before assignment, independently verify:

1. Exact predecessor anchors, complete protected inventory and existing read-only
   result judgment, including its accepted ancestry and source/native proofs.
1. One qualified installation 0, actual zero exit, exact candidate identity,
   complete v2 host timing, device observations, all five passing capture/timing
   checks and the original failed overall judgment.
1. The exact earliest failure, absence of after-accounting/restoration receipts,
   and ordered ready -> stopping -> baseline_confirmed -> closed observations.
   The closed state must retain all preservation and explicit-restoration facts.
1. Actual browser closure, supervisor exit, operator disposition and bounded
   parent cleanup joins. Validate resource cleanup independently of the missing
   accounting joins; never relabel historical `cleanup.complete: false`.
1. Absence of another installation or new mining, grants, reservations, renewals, work,
   fixtures or fault campaigns. Verify the whole inventory/journal and command
   claims, not just one optional filename. Preserved cumulative counters need not
   equal zero.
1. Current absence of the recorded owners, descendants, serial holders and local
   listeners before new admission. Historical review remains read-only and does
   not require old PIDs to remain unused after a later context starts.

Bind the successor physical anchor to attempt 003's `install-0.claim.json`.
Derive `beforeSource` from its installed package and same-pair final observations.
Bind `originalCampaign.record` honestly to attempt 003's `accounting-before.json`;
its original campaign ID is preserved. The ledger in that file was observed on
the prior installed image before installation 0. It is a continuity expectation,
not a new after-install ledger.

Require fresh authenticated successor accounting before any write: next ordinal
18, last completed 17, charged 1,560,000 ms, pending false; the original campaign
remains 240,000 ms with masks 7/7 and pending false. Any mismatch blocks effects.
No reservation, refund or ledger reset is permitted.

Require a changed clean published firmware/package/evaluator and unchanged or
separately verified exact Gate. Reuse encoded source snapshots, the stable
source-bound Rust/Node regression proof, native checks and validated staging
before exclusive assignment. Add the collector and guarded-predecessor regressions
to the fixed Node proof command; callers still cannot select commands or tests.
Preserve its 120-second complete-check limit, 64-KiB combined output bound,
actual native Node identity, actual child exits/cleanup and nonzero test counts.
Historical proof receipts/readers retain their original interpretation.

Keep the `usb-bootstrap-reader-correction-check-v1` receipt shape and command
name unchanged. Select the expanded fixed Node arguments only for context v4;
v3 historical proof retains its original two-file interpretation.

The fixed version-4 Node proof consists of `host-timing.test.mjs`,
`correction-judge.test.mjs`, `restored-accounting.test.mjs` and
`restored-predecessor.test.mjs` under `scripts/usb-bootstrap-measure/`, using the
existing `--test --test-concurrency=1` arguments. The identity and Rust capture
commands remain exact and unchanged. Bind and retain every validator/artifact
identity before effects.

## One hardware measurement and unchanged judgment

Use fresh detector admission and native foreground browser gestures. Preserve
NVS/Device Identity with the existing disjoint update segments and ROM ownership
checks. Perform one installation 0 and one full 30-second early-reader capture,
with both admission passes and the same reset backend. No retry, second write,
extra reset, rollback, factory operation, mining or V2 qualification is admitted.

Retain every correction limit: ordinary capture qualified; one descriptor and
actual joined/released ownership; reader opened within 1,500 host milliseconds of
reset-call start; first bootstrap completed within 2,000 device milliseconds;
zero captured TX/integrity failures; unchanged accounting and preservation; and
complete explicit restoration plus real host cleanup. Host/device clocks remain
separate. Zero-failure coverage ends at the last observed device snapshot.

Reuse result-v2 and its exact correction checks. A complete negative measurement
remains negative; failures cannot be tolerated merely because their capture passed.
Seal and independently review the outcome. Leave the measured candidate installed,
mine-on-boot false, leases inactive and owned resources released. No fifth attempt
or parity credit is added. Another failure requires verified targeted progress and
a new prospective bound; existing authorization does not permit an unchanged retry.

## Verification and completion

Test the actual page collector and production server through Restore ->
`baseline_confirmed` -> refresh -> fresh possession/accounting reads -> after
receipt -> closed final state, with simulated device boundaries clearly labeled.
Test missing restoration, stale/disconnected state, initial-stage rejection,
changed identity/settings/high-water/baseline/accounting, observed work, duplicate
receipts, private-field rejection and zero Start/Load/Renew calls.

Test exact failed-predecessor acceptance and rejection of altered/missing evidence,
wrong failure, fabricated after-ledger, incomplete resource cleanup, extra effects,
wrong before-source, duplicate/interrupted assignment, archived-task authority,
old-context effects and validator/tool drift. Old readers must reproduce the sealed
003 result without upgrading it. Software simulations do not replace the new live
restoration/accounting collection.

Run ordered Cargo checks, affected canonical/Gate checks, native packaging and
ownership, standards, reference, redaction, task-ID and read-only parity/progress
checks. Publish and build clean before preflight/effects. On a complete pass, add
an exact-pair report and completion review, archive only the completed bootstrap
task, and retain separate V2 continuity/channel/share obligations. On failure,
keep it active with the precise gap. Parity stays 90/95 throughout.
