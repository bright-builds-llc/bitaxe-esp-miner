# USB bootstrap early-reader correction

Contract ID: `usb-bootstrap-correction-measurement-v1`.
Owner: `task-usb-bootstrap-drain-observability`.
This prospective contract authorizes one fresh no-mining correction measurement
only after implementation, verification, publication and clean native packaging.
It does not authorize another write in an existing context or V2 qualification.

## Evidence and targeted change

[Measurement 002](../parity/evidence/20260927-usb-bootstrap-reader-gap-measurement.md)
completed with its ordinary capture still unqualified. Its pins are:

- Context: `80cf7437feb6df2efa6130a7c8f463e4b679506b96e911d439c57d7c0c864ee0`.
- Result: `6b3329af0d9188b7618377b18317646b4d24a2b13bcc13725fde2059f932470d`.
- Seal: `a8ad2ef03819f3f413edb85c2fb85b8623c556fe0077ff01e7574f0265cb6386`.

Its host reader opened 4,423.674 ms after reset-command return. The first 92-byte
startup record queued successfully but exhausted its native drain deadline.
The actionable hypothesis is delayed host receiving. The evidence does not identify
which private IDF wait timed out or establish that earlier receiving is sufficient.

Keep the existing qualified reset command and backend unchanged. Change ordinary,
bounded, state-preserving `flash-monitor` capture to start one actively draining
receive owner earlier, while preserving both existing admission passes. Do not
combine this first correction with admission deduplication, a driver fork, a new
flashing dependency, firmware deadline changes or relaxed qualification judgments.
Non-capture flashing, standalone monitoring, recovery and campaign/UAT streaming
callback paths retain their existing behavior.

## Receive owner and admission

Prepare one bounded capture intent before reset. Only after the actual reset child
has exited and been reaped may a fresh candidate start the receiver. The candidate
must be unambiguous, match the retained physical digest, expose a known fixed
Serial/JTAG profile, be an accessible character-device node and have no foreign
holder. Derive its profile from the same existing I/O Registry candidate scan;
do not add another blocking scan before opening.

One receive-owner thread opens exactly one existing receive-only descriptor.
Use the existing read-only adapter, with no serial write, line-toggle/reset, ROM
sync or alternate-device discovery. Bind its physical digest, path, enumeration
token and opened descriptor's device/inode/character-device identity. Reject
candidate, node or binding changes; do not silently replace or reopen the reader.

Retain all three Handoff samples, final profile inspection and all three
MonitorAdmission samples while receiving continues independently. Existing
physical matching, foreign-holder checks and their deadlines remain mandatory.
Quarantine received bytes until every admission check succeeds. No classifier,
application identity claim or evidence promotion may consume them earlier.
Keep early diagnostic failures; never discard a timeout marker to obtain a pass.

Ordinary capture already returns its accumulated buffer before sanitization and
assessment. Preserve that behavior with one heap buffer bounded by the existing
16-MiB `MAX_MONITOR_BYTES`, bounded 4-KiB reads and a capacity-one terminal-result
channel and one fixed shared metadata/status record for binding and progress.
Overflow is a terminal integrity failure, never silent truncation. Start the
caller-configured capture interval at actual reader opening (30 seconds for this
qualification); admission must finish before it expires. Keep the existing 25-ms
idle poll interval. Request a 256-KiB host thread stack, keeping the payload on the
heap and the read chunk at 4 KiB. Spawn failure is terminal; the requested stack is
not a claim about actual OS memory use. Firmware stacks remain unchanged.

The existing physical lock's `File` may be held through `Arc<File>` references;
do not duplicate or replace its descriptor. The worker holds a reference before
attempting open and drops the serial reader before releasing that reference.
Thus failed join cannot release the physical lock while the worker retains a
serial descriptor. Cancellation must request worker stop and permit at most five
seconds for actual completion/join. A failed join forbids lease completion and
inferred reader-closed claims, preserves the crash journal and remains subject to the
existing outer supervised-process bound. No detached-success path or forced
thread termination is permitted. An actual descriptor-drop observation may still
record reader closure after failed join; it cannot establish joined completion.

Hold one signal-supervision guard from the reset/capture boundary through join.
Preserve any pending interruption at that boundary. The worker polls cancellation
and the existing signal state; the warm capture consumer must not reacquire or
clear that guard. Ctrl-C and every error path must still release actual resources
or report cleanup failure without inventing quiescence.

## Host timing version 2

The unchanged opt-in `--capture-bootstrap-timing` flag emits the new separate
`bootstrap-host-timing-v2.json` for this corrected capture. Keep v1 artifacts and
readers unchanged. Context version selects the exact expected filename/schema;
there is no fallback or reinterpretation of old measurements.

Keep all v1 identity, clock, event, counter, completion and earliest-failure fields.
Use schema `bootstrap-host-timing-v2` and add exactly:

- `captureMode`: `early_quarantined`.
- `captureDurationMs`: the configured duration, 30,000 for this qualification.
- `readerBindingSha256`: nullable hash of the bound candidate/descriptor metadata;
  no raw path, serial number or descriptor metadata is copied into the sidecar.
- `quarantinedBytes` and `capturedBytes`: bounded byte counts, not delivery claims.
- `quarantineReleased`, `readerJoined`, `captureOverflow`: explicit booleans.

The binding hash uses UTF-8 JSON with ordered keys `physicalIdentityDigest`,
`enumerationToken`, `port`, `fdDev`, `fdIno`, `fdRdev`. All values are strings;
descriptor numbers use unsigned decimal strings. This commits physical digest,
enumeration token, port path and opened descriptor device/inode/rdev values.
It binds the reader to the same candidate; it does not authenticate an application.

Retain the ten original event labels and add `candidate_observed`, `reader_bound`,
`quarantine_released`, `capture_deadline_reached`, `reader_joined` and optional
`cancellation_requested`. Bound events to 24 entries. Record timestamps at their
source against the same session `Instant`, including worker-thread events; never
substitute the later time that a result message is received.

Validate actual partial order, not the old total order:

`reset end -> candidate -> open start -> opened -> bound -> Handoff admitted -> MonitorAdmission admitted -> quarantine released`.

Both admission-start events remain real call bounds. First read may occur anytime
after opening and before closure. Closure precedes join. Successful completion
requires the full configured interval, actual closure/join, released quarantine,
one reader, no reopen and no integrity failure. Pending cancellation or an expired
capture before admission cannot become complete evidence.

Existing closed failure stages/categories remain valid. Version 2 may additionally
use stages `binding`, `quarantine` and `join`, and categories
`reader_binding_changed`, `capture_overflow`, `reader_join_timeout` and
`admission_incomplete`. Preserve the earliest observed cause, including failures
observed in the receive thread. Never serialize native error detail or payloads.
The receive thread records `quarantine_released` when it acknowledges completed
admission; `quarantinedBytes` is its accumulated byte count at that acknowledgment.
If admission never releases, it equals the final quarantined count. No bytes are
classified until admission, even if buffering continues until the terminal result.
Host and device clocks remain separate.

## One fresh context and effect boundary

Only `scratch/usb-bootstrap-measure/attempt-003` and exclusive sibling
`attempt-ordinal-3.json` are admitted. Use a fresh attempt identity.
`preflight --predecessor-receipt` must identify measurement002's pinned result.
Reject `--supersede-preflight` for this version: an accepted measurement supplies
ancestry, and no new failure-closure workflow is needed.

Use `usb-bootstrap-measure-context-v3`. Retain the base context fields, with the
existing predecessor object now binding measurement002's root/context/result/seal.
Add only `correctionAmendment` with this contract's path and hash. Do not include
v2's preflight-amendment/supersession fields or reinterpret their old semantics.
Derive the before-source and accounting expectations from independently reviewed
measurement002; do not alter the constants used by historical v1/v2 readers.
Bind `originalCampaign.record` to measurement002's `accounting-after.json`, and
join its before/after equality and original campaign ID. Its `install-0.claim.json`
supplies the physical-device anchor, replacing the old Channel005 claim path
only for version 3.

Require a changed clean published firmware/package/evaluator, exact Gate/tool and
native identities, the targeted production-seam regressions and a verified
measurement002 result/restoration/cleanup before assignment. Preserve encoded
source snapshots and validated, non-authoritative staging before exclusive
assignment. Publish the final inventory last. Existing, conflicting and interrupted
assignments retain their rejection/consumption rules; staging is never effect-eligible.
Archived tasks and prior contexts remain unavailable for effects.

Expected installed before-source is `3951a441606115798ea47eba0de1052d6590b3a0`, ELF
`0b9d275b5c75c58b3e25f8221d0be7dbb909313c4d9083aba2faf480af7b283f`.
Require fresh detector admission before the native page, fresh authenticated
possession, preservation and both ledgers before one installation 0. Expect next
ordinal 18, last completed 17, 1,560,000 ms charged and pending false; the original
240,000-ms ledger stays at masks 7. These remain expectations until freshly observed.
Use one same-page private baseline, unchanged NVS-disjoint segments and the same
reset. Native post-install possession, explicit restoration, unchanged accounting,
serial/window release and actual host cleanup remain required.

No mining, allowance, pool connection, fixture, probe, second installation/reset,
factory reset, rollback, fourth attempt, V2 authority or parity credit is added.
On failure, preserve the earliest cause and complete bounded cleanup; no replay.

## Prospective correction acceptance

Version 3 uses `usb-bootstrap-measure-result-v2`: retain all version-1 result
fields and add only `correction`, with `accepted` and closed boolean `checks`:
`captureQualified`, `hostTimingComplete`, `readerHeadroom`,
`nativeBootstrapComplete`, `zeroTxFailures`, `preservationAndAccounting`, and
`cleanupComplete`. Derive every check from retained evidence; `accepted` is their
conjunction and requires `measurement_complete`. Historical v1 results remain
unchanged. The compact review response adds `correction_accepted` for version 3;
all hardware/mining/qualification-credit non-claims remain unchanged.

Keep `measurement_complete` distinct from correction acceptance. Complete negative
measurements remain useful data and confer no qualification credit. A correction
passes only when all of the following are independently verified:

1. Ordinary capture is qualified, with exact identity, healthy startup and stable boot.
1. Complete v2 host timing proves the required early-reader ordering, one descriptor,
   admission before quarantine release, full 30-second capture and actual join.
1. Reader opened within 1,500 host milliseconds of reset-command call start. This
   is a new prospective headroom guard, not an upstream guarantee or clock alignment.
1. First bootstrap completed within the unchanged 2,000-device-ms budget; actual
   TX failures and measurement integrity flags are zero in every captured,
   boot-bound observation, with no retained failure marker. Periodic snapshots
   precede their own transmissions: this claim ends at the last observed snapshot,
   not at an unobserved final instant of the 30-second window.
1. Both ledgers, identity/settings and authorization high-water are preserved,
   mine-on-boot is false, leases are inactive and restoration/cleanup are complete.
   The final closed state must also confirm explicit device restoration.

Do not relax these limits after observing results. Preserve historical judges and
failed evidence. Even a passing correction does not replace the separate four-cycle
continuity and V2 live-qualification obligations.

## Fixed software admission proof

Build `//tools/device-session:tests` before preflight. Embed only stable source
commit and dirty status from Bazel's stable workspace status through a declared
`rustc_env_files` input; do not bind a volatile build timestamp. The test
`reader_correction_identity::reports_build_identity` emits closed
`usb-bootstrap-reader-test-identity-v1` metadata with `sourceCommit` and
`sourceDirty`. Admission requires the exact current published commit and false.
Unstamped Cargo builds cannot supply this admission proof.

The repository-owned checker executes the already-built binary with fixed arguments:

1. `--exact reader_correction_identity::reports_build_identity --nocapture --test-threads=1`.
1. `usb::early_capture::tests:: --test-threads=1`.
1. The admitted Node executable runs `--test --test-concurrency=1` with only
   `scripts/usb-bootstrap-measure/host-timing.test.mjs` and
   `scripts/usb-bootstrap-measure/correction-judge.test.mjs`.

Require actual zero exits, no signal, nonzero passed tests and zero failed or
ignored tests; the identity run must execute exactly one test. Bound the complete
check to 120 seconds and combined output to 64 KiB. No caller-selected program,
filter, test or nested build is permitted. Retain the exact test binary, stable
build provenance and `snapshot/reader-correction-check.json`. The closed receipt
`usb-bootstrap-reader-correction-check-v1` binds command
`reader-correction-regression`, firmware commit, source-inventory SHA256, Node and
test-binary SHA256, embedded identity, and the three fixed run names
`identity`, `rust-capture`, `node-judges` with exit/signal/pass/fail/ignored counts.
Historical review validates these bindings and artifact hashes without executing
software; hot admission rechecks the bound pins. No test output or arbitrary logs
become public evidence.

## Verification and publication

Test real PTY traffic during deliberately slow admissions, same-descriptor retention,
no pre-admission publication, profile/physical/enumeration/node changes, foreign
holders, overflow, cancellation, failed join with retained lock ownership, signal
handoff, deadline boundaries and cleanup on every failure. Exercise the production
ordinary-capture path, preserving separate callback paths. Test v1/v2 historical
readers, v3 predecessor/source/version rejection and correction judgment negatives.

Run ordered Cargo checks, affected canonical/Gate checks, native packaging/stack
and ownership checks, standards, reference, redaction and read-only parity/progress.
Publish the reviewed correction and this contract before a fresh context/effect.
Also port the already tested evidence-write-safe cleanup to the older V2 operator
and verify its real-process failure cases before any later V2 campaign; that fix
must not change historical evidence schemas. Parity remains 90/95 throughout.
