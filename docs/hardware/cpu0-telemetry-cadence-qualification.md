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
