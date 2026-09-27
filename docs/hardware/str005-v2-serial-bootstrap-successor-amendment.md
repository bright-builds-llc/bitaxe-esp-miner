# V2 successor after accepted bootstrap correction

Contract ID: `str005-v2-serial-bootstrap-successor-v1`.
Owner: `task-str005-v2-bootstrap-successor-amendment` for specification and
publication; `task-str005-v2-serial-qualification` for implementation and later
Channel/Share effects. This supplements the immutable
[base V2 contract](str005-v2-serial-qualification.md) and its published amendments.
Publish this reviewed contract before implementation, then publish verified
implementation and one clean exact package before effects. Existing explicit
owner authorization covers the bounded real-mining stage without another approval.

## Scope and unchanged limits

Admit exactly Channel006, followed only after independent acceptance by Share002
on the same clean published firmware/Gate/package/fixture/observer/evaluator pair.
Preserve every failed preparation, sealed result and consumed host assignment.
No earlier context becomes reusable. No crypto fork, protocol change, external
pool, persistent pool setting, factory reset, recovery-006, automatic rollback,
mining-budget reset or parity promotion is introduced.

The [accepted bootstrap correction](../parity/evidence/20260927-usb-bootstrap-correction-accepted.md)
resolves host receiving and collector completion. It supplies no continuity-cycle,
channel, share or mining credit. Native image/stack/resource gates, identity,
privacy, independent safety enforcement and every base acceptance criterion remain.

## Exact evidence and installed baseline

Channel005 remains sealed/unverified, with the original installation4 capture
failure and only three completed continuity cycles:

- Context: `dcfce9bd9bdaf8ea8fb59d021b0449fec10b58a651a353aa24ce360e4a4ff79a`.
- Result: `1b687eda11fe233677f77c4a77ee462912a6ddb5caf57ad9bfc7567fd4782c2e`.
- Seal: `3b77bd5d8b83433ba18b758a3ec579806a91c866e594cb0254656d7a313ea342`.

Accepted bootstrap004 is a separate measurement, with all seven correction checks
and complete restoration/accounting/resource joins:

- Context: `804f022dbdf4bb1f1f2298558486a5eb659c84fd851ae32a9e5a97cf1c542c78`.
- Result: `be8339ce97e8e91bf5194076181caa11e9e9407892bf6ab376f8a06fdfbd34c8`.
- Seal: `b9f85846b4c7875f4908cd44dc38067a11b7ac84a99c3c9b57391544ddc7a424`.
- Installed firmware: `77e2e4a431663ab2c2948d1e81183472a04e385c`.
- Installed ELF: `d32d799586a72bcdf58871e0905a756a6499442c23abcce4e9de346e1fafaf51`.
- Gate: `e20c0fd52d2216596f904992ffa54fda33be9025`.

Keep the original accepted Noise protocol ancestor at
`scratch/str005-noise-serial/attempt-001`, result
`9fe8f1ed6755e2c9a8573fa6dc5ae867fca044f6a79636cc584415c60744c40a`, seal
`cc8e278543447e026b908b2a0e9e5b626ff1f7d47ec245060325ccf91d49f4d2`.
Its older pair supports the protocol design, not the successor's fresh measurements.

Share001 remains failed before mining. Preserve its context
`e96e658911fd438e222ba7d0088a8d8720e6542c4a93ed7a43914cea92a7b79e`, result
`da7d9aa759a071febd83ec062fb046f360affe6c790dbc33d27ee65cc506c994`, seal
`0449a33a703c9ccab27c92ed573156912449d9f67feabd8fb59d5c36aabc8aa1`, and sibling
readiness receipt SHA256
`62b7bbad76b3e9fc6892861996d2260e97d4a2493657b9bb9da2affdbf1edd95`.
Its original qualification-ordinal18 marker SHA256 is
`a75ffc361c41209981ed90230c6e8945da0ba00e36d6d584f5d69e42c89fe13b`.
No old receipt, marker or ledger is rewritten or refunded.

## Closed version-6 admission

Use `str005-v2-serial-context-v6`, preserving the existing closed fields and adding
only `bootstrapCorrection`. Extend `contracts` with `bootstrapCorrection` holding
this document's `{path, sha256}`; include it in the existing canonical contract hash.

`bootstrapCorrection` has exactly:

- `failedRoot`, `failedContextSha256`, `failedResultSha256`, `failedSealSha256`.
- `acceptedRoot`, `acceptedContextSha256`, `acceptedResultSha256`, `acceptedSealSha256`.

The roots are exactly the firmware workspace's
`scratch/str005-v2-serial/channel-005` and
`scratch/usb-bootstrap-measure/attempt-004`, with the pins above. No supplied verdict,
endpoint, baseline ID, credential, arbitrary source path or policy override belongs
in this binding.

Channel006 uses the existing `preflight --scope channel` arguments and accepted
Noise `--predecessor-receipt`, plus one channel-only `--bootstrap-receipt` pointing
to bootstrap004's exact `final-result.json`. Reject other supersession flags.
Copy the already verified historical `shareSupersession` binding from Channel005;
permission/cleanup supersession fields remain null. Derive `before_source` from
bootstrap004's installed package and verified final observations. Bind the first
physical-device claim to bootstrap004's `install-0.claim.json`, not the older
Share001 image or Channel005 claim.

Share002 takes independently passed Channel006 as its ordinary predecessor,
inherits the exact bootstrap and historical share bindings, and rejects all
bootstrap/supersession flags. Require equality of the exact source, Gate, trust,
package, fixture, observer, evaluator and contract identities. The before-image is
that accepted Channel006 candidate. Channel finalization stays private; do not
commit tracked documentation or change the pair between scopes.

Only new effects from v6 are eligible. Versions1–5 and every sealed root remain
read-only. Keep their validators and historical source/checker domains unchanged.
In particular, validate Share001's old readiness checker against its own published
creator/source inventory. A v6-only binding must not pretend that old checker was
built at the new firmware commit or that Share001's before-image is still installed.
The accepted bootstrap evidence supplies the intervening installed-image/accounting
bridge. Do not relax v5's original equality checks.

Extend the v6 source inventory with the complete material bootstrap readers,
corrected host receiver, build rules, this amendment and transitive validation
inputs. Do not expand the older checker inventory retrospectively. Preflight may
run already-built fixed software regressions but cannot build, fetch, discover an
alternate device, read signing credentials or accept caller-selected tests.

Before any assignment, independently verify the full sealed Noise/Channel005/
Share001/readiness ancestry and the finite bootstrap001–004 lineage. Reproduce
original failures without upgrading them. Verify bootstrap004's qualified capture,
all seven accepted checks, exact installed identity, fresh after-accounting,
restoration, actual browser/serial/process cleanup and unchanged ledgers. Recheck
current absence of recorded owners and listeners for admission; historical review
uses the original sealed observations. Deep ancestry verification occurs before
assignment; hot effect gates recheck immutable pins, complete current source/tool
identities and ownership without recursively rerunning the historical campaigns.

Before the first write in either scope, require fresh authenticated reads of
both ledgers on that scope's native page and admitted installed image. Bootstrap004
accounting is an expectation, not live admission. Require next18/last17/1,560,000ms
with pendingfalse and the unchanged original240,000ms/masks7/7 budget. Share must
repeat fresh accounting before issuance after its continuity and cooling steps.

Reserve `channel-ordinal-6.json` and `channel-006` exclusively. Channel allocates no
mining ordinal or signing authority. Share later exclusively reserves
`share-ordinal-2.json`, `share-002` and the existing
`qualification-ordinal-18.successor-2.json` through its guarded pending-file path.
The successor marker joins the original ordinal18 marker, Share001 receipt and new
Share context/attempt. Interrupted or conflicting assignment remains consumed;
never overwrite an old marker or restart a partial context.

Require the live task to remain active. Archived bootstrap/amendment tasks provide
read-only evidence, never authority. Require a changed clean published candidate
relative to the failed Channel005 pair, bound native audits and all thirteen
immutable runtime artifacts before effects.

## Prospective restoration strengthening

Keep the actual V2 workflow distinct from the bootstrap measurement:

1. Channel's run completes its diagnostic, restores and closes automatically.
1. Share performs accepted work, heartbeat suppression, ordered shutdown and its
   existing observation/cleanup sequence, then closes and waits before recovery.
1. Reconnect through a fresh native gesture; fresh possession and observations
   produce `ready`. Then use **Record restoration** to collect retained completion,
   preservation and fresh accounting. It reads proof; it is not another Stop.

Manual Stop/Restore remains an emergency abort that preserves failure. Do not
silently invoke it as part of normal post-reconnect record collection, widen the
V2 page to accept `baseline_confirmed`, or copy the bootstrap collector's ordering.

For v6 only, require `deviceRestorationConfirmed === true` at restoration,
after-accounting, final closed-state and complete-cleanup acceptance. A baseline
whose restoration is `not_required` is insufficient for these completion joins.
Keep initial admission's existing checks. Retain raw reported status and every
existing fresh health, inactive-lease, identity/settings and baseline-ID check.

Share's legitimate work advances authorization high-water. Preserve the existing
fresh-session authorization-recovery checkpoint and its matched generation joins;
do not require the original comparison to remain true or normalize away a mismatch.
Unexpected advancement, stale checkpoint or wrong generation still fails.

Keep Share's existing 145,000-ms post-fault wait. The UI must disable restoration
recording until the coordinator-reported remaining interval expires on a monotonic
clock, and until fresh ready/explicit-restoration conditions hold. Latch that UI
deadline once when the run returns; later observations cannot restart it. The coordinator
continues to enforce the authoritative deadline. Reconnection or UI refresh cannot
restart or shorten the origin; UI elapsed time is not device-restoration proof.
Prevent duplicate export after success while keeping server-side exclusive writes.

## Two bounded live stages

Use one detector-admitted Ultra205, the existing fixed Serial/JTAG owner and fresh
native browser gestures. Keep the conservative profile and existing safety rules.

Channel006 performs installation0 and four complete state-preserving update/
reconnect/65,536-byte bidirectional-probe cycles, each on its own fresh admission
and the same private page baseline. Verify exact identity/settings, startup/heap
health and resource release. Run one authenticated local-fixture V2 exchange
through SetupConnection, Standard channel opening, target and job/WorkReady. No
ASIC work, grant, reservation or signing is permitted. Preserve the 120,000-ms
authority and 125,000-ms observation horizon, then restore, freshly reconnect,
collect accounting and fully release resources. Seal and independently pass the
private channel result before Share admission.

Share002 uses a new root/page/private baseline/permission/possession and performs
four fresh cycles (install1–4, no extra initial reinstall). Complete the existing
fan-only cooling proof and baseline restoration. Obtain fresh authenticated
accounting before issuance: expected next18, last17, charged1,560,000 ms, pending
false; original budget240,000 ms with masks7/7 and pendingfalse. Historical values
are expectations, never substitutes for the current reads.

Use one fresh signed normal allowance reserving the full 180,000 ms. Keep 60,000-ms
leases, renewal after20,000 ms only after completed Start/Active, the 30,000-ms Gate
Start-reply limit and 164,450-ms maximum work gate with 15,550 ms reserved for
shutdown. Preserve both existing ten-second checks: client fixture-request→Start
and fixture-ready→Start, each on its own host clock. Setup/source audits and human
readiness precede fixture launch; no refresh or retry extends a deadline.

Require a real BM1366 nonce, independently verified target/header/ASIC-payload
correlation, encrypted submission and fixture acceptance that the device actually
acknowledged. Keep400MHz,1100mV,fan100%, fresh4.5–5.5V, power≤15W, temperature\<75C
and nonzero RPM. Select a fully correlated accepted share and prove at least5,000ms
remaining on both lease and work gate before suppressing application heartbeats.
Stop renewals; require unchanged2,800-ms expiry and actual device revocation plus
shutdown initiation within3,000ms of the last valid advancing heartbeat. Keep the
passive observer through the fault plus at least5,000ms.

Retain ordered shutdown, fan100% until fresh≤45C then qualified30%, cooling within
120,000ms and the145,000-ms recovery wait. Reconnect natively, verify explicit
restoration and the post-work authorization checkpoint, and collect final ledgers.
Expected consumed completion: next19, last18, total1,740,000ms, pendingfalse. Charge
the full reservation even on failure or early success; never refund or reset it.

Use only the owned local V2 fixture and the existing protected Share authority
input. Never inspect, print or persist private signing material, runtime endpoints,
user strings or pipe contents. Channel rejects the authority input. No pool or
Wi-Fi credential files are needed. Preserve all base fixture/observer lifetimes,
connection inventory, redaction and cleanup requirements.

## Verification, failure and completion

Before effects, test the actual native-page orchestration and production
coordinator/routes/accounting joins for both scopes with clearly simulated device
boundaries. Test automatic close→wait→fresh ready→Record restoration, early
reconnect, monotonic wait handling, missing explicit restoration, matched and
mismatched authorization checkpoints, emergency abort, duplicate collection and
unchanged v1–v5 interpretation. UI hints never supply acceptance.

Compose Share preparation with the real repository fixture and test-only authority
material through the production signing path; verify the existing ten-second
admission window and cleanup. No owner-authorized or installed-device-usable grant, device access or real
credential read is part of this software test. Test keys and cryptographic artifacts
remain private and never reach the device. Reject already-expired preparation
before issuing or admitting further work. If signing or delivery overruns after an
issuance claim was consumed, retain actual issuance/delivery/load facts and that
claim, prohibit Start, and preserve failure. Do not relabel it pre-issuance or
refund/reuse it. Cover both early-expired and late-signing cases; do not solve
latency by moving a clock origin.

Test wrong/missing/changed predecessor pins, failed Channel006, cross-pair Share
admission, altered inventories, missing fresh accounting, wrong ordinal, duplicate
or interrupted assignment, source/validator drift, private-field rejection and
actual process/socket cleanup. Run ordered Cargo format/Clippy/build/tests,
affected Gate/supervisor checks, canonical Bazel tests, native packaging/resource
and ownership checks, standards, references, redaction, task-ID/dependency and
read-only parity/progress checks. Keep Cargo and Bazel verification separate.

On failure, retain the earliest cause and consumed claims, stop safely and preserve
all available accounting/cleanup evidence. Do not replay a context, relax a threshold
or start Channel007/Share003 under this amendment. Any later attempt requires a
verified targeted correction and a new published finite admission. Existing owner
authorization avoids another permission question; it does not waive these gates.

On complete same-pair success, seal and independently review both scopes, publish
only permitted projections and exact-pair reports, and archive the completed live
task with its full native record. Hand cumulative STR-005 promotion to its separate
task. Parity remains90/95 throughout this qualification work.
