# CPU0 telemetry cadence qualification

Owner: `task-cpu0-telemetry-cadence-qualification`, explicitly requested 2026-09-13.
This is prospective authority and evidence policy. No previous result changes.
Preserve ADR-0021/0023, the accepted four-cycle amendment, the fixed USB ownership
rules, protected evidence policy and progress-gated hardware-attempt policy.

## Measurements and fixed acceptance criteria

Preserve the actual 500-ms sleep, CPU0 affinity, priority 5, existing stacks and
safety deadlines. Measure device-monotonic iteration intervals, total processing,
live-publication/log/pruning stage durations, observed CPU/priority and closed
publication outcomes. Unchanged data/no subscribers are distinct from errors;
queue acceptance is distinct from asynchronous completion and host receipt.
Three fixed 60-second capture summaries use no more than 2048 static bytes.
Recording never allocates, logs, persists or waits on locks. Loss, overflow,
clock discontinuity, reboot and incomplete captures reject qualification.
Frozen evidence survives shutdown/cooling; uncollected summaries cannot be rearmed.

Each phase requires at least 60 complete intervals, CPU0 / priority 5 throughout,
at least 95 percent of intervals \<=750 ms, every interval \<=1500 ms, every
processing duration \<=500 ms, zero projection/serialization/queue/asynchronous
send failures, and one connected live subscriber throughout. Include boundary
intervals so truncation cannot hide a stall. These are prospective qualification
guardrails, not an upstream timing guarantee or an exact 500-ms period.

Idle/USB capture starts after authenticated arming; mining is armed before
Start and starts at first successful work dispatch of the next admitted generation.
The endpoint observation reads only the currently connected STA IPv4, actual
HTTP port, readiness/freshness and admitted boot/session binding. No SSID/MAC,
credentials, raw Wi-Fi snapshot or stale address fallback. Arm/review requires
idle fresh possession, rejects unfinished cleanup and grants no work/liveness.

## Passive observation

A repo-owned observer using `PlainWebSocket` connects only to the exact fresh
endpoint and `/api/ws/live`, with no Origin header (the existing gate allows it).
It performs no control requests, redirects, discovery or `/api/ws` log subscription.
It holds one connection for at most 360 seconds, bounds frames/read/connection
operations and retains only allowlisted timing/count/outcome metadata. Payloads
and endpoint values never enter published evidence; endpoint remains protected
local input. Host and device clocks are independent. Web Serial remains the sole
application control transport. This contract expressly authorizes passive network
observation, unlike the completed Hello-only contract.

## Source, commands and admission

Create protected mode-0700 parent and absent supervisor child; use `umask 077` and
separate mode-0600 sibling logs. Bind all reachable validators, observer binary,
firmware/Gate clean pushed commits, bundle/page and package/artifact hashes.
No factory reset or NVS seed; preserve original snapshots and immutable attempts.

Use `just fixed-usb-qualification cadence-preflight` with firmware/gate roots,
exact commits, canonical manifest, private root, protected authority directory,
previous sealed result, verified-progress input and suggested difficulty 1000.
This creates only the dedicated cadence context; old contexts/judges/tasks never
gain new authority. Use `serve`, `record-cycle` and `cadence-judge` for this context.
The observer binary is launched by the supervisor using its admitted exact hash
and a private stdin endpoint handoff; it never receives pool credentials.

Run all relevant tests and required ordered Cargo/Bazel/Gate/native/reference/
redaction/standards checks. Publish Gate, pin its archive, publish firmware and
build the exact clean package before hardware. Run `just detect-ultra205`, require
one admitted physical device and native profile, preserve its physical lease and
pass repo-owned board-info before any ROM write. Use `just flash-monitor --board 205 --port <admitted> --manifest <exact manifest> --evidence-dir <fresh child> --capture-timeout-seconds 30 --redact-evidence` for the contract's bounded startup
capture. Admit only disjoint ordinary-update segments. Prove complete cleanup
before browser reuse. Four fresh update/reconnect cycles of the exact pair require
stable startup, fresh possession, 65536-byte requests/responses, identity/settings
preservation, mining disabled and zero owned holders. Snapshot all runtime and
observer artifacts before subsequent builds can replace them.

## Phases and live allowance

After four cycles, obtain fresh authenticated endpoint evidence, start the passive
observer and prove handshake before capture. Keep the same admitted boot and
subscriber through the three ordered phases:

1. Idle: 60 device seconds, normal serial supervision and one live subscriber.
1. USB: 60 device seconds, twelve 65536-byte request/response probes scheduled
   five seconds apart. Never overlap requests; missing/incomplete probes fail.
1. Mining: 60 seconds from first successful dispatch; signed renewals, continued
   running state and increasing dispatch counts in each ten-second segment.

Use the existing signed normal allowance type, reserving its full 180000 ms with
no refunds. This cadence judge requires real work, not an accepted share; old
normal-acceptance judges retain their accepted-share requirement. The prior
sealed recovery result selects expected ordinal 16 and 1200000 ms charged;
a fresh authenticated ledger must agree with the validated predecessor. Never
reset the exhausted original 240000-ms campaign or the separate attempt ledger.
Each verified-fix continuation needs a fresh accounted ordinal and exact pair.

Keep the conservative 400 MHz / 1100 mV / fan 100% profile, fresh 4.5–5.5 V, \<=15 W,
\<75 C and nonzero RPM. Preserve independent 2800-ms heartbeat revocation and
shutdown initiation \<=3000 ms after the last valid advancing heartbeat.
After the fixed 60-second mining capture and its at-most two-second final boundary
interval/processing tail, suppress only application heartbeats using the existing
qualified harness mechanism while the passive observer stays connected. Require
actual device-local revocation/shutdown timing, ordered safe stop and bounded
cooling; fan 100% until fresh \<=45 C, then qualified 30%, within 120 seconds.
Retain the existing pre-reset shutdown reserve. No electrical fault injection.
Keep the observer through the recorded suppression event plus at least five
seconds, covering the required three-second shutdown initiation. Then close and
reap it while cooling continues; retain its measurement-end witness separately
from the later collection timestamp. The 360-second observer lifetime does not
bound or shorten cooling. Wait at least 145 seconds after the fault before fresh
admission; elapsed time is not restoration proof. Fresh possession and review
occur only after restoration. Snapshot/export of
diagnostics must not block ongoing authority enforcement.

## Failure, sealing and completion

Any identity, health, timing, subscriber, observer, ledger, integrity or cleanup
failure preserves the earliest cause, triggers existing safe restoration where
possible and leaves qualification unverified. No host timeout is stop proof.
Keep diagnostics through cooling; reap observer/supervisor descendants and
release browser streams/locks and serial ownership. Human readiness waits have
no deadline; machine protocol/effect/cleanup bounds remain enforced.

Seal context, exact artifacts, summaries, observer receipts, phase/probe evidence,
generation/accounting joins and actual cleanup. Commit only redacted metadata.
No credentials, signed grants, pool endpoints/users/addresses, private device
fingerprints, raw network frames or operational identifiers are published.

A pass closes cadence, USB qualification and migration with exact evidence; BWG
and STR-005 still need their own successor contracts. Failure retains those tasks
with the precise blocker. Never relax thresholds after measurement, retry an
unchanged boundary, refund a reservation or automatically promote parity.

## Unissued preparation supersession

A permission-stage preparation failure before any signed issuance must remain
unverified. Use a native foreground click for Chrome's original user gesture;
do not bypass the SDK's gesture guard or synthesize possession. Read-only
recovery may verify the exact installed identity, safe baseline and unchanged
ledgers, then close all owned browser, USB and supervisor resources.

`just fixed-usb-qualification cadence-close-unissued --private-root <old root> --input <protected observed closure input>` is an effect-free classifier. It
requires no issued/consumed/result or mining evidence, closed safe-baseline
journal state, no running observations, unchanged authenticated ledgers,
parent-observed cleanup and the actual original failure. It seals all original
files and a typed unissued closure without claiming hardware qualification.

After verified native-gesture remediation and publication of the bookkeeping
correction, `cadence-preflight` may accept
`--supersede-unissued <old root/unissued-closure.json>`. The completed charged
predecessor must remain the same. Increment a separate preparation ordinal,
create a fresh context and allowance identifier, preserve the original
`ordinal-N.json`, and append `ordinal-N-preparation-K.json`. The device ordinal
N is still unused, not refunded or replayed. Validate the sealed predecessor,
exclusive assignment, corrected source pair and unchanged next/charged ledger
before any issuance. No issued, consumed, funded, active or unclean attempt is
eligible. Old preparations cannot be resumed as active authority. Four fresh
cycles and all original measurement, safety, privacy and cleanup bounds apply.

The sealed closure records `continue_after_manual_remediation` for this native
gesture boundary; the host bookkeeping repair is separate. A progress reason of
`manual_remediation` is admitted only with that validated, sealed unissued
predecessor and its native-gesture evidence. Ordinary first preflights still
require `software_correction`. No consumed allowance is eligible for this path.

## Pre-mining failure closure and preparation 3

The 2026-09-13 owner-approved continuation implements
`cadence-close-premining --private-root <failed preparation>` and
`cadence-review-premining --private-root <failed preparation>` through
`just fixed-usb-qualification`. They perform no hardware, credential or signing
access. The close command exclusively writes the protected sibling
`<failed preparation>.premining-closure.json`; review is read-only. Neither
command adds, rewrites or removes any file inside the already sealed root.

The supported failure class is the observed idle-review `probe_admission`
before USB-load/mining. Independently verify its sealed inventory, exact
context/charged predecessor and permission-failure ancestry, thirteen artifacts,
five flash/process receipts, four cycle audits, idle arm, observer journal,
parent-observed earliest failure and authenticated ledger observation, final
safe baseline and real host cleanup. Preserve parent-observed provenance:
do not manufacture a browser journal failure or missing device summary.

Initial closure must match the already published preparation-2 inventory SHA-256
`6895f2fcfe161a0d7617b51b02b9f12fcdf49454c9ee540295c31f40b463a5a7`
and its recognized producer. Internal consistency alone cannot admit a
replacement inventory. The CLI and environment provide no override for this
accepted evidence anchor; coherent evidence-plus-inventory rewrites fail.

Require no issuance, consumption, loaded grant, work, renewal, suppression or
later phase evidence. Fan-only cooling and cleanup transitions are valid when
consistent with that no-work evidence; they may temporarily report a pending
restoration, but the final baseline and ledger must be inactive and unchanged.
Any missing, conflicting, altered or unsafe-path evidence rejects closure.
The receipt is unverified, retains the original failure, records unchanged
accounting and grants no continuation authority. Existing permission-only
closure and funded-result judges remain unchanged.

`cadence-preflight --supersede-premining <sibling closure>` is mutually exclusive
with `--supersede-unissued`. It revalidates all closed evidence and requires
`software_correction`, a changed clean published pair, the same charged
predecessor, an exclusively assigned preparation number, a fresh allowance ID
and an absent protected evidence child. Preparation 3 expects still-unused
ordinal 16 and 1200000 ms charged; fresh authenticated device accounting must agree
before issuance. Preserve all earlier markers. Reject duplicate/conflicting or
partial assignments, recursive ancestry, archived-task authority and serving
sealed failed preparations; historical read-only validation remains available.

Publish and test these changes with corrected Gate ad16c23d, build the exact
clean package, close/review preparation 2 and create the new preparation context
before hardware. Repeat initial installation and all four cycles; no old cycle
or capture is transferred. Then run the existing idle/USB/mining sequence,
heartbeat suppression, observer tail/reaping, ordered shutdown/cooling and
fresh authenticated final review. Timing, storage, stack, publication, safety,
privacy and 360-second observer limits above are unchanged. Fully charge the
one 180000-ms reservation once consumed: expected next 17 / last 16 / 1380000 ms with no
pending reservation. An accepted share remains unnecessary for this judge.

A missing or failed proof keeps qualification unverified. Another attempt needs
verified targeted progress and appropriate guarded admission; recurrence of the
same authoritative boundary after its verified fix stops under the existing
hardware-attempt policy. On complete evidence, finalize/archive cadence,
qualification and migration and update BWG/STR-005 dependencies without granting
them effects. Otherwise retain precise active blockers and publish the truthful
result. Parity remains 90/95 throughout.

## Verified correction after preparation 3 USB capture failure

The sealed preparation-3 USB failure is eligible only for a distinct v2
pre-mining classification, never the v1 idle-review matcher. Its accepted
inventory SHA-256 is
`1936ad2be4fffa59dbc650fd833bc335e280dd31394ab3378e5d2760aa1ab580`,
with producer SHA-256
`66653e866e924d7811acf1b13b3889f20fec2c92c5c31588d00f4f040b171d11`.
Both are anchored before new effects. Close/review may dispatch by supported
sealed schema and exact hash; no arbitrary failure fallback or CLI/environment
anchor override is allowed. Independently validate the original passing idle
receipt, all twelve bounded probes, first failed USB review and later identical
fresh diagnostic review, unchanged numeric idle summary, absence of mining and
issuance, unchanged ledger, full artifact/cycle/observer chain and actual cleanup.
Preserve the old global-loss-derived idle flag change as failed evidence.

The v2 sibling receipt remains unverified and grants no continuation authority.
Historical v1/v2 readers revalidate complete ancestry without reviving old roots.
Only verified targeted correction, a changed clean published pair, the same
charged predecessor, fresh accounting and an exclusive new preparation/allowance
identity admit preparation 4 for still-unused ordinal 16. Preserve both earlier
sibling receipts, all markers and every sealed byte.

The targeted correction separates asynchronous outcome accounting from timing
ownership contention and avoids the unused retained-log snapshot when there are
no log subscribers. It does not relax real capture-loss, overflow, timing,
publication, storage or safety requirements. The same complete hardware workflow
above must repeat: initial state-preserving installation, four new exact-pair
cycles, idle/USB/mining captures, heartbeat suppression, ordered shutdown,
cooling and cleanup. Every reservation actually consumed remains fully charged.
No accepted-share requirement is added, no exhausted allowance is reused and
no parity transition occurs. A repeated authoritative boundary after its
verified fix stops under the existing hardware-attempt policy.

## Ordinal 17 live-stage diagnosis

Ordinal 16 completed its charged work and safe restoration but failed the
mining interval percentile. Its canonical completed unverified result is the
next charged predecessor; ordinary cadence preflight must allocate ordinal 17
with a fresh normal allowance and the unchanged 180000-ms reservation. Do not
use unissued supersession or refund any of the 1380000ms already charged.

Before effects, publish the positive-only boot/mount installed-asset version
fact and diagnostics v2. The fact reads/parses the actual mounted version until
success; missing/malformed initial reads remain retryable, and the firmware
build label is never substituted. Dynamic facts stay freshly collected. A
future same-boot WWW replacement/remount must invalidate the retained version;
current supported updates reboot. Retaining version identity intentionally does
not model later transient filesystem-read availability.

V2 retains all original fields and adds each phase's `maximumLiveStagesUs`
(fixed array of eleven independent maxima) and `worstInterval` containing
`previousExecutionUs`, `previousLiveStagesUs` (eleven exclusive durations) and
`gapUs`. Stage order is: visible state, platform identity, health/safety,
confirmed settings, settings transaction wait, settings NVS read, Wi-Fi,
publication-order wait, projection completion, retention, serialization/queue.
The worst interval is joined to the PREVIOUS iteration's work. Its gap includes
nominal sleep and other scheduling/overhead; it is not attributed to a cause.
Never sum independent maxima as if they describe one iteration.

Measure on the stack and merge once per iteration. Recording still cannot
allocate, log, persist or wait on a lock; total added static storage remains at
most2048 bytes. Missing/invalid current measured-stage observations fail rather
than appearing as measured zeros. A pre-capture previous iteration with no live
subscriber may legitimately have zero stage durations in its boundary witness.

New contexts set `cadence_diagnostics_version:2`; their phase completion and
signing require v2 metadata. Historical contexts without that field retain v1
validation. Gate and host readers preserve old evidence, while current v2
qualification checks coherent worst-interval arithmetic and stage bounds in
addition to every unchanged cadence/publication/safety criterion. Publish the
coordinated pair, run all required software/native gates, preserve exact
artifacts and collect four fresh cycles and all three phases. On completion,
expected accounting is next18/last17/1560000ms with no pending reservation.
