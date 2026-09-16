# STR-005 Noise qualification through fixed Serial/JTAG

## Status and ownership

Prospective contract `str005-noise-serial-v1`, approved for specification on
2026-09-16. The commands and types below are **not implemented by publication of
this document**. This is not an effect permit, hardware result or parity claim.
Freeze this document's byte digest in every implementing context. A later
semantic or interface change requires a new linked contract-amendment task;
archived task records and historical evidence remain immutable.

| Owner task                             | Deliverable and boundary                                                                                                              |
| -------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| `task-str005-noise-successor-contract` | This reviewed contract and the dependent-task handoff; no runtime implementation or hardware                                          |
| `task-str005-noise-runtime-readiness`  | Firmware/Gate adapters, production-seam regressions and native resource proof; no connected-device effects                            |
| `task-str005-noise-fixture-evidence`   | Fixture, host supervisor, contexts, independent judge and closed projection; synthetic/loopback tests only                            |
| `task-str005-noise-auth-205`           | One fresh positive hardware exchange, restoration, cleanup, sealing and independent acceptance after both implementation tasks finish |

Runtime and fixture work may proceed in parallel against these interfaces. Final
fixture integration uses the published runtime candidate. Shared interface
changes require reviewed amendments; Gate remains the Controller
wire-specification owner.

The hardware milestone is one positive exchange. Wrong-authority and other
negative cases belong to production-seam software verification for this version.
Channel/job qualification, ASIC work, share submission, mining, external pools,
BWG's separate restoration/replay/clock matrix and STR-005 promotion are
excluded.

## Evidence baseline and permitted lifecycle

| Existing evidence or implementation                                                                                                                                                                                                                                    | What may be reused                                                                   | What must be newly proved                                                                        |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| [TCP projection009](../parity/evidence/str005-tcp-payload/tcp-payload-projection-009.json), firmware `e0398abb74d710d6a3918f226f1c08fd3203d35f`, ELF `0f92f18f43d2eb2e958da5a1e973b1783462497c53cdbc90ba5194cf7996f089`                                                | Design and evidence-join precedent for exact TCP delivery                            | TCP delivery on the successor pair through its own act-one receipt                               |
| [Accepted CPU0/USB qualification](../parity/evidence/20260915-cpu0-cadence-qualified.md), firmware `8af54d8d8919aeb0f309203f790269803365fb9a`, Gate `72235d884c3605ff77d484f210fcaa0d7218b47e`, ELF `35ea53e7a4153fb17f21b8bb8b64653506bb909be1bd9d61cb7e96b130820b6a` | Fixed ownership, fresh possession, preservation, cleanup and bounded-stop mechanisms | Fresh identity, baseline, resource health and restoration on the Noise candidate                 |
| [September 11 Hello recovery](../parity/evidence/20260911-hello-recovery-live.md)                                                                                                                                                                                      | Fresh-session recovery design at its recorded pair                                   | Current-pair recovery after this diagnostic; no transfer of an old runtime measurement           |
| [August Noise plan](../parity/work-plans/20260829T143226Z-STR-005-NOISE-AUTH/PLAN.md) and diagnostic001 failure                                                                                                                                                        | Historical conclusions and reusable crypto/fixture logic                             | A new runtime lifecycle, namespace and evidence version; no recovery006 or old-command authority |

Use the ordinary qualified boot path, Controller 0.4, serial 0.2 and possession
0.2. Keep the Worker controller available throughout. Do not use the old
consume-once NVS diagnostic tuple, seed Wi-Fi credentials, alter `stratumprot`,
modify persistent pool settings or select the old alternate boot owner.

The future live task permits detector admission of exactly one Ultra 205,
state-preserving installation of the frozen candidate through existing repo
commands, four fresh update/reconnect/max-frame cycles for a changed runtime
pair, direct browser possession, one local fixture exchange and state
restoration. Same-pair cycle reuse requires independently verified
exact-pair/baseline/hash continuity; unrelated historical cycles cannot be
substituted. Capture the fresh private preservation baseline before the first
admitted update and compare it after installation and every cycle. Missing
baseline proof blocks writes.

The validated predecessor supplies the prior exact runtime identity, artifact
receipt and original campaign ID privately; it supplies no reusable page
baseline. Use the candidate Gate page with explicit, separately pinned
before/candidate identity configurations and one retained in-memory preservation
baseline across updates. Compatibility with both exact identities is a
runtime-readiness gate, not an open-ended identity allowlist.

Preserve the physical-device lease, same-device ROM board-info admission,
disjoint write segments, NVS/identity/replay state and existing bounded flash
operations. The fixed-serial startup capture is 30 seconds per installation,
using the existing qualified capture/assessment path. Browser streams must be
released before CLI ownership, with actual holder/process cleanup before reuse.
No factory reset, erase, direct UART/pins, network discovery, arbitrary control,
implicit reset or automatic rollback is permitted.

`serve` owns installation coordination: at most five writes (initial install0,
then updates1–4). Each has an exclusive `install-N.claim.json`, consumed before
child creation/launch and retained after ambiguous launch/result. Invoke only
`just flash-monitor --board 205 --port <admitted-port> --manifest <frozen-canonical-manifest> --evidence-dir <new-install-child> --capture-timeout-seconds 30 --redact-evidence`.
Prearm its process observer, require actual browser release/zero holders, and
validate exact healthy startup and owner release before reconnect/probing. There
is no sixth write, retry of a claimed installation or recovery reflash.

The fixture uses one literal private IPv4 address on the admitted host interface
and an ephemeral listening port. Match that interface's actual subnet to a fresh
authenticated station IPv4 observation; reject absent, stale or ambiguous
selection. No hostname/DNS, address retries, old-log endpoint fallback or pool
credential files. Generate fresh fixture authority material in memory for each
attempt and pass only its required public key as volatile diagnostic input.

Require no active/pending Work Lease, pending replacement possession, unfinished
restoration or other effect owner, and `mine_on_boot=false`. Issue no Work Lease
or mining allowance. Read both accounting ledgers before and after; their values
must remain identical and agree with validated predecessor evidence. The latest
accepted qualification baseline is next18/last17/1560000ms, pending false; the
original campaign is exhausted at240000ms/masks7/7. These are expectations to
verify freshly, not permission to overwrite drift. Require no new work dispatch
or share submission, rather than zero historical cumulative counters. Sample
those counters immediately before diagnostic Start on the final candidate boot
and after restoration; do not compare counters across installation resets.
Ordinary safe startup and independent safety actions must be recorded separately
from the prohibited diagnostic-induced ASIC/fan/voltage effects; do not claim
that hardware was never touched during installation.

## Frozen prospective interfaces

### Host command boundary

The new family is `just stratum-v2-noise-serial <action>`. It must route through
the canonical Bazel graph. It must not alias or widen the old
`stratum-v2-noise-auth` effectful entrypoints. Retire legacy effect admission
under this successor; retain historical read-only validators and schemas.

All actions accept `--private-root <absolute-root>`. Only `preflight` also
accepts `--firmware-root`, `--gate-root`, `--package-manifest`,
`--fixture-binary`, `--attempt-ordinal`, and `--predecessor-receipt`, each with
one value. Sources come from clean published HEADs; the fixture must be the
canonical build product. Other identities and this contract's digest are derived
and frozen, not trusted caller assertions. `recover` additionally requires
`--attempt-root` naming the original context. `finalize` accepts
`--cleanup-receipt` and an optional `--recovery-receipt`. `serve` alone may
accept `--port` for its loopback server, defaulting to an OS-assigned port.
Reject unknown or duplicate flags before sensitive input access. No pool/Wi-Fi
credentials, authority-directory, grant, mining-budget, arbitrary URL, image
override or force flag is accepted.

| Action      | Meaning and allowed effects                                                                                                                                                                                                                                                |
| ----------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `preflight` | Device/network-effect-free validation of the active live task, completed implementation receipts, published sources, package/reference/fixture/validator/contract identities and fresh protected root; exclusively reserves the host attempt and snapshots exact artifacts |
| `serve`     | Exposes only the task's loopback browser workflow; performs fresh baseline/accounting/continuity admission and starts the bounded fixture/job once from the admitted operator trigger; no Work Lease signer/grant routes                                                   |
| `recover`   | Creates a separate exclusive recovery-only child bound to the original attempt; fresh same-device possession, cancellation, retained evidence collection and state restoration only; no diagnostic restart, flash or reset                                                 |
| `finalize`  | No hardware or network effects; joins immutable diagnostic/restoration evidence and independently observed host cleanup after owners exit, writes the terminal seal and separately validated public projection exclusively                                                 |
| `review`    | Read-only revalidation of seals, inventories, joins and verdict; never rewrites an outcome or requires archived tasks to become active                                                                                                                                     |

Namespace: `scratch/str005-noise-serial/attempt-NNN` and sibling exclusive
`ordinal-N.json` assignments, beginning at1 only in this new namespace. `N` is a
positive host diagnostic ordinal, not a mining ordinal. Reserve the assignment
before child creation; interruption never frees it. Each context has a fresh
16-byte canonical base64url attempt ID. Recovery children are
`recovery-NNN-RRR`, with separately consumed recovery claims; they inherit no
diagnostic-start capability. One failed recovery requires verified progress and
a new bounded recovery context, never an in-place retry.

Parents are ignored mode0700 directories; files are mode0600; symlinks and
existing supervisor-owned children are rejected. Wrapper logs are distinct
siblings outside the initially absent child. Artifact snapshots include both
runtime sides, firmware images/manifest, fixture and all material validator
sources, with membership and byte digests. Private operational paths do not
enter the public projection. Publication uses the implementation's canonical
projection path, `docs/parity/evidence/str005-noise-serial/attempt-NNN.json`.

Preflight creates no fixture listener or device session. The fixture is started
only after native browser permission, fresh possession and all continuity gates.
No wall-clock deadline applies to a safe wait for a person. Start the automatic
budgets only after that admitted trigger; an expired fixture cannot be reused.
Fixture failure and host admission failure are sticky. Recheck fixture liveness,
its exact ready receipt, source bindings and the absence of terminal failure
after each awaited admission check, after the exclusive start-claim write and
immediately before sending Start. A claim-write/liveness race retains the claim
and fails closed; an earlier failure cannot be cleared by a late ready result.

### Controller and Gate boundary

Use existing envelopes, payload integrity, receive credits and the sole writer.
These are additive, candidate-specific commands, not a claim that every existing
Controller0.4 image supports them. Gate dispatches them only to the exact
compatible pair admitted by preflight. No diagnostic command extends possession,
heartbeat authority or any Work Lease.

| Command                   | Closed payload                                                                                                                                             | Response                                                                                                              |
| ------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `noise_diagnostic_start`  | `schema:"worker-noise-diagnostic-start-v1"`, `attemptId`, `expectedBootOrdinal`, `networkObservedAtUs`, `fixtureIpv4`, `fixturePort`, `authorityPublicKey` | `worker-noise-diagnostic-status-v1` in `admitted` state; not execution or delivery success                            |
| `noise_diagnostic_status` | `schema:"worker-noise-diagnostic-query-v1"`, `attemptId` (null only for idle/network observation)                                                          | Current observation or the exact retained job record; no mutation or rearm                                            |
| `noise_diagnostic_cancel` | `schema:"worker-noise-diagnostic-query-v1"`, non-null `attemptId`                                                                                          | Matching job with cancellation latched, or its existing terminal record; never an unsupported success for another job |

Reject unknown fields. `attemptId` is canonical unpadded base64url for16 bytes;
`authorityPublicKey` is canonical unpadded base64url for32 bytes; `fixturePort`
is1–65535; IPv4 is a literal private unicast address. Numeric counters, ordinals
and device timestamps must be nonnegative safe JSON integers, with positive
boot/attempt ordinals. The existing envelope and Gate request binding pin the
current session; the device pins its current generation/transport epoch and
rechecks them at dispatch. Do not invent an independently trusted session label
inside the payload.

Gate exposes `noiseDiagnosticStart(input, expectedBinding)`,
`noiseDiagnosticStatus(attemptIdOrNull, expectedBinding)` and
`noiseDiagnosticCancel(attemptId, expectedBinding)`, each returning a promise of
the status type below. `expectedBinding` is the existing Gate control-session
binding, checked before dispatch and on completion, not an extra wire field. All
three require authenticated possession; idle observation/new Start use fresh
admission, running status/cancel use the live job binding, and terminal review
after reconnect uses fresh admission plus the retained original job identity.

After the fixture is ready, query status again immediately before Start. The
device retains the last successfully delivered idle observation in that epoch.
Start must echo its exact `networkObservedAtUs`, match its
boot/generation/epoch, be no more than 5000 device milliseconds later, and still
have connected Wi-Fi with the same station IPv4. Clock rollback or
changed/disconnected station state rejects Start. The host also rechecks its
selected interface/subnet and fixture binding. A later query supersedes the
earlier observation; reconnect invalidates it. Host clocks cannot refresh it. No
MAC, SSID, credentials or full Wi-Fi snapshot is exposed.

The exact field dictionary below is authoritative. `job` is null only in `idle`.
Unobserved measurements remain null, never fabricated zeros. Observation
sequences strictly increase and usable device times are nondecreasing; repeated
status reads cannot manufacture progress. Retain the local socket port at
`tcp_connected`, actual transfer counts and operation durations. Missing stages
are not success.

Keep the first failure separately, latched at occurrence. Terminal outcome is
null until resources are released or their cleanup deadline expires, then
immutable: `accepted|rejected|cancelled|expired|incomplete`. It carries the
earliest closed failure stage/category, or no failure for `accepted`. Categories
are
`preparation|connect|write|read|authentication|proof|authority_lost|clock_invalid|cleanup|evidence_incomplete`;
the exact subcategories are frozen below. On timely cleanup, an earlier protocol
failure yields `rejected`, explicit/session cancellation yields `cancelled`, and
a deadline/heartbeat expiry yields `expired`; other non-expiry failures also
yield `rejected` when cleanup timing is proved. Cleanup deadline failure yields
`incomplete` while retaining the original first cause, plus a separate cleanup
failure. `accepted` requires no failure, protocol completion and confirmed
socket close/worker quiescence before the authority deadline; it is still only
the device verdict, not final acceptance. Explicitly record whether resource
release completed, when it completed and whether its deadline was met. A late
cleanup cannot erase an earlier violation.

Keep one bounded job record with append-only observations and an immutable
terminal through restoration and fresh-session review until reboot. Late
resource-release facts may fill previously absent fields but cannot revise the
terminal or make `deadlineMet` true after a missed deadline. This version
exposes no clear/rearm operation. An admitted job consumes that boot's
diagnostic slot even when its reply is lost; a new connection cannot revive it.
A later attempt requires its own qualified installation/boot and context, not an
implicit diagnostic reset.

The following field dictionary is normative JSON shape, expressed as TypeScript
for both implementation owners. Reject additional object fields. `UInt` is a
safe nonnegative JSON integer; `Digest` is64 lowercase hex characters; `Commit`
is40 lowercase hex characters. IDs/keys use the lengths/encoding above and
convey no authority by themselves. No strings below admit arbitrary exception
text.

```ts
type UInt = number;
type Digest = string;
type Commit = string;
type Stage = "noise_prepared" | "tcp_connected" | "act_one_written"
  | "act_two_received" | "authority_verified" | "proof_written"
  | "socket_closed" | "worker_quiescent";
type FailureStage = Stage | "admission" | "fixture_ready"
  | "candidate_inventory" | "proof_received" | "cleanup" | "evidence";
type Category = "preparation" | "connect" | "write" | "read"
  | "authentication" | "proof" | "authority_lost" | "clock_invalid"
  | "cleanup" | "evidence_incomplete";
type Detail = "timeout" | "eof" | "partial" | "extra" | "malformed" | "io"
  | "wrong_authority" | "certificate_time" | "session_replaced"
  | "heartbeat_expired" | "cancel_requested" | "clock_discontinuity"
  | "delivery_ambiguous" | "resource_unreleased" | "identity_conflict"
  | "peer_conflict" | "overflow" | "missing" | "stale" | "duplicate"
  | "rng" | "allocation" | "before_epoch" | "time_overflow";
type Cause = { stage: FailureStage; category: Category; detail: Detail };
type DeviceFailure = Cause & { atUs: UInt | null };
type Outcome = "accepted" | "rejected" | "cancelled" | "expired" | "incomplete";
type StageObservation = {
  stage: Stage; sequence: UInt; atUs: UInt | null;
  durationUs: UInt | null; bytes: UInt | null;
};
type Release = {
  socketState: "not_created" | "open" | "closed";
  workerState: "not_started" | "running" | "quiescent";
  volatileInputsDisposed: boolean;
  startedAtUs: UInt | null; deadlineAtUs: UInt | null;
  releasedAtUs: UInt | null; deadlineMet: boolean | null;
  failure: DeviceFailure | null;
};
type NoiseJob = {
  attemptId: string; inputSha256: Digest;
  bootOrdinal: UInt; workerGeneration: UInt; transportEpoch: UInt;
  admittedAtUs: UInt; authorityDeadlineUs: UInt;
  localSocketPort: UInt | null;
  stages: StageObservation[];
  firstFailure: DeviceFailure | null;
  terminal: { outcome: Outcome; decidedAtUs: UInt | null } | null;
  resources: Release;
};
type NoiseStatus = {
  schema: "worker-noise-diagnostic-status-v1";
  state: "idle" | "admitted" | "running" | "cancelling" | "terminal";
  observation: {
    bootOrdinal: UInt; workerGeneration: UInt; transportEpoch: UInt;
    observedAtUs: UInt; stationIpv4: string | null; wifiConnected: boolean;
  };
  job: NoiseJob | null;
};
```

`inputSha256` hashes the Start payload using the existing canonical JSON
encoder; it is private correlation evidence, not a public hash of operational
data. Stages have at most8 entries, each key once, sequence starting at1 and
increasing by1; skipped stages remain absent. Byte counts apply only to
act-one/act-two/proof transfer stages. Other counts and unavailable durations
are null. Act-two length must equal the pinned Noise library's
responder-handshake constant. Nullable device times are permitted only for
unavailable cleanup/failure timing after a clock failure, never an accepted
result. Admission must have a usable clock. Overflow, duplicate/reordered stages
or contradictory state is an evidence failure. Accepted jobs contain all8 stages
in order, no failure, non-null valid timings, closed socket, quiescent worker,
disposed inputs and timely release. An unstarted worker/socket is distinguished
from one actually stopped/closed. Worker quiescence is recorded by the owning
supervisor after actual completion or join, not by the worker announcing that it
intends to exit. Owned secret buffers use zeroizing wrappers and bounded
ownership. Disposal does not claim erasure of opaque library internals or
whole-memory credential absence.

`durationUs` measures named operation entry through completion: crypto
preparation, single connect, act-one write/flush, complete act-two read,
authority verification, proof write/flush, and socket-close request through
confirmed closure respectively. For `worker_quiescent`, measure from
`resources.startedAtUs` (cleanup request) to the owner's observed
completion/join. Do not measure gaps since the preceding stage or infer an
unobserved duration from host time.

### Fixture and host evidence handoff

Add a prospective `noise-serial` fixture mode without changing historical
`noise-auth` semantics. `serve` launches its frozen canonical binary with
`--mode noise-serial --private-root <new-fixture-child> --listen-address <admitted-ipv4>:0 --expected-peer-address <fresh-station-ipv4> --attempt-id <attempt-id> --accept-timeout-seconds 120 --read-timeout-seconds 10 --lifetime-seconds 150`.
These new options/mode require implementation; they are not today's CLI. No
authority private key is passed through arguments, environment or files. Capture
both streams through bounded, separately identified sanitizers (at most 1MiB per
stream; overflow is a terminal failure), never inherited output.

The fixture writes exclusive protected `ready.json` and `terminal.json`. Ready
has exactly `schema:"noise-serial-fixture-ready-v1"`, `attemptId`, `listenIpv4`,
`listenPort`, and `authorityPublicKey` in the Controller encoding. The parent
records actual ready arrival and process identity, verifies the requested
interface, and freezes this receipt before Start. Fixture terminal has this
shape:

```ts
type FixtureTerminal = {
  schema: "noise-serial-fixture-terminal-v1"; attemptId: string;
  outcome: Outcome; failure: Cause | null; elapsedMs: UInt;
  expectedPeerConnectionCount: UInt; unexpectedPeerCount: UInt;
  candidateOverflow: boolean; selectedIndex: UInt | null;
  candidates: {
    remotePort: UInt; actOneBytes: UInt;
    readOutcome: "complete" | "partial" | "eof" | "timeout" | "io" | "malformed";
  }[];
  actTwoBytesWritten: UInt; proofBytesReceived: UInt;
  extraBytesReceived: UInt; encryptedProofExact: boolean;
  peerClosed: boolean; socketClosed: boolean;
};
type CleanupReceipt = {
  schema: "noise-serial-cleanup-v1"; contextSha256: Digest;
  browserClosed: boolean; browserOwnershipReleased: boolean;
  fixtureExited: boolean; fixtureExitCode: number | null;
  supervisorExited: boolean; supervisorExitCode: number | null;
  remainingOwnedProcesses: UInt; listenerAbsent: boolean;
  serialHoldersAbsent: boolean; deviceResourcesReleased: boolean;
  observedAtHostUnixMs: UInt;
  witnesses: { browser: Digest; fixture: Digest; supervisor: Digest; resources: Digest };
};
```

Fixture candidate arrays contain at most3 entries; selectedIndex is an in-range
index or null, and ports are1–65535. Count absent bytes as zero actually
observed, not as success. `elapsedMs` uses the fixture's own monotonic clock
from ready; exit codes are signed32-bit integers or null when unobserved. Only
an accepted terminal with one expected candidate, zero unexpected peers, no
overflow,64 act-one bytes, the exact act-two size and22 received encrypted proof
bytes decrypting to the exact6-byte empty-payload header can pass. Require peer
EOF within5000ms after the proof, zero extra bytes and actual socket closure.
The10-second read bound covers each complete read phase, not the entire fixture
lifetime. Candidate read time starts with the first expected connection; new
candidates do not restart it. After selection, observe a full500ms for extras,
even when selection occurs near the read deadline, still within the fixture's
absolute lifetime. Local socket closure alone does not establish peer EOF. The
fixture's crypto proof is distinct from device authority verification.

The cleanup parent creates a fresh sibling `<private-root>.cleanup` directory.
`--cleanup-receipt` names its `receipt.json`; the four digests bind fixed
`browser.json`, `fixture.json`, `supervisor.json`, and `resources.json` there.
They contain the existing qualified parent browser/exit observations and
process-identity/socket/serial snapshots, interpreted by the frozen host
validator source inventory. No arbitrary external witness paths are followed.
Every exit must be observed; zero process counts are freshly measured against
the original owner identities/groups. A stored receipt's booleans alone cannot
replace those checks. Original nonzero exits remain nonzero, even after cleanup.
Positive acceptance requires observed fixture and supervisor exit codes0, every
cleanup boolean true, and `remainingOwnedProcesses===0`.

### Authority, cancellation and deadlines

Start requires the existing fresh admission (younger than60000ms), an idle
baseline and an unconsumed slot. Reserve the diagnostic fence before preparing
the reply. Execution follows the existing completed-response dispatch barrier,
with fresh clock/session/generation/idle rechecks. A queued reply is not a
delivered reply. Consume the host start claim before sending; do not resend a
start whose reply was lost or ambiguous. Preserve that failure, cancel/close,
and collect retained evidence through fresh possession. A late receipt cannot
turn that failed admission into a new success.

While running, status/cancel use the admitted job's still-current authenticated
session and heartbeat binding, not a new start-age test or renewed admission.
After session replacement, only fresh possession can read the retained old job;
it cannot resume it. Block mining Start, restart, configuration mutation,
fan-only qualification and competing effectful diagnostics until quiescence. Do
not rely on today's idle `safe_stop` path, which does not yet own this job.

| Bound                         | Definition                                                                                                                                                                                                                                                  |
| ----------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Diagnostic authority          | 120000ms from device admission, including reply delay and every successful-operation phase/cleanup; never restarted by status, progress or reconnect                                                                                                        |
| Crypto preparation            | 60000ms absolute from entry, also bounded by remaining diagnostic authority                                                                                                                                                                                 |
| TCP connection                | One address, one attempt, at most5000ms                                                                                                                                                                                                                     |
| Act-two reception             | 10000ms total, including every partial read                                                                                                                                                                                                                 |
| Each write and flush          | 2000ms combined for act one and separately for the encrypted proof                                                                                                                                                                                          |
| Heartbeat revocation          | Existing2800ms from last advancing valid heartbeat, enforced independently of crypto/I/O                                                                                                                                                                    |
| Cancellation/terminal cleanup | Socket close and worker quiescence within5000ms from the earliest cancellation or protocol terminal decision; expiry at120000ms permits cleanup-only work through125000ms, never later acceptance; host children also have5000ms to close/reap when stopped |
| Fixture                       | Ready within5000ms of launch; maximum150000ms lifetime after ready; inventory at most3 candidates, read bound10000ms and500ms post-selection observation; success requires exactly one expected peer and no unexpected peer/overflow                        |

Use absolute deadlines; progress cannot restart a per-call timeout. Clock
discontinuity revokes authority. Check the fence before every new protocol I/O
and successful completion. Cleanup-only socket close/disposal and retained
non-success/cleanup observations remain allowed after revocation; no further
handshake/proof traffic does. Restore/Cancel, close/disconnect, session
replacement, heartbeat expiry and deadline expiry all latch cancellation. The
diagnostic must not delay ordinary work revocation or existing safety tasks.
Later read-only delivery of a timely retained accepted terminal is valid; it is
not a new acceptance decision or permission to resume effects.

The old synchronous preparation check runs only after crypto returns; it does
not meet this contract by itself. Runtime readiness must demonstrate independent
deadline/revocation handling and bounded worker cleanup, or block hardware.
Revoked authority is not proof that the thread stopped. If cleanup cannot be
proved in time, retain `incomplete`, deny new effect owners, preserve evidence
and stop for a separately reviewed recovery contract. Do not kill a firmware
thread unsafely, reset/reflash implicitly or fall back to the old boot mode.

Crypto must not execute on the control, receive or main telemetry stacks. The
old24-KiB diagnostic owner replaced normal owners; it proves no fit alongside
them. Keep existing stacks, affinities, priorities, USB/Wi-Fi buffers, reserve
and safety deadlines unchanged. Before installation, runtime readiness must pass
native image/static/stack and allocation-failure checks for the combined owner
set. During the live task's admitted installation/cycles, require fresh healthy
startup and internal-heap/resource observations before diagnostic Start. This
assigns no device access to the software preparation tasks. If either gate
fails, block the next effect and require a reviewed amendment for changed
limits.

## Evidence, recovery and acceptance

Use a new closed `bitaxe-stratum-v2-noise-serial-projection-v1`, not the
historical Noise projection schema. Firmware, Gate and fixture observations
remain separate inputs. The independent evaluator derives the verdict;
caller-provided success booleans or hashes cannot replace verification. Bind
every materially reachable validator and inventory member by path and source
bytes in build/runfiles data.

The public projection has the exact shape below. All criteria must be true and
all numeric evidence present for `accepted`; unverified private results do not
produce this admitted projection. Failure reports remain separately redacted
non-promotional reports. Register this closed schema with the existing semantic
redaction verifier, with no exception or wildcard bypass.

```ts
type NoiseProjection = {
  schema_version: "bitaxe-stratum-v2-noise-serial-projection-v1";
  status: "accepted"; board: 205; diagnostic_ordinal: UInt;
  source_commit: Commit; gate_commit: Commit; reference_commit: Commit;
  provenance: Record<"app_elf" | "package_manifest" | "contract" | "fixture"
    | "evaluator" | "sealed_inventory" | "private_result", Digest>;
  criteria: Record<"identity" | "continuity" | "authority" | "tcp_delivery"
    | "noise_authentication" | "encrypted_proof" | "no_new_work"
    | "preservation" | "accounting" | "restoration" | "cleanup" | "privacy", true>;
  timings_ms: Record<"preparation" | "connect" | "act_one_write"
    | "act_two_read" | "proof_write" | "diagnostic" | "device_cleanup"
    | "fixture_lifetime" | "host_cleanup", UInt>;
  counts: Record<"exact_peer_connections" | "unexpected_peer_connections"
    | "act_one_written" | "act_one_received" | "proof_written"
    | "proof_received" | "new_work" | "new_shares", UInt>;
  redaction_status: "passed";
};
```

Round positive microsecond durations upward to whole milliseconds for public
reporting; judge original device microseconds before rounding. Host durations
use their own identified monotonic clocks. The private `final-result.json`
contains exactly `schema:"noise-serial-result-v1"`, `contextSha256`,
`status:"passed"|"unverified"`, `firstFailure` (Cause or null), `outcome` (one
progress-policy outcome), and `inputs` (closed named digests for
`deviceJournal`, `fixtureReady`, `fixtureTerminal`, `preservation`,
`accounting`, `cleanup`, and `recovery`, with null only for absent/unneeded
evidence). The contract's frozen host validator owns those private evidence
interpretations. Preserve every missing-proof reason in private classification;
no null can satisfy a required acceptance join. Write this result before sealing
its inputs; build the public projection afterward from that sealed inventory,
avoiding a self-referential projection/inventory digest. No private paths or IDs
enter the projection.

| Requirement               | Required evidence and join                                                                                                                                                            |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Exact candidate           | Clean published firmware/Gate/reference, ELF/package/artifact, contract, fixture and evaluator identities; fresh signed serial identity and private device-baseline comparison        |
| Fresh authority           | Exclusive ordinal/context/start claims; original boot/session/generation binding; ordered admitted/run/terminal records; no duplicate execution                                       |
| TCP delivery              | Device reports64 act-one bytes written and fixture receives exactly64, joined by private local/remote socket tuple and one admitted device/attempt                                    |
| Noise and encrypted proof | Device verifies the configured authority and reports22 proof bytes; the same fixture connection decrypts the exact empty diagnostic frame; no channel/job/share traffic               |
| Time bounds               | Device-local stage and deadline evidence, separate host fixture/cleanup durations; no host-minus-device timestamp arithmetic                                                          |
| No mining/effects         | No Work Lease or allowance issuance/consumption, no new work/share counters, no diagnostic-driven ASIC/fan/voltage action; record ordinary startup/safety separately                  |
| Preservation/accounting   | Fresh same-pair identity/settings/replay comparison and unchanged authenticated before/after ledgers; no active or pending grant and mine-on-boot false                               |
| Restoration               | Candidate remains installed; diagnostic socket/worker stopped, volatile diagnostic inputs disposed, healthy runtime, fresh same-device possession and retained original job join      |
| Host cleanup              | Actual fixture exit, browser stream/lock release and closure, supervisor exit, listener/owned-child/USB-holder absence; incomplete supplementary observations supply no cleanup proof |
| Durable result            | Final ordered journal flush and immutable artifact inventory; independent verdict; eligible redacted projection only after restoration and all cleanup are proved                     |

No separate fixed-pattern TCP phase is required: act-one receipt supplies fresh
delivery evidence on the candidate. No static device-certificate authentication
is inferred from the fixture possessing negotiated transport keys. The fixture's
certificate interval does not establish synchronized device wall time. Preserve
checked device `SystemTime` conversion to Unix seconds in the u32 range;
before-epoch or overflow values fail closed without substituting a time. The
positive fixture retains the existing certificate interval starting at epoch0
with u32::MAX-second validity. Do not add time synchronization or bypass
certificate validation; software tests use controlled times for expiry/future
and invalid-clock cases.

Normal restoration uses the same candidate, fresh possession and the existing
bounded serial reacquisition/restoration operations. It does not install an old
image. A separate recovery-only context may cancel and collect the original job
and prove restoration; it cannot extend authority or clear a consumed slot. Loss
of retained evidence, a reboot, source/identity conflict, an uninterruptible
owner or unproved restoration leaves the original diagnostic unverified. Retain
the same loaded Gate page and its in-memory preservation baseline during
fresh-session recovery; close its serial stream before binding the recovery-only
context. A new page cannot import or recreate that baseline. If the page or
baseline was lost, recovery may establish present safety/accounting and cleanup,
but original preservation remains unverified and the original result cannot
pass. Recovery rebinds the original loopback origin recorded by `serve`, only
after that server has exited. Reconfigure the same page to the bound recovery
child; if that origin is unavailable, do not reload onto another origin or
manufacture a replacement baseline. The predecessor reader supplies
`original_campaign_id` privately for accounting; no extra campaign-credential
file is accepted.

A host workflow may wait safely for its planned fresh-possession recovery with
all original writers stopped and inputs frozen, without declaring a terminal
workflow result yet. `recover` may complete that unfinalized restoration stage.
Any latched admission/protocol/deadline failure remains a failure even if
recovery later proves safety. Once `finalize` has sealed an unverified original
result, recovery can produce a separate safety receipt only; it cannot rewrite
or promote that original result. A failed automated reacquisition is not an
unbounded human wait. Apply its existing finite bounds and retain the failure.

The protected recovery receipt is `noise-serial-recovery-v1`, with exactly
`originalContextSha256`, `recoveryContextSha256`, `originalJournalSha256`,
`finalJournalSha256`, `cleanupReceiptSha256` (Digest), `outcome`
(`restored|unverified`), and booleans `baselineRetained`, `originalBootMatch`,
`jobMatched`, `identityExact`, `healthyRuntime`, `preservationVerified`,
`leasesInactive`, `mineOnBootFalse`, `accountingUnchanged`,
`deviceResourcesReleased`. The discriminator field is `schema`. A restored join
requires every boolean true and independently verified underlying observations;
late safety alone is not original preservation or diagnostic success. No further
diagnostic is eligible until cleanup/restoration is independently proved.
Recovery006's historical absence is neither erased nor a fallback.

Stop the fixture, flush the final device journal, release/close the browser and
stop/reap the supervisor before `finalize`. The server cannot certify its own
exit. A parent supervisor/operator records actual exits and fresh kernel cleanup
observations outside the terminated owner; independently hash-bind them to the
context. Snapshot classifier inputs before classification; finalize seals once
and creates a distinct shareable projection without rewriting private inputs.
Missing proof yields a sealed non-promotional result, never partial acceptance.

Apply the [evidence policy](../parity/evidence-policy.md) before the first
write. No live pool credentials are required or accepted. Fixture private keys
and secret-bearing handshake/cryptographic material stay in memory and are
disposed on cleanup. Raw control payloads and arbitrary log streams are not
evidence interfaces. Operational addresses, ports, identities, session bindings
and process details stay protected when retained; never publish low-entropy
hashes as purported anonymization. Public data is limited to closed categories,
comparison booleans, bounded counts/durations and safe provenance digests.

Every attempt receives one outcome from the existing
[progress-gated policy](hardware-attempt-policy.md). Preserve the earliest
authoritative signature and all failed/consumed roots. Another attempt needs
verified targeted progress, published sources, fresh admission and a fresh
ordinal. The same post-fix signature stops; renamed categories, waiting or an
unchanged rerun are not progress. Safe human waits have no elapsed deadline.

## Verification and publication gates

The following are obligations of the two implementation tasks, not tests claimed
executed by publishing this specification:

- Actual firmware/Gate parsers: missing/wrong authority, malformed certificates,
  expired/future certificate intervals and invalid-time policy; unknown/private
  fields, stale boot/session/nonce and duplicate admission.
- Production transport plus real local fixture: truncated, extra, altered,
  fragmented and slowly delivered bytes; write success without peer delivery;
  tuple mismatch, unexpected/duplicate peers and inventory overflow.
- Production owner seams: delayed/failed reply delivery, generation replacement
  before dispatch, preparation overrun, independent heartbeat revocation,
  cancellation during every phase, clock discontinuity, late completion and
  worker/socket cleanup failure. Prove exclusion of conflicting effects and
  unchanged accounting without live keys or device access.
- Actual Gate/supervisor/fixture composition: response-body consumption, final
  journal ordering, ambiguous admission without resend, recovery-only joins and
  parent-observed cleanup before finalization. A late success cannot overwrite
  an earlier failure. Simulator-only results do not replace these boundaries.
- Independent validator: missing/contradictory evidence, forged pass flags,
  immutable inventory drift, source/path/membership/evaluator drift, unknown
  fields, privacy leakage, archived-task admission and old-command rejection.
- Native resource review and ordinary combined-owner startup; no hardware
  negative, wall-time, arbitrary-load, other-board or full-callgraph claim from
  software-only or selected-path tests.

Require ordered Cargo format/Clippy/build/tests, applicable Gate and canonical
tests, native package/resource and ownership checks, reference integrity,
semantic redaction, standards and diff review before the live task becomes
eligible. Keep Cargo and Bazel verification runs separate. Hardware execution
requires both implementation tasks complete and their exact sources published;
build the clean package afterward. Do not resume an archived cadence/Hello
context or borrow its authority to qualify Noise.

For this specification task, verify source/requirement traceability, interface
consistency, evidence links, task IDs/dependencies, privacy and unchanged
historical evidence; run the repository's mandatory pre-commit Cargo and
standards/reference/redaction/parity checks. Archive its completed native task
record in the publication commit. Mark runtime readiness and fixture preparation
ready for their bounded software work, while live qualification stays blocked.
Close the live task only after the new independent hardware result passes.
STR-005 remains implemented and overall parity remains90/95 until the separate
channel/job, accepted-share and final promotion tasks satisfy their own
evidence.
