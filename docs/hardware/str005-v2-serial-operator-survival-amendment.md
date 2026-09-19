# Durable operator and interrupted Share successor

Contract ID: `str005-v2-serial-operator-survival-v1`.

This prospective amendment belongs to
`task-str005-durable-operator-share-successor`. The existing
`task-str005-v2-serial-qualification` owns hardware and mining. Publish this
reviewed contract before implementation, and verified implementation before
any effect. Standing authorization covers this finite continuation.

The immutable [base](str005-v2-serial-qualification.md),
[clock](str005-v2-serial-clock-amendment.md),
[permission](str005-v2-serial-permission-amendment.md),
[cleanup successor](str005-v2-serial-cleanup-successor-amendment.md),
[installation review](str005-v2-serial-install-review-amendment.md) and
[installation ownership](str005-v2-serial-install-ownership-amendment.md)
contracts remain historical inputs. Only the prospective admission and operator
interfaces explicitly defined here change. No historical reader or result is
rewritten.

## Exact interruption and non-claims

Share001 is permanently unverified. Pin its sealed identities before inspecting
operational paths:

- Context: `e96e658911fd438e222ba7d0088a8d8720e6542c4a93ed7a43914cea92a7b79e`.
- Result: `da7d9aa759a071febd83ec062fb046f360affe6c790dbc33d27ee65cc506c994`.
- Seal: `0449a33a703c9ccab27c92ed573156912449d9f67feabd8fb59d5c36aabc8aa1`.

The interactive parent disappeared after installation1. Its detached supervisor
survived and was later stopped by the resumed operator. The interruption time
and signal are unknown; reproducing a vulnerable owner lifetime does not prove
the cause of that historical disappearance. Preserve unknown original
parent and supervisor exit codes, the original reviewer disposition, actual
resumed cleanup provenance and the missing fresh post-install baseline and
accounting. Present process absence is not historical zero exit or restoration.
No passing cleanup receipt may be manufactured from these observations.

The trusted installation supports expected installed firmware
`0a9b29bf96b4d4797b46128f0d3850755f5637f2`, ELF
`c24bcfee9a2feb9f3932ce7b7f1f81cc012360420b47329f4f5c11bded218d52`.
Fresh authenticated possession must establish that identity, a new private safe
baseline and both ledgers before any successor write. Do not transfer the failed
page's baseline or infer settings preservation across its missing observation.

Accepted Channel004 remains valid for its actual pair only: result
`68997b3cb71933a8aae0715f2f659e0d0c5253cfc9f33e95437572efc449b907`, seal
`16fd7dddce80530c3b2f1b4b0b94ce6592a51c99ee5008b75bc4c602ab95beea`.
The new implementation uses the current clean published firmware, driver and
validator together. It must qualify a new Channel005 and same-pair Share002;
it does not introduce separate runtime and driver provenance roots.

## Persistent operator boundary

Add repository-owned commands under `just stratum-v2-serial`:

- `operator-start --private-root ROOT` exclusively starts one detached operator
  daemon for the assigned context and acknowledges creation/initialization within
  10000 ms with the closed `str005-v2-operator-status-v1` response below in
  phase `initializing`, without claiming hardware readiness. Share requires
  `--authority-directory DIRECTORY`; Channel rejects that option before accessing
  it. The daemon retains this private path without emitting or persisting it.
- `operator-request --private-root ROOT --request FILE` submits one protected,
  typed request to that existing daemon and returns an acknowledgment or result.
- `operator-status --private-root ROOT [--request-id ID]` reads daemon status or
  an existing request's pending/terminal state without replaying it.

The daemon launches the existing repository-owned supervisor, which performs
full admission; the launcher does not duplicate complete ancestry inspection.
Attach the actual ChildProcess exit observer immediately upon creation. A cheap
structural/source/snapshot bootstrap may precede full supervisor admission but
creates no device authority. For context-v5, public `serve` must verify the
actual admitted daemon parent and its one-shot IPC startup binding before
authority input is accessed; a private fixed supervisor entrypoint may enforce
the same condition. Reject direct unmanaged serve. No production flag or
environment variable bypasses this requirement.
After supervisor admission, bounded `loadEffectContext` establishes readiness
against the actual live owner. Automated initialization has a 900000-ms ceiling;
there is no idle human-response deadline. Readiness is reported only after all
existing admission checks pass.

The daemon owns and observes the actual supervisor child from creation through
exit, retains the opaque cleanup handle in memory, and survives CLI exit,
standard-input EOF, PTY closure and client disconnection. It is independent of
an interactive shell or long-lived tool session. Reconnecting a short client
addresses the same checked daemon identity; it does not launch, adopt or replace
an owner. If the daemon itself dies, the attempt becomes unverified: no restart,
process adoption, cleanup-handle reconstruction or effect replay is permitted.
Existing bounded, identity-checked failure cleanup may stop remaining owned
resources without upgrading missing observations. Explicit operator stop or a
handled signal must not abandon in-flight children: finish an already admitted
bounded operation, or preserve failure and perform identity-checked cleanup.
Never arbitrarily interrupt a flash. This contract does not promise SIGKILL
survival or a zero exit after unexpected daemon loss.

Use one owner-only Unix socket in a fresh canonical OS-temporary directory with
mode0700 and a socket pathname of at most103 UTF-8 bytes, satisfying Darwin
`sun_path`. Keep its protected endpoint/owner locator under `ROOT.operator`
with mode0600; never place the socket in the sealed campaign root. Request,
status and locator claims also belong in that protected `ROOT.operator` sibling.
Genuine child observations and canonical cleanup receipts remain in the campaign
root. Before finalization, stop and establish absence of the daemon, require its
complete source-bound disposition, and ensure no socket or live writer remains
inside the campaign root. Absence never supplies a daemon exit code.
Require owner UID, 0700 directories, 0600 files/socket, no symlinks, exclusive
creation and recorded PID/start/process-group identity. Reject an occupied or
stale socket instead of unlinking it to restart. Never expose a TCP operator
control service. Bound each JSON message to 16384 bytes and each client exchange
to 5000 ms; exceeding either fails the client without cancelling or restarting an
already admitted operation. Serialize effectful operations. Long operations run
inside the daemon under their existing bounds and are queried by short clients.

Requests have exactly `schema`, `contextSha256`, `requestId`, `action` and
`payload`; schema is `str005-v2-operator-request-v1`. `requestId` is a fresh
32-character lowercase hexadecimal identifier. Actions and closed payloads are:

| Action            | Payload                                                        |
| ----------------- | -------------------------------------------------------------- |
| `install`         | `{index}` using the existing scope-specific installation range |
| `prepare-cleanup` | `{}`                                                           |
| `finish-cleanup`  | `{browserWitness}` containing the v3 witness below             |
| `stop`            | `{}`; only after bounded cleanup and proven `hostStopped`      |

Status and ordinary readiness queries are read-only and never authorize device
operations. Their closed IPC shape has exactly `schema`, `contextSha256` and
`maybeRequestId`, with schema `str005-v2-operator-query-v1`; the ID is null or
a valid request identifier. Queries carry no action or payload.
With a request ID, responses have exactly `schema`, `contextSha256`, `requestId`,
`status` and `code`; schema is `str005-v2-operator-response-v1`, status is
`pending`, `succeeded` or `failed`, and code is null or a fixed allowlisted
category. Without a request ID, status has exactly `schema`, `contextSha256`,
`phase`, `maybeRequestId`, `maybeAction`, `maybeIndex`, `maybeCode`,
`maybeSupervisorOrigin`, `lastSequence` and `lastStateSha256`. Schema is
`str005-v2-operator-status-v1`; phase is `initializing`, `ready`, `busy`,
`failed`, `stopping` or `stopped`. Nullable fields use null when unavailable.
Request/action/index/code values use their request-domain types; journal
sequence/hash represent the last actual persisted row for an independent
browser witness. The only permitted supervisor origin is numeric-port
`http://127.0.0.1:PORT`; this protected loopback metadata may be returned.
Never return a fixture endpoint or authority-directory path. Persist terminal
status so clients can read it after a clean stopped actor without adoption,
relaunch or a new socket owner.

Persist exclusive request acceptance before dispatch. Compare canonical JSON
body bytes after strict parsing, so whitespace does not create a distinct
request. An identical request ID and canonical body returns existing status; a conflicting body
fails. Interrupted acceptance remains consumed. Client timeout or missing result
never authorizes another ID for the same phase. Phase state and original effect
claims independently prohibit duplicate installs and cleanup execution.

Revalidate clean publication, context/source/artifact/native identities, current
owner identity and the original operation's prerequisites at dispatch. Preserve
the detector freshness check after the final await. Neither IPC nor daemon
liveness extends possession, heartbeat, Work Lease or preparation deadlines.
Actual child exit remains an observed exit event; process absence alone cannot
supply an exit code. Retain the existing failed-only bounded reaping behavior.

## Native page controls and browser provenance

Add fixed visible controls to the existing qualification page for recording
accounting, configuring the candidate, recording the next cycle, running the
scope, restoration and journal flushing. Each calls its existing coordinator
method with fixed phase-derived arguments and displays typed completion/failure.
Connect remains the genuine foreground native Web Serial gesture. This workflow
must work through normal browser UI without a debugger, CDP or injected script.

Do not reload the page, export/import its private baseline, call controller
internals or synthesize browser events to recover interrupted coordination.
Loss of that page's private baseline makes restoration unverified. Browser
navigation or client reconnection cannot grant new application authority.

A separate actual UI observer closes the exact owned tab only after the page
reports closed, disconnected, serial ownership released and its final journal
has flushed; it verifies that tab is absent. It supplies the daemon a witness
with exactly `schema`, `source`, `contextSha256`, `closed`, `lastSequence`,
`lastStateSha256` and `observedAtUnixMs`. Schema is
`noise-serial-browser-closure-v3`, source is `native-ui-observer`, and closed is
true. Sequence and digest bind the final actual journal row; timestamp is a
nonnegative integer no later than collection. The daemon validates and persists
the witness unchanged. It must not label its input as parent-observed or claim
that it inspected browser tabs. A caller assertion unsupported by actual UI
observation is not evidence. The validator accepts v3 only for the new context;
v1–v4 historical witness rules stay unchanged. Missing witness permits failure
cleanup but never a passing receipt.

## Guarded Share interruption readiness

Add `prepare-share-successor --private-root FAILED` and
`review-share-successor --private-root FAILED`. They exclusively create or
read-only verify deterministic protected sibling
`FAILED.share-successor-readiness.json`. These are effect-free commands: no
hardware, credentials, signing authority, fixture or new effect context. They
never modify the sealed tree. Reject pending/existing receipts, unsafe paths,
changed membership/bytes, conflicting failures or incomplete proof.

This classifier accepts only exact sealed Share001 above. Independently verify
its full protected inventory, context/assignment, original ordinal marker,
thirteen artifacts, complete transitive source/native/evaluator identities,
accepted Channel004 and its ancestry, original verdict and sealed cleanup
observations. Verify authenticated initial accounting, closed-page release,
exactly installation1's admission/claim/physical identity/actual exit and trusted
healthy candidate receipt. Require absence of cycles and later installations.

Permit exactly `signer-01.exit.json`, index1 and operation `public-trust`,
with its genuine zero exit, null signal, nonempty bounded stdout, zero stderr
and no overflow/input failure. Verify its context/source join and subsequent
successful server admission: the captured implementation compares the returned
trust against deployment trust before creating that server. The raw returned
trust was not retained; do not invent a result artifact or an independent
retrospective comparison. This lookup is not Work Lease signing. Reject any other signer operation, Work Lease signing, `issuance.claim`,
issued or loaded grants, reservation consumption/pending reservation, fixture
creation, work, shares, renewals, observer or heartbeat suppression. Check both
complete inventory and journals; filename absence alone is insufficient.
Initial ledger must remain next18, last17, charged1560000 ms, pending false,
and the original exhausted240000-ms ledger remains unchanged. These are initial
observations, not a replacement current review.

Collect fresh absence of every recorded owned actor, process group, descendant,
serial holder and known supervisor listener using checked identity and the
existing detached ownership boundary. Retain unidentified parent references
conservatively; never include unrelated external launcher ancestors as owned
resources. No fixture or private pool-port absence is invented.

Receipt schema is `str005-v2-share-successor-readiness-v1`. Its exact fields are
`schema`, `amendmentSha256`, `failedRoot`, `failedContextSha256`,
`failedResultSha256`, `failedSealSha256`, `status`, `classification`,
`hardwareQualified`, `historicalCleanupComplete`, `afterBaselineObserved`,
`beforeSource`, `predecessor`, `initialAccounting`, `ordinalMarker`,
`inspectedInputs`, `currentOwnership`, `checkerIdentity` and `nonClaims`.
Status is `unverified`; classification is `ready_for_fresh_channel`; the three
boolean proof fields are false. Reuse the prior installation-readiness closed
shapes for beforeSource, predecessor, initialAccounting, inspectedInputs,
currentOwnership and checkerIdentity. Predecessor binds accepted Channel004;
ordinalMarker is exactly `{path, sha256, length}` for the unchanged original
ordinal18 marker. Its path is exactly `qualification-ordinal-18.json`, resolved
only beside the failed root; it is not an arbitrary operational path. NonClaims is exactly: `historical-parent-exit`,
`historical-supervisor-exit`, `post-install-baseline`, `post-install-accounting`,
`historical-cleanup-complete`, `mining-acceptance`, `cycle-or-baseline-transfer`,
`effect-authority`.

Historical receipt review checks recorded source identities against Git and
sealed inputs without recollecting present ownership. Recollect current absence
immediately before assignment and serve. A receipt grants no effect authority.

## Finite context and allowance assignment

New admissions use `str005-v2-serial-context-v5`; v1–v4 remain readable but cannot
regain effect eligibility. Add exact document digest contract binding
`operatorSurvival`. Preserve complete current clean published pair, package,
Gate, observer, fixture, native and transitive validator/source bindings.

Add `preflight --supersede-share RECEIPT`, mutually exclusive with all earlier
supersession options and accepted only for Channel005. Channel005 requires this
exact Share001 readiness. Share002 rejects supersession flags and inherits the
readiness binding from its independently accepted Channel005 predecessor. Add
closed context field `shareSupersession` containing exactly `failedRoot`,
`failedContextSha256`, `failedResultSha256`, `failedSealSha256`, `receiptPath`
and `receiptSha256`. Both Channel005 and Share002 retain that same binding;
permissionSupersession and cleanupSupersession are null. Channel005 keeps the
accepted Noise protocol ancestor and the inspected installed beforeSource.
Prove that Noise ancestor through receipt predecessor Channel004's verified
ancestry; do not require direct equality between Noise and receipt predecessor.
Share002 requires independently accepted same-pair v5 Channel005 and its exact
candidate beforeSource. It inherits and revalidates the same readiness binding.
No other Channel or Share ordinal is eligible under v5.

Preserve `qualification-ordinal-18.json` byte-for-byte. Exclusively create
`qualification-ordinal-18.successor-2.json` for Share002. Its closed schema
`str005-v2-qualification-ordinal-successor-v1` has exactly `schema`, `ordinal`,
`originalMarker`, `failedContextSha256`, `failedResultSha256`, `failedSealSha256`,
`receiptSha256`, `contextSha256` and `attemptId`. OriginalMarker is the original
marker `{path, sha256, length}`; ordinal is18. Bind all hashes before publishing
through the existing exclusive pending-file protocol. Duplicate, conflicting or
interrupted assignment stays consumed; neither old marker nor device ordinal
is reset. A host successor assignment is not a funded reservation.

## Verification and bounded qualification

Reproduce the old parent-loss boundary with real processes first. Then prove the
new daemon survives EOF, PTY/client loss and reconnect; pending operations run
once, queries do not replay effects, and actual supervisor exits remain observed.
Exercise real owner-only Unix sockets, fragmented/coalesced/malformed/oversized
requests, conflicting IDs, stale owners, daemon death, missing results, private
cleanup-handle retention and actual finalizer/reviewer composition. No production
test flag or caller-supplied pass verdict is allowed.

Test native page controls, phase/duplicate rejection, baseline loss and v3 UI
witness joins/source/timing. Mutate the precise Share failure inventory, signer
operation, issuance claim, ledger, markers, ancestry, sources and cleanup proof;
all unsupported classes must fail. Test archived tasks, interrupted assignments
and v1–v4 historical verdict preservation. Re-run ordered Cargo checks, Gate,
canonical Bazel, native/package, ownership, reference, privacy, standards and
parity/progress checks without overlapping Cargo and Bazel. Bind verified
corrections before publishing implementation and building the clean package.

After publication, prepare/review readiness, reserve Channel005, detector-admit
one Ultra205 and freshly authenticate the expected installed pair plus both
ledgers before any write. Collect a new baseline. Perform initial installation
and four fresh update/reconnect/65536-byte bidirectional cycles, the Channel
exchange, restoration/accounting and complete cleanup; seal and independently
review. Do not change HEAD/pair before Share002.

Share002 requires a fresh page/baseline, authenticated exact same-pair identity
and both unchanged ledgers before its first write, four new same-pair cycles,
then fresh
next18/last17/1560000-ms/pending-false accounting before signing. Consume one
normal180000-ms reservation. Prove a real accepted BM1366 share and its device
acknowledgment, heartbeat suppression, revocation and shutdown within three
seconds, ordered stop/cooling, restoration and complete actual host cleanup.
Expected completed ledger is next19, last18, charged1740000 ms, pending false.
No refund or ledger reset is permitted.

All existing thresholds and bounds remain: conservative400MHz/1100mV profile,
2800-ms heartbeat deadline, work-clock/renewal limits, observer360-second
lifetime, recovery wait, cooling and resource margins. No external pool,
factory reset, automatic rollback, crypto fork, UART/pin access or fallback
control transport is introduced. Keep keys, credentials, grants and raw private
endpoints out of logs, IPC status and evidence; retain private cleanup material
only in its existing owning memory. Persist only allowlisted counts, timings,
identity hashes, categories and provenance.

Preserve earliest failures and finish possible bounded cleanup. This amendment
admits only Channel005 and Share002; further failure needs verified correction
and another applicable prospective admission contract. No old result gains
acceptance. The software task closes after verified publication; live
qualification and separate parity promotion remain independently gated. Parity
stays90/95 until the separate promotion task passes.
