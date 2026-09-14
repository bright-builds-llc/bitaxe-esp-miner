# Prospective reset-origin observation and recovery

Owner authorization: 2026-09-14, under active
`task-cpu0-telemetry-cadence-qualification`. Prior startup attempts and their
seals remain failed historical evidence. This contract does not reinterpret
their verdicts or grant authority to reopen their contexts.

## Stage A: observe the installed runtime

Use `just fixed-usb-qualification reset-origin-preflight`,
`reset-origin-serve`, `reset-origin-judge` and read-only `reset-origin-review`.
The preflight binds the exact retained firmware
`daa69bb03274968e78d131e9b0bf13ff12f64ff0`, ELF
`49643f417a27b1842865c92109facf253efc8b3837c3906074ccc20a41ce3545`,
Gate `82992de904616cbfc313044b91aa376fef3d91c9`, thirteen retained artifacts,
failure seal `0ed37434d6bb8587ea51dec6b1e3cf41128ee555635058bd74fd2c84b702e834`,
and the last validated charged predecessor. Bind the current clean published
host driver and all reachable validators separately. An explicit host-only
source allowlist prevents an unexamined firmware change being treated as the
installed image. Runtime identity still requires a fresh authenticated session.

Create a new protected observation context exclusively. It authorizes one
detector admission and one fresh foreground browser Web Serial session, with
read-only status/diagnostic/accounting requests and ordinary Close. There are
no installation, reset, signer, pool-input, grant, work or fault-injection
routes. Do not open another USB owner while the browser holds the port.

Require a confirmed idle baseline, inactive leases, mine-on-boot false and
fresh original/qualification accounting before observation. Expected values
are original budget 240000 ms exhausted, and qualification next ordinal 17,
last completed 16, 1380000 ms charged, pending false. They must be observed;
the predecessor does not substitute for a fresh review.

Prime the bounded diagnostic history, then record fresh changes continuously.
The host assigns sequence numbers and monotonic receipt timestamps. Cached
snapshots cannot count as new device progression. Compare the first fresh
boot/startup observations to the prime so a boundary reset cannot disappear.
Persist only the selected closed metadata. Informational memory checkpoints
and undecodable/unavailable preparation-receipt markers are not current health
or crash proof; a decoded preparation receipt requires separate review.
Require all of the following, fixed before the new capture:

- At least 120000 ms of host coverage and separate device boot/startup spans.
- At least thirty advancing boot pairs and 120 healthy-startup advances.
- No initial, interior or trailing observation gap greater than 6000 ms.
- One boot ordinal/reset category throughout, advancing device clocks, exact
  source/ELF identity, complete error-free runtime startup and ready storage/HTTP.
- No disconnect, control/transport failure, malformed selected observation,
  missing sequence, clock regression, overflow or unsupported record.
- No retained Rust panic/allocation receipt; the initial reset category alone
  may remain `panic` with its cause and firmware attribution explicitly unknown.

Use a 130-second recording target with a hard 135000-ms host envelope to cover
periodic-marker boundaries. Retain at most 4096 measured records and eight boot
segments; a transition fails this observation even though the reducer reports it.
Record host and device timing independently. This is sampled diagnostic
coverage, not a claim to have retained every byte or every emitted diagnostic.
Read-only health covers baseline, liveness, startup and storage readiness;
it does not claim fresh idle sensor or owner-stack measurements absent from
the installed interface.

Diagnostic POSTs have a five-second abort bound. Source-verifying start/end
requests have a thirty-second abort bound; accounting and journal waits are
bounded to sixty seconds outside the measurement envelope. A timed-out wait is
not cancellation or cleanup proof: close the Worker transport, close the page
and verify actual resource release before sealing a failed result.

Review both ledgers again in the same admitted session, confirm unchanged
baseline/accounting, then close the browser and prove host/process/USB cleanup.
The independent result may be `observed_stable`; it is not a recovered result,
does not identify the preceding panic, and grants no cadence/mining authority.
Preserve failed observations and their earliest cause. Any incomplete capture
or cleanup blocks continuation; never restart the same context.

### Unstarted HTML-serving failure

Observation 001 exposed JSON-encoded HTML before configuration or USB control.
Preserve its exact parent-observed page failure, original server-close marker,
zero-activity evidence and actual browser/process/USB cleanup. Its failure seal
cannot authorize work or turn it into an observation result.

After the real HTTP response regression passes, `reset-origin-preflight --supersede-unstarted <failed-root>` may create observation 002 with a fresh
context/marker only for this exact verified unused failure and a changed clean
published driver. Require a software-correction plan, identical retained
runtime/accounting, and preserved original context/assignment/inventory.
Conflicting or duplicate successors, changed evidence, any prior USB/browser
control or observation activity, and incomplete cleanup remain ineligible.
The new context repeats the complete observation under unchanged limits.

## Stage B: controlled restart interface and observation

The installed Worker interface has no authenticated software-restart command.
Before any restart effect, implement and independently verify a narrow idle,
possession-bound command and expected-reset observation lifecycle on the sole
Web Serial transport. Publish the exact firmware/Gate pair and finalize its
installation/capture contract before effects. HTTP control, DTR/RTS reset,
ROM reset and an unobserved power cycle are not substitutes.

The command must reject active work, cleanup and stale/changed sessions, consume
one request, acknowledge its bound request/boot identity before committing the
restart, and never acquire mining authority. A delivered device reply is not
host receipt; missing matching host acknowledgement remains unverified.
Prearm bounded observation before the request and handle coalesced reply/boot
bytes correctly. Prove exactly the expected next boot, software-reset category,
same firmware/Device Identity, healthy advancing startup and fresh unchanged
accounting. Any stream interruption/reopen must remain explicit; it cannot be
called uninterrupted byte capture. Existing safety deadlines/stacks remain.

Stage B is not effect-eligible until its concrete repo command, bounds,
cleanup and tests are recorded. A passing Stage A result cannot supply those
missing capabilities or authorize a reset by itself.

## Continuation and completion

Only independently accepted recovery can support a new cadence preparation.
Repeat all required update/reconnect cycles and three cadence phases on its
exact pair under every existing timing/publication/safety criterion. The next
mining reservation remains the normal 180000-ms type and requires a fresh
authenticated ledger review; no refund or ledger reset is allowed.

Use only provided USB/barrel interfaces. Keep operational evidence protected,
persist only closed allowlisted metadata and redact before sharing. No
credentials, private proofs or raw payloads may enter evidence. Require actual
owned-process and USB release, preserve the earliest failure and apply the
existing progress-backed retry/repeated-boundary policy. Parity remains 90/95;
archive cadence/qualification/migration only after complete accepted evidence.
