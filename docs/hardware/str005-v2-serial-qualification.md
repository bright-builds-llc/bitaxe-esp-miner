# STR-005 V2 Serial channel and accepted-share qualification

## Contract status and authority

Frozen prospective contract under `task-str005-v2-serial-qualification`, owning
both stages. Contract ID: `str005-v2-serial-v1`. Reviewed on 2026-09-16; later
semantic changes require an explicit amendment task. The earlier channel and
share tasks are prospectively superseded, preserving their native records and
unchecked historical items; neither becomes completed hardware evidence.
Interfaces below are frozen requirements, not presently available commands. This
document grants no device effects until implementation is verified and published
with the exact firmware/Gate/package pair. The active live task must authorize
the corresponding scope before its preflight succeeds.

The owner authorizes progression through real V2 mining after these gates; no
additional per-attempt approval is required. There are two separately judged
scopes on one candidate: `channel`, then `share`. Channel acceptance supplies no
mining authority. Share acceptance does not by itself promote all of STR-005;
the evidence-promotion task still owns the cumulative review.

The accepted Noise result is a validated predecessor, never a reusable runtime
context. Preserve the original Noise contracts, schemas, sealed roots, v1 Work
Lease canonical bytes, historical pool fixture receipts and historical readers.
Do not enable old boot/NVS campaigns, recovery-006, factory reset, persistent
pool configuration, external pools, protocol fallback or automatic reflashing.

## Scope, reuse and source admission

Both scopes use ordinary fixed USB Serial/JTAG, Controller 0.4 possession and
exact firmware source/ELF/Device Identity admission. A versioned V2 grant
extension requires explicit candidate capability admission; old V1 readers
remain strict. No HTTP control or raw serial substitute is permitted.

Reuse `V2Session`, V2 frame/message types and the accepted Noise initiator and
transport. Factor owned authenticated transport without changing cryptographic
math. `SetupConnection` is the first encrypted application frame: the reserved
empty Noise diagnostic proof is not sent to this fixture. Channel scope runs the
heap-owned session on the existing borrowed 12KiB primary transport worker and
stops at validated `WorkReady`, without calling any ASIC dispatch path. Share
scope uses the ordinary signed Work Lease, production owner, ASIC executor and
independent safety enforcement. A diagnostic fence cannot become a Work Lease.
The historical 24KiB boot V2 owner is not this adapter. Retire its still-linked
startup dispatch/owners when the ordinary signed replacement lands; preserve
historical evidence/readers and reusable pure core, not obsolete boot execution
compatibility. Re-audit actual native image/stack/heap after that replacement.

Preflight derives exact source/package/Gate/fixture/evaluator identities, checks
clean published sources, closed active-task admission, native image/stack and
allocation-failure gates, and thirteen immutable runtime artifacts. Include
every material transitive parser, signing canonicalizer, transport reducer,
header verifier, observer and native audit source in declared build/runfiles
inventories. Native measured fit is not a complete callgraph or hardware claim.
Reuse current native auditor gates: primary stack12288 bytes with selected-path
margin≥512, app package≤4194304 bytes, no new owner/stack increase or internal
reserve change. Accepted-baseline headroom does not relax these criteria.

Use the accepted Noise reader to validate its whole sealed ancestry and cleanup.
For a changed pair, capture a fresh private before-update baseline on the new
Gate page, install state-preservingly, then perform four fresh update/reconnect/
maximum-exchange cycles with existing physical lease, ROM and process-observer
checks. Bind each fresh probe to its own reconnect, nonce and journal sequence.
Keep the same page/private baseline across installation; never import a baseline
ID as proof. Require healthy startup and measured internal-heap observations.

Implement both scopes before publishing one candidate. Keep that exact clean
published firmware HEAD, Gate, trust and package unchanged between stages. The
consolidated task stays active until both pass. Channel finalization seals only
private evidence and a private projection; it writes no tracked report,
projection or task closure. Independent read-only acceptance then enables Share
preflight against the same current HEAD/package, with no provenance exception.

Share has a fresh root, page, private baseline, native permission/session,
identity and accounting. Before identity equals the accepted channel candidate;
freshly prove it is installed before any write. Perform four fresh Share update/
reconnect/max-exchange cycles on that same page; no fifth initial reinstall.
Channel cycles remain ancestor evidence, not Share continuity credit. Both
scopes fully close their own pages/owners. No retained-page handoff, imported
baseline or cross-page fingerprint comparison is introduced. Any material change
requires new channel qualification on a newly published current-HEAD package;
Share cannot substitute another pair or silently reuse a failed context.

## Command and context surface

The prospective human surface is `just stratum-v2-serial ACTION`. Bazel owns its
implementation and tests. All options are single-use; unknown, duplicate or
conflicting flags fail before sensitive inputs or root assignment.

- `preflight`: exactly `--scope channel` or `--scope share`,
  `--private-root PATH --firmware-root PATH --gate-root PATH`,
  `--package-manifest PATH --fixture-binary PATH --predecessor-receipt PATH`.
  Effect-free validation reserves an exclusive host assignment before creating
  the child. Channel predecessor is accepted Noise; Share predecessor is the
  accepted private channel result/seal on the same pair.
- `serve`: `--private-root PATH`; Share additionally requires
  `--authority-directory PATH`. Channel rejects it before reading it. Start one
  localhost workflow with protected stdout and filtered child environments.
- `finalize`: `--private-root PATH --cleanup-receipt PATH`; stopped writers are
  mandatory. Channel seals privately only. Share validates both scopes after
  cleanup, seals, then may publish both admitted projections together.
- `review`: `--private-root PATH`; read-only independent validation of immutable
  inputs, private disposition and any eligible projection.
- `recover`: fixed `v2_serial_recovery_unavailable`; no reset or new install.

There is no `--pool-credentials`, Wi-Fi input, endpoint, arbitrary argv,
factory, force, allowance override, caller readiness receipt or supplied verdict
option. The canonical fixture binary has a source-bound build receipt. Preflight
does not build or fetch dependencies. The runtime obtains its pool configuration
only from the newly owned fixture's bounded private readiness channel.

The closed context `str005-v2-serial-context-v1` contains `scope`, fresh
`attemptId`, `hostOrdinal`, `contractSha256`, exact source/artifact/evaluator
inventories, accepted predecessor `{root, resultSha256, sealSha256}`, original
campaign ID for private accounting, before/candidate identities, client and
operator hashes, native audit, and the fixed policy values in this document.
Share additionally contains `qualificationAttempt` and `expectedLedgerBefore`;
channel rejects those fields. No endpoint, port, user, signed grant, binding,
private key or secret-derived fingerprint belongs in the context.

Namespace: ignored `scratch/str005-v2-serial/channel-NNN` and `share-NNN` roots,
mode0700 and mode0600 files. Reserve each protected sibling host-ordinal marker
exclusively before child creation/copying; never roll back a consumed marker.
Share additionally reserves the fresh qualification ordinal assignment before
issuance. Partial contexts cannot serve or finalize as accepted. Sealed roots
cannot serve, issue or execute. Historical readers remain available.

The source-bound operator admits exactly five Channel claims `install-0` through
`install-4`, and four Share claims `install-1` through `install-4`; Share
rejects index0. Each scope independently proves its four cycles. Consume each
exclusive claim before its child executes. Derive exact
`just flash-monitor --board 205 --port <fresh admitted port> --manifest <bound manifest> --evidence-dir <new child> --capture-timeout-seconds 30 --redact-evidence`
arguments from the context and fresh detector. Recheck terminal-failure absence,
context/claim digest, physical identity, native ownership and durable observer
arming before dispatch. Failure during claim write leaves it consumed. Share's
first update binds its fresh admitted baseline and accepted channel ancestor,
not a fabricated local install-0 proof.

## Controller and browser boundaries

The prospective channel commands are `stratum_v2_channel_start`,
`stratum_v2_channel_cancel`; generic read-only `stratum_v2_status` serves both
scopes. Channel Start takes the exact closed in-memory shape:

```ts
type ChannelStart = {
  schema: "worker-stratum-v2-channel-start-v1";
  attemptId: Nonce16; expectedBootOrdinal: UInt;
  networkObservedAtUs: UInt; stratum: V2Stratum;
};
```

`Nonce16`/`Key32` are canonical unpadded base64url16/32-byte values; UInt is a
nonnegative safe integer and Port is1–65535. `userIdentity` is a fresh synthetic
non-owner value generated in memory, UTF-8 length1–255. This shape and status's
network tuple are runtime input only and must never be serialized as evidence.
PrivateIpv4 uses the existing qualified parser: four canonical decimal
octets0–255, in10/8,172.16/12 or192.168/16 only. Reject leading-zero, octal,
abbreviated, mapped, loopback, link-local and public forms before URL parsing;
no alternate spelling. No hostname, DNS, alternate address, optional authority
or pool file is accepted. The supplied network observation must be the current
delivered idle observation for the same boot/generation/transport epoch and be
at most5000000 device microseconds old. Refreshing possession replaces the old
binding; retained job fields remain immutable and are not rewritten to the new
session.

Start consumes the shared once-per-boot Noise/channel diagnostic slot before
returning `admitted`; ACK is not execution. Cleanup releases the owner fence,
never rearms the consumed slot. Lost/ambiguous Start is not resent. Share uses
signed Start/Renew/Stop; no diagnostic fence becomes work authority.

```ts
type V2Query = {
  schema: "worker-stratum-v2-query-v1"; scope: "channel" | "share";
  attemptId: Nonce16 | null;
};
type ChannelCancel = {
  schema: "worker-stratum-v2-channel-cancel-v1"; attemptId: Nonce16;
};
type SocketTuple = { localIpv4: PrivateIpv4; localPort: Port; remoteIpv4: PrivateIpv4; remotePort: Port };
type CurrentObservation = {
  bootOrdinal: UInt; workerGeneration: UInt; serialTransportEpoch: UInt;
  observedAtUs: UInt | null; clockValid: boolean;
  stationIpv4: PrivateIpv4 | null; wifiConnected: boolean; socket: SocketTuple | null;
};
type RetainedConnection = {
  observedAtUs: UInt; bootOrdinal: UInt; workerGeneration: UInt;
  serialTransportEpoch: UInt; poolSessionGeneration: UInt;
  poolTransportEpoch: UInt; socket: SocketTuple;
};
type V2Status = {
  schema: "worker-stratum-v2-status-v1"; scope: "channel" | "share";
  state: "idle" | "admitted" | "running" | "terminal";
  observation: CurrentObservation; connection: RetainedConnection | null;
  record: DeviceRecord | null;
};
```

An idle query has null attempt ID and null record; its scope matches the
immutable context/page. A non-null query/cancel must match the retained attempt;
unknown IDs fail. Channel Start/Cancel and generic query return this wrapper;
Cancel requests revocation, not release proof. Current observation is distinct
from the retained original job identity. Reconnection requires fresh exact-image
possession; it cannot reset deadlines or hide a changed boot. Null socket means
no currently owned socket, not unproved cleanup. Null observed time requires
clockValid:false and a retained job's first or secondary clock failure; non-null
time requires true. Idle invalid/absent clock fails the command before returning
a wrapper. Capture one RetainedConnection at actual connect; its original
time/identity/tuple survives RAM-only until boot, even if the socket closes
before polling. This narrow diagnostic snapshot is not resource ownership:
crypto/keys/signed inputs are disposed on job return. Strip both current and
retained tuples before every evidence write.

Only Start requires fresh possession and the≤5s current idle network
observation; a new delivered idle observation supersedes the previous Start
network proof. Running/terminal query and Cancel remain usable with the admitted
current-session binding past60s; do not refresh proof on every poll or reapply
Start's freshness window. Revocation does not prevent status/cleanup
observation. None of these operations extends authority or rearms the slot. The
host projects this runtime wrapper into its closed evidence type before the
first write, excluding all network tuples and secret-bearing inputs; no raw
status dump is allowed. Share context attemptId equals its signed
qualificationAttempt.id, and Share DeviceRecord is seeded only by actual
admitted funded Start. A Share idle status query supplies fresh private network
observation without consuming the diagnostic slot or authority. Active queries
use the same admitted binding past60s; fresh-session observation requires fresh
possession and preserves original record IDs. Share rejects diagnostic
Start/Cancel and uses existing signed Stop/Restore. Existing
worker-qualification-v1 is unchanged; this generic status wrapper is the
explicit new ShareFact/connection collection path.

Gate configuration has exactly `expectedGateCommit`,
`expectedFirmwareSourceCommit`, `expectedAppElfSha256`, `trust`,
`stratumV2Qualification:"before"|"candidate"`,
`stratumV2Identities:{before,candidate}` and `stratumV2Scope:"channel"|"share"`.
Each identity is exactly `{firmwareSourceCommit,appElfSha256}`. Gate commit,
scope and identities remain immutable for the page. Before→candidate preserves
the same page's private baseline and requires serial ownership released;
returning to an arbitrary pair is rejected. Share uses a fresh exact-candidate
page/root with before identity taken from the accepted channel candidate (before
may equal candidate), and captures its own new private baseline; accepted
channel continuity proof is ancestry, not an imported private baseline.

The prospective page methods are `stratumV2Possession()`,
`stratumV2ChannelStart(input,binding)`,
`stratumV2Status(scope,attemptIdOrNull,binding)` and
`stratumV2ChannelCancel(attemptId,binding)`. Possession is fresh and
memory-only. Share reuses `prepareStartAuthorization`, `loadSignedWindow`,
`startWindow`, qualification reviews and signed Renew/Stop. Channel rejects
those work/signing routes; share rejects channel Start. Before phase permits
inspection only.

The exact proposed inner grant variant is:

```ts
type V2Stratum = {
  profile: "bwg-worker-stratum-v2-standard/0.1";
  endpoint: string; authorityPublicKey: Key32; userIdentity: string;
};
```

Endpoint must be `stratum+tcp://<literal-private-ipv4>:<port>/`, with no
credentials, query, fragment, DNS or redirect. The signed profile fixes standard
channel only, mining protocol0, minimum/maximum version2, setup flags5 and
returned version2/flags0; there are no configurable duplicate policy fields. The
outer Controller0.4 Start/Renew envelopes and existing signature mechanism
remain. Presence of `profile` selects this exact parser; unknown profiles never
fall back to V1. Legacy untagged V1 canonical bytes remain byte-identical.
Require exact-candidate capability opt-in; generic signed capability0.2 alone
does not advertise V2. Signature covers this entire stratum value and the normal
qualification attempt. Mixed variants, missing authority, field mutation,
downgrade and unsupported candidates fail before activation.

## Wire exchange, target and version rules

Support only standard mining channels. The fixture validates the exact setup
protocol/version range and request flags5, responds with version2/flags0, echoes
the channel request ID, then sends channel success, one future `NewMiningJob`,
explicit `SetTarget` and matching `SetNewPrevHash`, in that order. The primary
specification's §5.3.15 requires NewMiningJob first after channel success. No
extended channel, extranonce negotiation, fallback or unrelated server message
is admitted.

The
[primary mining specification](https://stratumprotocol.org/specification/05-mining-protocol/)
assigns request bits0/2 to Standard jobs/version rolling, respectively; success
bit0 instead requires a fixed version. Therefore request5/success0 is
deliberate: reject fixed-version, Extended-required or unknown success flags.
The existing fixture's1/1 is not this contract. The
[primary message table](https://github.com/stratum-mining/sv2-spec/blob/main/08-Message-Types.md)
fixes channel_msg0 for Setup/Open and1 for Job/PrevHash/Target/Submit/ack. The
hardware rolling subset remains0x1fffe000; do not widen it to a protocol
maximum.

Freeze share difficulty1024 using exact integer arithmetic:

```text
difficulty_one = 0xffff * 2^208
target = difficulty_one / 1024
big-endian numeric hex:
00000000003fffc0000000000000000000000000000000000000000000000000
32-byte little-endian wire hex:
000000000000000000000000000000000000000000000000c0ff3f0000000000
```

The division is exact. This matches the little-endian target interpretation of
`crates/bitaxe-stratum/src/v2/work.rs::target_to_pdiff`; its floating-point
helper is not the independent target oracle. Both channel-success target and
explicit SetTarget use these exact bytes; never substitute all-FF or caller
difficulty.

Use base version `0x20000000`, nbits `0x207fffff`, and fresh fixture ntime. Keep
ntime fixed for this one job; NewMiningJob has maybe_min_ntime=None and the
matching SetNewPrevHash supplies the exact ntime. Permit rolled bits only within
`0x1fffe000`, the existing BM1366 initialization mask. The device's result
decoder shifts its 16-bit returned version field left13; existing V2 work uses
base OR returned bits. Therefore require
`(submittedVersion & ~0x1fffe000) == 0x20000000` and
`submittedVersion == (baseVersion | parsedAsicVersionBits)`. Reject out-of-mask
bits rather than accepting arbitrary supersets as the old fixture did. Do not
claim coverage of every roll pattern or multi-midstate behavior from one share.
Keep the ASIC ticket filter at its existing catalog difficulty256; the pool
share target1024 must not rewrite the ASIC difficulty/mask registers. Standard
work keeps polling until a new valid job/previous-hash event or revocation; the
V1 two-second regenerate/extranonce policy must not invent Standard V2 jobs.

Bound plaintext payload to2048 bytes, encrypted header to22, encrypted payload
to2064 and complete encrypted frame to2086 bytes. Validate extension ID0 and the
correct channel-message flag before decoding payloads: setup and channel-open
frames have extension_type0; SetTarget/NewMiningJob/SetNewPrevHash and share
submission/success use0x8000. Unknown types/flags/extensions, trailing bytes,
short frames or surplus bytes inside a declared frame fail. Correctly coalesced
subsequent frames remain buffered and parsed in order; they are not surplus. One
retained active job avoids ASIC job-ID wrap ambiguity.

The fixture generates fresh high-entropy32-byte merkle root and previous hash,
nonzero channel/job IDs and one fixed job. Canonical job commitment is SHA256 of
the exact permitted plaintext frame encodings for channel success, NewMiningJob,
SetTarget and SetNewPrevHash, each prefixed by its four-byte little-endian
length. It excludes Setup/Open payloads containing endpoint or user data. The
independently received device commitment must equal the fixture commitment.

The independent verifier must not call `V2MiningWork`, its header builder or its
qualification comparison. Construct the80-byte header directly as version LE4,
wire previous-hash32, wire merkle-root32, ntime LE4, nbits LE4 and nonce LE4.
Compute SHA256(SHA256(header)); interpret the resulting32 digest bytes as a
little-endian integer and require it ≤ the frozen target. Golden tests establish
all byte orders, including the native ASIC work word transformations.

## Fixture and memory-only privacy

Apply `docs/parity/evidence-policy.md` before the first write. Actual fixture
pool endpoints, pool ports, user identities, signed grants, credentials and
private keys remain memory-only even below a protected root. Do not reinterpret
Noise's diagnostic endpoint rule as a mining-pool exception. No plain hash of a
low-entropy endpoint/user may serve as anonymization.

The prospective fixture mode is `--mode v2-serial`, with non-secret command
arguments `--scope channel|share --private-root PATH --attempt-id ID`. One
bounded private stdin record provides the listener's admitted IPv4, expected
peer IPv4 and synthetic user identity. One bounded private readiness pipe
returns the actual listener tuple and authority public key to the owning
supervisor. No raw pipe bytes reach files, inherited stdout/stderr, command
transcripts or terminal. The source-bound parser constructs a different closed
persistable receipt. Ephemeral authority material and transport keys are
zeroized on ownership end.

Each private pipe carries one compact UTF-8 JSON object followed by LF, at
most4096 bytes including LF; reject trailing records. Input has exactly
`{schema:"str005-v2-fixture-input-v1",scope,attemptId,listenIpv4,expectedPeerIpv4,userIdentity}`.
Ready has exactly
`{schema:"str005-v2-fixture-ready-runtime-v1",scope,attemptId,instanceId,listenIpv4,listenPort,authorityPublicKey}`.
Both repeat and match the admitted scope/attempt. Listen port is chosen by
binding port0; no caller-selected live pool port is accepted. Input/readiness
pipes close after their one message. These runtime schemas are forbidden
evidence inputs.

Use exact ready/startup≤5000ms, channel fixture lifetime150000ms, share lifetime
300000ms, and actual requested host-child close/reap≤5000ms. Lifetime begins at
actual ready publication. All setup/expensive source audits and human readiness
precede fixture launch; ready→Start delivery is at most10000 host milliseconds.
No readiness/status refresh or retry extends an origin. The share fixture stays
alive through the heartbeat fault and actual peer EOF, then closes naturally; it
need not remain alive throughout later cooling/fresh-session waiting.

Retain at most3 candidate connections with the existing bounded complete-read
and500ms post-selection peer inventory. Accept exactly one expected peer, reject
unexpected/duplicate peers/overflow, and retain all first failures. Use absolute
remaining-time bounds before and after accept/read/write operations; partial
data or repeated Interrupted cannot refresh deadlines. Close the listener at the
final observed inventory boundary. This is not proof of future network silence.
Ready and terminal receipts publish atomically and exclusively.

A live source-bound collector compares the actual device and fixture socket
tuples in memory and emits a typed comparison result with observation
provenance. The offline judge verifies that result's source/attempt/connection
binding; it cannot reconstruct an intentionally unpersisted endpoint.
Independent encrypted job commitments and share verification provide the
retained protocol joins. A fresh opaque connection ID correlates one owned
fixture socket and its records; caller-provided IDs/booleans never replace
actual socket/producer observations.

Use bounded closed per-frame summaries, never raw encrypted bytes, plaintext
Setup/Open payloads or arbitrary logs. Persistable readiness contains only
schema/scope/attempt/fixture instance/source identity, authority-key commitment,
comparison classification, monotonic ready time and actual process ownership.
Local commands/lsof output are sanitized before writing: pool port/endpoint must
not leak through argv, command echo, error text or cleanup snapshots. The parent
receives necessary listener details through RAM/IPC and records actual absence
results without the forbidden tuple.

## Channel acceptance and accounting

Channel's stages are admitted, preparing, connected, authenticated, setup,
channel, job, target, work_ready, socket_closed and worker_quiescent. Require
monotonic ordered events, exact source/boot/generation/epoch, configured
authority verification, explicit target and coherent job/previous-hash pair.
WorkReady means converted work exists; no ASIC queue/dispatch or submission
occurs.

Authority lasts120000 device milliseconds from admission. Observe through the
fixed125000-ms horizon, without extending admission. Retain the Noise v2
60000-ms preparation bound,5000-ms connect,10000-ms complete read and2000-ms
write phase bounds, all within the absolute authority. Add no unbounded
per-frame loop. Success requires actual socket and worker release
within120000ms. Late release can improve cleanup facts but never change a
terminal failure to success.

Independently start a125000-ms host observation deadline immediately before the
single Channel Start request using the supervisor's monotonic clock. Freeze its
origin/deadline with the consumed Start claim; neither response latency nor
polling/reconnect refreshes it. If the device clock is lost, the host still ends
collection as unverified/incomplete at that bound and performs bounded cleanup.
Host elapsed time never substitutes for accepted device-local timing.

Fresh before/after authenticated ledgers must be next18,last17,total1560000ms,
pendingfalse, unless the validated predecessor and newly reviewed contract
explicitly establish a later current ordinal. Original campaign masks remain7/7,
charged240000ms,pendingfalse. Compare same-boot work/submitted/accepted/rejected
counters with zero deltas; historical totals need not be zero. No signing,
allowance issue/consumption, ASIC effects, restart or settings writes are
allowed.

## Share reservation, work and safety

Share preflight requires independently accepted channel evidence for the exact
candidate and complete restoration/cleanup, not merely the work_ready stage.
Fresh possession, temperature/voltage/power/fan health, unchanged preserved
settings/identity/high-water and both authenticated ledgers precede issuance.
Run the existing fan-only cooling proof and restore its baseline before Start.

Use a fresh normal `worker-qualification-attempt-v1`: expected ordinal18,
maximumActiveMilliseconds180000 and a fresh ID. The device reserves the complete
180000ms once; no failure, early success or interrupted completion refunds it.
Expected completion is next19,last18,total1740000ms,pendingfalse. Original
240000ms/masks7/7 stay unchanged. Host-only assignment is not device
consumption; issued/delivery-attempted/reserved/completed are distinct retained
facts.

Initial and normal renewal leases are exactly60000ms with renewAfter20000ms.
Preparation must reach Active before its expiry; Renew remains unavailable while
preparing. The existing Gate Start reply deadline remains30000 host
milliseconds: positive activation and reply must satisfy that bound as well as
the60000-ms device lease. A lost/late reply consumes the admission and cannot be
retried. Initial renewAfter is20000ms; renewal starts only after observed
completed Start and actual running state. No automatic fresh Start or
preparing-renew extension is introduced. Subsequent normal renewals follow the
existing signed policy only after Active. The180000-ms generation ceiling
retains its15550-ms shutdown reserve: actual work gate is at most164450ms.
Crypto and networking execute separately from the production/safety/control
owners; late crypto after revocation may only clean up, never connect, submit or
activate.

Keep400MHz,1100mV,100% fan; fresh4.5–5.5V,≤15W,\<75C and nonzero RPM. Select the
first fully correlated accepted share whose matching ACK the device actually
observed. Before cutting heartbeats, require fresh device evidence of≥5000ms
remaining on both the effective lease and work gate. Otherwise fault proof is
unverified; grant no extra allowance or after-result relaxation. Stop renewals
and suppress only application heartbeats through the qualified mechanism.
Require `revocation_reason:heartbeat_timeout`, the unchanged2800-ms cutoff and
last-valid-advancing-heartbeat→actual revocation/shutdown initiation≤3000ms.
Keep the passive observer through the fault plus≥5000ms. Browser timing and
fixture EOF do not prove device safety.

Require ordered safe stop and fan100% until fresh≤45C, then qualified30%, within
120000ms. Retain the existing145000-ms post-fault wait before fresh admission;
elapsed time is not restoration proof. Cooling and observation bounds are not
shortened to fit fixture lifetime. The300000-ms fixture limit covers ready→Start
≤10000ms plus the≤180000ms generation and a bounded observed close window;
actual EOF must still occur before its deadline. Opaque non-returning work
cannot be assumed to fit that remainder: record incomplete and stop safely if it
does not.

Retain every legitimate submission/acknowledgement, including post-cut or
post-revocation arrival of bytes already committed before revocation. Arrival is
not a new device write; prove the write/queue event's own device ordering. Each
selected accepted submission must have matching channel/job/ sequence and
fixture `accepted_count:1`, with no duplicate credit. The first correlated
accepted share suffices; global accepted total need not equal1. Prove
source-bound ASIC dispatch and nonce decoding for each credited share, not
merely a fixture's receipt of a self-generated valid header.

After revocation, no new device work, protocol write/submit or renewal is
permitted; late matching ACK observation may settle previously submitted work
without new credit or authority. The fixture keeps reading bounded valid
protocol traffic until actual peer EOF, then produces its terminal. It must not
close immediately after the first success and manufacture a disconnect before
the required heartbeat fault. Bound retained frame/share records to1024 each;
overflow is a closed unverified outcome, never silent loss.

## Evidence interfaces and independent finalization

All new evidence objects use closed versioned schemas, bounded arrays and safe
integer timestamps. Runtime-only values use distinct types from persistable
records. The frozen persisted envelopes are:

Digest is lowercase64-character SHA256 hex. Nonce16 is canonical16-byte
base64url; every stored identity, duration and sequence retains its declared
units.

```ts
type Event = {
  sequence: UInt; atDeviceUs: UInt | null; kind: Stage;
  channelId: UInt | null; jobId: UInt | null;
  submissionSequence: UInt | null; payloadSha256: Digest | null;
};
type Timing = {
  operation: Operation; count: UInt; failedCount: UInt;
  maxDurationUs: UInt | null; totalDurationUs: UInt | null;
  firstStartedAtDeviceUs: UInt | null; lastFinishedAtDeviceUs: UInt | null;
  inFlightStartedAtDeviceUs: UInt | null;
};
type ShareFact = {
  dispatchSequence: UInt; asicJobId: UInt; workFieldsSha256: Digest;
  dispatchedAtDeviceUs: UInt; nonceAtDeviceUs: UInt;
  writeStartedAtDeviceUs: UInt | null; writeCompletedAtDeviceUs: UInt | null;
  nonce: UInt; versionBits: UInt; asicIndex: UInt; coreId: UInt; smallCoreId: UInt;
  channelId: UInt; jobId: UInt; submissionSequence: UInt; ntime: UInt; version: UInt;
  ackAtDeviceUs: UInt | null; ackLastSequence: UInt | null;
  ackAcceptedCount: UInt | null; ackSharesSum: UInt | null; matchedSubmitCount: UInt | null;
};
type DeviceRecord = {
  schema: "worker-v2-serial-evidence-v1"; scope: "channel" | "share";
  attemptId: Nonce16; bootOrdinal: UInt; workerGeneration: UInt;
  poolSessionGeneration: UInt | null; poolTransportEpoch: UInt | null;
  serialTransportEpoch: UInt;
  jobCommitment: Digest | null;
  observedAtUs: UInt | null; state: "admitted" | "running" | "terminal";
  admittedAtDeviceUs: UInt; authorityDeadlineDeviceUs: UInt;
  observationDeadlineDeviceUs: UInt | null; terminalAtDeviceUs: UInt | null;
  outcome: "accepted" | "rejected" | "expired" | "cancelled" | "incomplete" | null;
  events: Event[]; timings: Timing[]; shareFacts: ShareFact[];
  firstFailure: Failure | null;
  secondaryFailures: Failure[]; resources: Resources;
};
type Failure = { stage: Stage; category: FailureCategory; atDeviceUs: UInt | null };
type Resources = {
  socketClosed: boolean; workerQuiescent: boolean; fenceRetained: boolean;
  socketClosedAtUs: UInt | null; workerQuiescentAtUs: UInt | null;
};
```

Stage is the channel-stage list above plus `asic_dispatch`, `nonce`,
`submission`, `accepted`, `revoked`, `shutdown`, `cooled`. FailureCategory is
admission, clock, allocation, authority, timeout, eof, extra, authentication,
protocol, channel_mismatch, job_mismatch, invalid_nonce, rejected_share, safety,
cleanup or evidence. At most16 secondary failures retain later distinct cleanup/
clock problems without replacing the first cause. Setup/Open events always have
null payloadSha256: hashing secret-bearing payloads is forbidden on every path.

Operation is initiator_construction, act_one_construction, connect,
act_one_write, act_two_read, act_two_authentication, frame_encrypt, frame_write,
frame_read, header_decrypt, payload_decrypt, socket_close or worker_join. Retain
one summary per operation, max64 stage events and max16 actual ShareFact rows;
Channel has none. Full serialized reply including Controller/runtime wrappers
is≤65536 bytes, proved by worst-case encoding tests. Overflow is sticky evidence
failure, never eviction or a reason for another Start. Host/fixture disk
receipts remain bounded separately; no large serial stream or cursor framework
is introduced.

Timing count/max/total cover actual completed invocations, including measured
failures; failedCount≤count. Preserve first/last times and the currently
in-flight start. Enforce maximum read≤10000000us, write≤2000000us,
connect≤5000000us, aggregate preparation≤60000000us and all applicable absolute
deadlines. Opaque crypto has before/after measurements, not an in-call
cancellation guarantee. Zero-count/unreached operations have null aggregate
timestamps/durations; in-flight start may exist before count1. A completed
invocation with invalid clock makes duration aggregates null and adds explicit
clock failure, never0.

Unreached close/join has false release flags/null timestamps. True release with
unknown time retains the fact plus a clock failure and cannot pass timing.
Preserve secondary cleanup/clock failures without replacing the first cause.
Channel authorityDeadline is admission+120000ms, observationDeadline is
admission+125000ms. Terminal outcome freezes by that horizon; late release
updates cleanup facts only. Share authorityDeadline is its fixed qualification
ceiling, not its renewable effective lease; its observationDeadline is null
because fault-relative cooling/fresh-admission rules govern collection. Required
safe-stop facts remain mandatory. The expected heartbeat_timeout after a
qualified ACK is a safety-test outcome, not a new protocol failure; other first
failures persist.

Native ShareFact rows are built from actual ASIC dispatch/result decoding,
submission and matched ACK events under the retained binding. Partial ACK fields
remain null until observed. The collector may derive separate immutable receipts
from these facts, never invent nonce/dispatch details from an event hash.
writeStartedAtDeviceUs records actual I/O entry; writeCompletedAtDeviceUs
records successful full socket write/flush completion, never queue acceptance.
Null completion means no proved completed submission. Any retained queue event
must have its own named fact. After revocation no new write operation may start;
an operation initiated earlier may complete later, with original
start/completion times retained and no subsequent I/O. Completion/arrival alone
cannot establish a new post-revocation write. Device ACK receipts take sharesSum
from actual ackSharesSum, never infer it from counts. DeviceRecord.jobCommitment
is computed natively from the four actual validated received frames in the
specified order; it stays null until all four exist. The judge compares that
native commitment with the independent fixture/job proof.

WorkerGeneration names native authority/revocation; PoolSessionGeneration names
the pool session; serialTransportEpoch names possession/reconnect;
poolTransportEpoch names the pool connection/work/nonce incarnation. These four
namespaces are never numerically interchangeable. Both scopes record the actual
pool pair once its session starts; null pool fields mean not yet started only.
Share activation records all four IDs; work/nonce/submit/ACK receipts join that
immutable binding. worker-qualification-v1.generation joins workerGeneration.
Fresh serial reconnect may change current worker/serial IDs while the retained
job's original binding stays fixed; cleanup/status cannot rewrite history.

`workerQuiescent` means the scoped V2 job returned, its socket/crypto/input
owners dropped and no queued effectful job remains. The existing persistent
primary OS network worker need not exit; never label scoped completion as that
worker's termination or allocate another stack owner to satisfy the evidence
vocabulary.

Separate immutable receipts use the closed envelope below. `kind` selects one
exact facts type; fields from any other kind are rejected. Device and fixture
clocks remain separate and timestamps are never subtracted across owners.

```ts
type Receipt<K, F> = {
  schema: "str005-v2-serial-receipt-v1"; kind: K;
  contextSha256: Digest; producerSha256: Digest; sequence: UInt;
  clock: "device-us" | "fixture-us" | "supervisor-ms" | "parent-hrtime-ms";
  at: UInt; facts: F;
};
type ProcessIdentity = { pid: UInt; pgid: UInt; startedAt: string };
type Ready = {
  scope: "channel" | "share"; attemptId: Nonce16; instanceId: Nonce16;
  authorityPublicKeySha256: Digest; owner: ProcessIdentity; readyAtMs: UInt;
};
type Connection = {
  attemptId: Nonce16; instanceId: Nonce16; connectionId: Nonce16;
  bootOrdinal: UInt; workerGeneration: UInt; serialTransportEpoch: UInt;
  poolSessionGeneration: UInt; poolTransportEpoch: UInt;
  expectedPeerMatch: boolean; tupleMatch: boolean; expectedPeerCount: UInt;
  unexpectedPeerCount: UInt; candidateOverflow: boolean;
  readinessSha256: Digest; deviceObservationSequence: UInt;
  comparisonStartedAtMs: UInt; comparisonCompletedAtMs: UInt;
};
type Job = {
  connectionId: Nonce16; jobCommitment: Digest;
  channelSuccess: Base64Bytes; newMiningJob: Base64Bytes;
  setTarget: Base64Bytes; setNewPrevHash: Base64Bytes;
};
type Dispatch = {
  jobCommitment: Digest; workerGeneration: UInt; poolSessionGeneration: UInt;
  serialTransportEpoch: UInt; poolTransportEpoch: UInt; dispatchSequence: UInt;
  asicJobId: UInt; workFieldsSha256: Digest;
};
type Nonce = {
  jobCommitment: Digest; workerGeneration: UInt; poolSessionGeneration: UInt;
  serialTransportEpoch: UInt; poolTransportEpoch: UInt; dispatchSequence: UInt;
  asicJobId: UInt; nonce: UInt; versionBits: UInt;
  asicIndex: UInt; coreId: UInt; smallCoreId: UInt;
};
type Submission = {
  connectionId: Nonce16; jobCommitment: Digest;
  workerGeneration: UInt; poolSessionGeneration: UInt;
  serialTransportEpoch: UInt; poolTransportEpoch: UInt; nonceReceiptSha256: Digest; channelId: UInt; jobId: UInt;
  sequenceNumber: UInt; nonce: UInt; ntime: UInt; version: UInt;
};
type Acknowledgement = {
  connectionId: Nonce16; workerGeneration: UInt; poolSessionGeneration: UInt;
  serialTransportEpoch: UInt; poolTransportEpoch: UInt; submissionSha256: Digest;
  channelId: UInt; lastSequenceNumber: UInt; matchedSubmitCount: UInt;
  acceptedCount: UInt; sharesSum: UInt; producer: "fixture" | "device";
};
type FixtureEvent = {
  sequence: UInt; atFixtureUs: UInt; kind: FixtureStage;
  payloadSha256: Digest | null; submissionSequence: UInt | null;
};
type FixtureEvents = { connectionId: Nonce16; events: FixtureEvent[] };
type FixtureTerminal = {
  instanceId: Nonce16; connectionId: Nonce16 | null;
  outcome: "accepted" | "unverified"; firstFailure: FixtureFailure | null;
  elapsedMs: UInt; receivedShares: UInt; acceptedShares: UInt;
  rejectedShares: UInt; duplicateShares: UInt;
  peerClosed: boolean; socketClosed: boolean; listenerClosed: boolean;
};
type FixtureFailure = { stage: FixtureStage; category: FailureCategory; atFixtureUs: UInt | null };
type Fault = {
  workerGeneration: UInt; poolSessionGeneration: UInt;
  serialTransportEpoch: UInt; poolTransportEpoch: UInt;
  selectedDeviceAckSha256: Digest; suppressionRequestedAtHostMs: UInt;
  headroomObservedAtDeviceUs: UInt; leaseRemainingMs: UInt;
  workGateRemainingMs: UInt; revocationReason: "heartbeat_timeout";
  lastValidHeartbeatAtDeviceUs: UInt; gateClosedAtDeviceUs: UInt;
  shutdownStartedAtDeviceUs: UInt; observerTailMs: UInt;
};
```

Kind names exactly match `fixture-ready`, `connection-comparison`, `job`,
`dispatch`, `nonce`, `submission`, `acknowledgement`, `fixture-events`,
`fixture-terminal` and `heartbeat-fault`, respectively. FixtureStage is one of
setup_received, channel_open_received, channel_success_sent, job_sent,
target_sent, prev_hash_sent, share_received, share_success_sent, peer_eof and
listener_closed. Payload digests are forbidden for the secret-bearing first two
stages; other payload digests cover only the permitted frames. Protocol fields
are u32, ASIC job/index/core fields use the existing bounded BM1366 parser, and
Base64Bytes is canonical base64 for at most2054 plaintext frame bytes.
Host/fixture frame and share arrays contain at most1024 records; runtime limits
above remain smaller. Process start strings are bounded128 UTF-8 bytes. Sequence
is positive and strictly increasing per producer; repeated identical status
snapshots add no events or progress. Conflicting repeats fail.

Kinds `accounting-before` and `accounting-after` have exactly
`{qualificationLedger,originalBudget,stateSequence,stateSha256}`; the two ledger
objects use unchanged `worker-qualification-ledger-v1` and
`worker-budget-review-v1` shapes and fresh authenticated review procedures.
`restoration` has exactly
`{stateSequence,stateSha256,preservation,resources, qualification}`:
preservation uses unchanged `worker-preservation-continuity-v1`, resources uses
Resources above, and qualification is the exact current
`worker-qualification-v1` object. Require source-bound current-image inactive
baseline, mine-on-boot false, healthy fresh sensors, safe stop and retained
job/resource release. Share Start/Renew legitimately advance durable
authorization high-water. Compare final authorization preservation to the
existing post-work authorization checkpoint, not its pre-work value. Channel
authorization high-water remains unchanged. Settings and Device Identity remain
unchanged in both scopes; both budget ledgers remain monotonic with no
reset/refund.

Parent browser/process/resource cleanup receipts use the existing qualified
Noise cleanup producer shapes and proof rules, bound to this context; the new
resource projection excludes every pool tuple/port. Any changed shape receives
an explicitly versioned parser rather than relabeling a Noise receipt. The
frozen context maps each receipt's producer digest to its exact evaluator source
inventory; a supplied digest is not evidence that code ran. Actual producer
entrypoints create receipts from their own observations, not JSON pass inputs.

The job receipt retains the four permitted encoded frames and independently
checks their digest, exact target/version/ntime/channel/job IDs. Dispatch/nonce
receipts retain native job slot, all four binding IDs, dispatch sequence, parsed
hardware nonce/version bits and job commitment privately. Submission/ack
receipts join that nonce to the encrypted fixture's decoded fields and exact
success response. These bounded noncredential inputs permit independent header
reconstruction. Never put raw nonce/job/socket/device identifiers in public
projections.

The final judge verifies fresh source/artifact/native inputs, claim chronology,
all required cycles, immutable producer histories, independent fixture math,
accounting, source-bound privacy comparisons, unchanged private preservation and
actual cleanup. It rechecks file membership/modes/digests before and after
reading. Current native audits and source admission finish before the≤5s fresh
endpoint window; metadata recording must not repeat expensive full audits per
event.

Parent cleanup is external to the exited server: actual browser closure,
stream/WebLock release, supervisor/fixture close-event receipts, listener/owned
process and every observed Serial node holder absence. Requested child stop→
close/reap is≤5000ms on that parent's monotonic clock. Natural fixture EOF/exit
has no invented stop timestamp. Keep supplementary incomplete observers and
nonzero exits separate; they cannot certify mandatory cleanup.

`finalize` freezes available partial inputs and actual parent cleanup, then
writes an exclusive result, complete inventory and any eligible closed
projection. Writers must already be stopped. Channel stores all three privately:
accepted channel evidence can admit Share, but causes no tracked publication,
task archival or HEAD change. `review` independently validates that private seal
before Share preflight; no `passed` argument can bypass it.

Share finalization, after actual restoration and complete cleanup, may publish
the independently accepted channel projection and, only if Share also passes,
its accepted `str005-v2-serial-projection-v1` projection. A failed Share does
not rewrite channel acceptance; publish its precise failed outcome without an
accepted-share projection. Incomplete Share restoration/cleanup blocks new
accepted-projection publication and retains the channel result privately. Each
accepted projection contains scope, accepted status, board205,
contract/source/package/evaluator/result/seal digests, closed
criteria/counts/durations and non-claims; Share also binds the channel
result/seal digests. Publication never overwrites an existing projection. The
combined task closes/archives only after both pass. Unverified evidence remains
sealed non-promotional and is never repaired by appending missing inputs.
`review` is always read-only.

## Failure, restoration and retries

Failure is sticky across awaited source checks, child creation, signing, writes,
late acknowledgements and cleanup. Consume claims before effects and recheck
terminal state immediately after awaits and before returning a permit. Retain
actual typed device/fixture failures before generic host failures. Raw exception
text and secret-bearing payloads never become evidence.

A failed exchange may still collect real cancellation, release, cooling,
restoration and both ledgers. This does not grant a new Start or change the
failed outcome. Use normal Stop/Close, native fresh Connect, fresh authenticated
retained status and baseline/accounting review. Settings/NVS remain untouched
and the tested candidate remains installed. If resources/authority/physical
identity or restoration cannot be proved, deny new owners and stop; no hidden
recovery mode.

There is no fixed retry count. Every retry requires retained failed evidence, a
concrete verified correction, a boundary regression, applicable software/native
gates, changed published implementation when correction requires it, a new
protected root and fresh exclusive assignments. A consumed mining reservation
always advances to the next authenticated ordinal; an unused host claim supplies
no replay authority. New failure classes need an explicit prospective guarded
continuation rather than a generic `force` or repeated same-signature retry.

## Verification and handoff

Before publication, verify actual shared production transport against the new
fixture, independently specified target/header golden vectors, fixed version
mask, explicit target receipt, Setup-first encrypted traffic, channel zero-ASIC/
zero-signing/zero-reservation, and real nonce/ack correlation. Software
negatives cover wrong authority/certificate, protocol/order/channel/job/target
errors, short/extra/late/fragmented frames, duplicate peers, invalid or stale
nonce, forged/duplicate/excess accepted counts, out-of-mask versions, non-exact
ntime, wrong extension/channel flags, fabricated V1-style regenerated jobs,
replay/ambiguous Start and frame bounds.

The existing asymmetric nonzero-version known-answer fixture proves byte order
and a negative at1024. Add a positive finite-target header oracle (genesis is a
verifier unit vector, not this hardware job: its nbits differs), exact target
equality and target+1 cases. Do not fabricate an all-FF positive proof.

Test initial lease expiry during crypto with no preparing Renew, existing V1
canonical vectors byte-for-byte, V2 field mutation/downgrade and fresh binding
requirements. Test independent heartbeat cutoff under stalled crypto/network,
late completion after revocation, cleanup timeout and nonpromotion, no refunds,
ledger drift, archived-task effects, unsafe paths, partial assignment, source
and inventory mutation, and endpoints/users/grants split across stdout/stderr
chunks being removed before any write. Unknown fields fail closed.

Run real Gate/client/supervisor/operator/fixture-child composition and immutable
finalizer/review tests, not only isolated mock pass flags. Negative hardware
injection is excluded; one positive channel attempt and one positive funded
share attempt are the intended live path, with failure policy governing any
deviation. Root owns ordered repository gates, publication, fresh clean package
and actual hardware; software preparation tasks gain no USB authority from this
document.
