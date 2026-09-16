# STR-005 Noise parity qualification amendment

## Status and precedence

Owner-authorized on 2026-09-16 under `task-str005-noise-parity-scope-amendment`.
Contract ID: `str005-noise-serial-v2`.

This amendment and the [v1 contract](str005-noise-serial-qualification.md)
together define v2. The v1 byte digest remains
`0da417fc198a89042eb62902999ac822be5365a5f0333df8220f15afe3447a62`. This
document overrides only the requirements explicitly identified below; other
identity, authentication, privacy, ownership, preservation, accounting,
publication and evidence requirements remain. Preserve the original contract,
archived task and every historical result. Later semantic changes require a new
linked amendment task.

This is a specification change, not implemented firmware capability or hardware
acceptance. Current v1 parsers and the blocked CLI must not be relabeled as v2.
Runtime and fixture preparation may resume against this amendment. Live effects
remain blocked until their applicable v2 implementation is verified and
published. No device access or new attempt is part of this amendment task.

## Parity objective and scope

Prove one fresh, authority-authenticated Noise handshake and one exact encrypted
diagnostic exchange on Ultra 205 through the ordinary fixed-Serial/JTAG runtime.
The fixture must receive the device's 64-byte act one and independently decrypt
the exact 22-byte encrypted empty diagnostic frame. Normal restoration and
actual resource release are necessary for an accepted result.

The pinned reference's
[`sv2_noise_handshake`](../../reference/esp-miner/components/stratum_v2/sv2_noise.c)
uses synchronous secp256k1 operations. That source and the STR-005 checklist do
not establish a five-second in-call cancellation guarantee. The
[readiness audit](str005-noise-runtime-readiness.md) correctly identifies a gap
against our stronger v1 contract; it does not demonstrate a hardware overrun or
a failure of Noise interoperability. That audit remains valid for v1.

V2 accepts the existing pinned synchronous crypto dependencies. A maintained
crypto fork, bounded ElligatorSwift internals, cancellation inside opaque native
calls and a universal worst-case crypto/cleanup proof are not prerequisites for
this parity milestone. They remain deferred under
`task-str005-noise-cooperative-cancellation`. Do not change cryptographic math,
key generation, certificate validation or the wire protocol to obtain a pass.

This positive Noise milestone does not complete all of STR-005. Channel/job
receipt and an accepted ASIC-generated V2 share still need their own fresh
contracts and hardware evidence before the final parity promotion. The relaxed
crypto cancellation claim here does not relax mining authority, work/submission
revocation, thermal protection or ordered shutdown in any later task.

## Minimal implementation and unchanged gates

Reuse the existing Noise initiator, transport, Worker lifecycle, Gate
possession, fixture and qualification helpers. Add only the asynchronous command
adapter, ordinary owner fence, bounded metadata and evidence joins needed for
this test. Use the existing public crypto operations as checkpoints; do not
duplicate the handshake or create a second general mining/recovery framework.

The following remain mandatory:

- One idle, network-only diagnostic per boot, fresh exact-pair authenticated
  possession, completed-response dispatch, consumed admission and no duplicate
  execution after a lost response.
- An ordinary separately owned worker for crypto, keeping control, receive,
  telemetry and independent safety enforcement responsive. Preserve existing
  stack sizes, priorities, CPU affinities, buffers and internal reserve. Native
  build/image and selected stack/allocation checks plus fresh startup/heap
  observations remain required. A measured fit is not a full-callgraph proof.
- No mining Start, restart, configuration mutation, competing diagnostic or
  fan-only qualification while diagnostic resources are not actually released.
  No Work Lease, allowance signing/consumption, jobs, shares or ASIC work in
  this diagnostic. Preserve both accounting ledgers and compare fresh same-boot
  work and share deltas; historical totals need not be zero.
- Literal admitted private fixture IPv4, required configured authority key,
  fresh station observation, exactly one correlated peer, exact receipt/decrypt
  proof and the existing privacy rules. No discovery, credentials or raw logs.
- State-preserving initial installation and four update/reconnect/max-exchange
  cycles for a changed pair, using existing ownership and ROM admission rules.
  Preserve the before-update page baseline, installed candidate and actual host
  process/browser/socket/serial cleanup. No automatic rollback or factory reset.
- Exact published source/package/fixture/evaluator identities, independently
  judged immutable evidence and unchanged historical schemas/readers. A
  successful handshake alone is insufficient for acceptance.

## Cancellation and time semantics

Revocation and worker termination are separate observations. The independent
2.8-second heartbeat revocation rule remains. Losing possession, replacing the
session, Restore/Cancel, disconnect, deadline expiry or clock discontinuity
revokes diagnostic authority without waiting for crypto or its worker lock.
Ordinary actual work revocation and shutdown initiation retain their existing
three-second requirement; this idle diagnostic must not delay them.

Check cancellation, session/generation/epoch and valid device time before and
after each supported opaque crypto operation and before every new protocol I/O
or accepted completion. In particular, use separate initiator construction,
act-one construction and act-two authentication calls, checking between them.
The existing observer-only preparation wrapper does not provide an early exit
and cannot be treated as a cancellation boundary by itself.

If cancellation arrives inside a synchronous library call, that call may finish.
After it returns, perform cleanup only: do not enter the next crypto operation,
connect, write a handshake/proof frame or declare success. This rule does not
claim to retract bytes already queued in TCP before revocation. Preserve the
first failure even if the cryptographic computation later succeeds.

| Requirement                       | V2 meaning                                                                                                                                                                                                                                              |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 120000-ms authority envelope      | Unchanged, from device admission. A positive result requires protocol completion, actual socket release and owner-observed worker completion/join within this envelope. Status, progress and reconnect do not extend it.                                |
| 60000-ms crypto preparation limit | An attempt acceptance limit and cancellation trigger, checked independently where available and again on return. It does not promise preemption or return from an opaque call within that time. Late completion cannot authorize I/O or success.        |
| Connect/read/write limits         | Keep one 5000-ms TCP connect, 10000-ms aggregate act-two reception and 2000-ms write/flush bounds. Partial progress never restarts a deadline.                                                                                                          |
| Firmware cleanup                  | Remove the universal cancellation-to-quiescence limit of 5000 ms. Record the actual duration and require actual release for acceptance. No new hard guarantee about an opaque call is claimed.                                                          |
| Failed-job observation horizon    | A fixed device deadline at admission plus 125000 ms, including the original 120000-ms envelope and a 5000-ms observation tail. This is not cancellation time plus 5000 ms and not a promise of worker exit.                                             |
| Host and fixture bounds           | Retain the fixture's 5000-ms readiness, 150000-ms lifetime, bounded inventory/reads and 500-ms extra-peer observation. Host children still have 5000 ms to close/reap when stopped. These host process bounds do not justify killing a firmware thread. |

When cleanup completes by the failed-job observation horizon, preserve the
appropriate rejected/cancelled/expired outcome. A protocol success cleaned up
after the 120000-ms authority envelope is expired, never accepted. At the
125000-ms horizon, missing cleanup becomes immutable `incomplete`, retaining the
earliest cause and a separate cleanup failure. Late cleanup facts can establish
present safety but cannot upgrade the diagnostic verdict.

An incomplete result does not release an owner: retain the job record and join
handle, continue cleanup-only observation, and reject competing effects until
actual socket release, disposal and join are observed. Do not detach the worker
and report success, kill/unwind a native firmware thread, reset/reflash, or
restart the diagnostic to conceal missing cleanup. If quiescence cannot be
proved, stop the attempt with a documented blocker; any further recovery effect
needs its own reviewed contract.

A clock discontinuity immediately prevents acceptance and revokes authority.
Unavailable device timing remains null under the existing closed failure rules;
do not infer it from a host clock. The supervisor's finite observation cannot
outlive its existing fixture lifetime and host cleanup bounds while waiting for
an automatic result. Human recovery planning may wait after host owners are
released, but supplies no missing device-cleanup proof or renewed authority.

## Versioned interface and evidence changes

Keep Controller 0.4, serial 0.2, possession 0.2, command names, Gate method
names, CLI arguments and the v1 field dictionaries except for the explicit
changes in this section. Updated parsers must select a version explicitly and
reject mixed v1/v2 evidence. Do not weaken or silently reinterpret the existing
v1 readers.

| Surface                                     | Version/binding                                                                                                                                  |
| ------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| Start payload                               | `worker-noise-diagnostic-start-v2`; same fields and canonical digest rules                                                                       |
| Status payload                              | `worker-noise-diagnostic-status-v2`; same fields, amended cleanup semantics below                                                                |
| Status/Cancel query                         | Retain `worker-noise-diagnostic-query-v1`; its shape is unchanged. Exact candidate context selects the expected v2 reply.                        |
| Private attempt context                     | `noise-serial-context-v2`; bind `str005-noise-serial-v2`, both contract files, exact sources and complete evaluator inventory before effects     |
| Private terminal result                     | `noise-serial-result-v2`; retain the closed v1 result fields and immutable-input joins                                                           |
| Public projection                           | `bitaxe-stratum-v2-noise-serial-projection-v2`; retain the v1 fields with amended timing validation                                              |
| Fixture, host-cleanup and recovery receipts | Their unchanged v1 shapes may be reused only when bound to the exact new context and observations. No old receipt can be copied in as new proof. |

`resources.startedAtUs` continues to mean the first cleanup request or protocol
terminal decision. When cleanup starts, `resources.deadlineAtUs` is the fixed
`authorityDeadlineUs + 5000000`, with checked arithmetic. `deadlineMet` reports
actual complete release by that horizon, not a five-second cleanup guarantee.
Missing release remains unknown until the horizon; a missed horizon latches
false permanently. A late observed release fills missing facts without flipping
false to true. The retained worker-quiescent duration is still measured from
cleanup request to actual owner-observed completion/join.

An accepted v2 status requires all eight ordered stages, exact byte counts, no
failure, a closed socket, quiescent worker, disposed owned inputs and usable
measurements. Release and terminal decision must be within the original
120000-ms authority envelope. Remove the v1 validator's 5000-ms maximum for
socket-close/worker-quiescent durations; require their actual timings and valid
ordering within that envelope. Public `timings_ms.device_cleanup` is rounded
upward from its measured duration and bounded by 120000 ms for accepted results.
Other positive timing/count checks remain unchanged.

The v2 public `provenance.contract` digest binds a canonical JSON object with
exactly `base` and `amendment`, each containing the repository-relative `path`
and file-byte `sha256`. Use paths
`docs/hardware/str005-noise-serial-qualification.md` and
`docs/hardware/str005-noise-parity-scope-amendment.md`, the existing sorted-key
canonical JSON encoder, UTF-8 and SHA-256. Freeze the object and both source
files in the private context/artifact inventory; expose only its digest in the
public projection. No caller-supplied digest can substitute for reading verified
source. The complete evaluator identity must include both v1 and v2 readers
where used.

## Restoration and failure handling

For the first positive hardware exchange, require the normal path: actual job
completion, existing bounded fresh-possession reacquisition/restoration,
retained same-page baseline comparison, unchanged ledgers, inactive leases,
`mine_on_boot=false`, healthy candidate runtime and complete parent-observed
host cleanup before finalization. Reuse existing qualified restoration
operations. Do not require an additional automatic recovery subsystem as a
prerequisite to this positive test.

The dedicated `recover` action and separate recovery-only context are optional
future implementation for v2. Until implemented and independently verified,
`recover` must reject before input-dependent effects with exit code 1 and the
existing closed CLI result fields: `ready:false`, `device_effects:false`,
`hardware_qualified:false`, `error:"noise_recovery_unavailable"`. Preflight for
the positive normal-path campaign must not claim or require that optional route.
If normal restoration fails, the attempt remains unverified; collect only
available bounded safe observations, release host owners, and stop. An
unavailable recovery route cannot be replaced by an implicit reset, reflash, new
page baseline or an unbounded automated wait.

`finalize` and `review` remain mandatory. Missing cleanup, preservation or
accounting evidence can only produce an unverified result. An optional recovery
later proving safety cannot promote a sealed failed attempt. Implementing future
recovery retains the v1 recovery-only restrictions and additionally obeys this
amendment's cancellation and fixed observation horizon.

## Verification and task handoff

The two software preparation tasks may proceed in parallel. Before live effects:

1. Exercise real production crypto/transport and the local fixture for authority
   rejection, malformed/truncated/tampered/fragmented input, aggregate timeouts,
   peer correlation and the exact encrypted exchange. Keep negative hardware
   cases outside this positive milestone.
1. Test cancellation before/after each supported opaque API call. Hold a fake
   synchronous operation across revocation, then release it and prove no next
   crypto phase, network write or late success occurs. Observe actual worker
   completion separately. These tests establish adapter behavior, not a hard
   bound inside the real native crypto implementation.
1. Test independent heartbeat/deadline enforcement, first-cause retention,
   consumed admission, failed cleanup horizon, late release, conflicting effects
   and rejection of optional unsupported recovery. Preserve clock-failure rules.
1. Test v1/v2 separation, changed cleanup validation, immutable inventories,
   private-field rejection and independent normal-path finalization only after
   actual restoration and host cleanup. Schema consistency is not acceptance.
1. Run required software/native build, resource and ownership checks; publish
   exact firmware/Gate/fixture/evaluator sources and build the clean package.
   Future admitted installation/cycles then supply fresh startup/resource proof.

No native inside-crypto cancellation or universal crypto WCET proof is required
for readiness. An actual failed build, insufficient resources, stalled control,
unsafe revocation, unsuccessful protocol exchange or unproved restoration still
blocks the next effect or prevents acceptance; do not relax limits after a run.

Update the runtime, fixture and live task records to use this amendment and keep
cooperative dependency cancellation deferred and non-blocking for v2. Later
channel/job, accepted-share and final promotion tasks retain their independent
contracts and evidence requirements, using the current state-preserving runtime
rather than reviving recovery-006. Parity stays 90/95 until the final STR-005
acceptance and promotion work is actually complete.
