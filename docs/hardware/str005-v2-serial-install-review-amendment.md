# V2 channel successor after installation review failure

Contract ID: `str005-v2-serial-install-review-successor-v1`.

Status: Prospective reviewed contract. Implementation requires publication of
this amendment; effects require subsequent verified implementation publication.

This prospective amendment belongs to `task-str005-v2-install-review-amendment`.
The existing `task-str005-v2-serial-qualification` owns the later hardware and
mining work. It supplements the immutable
[base](str005-v2-serial-qualification.md),
[clock](str005-v2-serial-clock-amendment.md),
[permission](str005-v2-serial-permission-amendment.md) and
[cleanup-successor](str005-v2-serial-cleanup-successor-amendment.md) contracts.
Publish this reviewed amendment before implementation; publish verified
implementation and the exact clean package before effects. Existing standing
authorization covers this bounded continuation without another approval.

## Pinned failed predecessor

Channel003 remains permanently `unverified` / `stop_evidence_incomplete`:

- Context: `031552692b777c324b5633dfb5b1cbbc4625da4cdea1adedcd8c1c3ee0cbd1b1`.
- Result: `cf219da3a6b6830f1818f03eeb7e3d88b0b912d589a99e6a4a1f8b8b0a02ad54`.
- Seal: `5941810323f06e9bdf86d79bc8c9f4aed6ce3a2f105716153e68b101e7f79bf3`.

The original first failure is supervisor `v2_operation_failed`, with no device
or fixture cause, recorded-first-observation ordering, and host observation
703528 ms. The canonical judgment is `v2_recorded_failure`, native code null.
Keep that disposition separate from the parent-observed `private_path_policy`
diagnosis in `parent-install-permissions.json`. The Rust installation evidence
producer created a directory/file tree with modes 0755/0644 beneath a protected
ancestor. The parent recorded the original modes and hardened permissions
without changing file bytes; this enabled forensic inspection, not restoration
of acceptance. Preserve that exact provenance and `acceptanceRestored: false`.

Installation0 completed and its trusted boot capture supports the expected
installed firmware `0d2b6d061edc7aab8957c17764ce77ee82f313c1`, ELF
`8e766fbadca0678e4db534acb067e7621d7fed29e1a749ce9d806c275559d178`. Gate was
`e20c0fd52d2216596f904992ffa54fda33be9025`. No authenticated post-install
possession/baseline was collected. Boot health does not establish application
authority, preserved settings, current accounting or restoration. The accepted
Noise result remains the protocol ancestor; its older installed image must not
replace this expected before-source.

No continuity cycle, Channel diagnostic, fixture, signing, grant, reservation,
mining, observer or heartbeat fault followed installation0. Initial accounting
was collected before that installation. It is not an after-install ledger. The
parent recorded `v2_parent_cleanup_state` because complete ordinary cleanup
could not establish the missing device state. Actual browser closure and
supervisor/parent exit remain useful ownership facts; they do not fill that gap.
No complete historical cleanup or accepted Channel projection is created.

## One finite successor classifier

Reuse `prepare-channel-successor --private-root FAILED` and
`review-channel-successor --private-root FAILED`, with deterministic protected
sibling `FAILED.successor-readiness.json`. Keep the existing Channel002 reader
and receipt schema exact. Add a separate dispatch for only the pinned sealed
Channel003 class above. A generic failure, another ordinal or merely a matching
error string is insufficient. Do not modify failed evidence or its siblings
while inspecting it, and never rerun the failed browser workflow.

Before creating the new receipt, independently verify:

1. Exact context/result/seal anchors before interpreting ancestry or operational
   paths; complete protected inventory, thirteen artifacts, source/native
   receipts, exclusive assignment, original reviewer disposition and the full
   accepted Noise / permission / Channel002 successor ancestry. Recheck bytes,
   membership and modes after inspection. Reject symlinks and conflicting
   evidence. Historical readers must reproduce the original verdicts.
1. The authenticated initial accounting row, its context/sequence/state join,
   and the later closed browser state that authorized installation0. Verify only
   the accounting actually observed: next18, last17, charged1560000 ms, pending
   false; original budget240000 ms and masks7/7. Do not call it final or current
   accounting.
1. Exactly installation0's claim, detector/physical admission, exact fixed
   arguments, prearmed process observations, actual zero exit, command receipt,
   trusted healthy startup capture and exact candidate package. Reuse the
   qualified installation inspector. Require the same physical-device ancestry.
   Do not write a replacement install-review receipt or infer a later serial
   session from the boot capture.
1. Exact parent permission diagnosis, original failure hash and all three
   recorded mode entries joined to their sealed files/directories. The recorded
   permission-only correction is historical evidence, not a general permission
   repair capability. Reject altered bytes, a different failure or an
   unsupported installation result.
1. Absence of install1–4, cycles/probes, Channel Start/device diagnostic
   records, fixture ownership/readiness/exchange, issuance/loaded grant,
   reservation, renewal, work, observer and fault evidence. Check the complete
   inventory and journal, not only one optional filename. Preserve missing
   after-baseline, after-accounting and restoration as missing proof.
1. Actual browser-close, supervisor-close and available parent/install/detector
   process witnesses. Recheck current absence of every recorded owner identity,
   process group and descendant, recorded parent references, both serial nodes'
   holders and the known supervisor listener. An occupied parent PID without a
   recorded start identity fails closed. Do not invent an uncollected exit,
   fixture instance or pool-port absence observation.

Creation remains effect-free: no device opening, new context, credentials,
fixture, signing or reservation. Require safe modes and exclusive publication
through the existing pending-file pattern; an interrupted or duplicate receipt
cannot be overwritten. Read-only historical receipt review verifies source and
sealed inputs without requiring current owners to remain absent after a later
attempt starts. Recollect current ownership at preassignment and serve
admission.

## Closed receipt and context binding

Use schema `str005-v2-channel-successor-readiness-v2`, with exactly: `schema`,
`amendmentSha256`, `failedRoot`, `failedContextSha256`, `failedResultSha256`,
`failedSealSha256`, `status`, `classification`, `hardwareQualified`,
`historicalCleanupComplete`, `afterBaselineObserved`, `beforeSource`,
`predecessor`, `initialAccounting`, `inspectedInputs`, `currentOwnership`,
`checkerIdentity` and `nonClaims`.

The fixed values are `status: unverified`,
`classification: ready_for_fresh_channel`, `hardwareQualified: false`,
`historicalCleanupComplete: false` and `afterBaselineObserved: false`.
`amendmentSha256` is this published document's digest. The failure fields bind
only Channel003 above. `beforeSource` has exactly `firmware_commit` and
`app_elf_sha256`, with the expected installed identity above. `predecessor`
keeps the accepted Noise `{root, resultSha256, sealSha256}` binding.

`initialAccounting` has exactly `path`, `sha256`, `observedSequence`, `ledger`
and `original`. Its path is `accounting-before-install.json`; its SHA binds that
whole sealed file, its sequence is the authenticated initial journal row, and
its ledgers reproduce that file's existing closed ledger/original-budget
schemas. This is explicitly pre-install evidence. No after-accounting field or
reusable possession/private-baseline identifier belongs in this receipt.

`inspectedInputs` is the complete sealed `{path, sha256, length}` inventory.
`checkerIdentity` retains exactly `{firmwareCommit, sources}` and the fixed
complete source inventory defined by the prior amendment, extended with the new
checker dependencies. Bind it to the clean published creator commit and the
successor evaluator; historical review verifies recorded Git/source identity,
not equality with a future HEAD.

`currentOwnership` has exactly `schema`, `source`, `observedAtUnixMs`,
`ownerCount`, `processGroupsAndChildrenAbsent`, `serialNodeCount`,
`serialHoldersAbsent`, `supervisorListenerAbsent`, `parentReferenceCount` and
`parentReferencesAbsent`. Its schema is
`str005-v2-install-successor-ownership-v1`, source is `successor-collector`,
counts come from deduplicated pinned records, the time is a nonnegative integer,
and all absence flags are true. There is no invented fixture/unclaimed-owner
field. `nonClaims` is exactly, in order: `post-install-baseline`,
`post-install-accounting`, `historical-cleanup-complete`, `accepted-channel`,
`mining-acceptance`, `cycle-or-baseline-transfer`, `effect-authority`.

Reuse `preflight --supersede-channel RECEIPT`; it remains mutually exclusive
with permission supersession and unavailable to Share. New admissions use
`str005-v2-serial-context-v4`. Preserve v1–v3 historical readers and prevent
their live reuse. The v4 shape reuses existing fields, with
`permissionSupersession: null` and the existing closed `cleanupSupersession`
binding for Channel004, now pointing only to the pinned Channel003 and its v2
readiness receipt. Add this document's exact digest as contract binding
`installReview`. Channel004 is the only new Channel ordinal; Share001 has both
supersessions null and requires independently accepted same-pair v4 Channel004.

Keep the existing inspector function names and safe seven-field CLI summary. For
the v2 receipt, return `initialAccounting` and `afterBaselineObserved: false`
instead of the v1 internal flat ledger/original fields; all other common binding
fields remain. Dispatch explicitly by context and receipt schema: v3 accepts
only v1/Channel002, v4 only v2/Channel003. Fresh admission must not mistake
initial accounting for current device evidence. Checker identity must match the
new candidate commit/evaluator. Preserve complete ancestry inspection at
admission and bounded source/snapshot/receipt/ancestor pin checks before
effects; the detector's 60-second bound is unchanged.

## Required software correction and verification

Require a changed clean published firmware/package/evaluator containing the host
corrections. Gate may remain unchanged at its verified pin. Do not relax
private-path validation to accept 0755/0644 evidence. Correct repository-owned
installation evidence creation so every new directory/file is 0700/0600 before
review. Apply a restrictive child umask at the owned launch boundary and verify
the actual Rust evidence producer, including nested directories and atomic
outputs. Scope any handling of existing paths narrowly; reject symlinks, foreign
ownership or unsupported files rather than recursively repairing an arbitrary
tree. No post-failure permission mutation may revive a terminal run.

Also correct the host-check launch environment: Bazel's generated Node wrapper
requires runtime fields that the filtered regression child currently omits. Use
the existing supported `nodeRuntimeEnvironment` allowlist/runfiles seam so the
already-built canonical Node toolchain works under a minimal environment. Do not
inherit unrelated environment or credentials, edit generated wrappers, add
verdict flags or substitute a prior pass. Preserve the existing three-test
host-correction argv, 180000-ms/65536-byte bounds and real zero exit. The
canonical `just` preflight must run the correction tests successfully through
its actual launcher before assignment. Keep the existing source-bound host/Gate
correction receipts and frozen old validators; new receipt validation must bind
the corrected checker source.

Before effects, require focused regressions for the real Rust producer beneath a
protected ancestor, actual child umask propagation, unchanged evidence bytes,
unsafe/partial trees, and the actual canonical filtered Node launch. Run the
permission/producer regressions separately before publication and bind their
sources in the evaluator; do not change the historical correction command.
Re-run the existing real host cleanup/lifecycle rehearsal. Add exact-classifier
positive and mutation tests, historical-verdict preservation, missing/changed
initial accounting, unsupported flash, absent/wrong fresh possession and
baseline, ledger drift, owner conflicts, duplicate/interrupted assignments,
archived tasks and rejection of Channel005. Run the repository's full ordered
verification, native/package checks and privacy/ownership gates before
publication. Tests may use explicit code-only synthetic device prerequisites;
production admits no test seam or caller pass flag.

## Fresh hardware workflow and stop conditions

After publication, prepare/review the exact failed003 successor receipt and
exclusively reserve Channel004 before its context directory exists. Recheck
current host ownership before assignment and serve. Detector-admit the same
physical Ultra205, then obtain a fresh foreground Web Serial connection,
possession, exact expected `0d2`/ELF identity, safe inactive baseline and both
authenticated ledgers before any write. Failure to obtain any of these stops
without installation or mining. Boot health alone cannot pass this gate.

Use a newly captured private baseline for this page; do not import the failed
page's baseline or claim preservation across its unobserved post-install gap.
Then perform initial installation0 of the newly published candidate and four
fresh update/reconnect/65536-byte bidirectional cycles, the Channel exchange,
fresh restoration/accounting, full resource cleanup and independent acceptance.
No Channel003 cycle or baseline credit transfers. Keep the exact pair/HEAD
unchanged between accepted Channel004 and Share001.

Share001 uses a fresh root/page/baseline, four fresh same-candidate cycles and
fresh next18/last17/charged1560000-ms/pending-false accounting. Only then
consume one normal180000-ms reservation and prove the real accepted BM1366
share, device acknowledgement, heartbeat fault, revocation/shutdown within three
seconds, ordered stop/cooling, post-work authorization checkpoint, restoration
and complete cleanup. Expected completion remains next19/last18/1740000 ms,
pending false. Neither ledger is reset and no reservation is refunded.

All existing protocol, target, stack, heap, privacy, lease, preparation,
fixture, observer, heartbeat, cooling and recovery bounds remain unchanged. No
crypto fork, external pool, factory reset, boot-controller fallback or new
hardware owner is introduced. Preserve earliest failures; terminal contexts
never resume. This amendment admits no Channel005 or Share002. Further failure
requires verified progress and another applicable published finite admission
contract. The software task closes only after verified implementation
publication; the main task remains open until both scopes pass. No parity row is
promoted here.
