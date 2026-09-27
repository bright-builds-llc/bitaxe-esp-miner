# USB bootstrap drain measurement contract

Contract ID: `usb-bootstrap-drain-measurement-v1`.
Owner: `task-usb-bootstrap-drain-observability`.
This is prospective: publication alone does not mean these interfaces exist.

## Objective and limits

Distinguish the host reader gap from native write/drain behavior observed in
sealed Channel005. Preserve that unverified result and its strict classifier.
One fresh no-mining measurement is authorized only after this contract and its
verified implementation are published and the clean package passes native checks.
This measurement confers no V2, continuity, mining or parity qualification.

Use ordinary fixed Serial/JTAG and native foreground Web Serial possession.
Preserve Controller 0.4 / serial 0.2 / possession 0.2, CPU/task configuration, stack sizes,
2000-ms native record budget, independent 2800-ms heartbeat revocation and existing
safety limits. No driver fork, private IDF structure offsets, register writes,
concurrent port owners, new reset sequence, factory reset, UART/pins, credentials,
Work Lease signer, allowance issuance, fixture, pool connection or ASIC work.
Do not replace the qualified reset backend during this measurement.

Three hypotheses remain open: host reader availability is late; native idle
completion is delayed/unobserved; or a deadline-edge defect contributes. A
separately identified sub-tick rounding behavior may end polling early. Correct
it only after a production-seam regression, without extending the absolute
2000-ms budget. It does not explain the historical exact 2000-ms failure.

## Exclusive attempt and command surface

New command family: `just usb-bootstrap-measure ACTION ...` backed by Bazel.

| Action | Arguments and behavior |
| --- | --- |
| `preflight` | `--private-root ROOT --firmware-root ROOT --gate-root ROOT --package-manifest FILE --predecessor-receipt FILE`; validate all bindings before exclusive assignment |
| `operator-start` | `--private-root ROOT`; start one persistent actual supervisor parent, return initializing acknowledgment |
| `operator-request` | `--private-root ROOT --request FILE`; one typed, idempotent operation |
| `operator-status` | `--private-root ROOT [--request-id ID]`; read-only progress/result |
| `finalize` | `--private-root ROOT`; seal available observations only after writers/owners stop |
| `review` | `--private-root ROOT`; independent read-only verification |

Only `scratch/usb-bootstrap-measure/attempt-001` and its exclusive sibling
`attempt-ordinal-1.json` are admitted by this version. Interrupted assignment
stays consumed. Do not touch any V2 attempt or qualification-ordinal marker.
Context schema is `usb-bootstrap-measure-context-v1`; result, seal and cleanup
schemas are separate `usb-bootstrap-measure-*-v1` records. Their closed parsers
must reject extra fields and source/evidence drift.

Bind the active task, this contract, clean pushed firmware/Gate commits, exact
package segments/source/ELF, public deployment trust, native resource audit,
page/client, complete collector/validator sources, actual host executables and
immutable predecessor evidence. The predecessor is exactly Channel005:
context `dcfce9bd9bdaf8ea8fb59d021b0449fec10b58a651a353aa24ce360e4a4ff79a`,
result `1b687eda11fe233677f77c4a77ee462912a6ddb5caf57ad9bfc7567fd4782c2e`,
seal `3b77bd5d8b83433ba18b758a3ec579806a91c866e594cb0254656d7a313ea342`.
Validate its original unverified verdict, complete inventory and released owners
read-only. Expected installed before-source is firmware
`14f6d6d98ad939db065542db66dd8aad77f097d7`, ELF
`cbe496ec96b5f20cfaeabf10f6e41105de916310e4dc34fc3a77f07d1091e8cf`.
That expectation requires fresh authenticated confirmation, not historical trust.

Reuse the verified durable-operator mechanics through fixed code-bound adapters,
not caller-selected loaders. Retain protected short Unix sockets,16-KiB bounded
messages, five-second exchanges, unique 32-hex request IDs, atomic results,
exclusive phase claims, actual child-exit observation and parent-loss rejection.
Initial acknowledgment is bounded to 10 seconds; automated setup to 900 seconds.
No idle human-response deadline. Daemon loss is terminal/unverified; no adoption,
restart or effect replay. Evidence-write failure must not prevent cleanup.

IPC schemas are `usb-bootstrap-measure-operator-request-v1`,
`usb-bootstrap-measure-operator-query-v1`,
`usb-bootstrap-measure-operator-response-v1` and
`usb-bootstrap-measure-operator-status-v1`. Reuse the existing closed field/null
semantics, but reject cross-task schemas. Browser witness schema is
`usb-bootstrap-measure-browser-closure-v1`, with exactly schema, source
(`native-ui-observer`), contextSha256, closed, lastSequence, lastStateSha256 and
observedAtUnixMs. It asserts disappearance of the exact owned tab; unrelated
browser windows/tabs must not be closed.

Requests contain schema/contextSha256/requestId/action/payload. Actions are only
`install` with `{index:0}`, `finish-cleanup` with `{browserWitness}` (nullable for failure-only cleanup), and `stop`
with `{}`. The command exposes no public unmanaged serve or signing option.
Internal reuse must preserve all historical V2 readers and archived-task rejection.

Context's closed top-level fields are `schema`, `taskId`, `contract`, `attempt`,
`firmwareRoot`, `gateRoot`, `package`, `beforeSource`, `gate`, `trustSha256`,
`predecessor`, `sourceInventory`, `nativeReadiness`, `hostTools`, `originalCampaign`,
`expectedAccounting`, `installIndices`, `miningAuthorized`. Contract is path/hash;
attempt is id/ordinal 1; beforeSource uses firmware_commit/app_elf_sha256;
predecessor uses root/contextSha256/resultSha256/sealSha256. Gate binds commit,
bundleSha256, pageSha256 and pageRelativePath. Host tools are closed pinned roles
for espflash, managed esptool and Node, including versions/digests. Source entries
are exactly path/sha256/length. InstallIndices is `[0]`; miningAuthorized is false.
Nested package/native/trust/accounting records come from fixed repository
inspectors with their existing exact shapes; no CLI-supplied verdict is accepted.
Runtime actor identities are observed when launched, not invented at preflight.
Require the predecessor's same physical device, but capture a new private baseline.

## Device and browser sequence

1. Detector-admit exactly one Ultra 205, retain its physical lease, and start the
   page from the admitted loopback supervisor in a dedicated native window.
2. Native Connect authenticates the expected installed pair. Capture a fresh
   private baseline and both ledgers. Require inactive leases, mine-on-boot false,
   next18/last17/1560000-ms/pending-false qualification accounting and the matching
   exhausted original 240000-ms ledger with reserved/completed masks 7.
3. Close/flush the page's serial session. Require actual ownership release.
4. Consume exactly one installation 0 claim. Use existing ROM board-info admission,
   validated NVS-disjoint segments, the existing qualified reset, and 30 seconds
   of receive-only capture, with the existing outer command/cleanup bounds.
5. Configure the candidate using the existing generic Gate configuration while
   retaining the same page's private baseline. Native reconnect must authenticate
   the exact candidate. Collect fresh preservation/restoration and both ledgers;
   require unchanged accounting and identity/settings/authorization high-water.
6. Close/flush serial, observe actual native window closure, stop/reap owned host
   children and daemon, and verify listener/socket/serial-holder absence.

The native page exposes only Connect, initial accounting, candidate configuration,
post-capture restoration/accounting and close/flush. No restart, probe, protocol
run, Prepare/Load/Start/Renew/Suppress controls or backend routes. Keep emergency
Stop/Restore available after failure solely for restoration, never restart, new
flash, work or an implicit second measurement. The existing generic Gate interface needs no new wire
command or new V2 scope. No authority or credential directory is accepted.

A failed capture does not authorize another write. Recovery-only candidate
configuration is permitted when independently checked artifacts prove the write
completed and the exact candidate is executing; then fresh possession must
confirm it. This permits collecting failure observations and restoration, never
upgrading the ordinary flash verdict. Unknown/partial writes or unavailable
identity remain unverified; do not add reset/reflash/rollback implicitly.

## Host observation interface

Add opt-in `flash-monitor --capture-bootstrap-timing`, requiring protected
`--evidence-dir`, `--redact-evidence`, ordinary state-preserving update and no
provisioning/stimulus/dry-run combination. Default commands and historical
capture/result schemas remain unchanged. Create exclusively, after capture and
cleanup attempts, `<evidence-dir>/bootstrap-host-timing-v1.json` with mode 0600.
Collect only bounded memory observations during the measured interval; existing
ownership journals remain intact. Failure must retain partial timing evidence.

Closed fields: `schema`=`bootstrap-host-timing-v1`, `clock`=`host_monotonic`,
`origin`=`usb_session_acquired`, `physicalIdentityDigest`, `sessionNonceSha256`,
`events`, `resetChildSequence`, `readerOpenCount`, `readerReopenCount`,
`firstReadBytes`, `missingStages`, `overflow`, `clockDiscontinuity`,
`captureComplete`, `cleanupComplete`, `earliestFailure`.
Use the actual retained lease nonce and one UsbSession Instant origin. When
session acquisition never succeeds, origin and both identity joins are null,
events are empty, required stages are missing and captureComplete is false;
never manufacture a nominal lease. A failure writing the sidecar is recorded by
the outer result, not falsely claimed to exist inside the missing sidecar. Each of
at most 16 events contains exactly `stage` and `elapsedUs`. Closed stages are:
`reset_command_call_start`, `reset_command_call_end`, `handoff_start`,
`handoff_admitted`, `monitor_admission_start`, `monitor_admitted`,
`reader_open_start`, `reader_opened`, `first_nonempty_read`, `reader_closed`.

Record first traversal; retries increment bounded counters. Missing stages are
derived, never caller-authored. A reopen, overflow or clock error makes the
initial chain incomplete. Reader closed means the actual reader object dropped local descriptor ownership,
not an observed successful close syscall; cleanup completion is independently
checked. Host earliestFailure is null or exactly stage/category. Closed failure
stages are preparation, session_admission, reset, handoff, monitor_admission,
open, read, close, clock, overflow, cleanup and evidence. Categories are the
following frozen native labels, plus preparation_failed, clock_discontinuity,
observation_overflow and evidence_write_failed:
`concurrent_repo_session|foreign_holder|transport_absent|identity_drift|
runtime_profile_unknown|handoff_unsupported|handoff_rejected_unsafe_state|
handoff_ready_timeout|handoff_commit_timeout|bus_reset_timeout|
same_worker_after_commit|handoff_transition_timeout|bootloader_ambiguous|
physical_identity_drift|bootloader_sync_failed|rom_admission_failed|
application_reappearance_timeout|application_identity_mismatch|recovery_required|
bootloader_connect_failed|usb_enumeration_lost|flash_failed_before_transfer|
flash_failed_after_transfer|monitor_failed|cleanup_failed|recovery_not_observed|
repeated_boundary`.
Never serialize UsbSessionError.detail. Preserve an earlier operation failure
when export fails. Host measurement failure and the ordinary device/capture
verdict remain separate. Nullable sequence/first-byte/failure fields
represent missing observations, not zero. Earliest failure uses a closed stage
and category, never arbitrary error text. Call bounds include supervised launch,
execution and reap; they are neither physical reset edges nor exact child spawn/
exit times. Do not subtract host timestamps from device uptime or infer peer
receipt from queue/FIFO state.

## Device observation interface

Preserve `usb_tx_failure schema=v1` bytes and its existing retention/Hello behavior.
Add separate boot-lifetime retention for the first bootstrap diagnostic attempt
(success or failure) and first write failure, plus saturating actual-failure and
measurement-replay counters. Never clear the new observations on Hello. Label
categories explicitly at their production call sites; byte length is not identity.

Emit at most one selected retained observation through an existing replay
opportunity, alternating slots. No immediate logging, allocation, durable write,
waiting lock or extra wait in the new recorder. Formatting happens afterward.
Target at most 512 bytes additional retained storage and 128 bytes per in-flight
measurement object; audit actual native stack frames separately without enlarging
stacks. Omit optional native register sampling initially: it cannot identify the
private mutex/idle wait and is unnecessary before the timing evidence exists.

Marker prefix: `usb_tx_observation schema=v1`. Closed fields are `slot`,
`category`, `record_kind`, `outcome`, `stage`, `start_ms`, `end_ms`, `record_bytes`,
`queued_bytes`, `queue_calls`, `queue_positive`, `queue_zero`, `last_queue_return`,
`drain_calls`, `drain_timeout`, `drain_success`, `drain_other`, `first_drain_ms`,
`last_drain_ms`, `max_drain_ms`, `last_drain_return`, `actual_failures`,
`replay_attempts`, `replay_completed`, `flags`, `redacted=true`.
Limit a marker to 1024 bytes. Slots are `first_bootstrap|first_failure`;
categories `bootstrap_diagnostic|diagnostic|diagnostic_replay|hello|heartbeat|
receive_credit|control_reply|resynchronization`; outcomes `completed|failed`;
stages `completed|write|write_timeout|flush_timeout|cancelled`.
Record kinds are `startup_progress|statistics_startup|boot_identity|admission|
retained_diagnostic|tx_observation|protocol|resynchronization`, passed explicitly
from producer sites. Ordinary diagnostics without a session use
bootstrap_diagnostic; replay retains diagnostic_replay even without possession.
The first-bootstrap slot selects the first ordinary bootstrap diagnostic.
When drain_calls is zero, both drain timestamps are zero; otherwise they contain
observed device uptime, which may itself be zero. The count distinguishes absence. Return values are
signed numeric native outcomes, not interpretations of private driver state.
Flags form a closed bitmap: 1 counter overflow, 2 clock discontinuity, 4 invalid
native result; other bits are forbidden. Counters describe the snapshot before the current marker transmission;
replay completion requires actual drain confirmation. A failed replay is an
actual failure but never replaces the retained first failure. Successful replay
never increments the failure count. No payload, session ID, key or credential
may enter these measurements.

## Judgment, verification and continuation

Bind device observations to one exact current identity and stable monotonic
boot-discriminator/startup stream in the same capture. Reject mixed/rebooted
captures; adjacency alone and cross-clock subtraction are not boot binding.
Reaching the configured 30-second capture end is normal observation completion;
it is distinct from a native write/drain timeout.

A measurement may be complete while the ordinary capture is unqualified. Its
result must retain that failed exit/verdict, exact first cause and timing flags;
it must always state `hardware_qualified:false`, `mining_authorized:false` and
no continuity/parity credit. Complete measurement requires exact identities,
ordered complete host timing, valid boot-bound device observations, fresh
before/after unchanged ledgers and baseline, and independent actual cleanup.
Missing stages/data or restoration remain unverified. Do not infer root cause
from a timeout code alone, a matching byte length or an absent failure.

Result fields are schema/contextSha256/status/firstFailure/capture/measurement/
restoration/cleanup/inputs/hardware_qualified/mining_authorized/qualification_credit.
Status is measurement_complete or unverified; the last three fields are `false`,
`false` and `"none"`. Capture has exitCode/flashVerdictSha256/qualified; measurement
has hostTimingSha256/deviceObservationsSha256/complete; restoration and cleanup
have receiptSha256 plus confirmed/complete respectively. Missing codes/digests
are null. FirstFailure is null or source/stage/code/observationSha256, using
closed allowlisted categories. Inputs is the exact input inventory before the final result/seal, including
retained operator facts and the derived cleanup receipt. Seal has
only schema/contextSha256/files. A complete measurement may retain a capture
failure; it never supplies qualification credit.

Finish-cleanup retains preliminary facts and stops the actual supervisor.
After stop writes its disposition and the daemon is absent, finalization performs
fresh absence checks and derives the final cleanup receipt from those facts.
No receipt may depend on a disposition that has not yet been written. A null
browser witness permits process cleanup but prevents a complete receipt.
The finalizer rejects any live daemon, supervisor, installation or detector even
for an unverified result; incomplete cleanup is not permission to seal live files.

The separate cleanup receipt joins actual daemon/supervisor/installation/detector
owners, native-ui-observer browser witness, actual child exits, serial-holder and
known listener/socket absence, and restored baseline/accounting. No dummy fixture,
private pool port or invented successful exit. Seal once after all writers stop;
review is read-only, uses retained inputs and cannot upgrade a failed result.
Its closed fields are schema/contextSha256/browserWitnessSha256/
baselineReceiptSha256/accountingBeforeSha256/accountingAfterSha256/owners/
operatorDispositionSha256/absence/complete. Owner entries bind role, checked
process identity and exitObservationSha256; the daemon exit digest may be null,
using its source-bound stopped disposition and actual absence without inventing
an exit code. Absence binds observedAtUnixMs, serialNodeCount,
serialHoldersAbsent, supervisorListenerAbsent, operatorSocketAbsent and
ownedGroupsAbsent. Complete requires all checks and fresh restoration/accounting.
Public projections contain only approved counts, timing, categories, booleans
and provenance; protected paths, identities and raw data stay private.

Tests must exercise real production queue/drain logic with 92-byte writes,
readiness below/at/above 2000 ms, cancellation, never-ready, clock/counter errors,
first retention across Hello and replay accounting. Test the sub-tick boundary
separately. Use real PTY/process boundaries for host ordering, fragmentation,
cleanup, parent/client loss and atomic results. Test authentication, same-baseline
retention, missing/altered artifacts, wrong pair/task/attempt, changed ledger,
extra routes, interrupted assignment, redaction and independent review. Verify
old v1 markers/readers/classifiers remain unchanged and still reject TX failures.

Before effects run ordered Cargo, canonical Bazel, Gate, native/package,
ownership, reference, privacy, standards, task-boundary and parity checks; never
overlap Cargo and Bazel. Publish exact identities first. The software work may
correct the separately proven sub-tick bug; it must not claim that correction
resolved Channel005. After the measurement, select a targeted mechanism-based
correction, verify it, and establish a fresh guarded qualification contract before
any further channel/mining attempt. Do not reuse Channel005, allocate Share002
from an unverified predecessor, refund/reset a ledger, or advance parity 90/95.
