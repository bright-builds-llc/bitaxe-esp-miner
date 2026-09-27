# USB bootstrap interrupted-preflight successor

Amendment ID: `usb-bootstrap-preflight-successor-v1`.
Owner: `task-usb-bootstrap-drain-observability`.
This prospective amendment extends the [base measurement contract](usb-bootstrap-drain-measurement.md),
SHA256 `f8534c50f47d7c3a210295b5523c46cc51679850281a222af5c97402133e9cc4`.
Its hardware limits, privacy rules, native gestures, preservation, accounting,
cleanup requirements and qualification non-claims remain unchanged.

## Observed interruption and immutable inputs

The `def310ec` preflight failed native-frame validation before assignment. Its
correction was published as `29cf472dc50e6763cd30f525efc2b013eea5dfba`.
That corrected preflight reserved attempt 001 and copied its inputs, then failed
while inventorying a published synthetic-fixture source filename containing
`credentials`. This is a source-snapshot layout defect; do not weaken the generic
private-evidence filename guard or interpret the interruption as hardware evidence.

Fixed failed root: `scratch/usb-bootstrap-measure/attempt-001`.
Fixed assignment: sibling `attempt-ordinal-1.json`.

| Input | SHA256 |
| --- | --- |
| Context identity | `71a82b212b63e7d3023138bb569f756e5415907d401c6dbf815cc3387ce91632` |
| Exact context file | `dd0084021071b22c4b2c465a0be5fa93dad68434f81081350ff387f887905dea` |
| Assignment file | `759685dc765117c4bef70d5a3888f980104b4677cc30095e8f4760c158219980` |
| Canonical file inventory | `a481daa922ec5d96565225c1b597a09304c2a246cb4fbc524c6b3c578345f9ff` |
| Original failed command log | `ff6af63a7d7f7b619ca9df84a204a0750ba5abd530c68255a55fc11509267999` |

The original log is the 1,438-byte protected file
`scratch/usb-bootstrap-measure-software-20260926/preflight-corrected.log`.
It records the parent-visible `bootstrap_operation_failed` and failed command.
The narrower `noise_forbidden_inventory_name` cause is a read-only reproduction,
not an invented device event or an original detailed process receipt.

The exact corpus contains 1,773 files and 216 directories including the failed
root (215 descendant directories): context, manifest,
public trust, two native audit receipts, Gate page/bundle, eight package artifacts
and 1,758 source snapshots. Root entries are exactly `context.json` and `snapshot`.
There is no preflight inventory, operator sibling, browser state, accounting,
installation, detector, issuance, work or cleanup record. No original device
baseline or historical process observation may be invented from that absence.

Preserve every byte, permission and assignment. Do not create files inside this
failed preparation, finish its missing inventory, serve it, repair its layout,
reuse its attempt identity, remove its assignment or refund anything.

## Read-only classification and sibling closure

Add only these effect-free actions to `just usb-bootstrap-measure`:

- `close-preflight --private-root FAILED`: independently verify the exact failed
  corpus and write one exclusive protected sibling `FAILED.preflight-closure.json`.
- `review-preflight --private-root FAILED`: independently recheck that receipt and
  every bound input without writing or acquiring hardware/signing authority.

The verifier must reject symlinks, hardlink aliases, unsafe modes, changed bytes,
missing or extra entries, conflicting context/assignment identities, unexpected
operator/effect artifacts and a different failure class. Require 0700 directories
and 0600 files inside the failed root and for the assignment/log/closure artifacts.
Derive expected membership from the closed original context and fixed snapshot
roles; verify source bytes against both its recorded hashes and the published
failed Git commit. Do not use a broad forbidden-name exception. The sole reserved
filename is the exact published source
`scripts/phase28.1.1.1-synthetic-pool-credentials.mjs`; it is accepted only as a
Git-bound source snapshot in this anchored interrupted preparation. All package,
Gate, trust, native and original Channel005 ancestry joins remain required.

The canonical inventory is a sorted array of `{path,sha256,length}` entries, encoded
with the existing canonical sorted-key JSON function before hashing. Check the
failed corpus, assignment and external command log twice around receipt construction. Publication uses
exclusive pending-file creation followed by an exclusive link; collisions and
interrupted publication remain visible and cannot overwrite an existing receipt.

Receipt schema: `usb-bootstrap-measure-preflight-closure-v1`. Closed fields:
`schema`, `amendmentSha256`, `failedRoot`, `failedContextSha256`,
`assignment` (`path`, `sha256`, `length`), `failedSourceCommit`,
`status` (`unverified`), `classification` (`interrupted_before_effects`),
`earliestCause` (`source`, `code`, `observationSha256`),
`diagnosedCause` (`source`, `code`, `repositoryPath`),
`inspectedInputs` (path/hash/length triples), `checkerIdentity` (`commit`, `sources`),
`effectsObserved` (`false`) and `nonClaims`.
`assignment.path` is exactly `attempt-ordinal-1.json`. `inspectedInputs` contains
exactly the 1,773 failed-root-relative file rows, excluding the external assignment
and command log, which have their separate fixed bindings.

Earliest cause uses source `parent-observed` and code `bootstrap_operation_failed` bound to the
original command log. Diagnosed cause uses source `read-only-reproduction` and code
`noise_forbidden_inventory_name` bound to the exact source path above.
Checker identity binds the clean published corrected checker and its complete
source dependencies. Historical review verifies that recorded identity without
requiring the old checker commit to be the current checkout. Effect admission
requires the admitted current checker identity to match the receipt.
`effectsObserved:false` describes the inspected corpus, not fresh device telemetry.
The fixed non-claims are `no_hardware_execution_proof`, `no_fresh_device_accounting`,
`no_qualification_credit`, and `no_continuation_authority`.
Closure alone authorizes no continuation.

## Corrected source snapshot layout

New contexts use `usb-bootstrap-measure-context-v2` and one explicit layout:
`snapshot/source/<SHA256(UTF8 repository-relative path)>.bin`.
Logical paths, content hashes and lengths remain in the strictly sorted,
duplicate-free `sourceInventory`. Path hashes identify storage locations; content
hashes independently verify bytes. This is naming, not content redaction.

Use one repository-owned getter for snapshot writing, source verification and the
fixed browser-client asset lookup. Reject duplicate logical paths, duplicate
storage keys, unindexed blobs, aliases, missing files and substituted bytes.
Only the admitted clean published Git source inventory may be copied. Keep the
normal forbidden-filename guard intact for all evidence, including unexpected
credential files alongside encoded sources. Version 1 retains its verbatim-path
interpretation for historical inspection; never fall back between layouts.
Version 1 cannot regain effect authority, and missing completion evidence remains
unverified.

## One fresh successor admission

Extend preflight with `--supersede-preflight CLOSURE`, required and exclusive for
`scratch/usb-bootstrap-measure/attempt-002`. Admit no other successor. Require the
verified sibling closure, targeted source-layout/privacy regression, a changed
clean published firmware/package/evaluator and complete exact source/tool/native
bindings before writing sibling `attempt-ordinal-2.json`. Gate may remain the
same published pin. Use a fresh attempt identity and consume assignment exactly
once; interrupted assignment remains consumed.

Version 2 retains every base context field and adds only:

- `preflightAmendment`: this amendment's path and SHA256.
- `preflightSupersession`: `failedRoot`, `failedContextSha256`, `assignmentSha256`,
  `closurePath`, and `closureSha256`.

Its attempt ordinal is 2. Bind the closure in its preflight snapshot and recheck
the closure, immutable failed inputs and exclusive assignment before effects.
Reject archived-task authority, stale source/package/evaluator identity, duplicate
or conflicting successors and attempts to serve the failed preparation. Historical
readers and failed contexts never acquire authority through this amendment.

Installed before-source remains `14f6d6d98ad939db065542db66dd8aad77f097d7`, ELF
`cbe496ec96b5f20cfaeabf10f6e41105de916310e4dc34fc3a77f07d1091e8cf`.
Fresh detector admission, native authenticated possession, preservation and both
accounting reviews are still mandatory. Expected accounting remains next ordinal 18,
last completed 17, 1,560,000 ms charged, pending false; the exhausted original
240,000-ms ledger remains at reserved/completed masks 7. These are expectations, not newly
observed facts.

The successor permits only one state-preserving installation 0, the existing reset
and 30-second capture, fresh same-pair restoration/accounting and actual cleanup.
No mining, allowance, fixture, probe, second write/reset, rollback, third attempt,
V2 continuation or parity credit is added. Initial detector inspection uses the
existing repo command before the native page is opened; install admission still
requires its own fresh detector proof.

## Fixed correction check

Before successor assignment, run the pinned Node executable with fixed arguments
`--test --test-concurrency=1 scripts/usb-bootstrap-measure/source-snapshot.test.mjs`
in the admitted firmware root. There is no caller-selected command, test or verdict.
Bound execution to 60 seconds and captured output to 64 KiB; only actual exit 0
with no signal passes. This test uses filesystem fixtures and reads published
source bytes; it cannot access a device, credentials or signing authority.

Retain `snapshot/source-layout-check.json` with closed schema
`usb-bootstrap-source-layout-check-v1` and fields `schema`, `command` (literal
`source-snapshot-regression`), `nodeSha256`, `sourceInventorySha256`, `exitCode` (0)
and `signal` (null). The inventory digest uses canonical sorted-key JSON of the
entire admitted source inventory. Verify these joins before effects and in
historical review; rerun the actual check for admission rather than accept a
caller-authored receipt. Recheck all source/tool identities after it completes.

## Verification before effects

Test the real published synthetic-source filename through snapshot creation,
inventory, loading and fixed asset serving. Keep rejection tests for real unexpected
credential artifacts and all layout mutations. Test exact interrupted-preflight
closure, altered/missing/extra inputs, unsafe paths/modes, effect contamination,
receipt tampering, duplicate publication, interrupted/duplicate successor assignment
and version 1 effect rejection. Reuse actual-process lifecycle and failed-capture
regressions; never weaken the hardware classifier or native stack limits.

Run ordered Cargo checks, affected canonical tests, standards, ownership, reference,
redaction and read-only parity/progress validation. Review and publish the amendment
and correction, then build the clean exact package. Create/review the sibling
closure and successor context only afterward. Parity remains 90/95. Failed or
missing proof remains unverified with the earliest cause retained; any further
attempt requires another prospective contract and verified correction.
