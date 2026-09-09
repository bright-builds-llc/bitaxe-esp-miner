# Worker qualification: reusable diagnostic guidance

These lessons come from the September 2026 Ultra 205 preparation and live
acceptance work. The [acceptance report](../parity/evidence/20260908-worker-preparation-live-acceptance.md)
owns measured outcomes and exact source/artifact identities. This document
explains how to avoid repeating the diagnostic and test-harness mistakes.
It grants no hardware effects or new mining allowances. Use the current active
task and [USB ownership contract](native-usb-ownership.md) before device work.

## Prepare the observer before obtaining short-lived authority

Set up and verify the observer while disconnected and mining disabled. Browser
debugger focus emulation can keep a background page logically visible: merely
switching tabs does not prove a foreground-loss fault. Disable that emulation
on the owned qualification tab and prove actual visible/hidden/visible
transitions before signing. Do not override document properties or synthesize
visibility events. Close owned tabs or release overrides during cleanup.

Await completed connection/admission and the published harness ready state.
An internal mutable `ready` value can precede the final status query; issuing
another command then can race the unfinished connection. Keep fan proof,
ledger review, fresh possession, signing and Start in a contiguous sequence
within the existing possession lifetime. Human/observer setup belongs before
that sequence, not after the signed grant is loaded. Do not extend the 60-second
possession limit to compensate for an unprepared observer.

## Measure resources in the actual owner and phase

A host build or successful Start does not establish adequate firmware stack
margin. Diagnostic 001 reported 28 bytes of untouched owner stack; the final
runtime used a 24576-byte allocation and observed at least 7800 bytes during
acceptance. Audit the exact ELF entry frame and measure the actual owner task
through preparation, active work and shutdown. Current qualification requires
at least 4096 bytes of observed headroom; historical values are not guarantees
for a changed image. Include heap free/largest-block observations separately.

Capture resources during normal execution, never by allocating, logging,
querying drivers or blocking inside a panic/retained-record writer. Snapshot
previous-boot records before current-boot initialization. Valid breadcrumbs
identify progress; corrupt or absent receipts leave the crash cause unknown.
An improved stack margin and successful run do not prove the original panic's
cause, nor does simulated native abort prove physical reset retention.

Treat a missing resource object as a diagnostic boundary, not automatically a
stalled owner. Reproduce coherent-publication and timestamp ordering races.
Preserve the preceding fresh observation during publication, check age after
a coherent read and keep retries bounded. Do not widen freshness or stack
thresholds merely to turn a missing observation into a passing result.

## Test repeated sessions and interpret counters precisely

Exercise Start, terminal cleanup and a second independently authorized Start.
Cleanup can remove volatile pool configuration while stale availability caches
incorrectly suppress its reload. A first-session success does not cover this
path; verify new configuration/transport, stale-event rejection and both stops.

Separate dispatched work, parsed responses, reconstructed filter matches,
below-target candidates, qualified candidates, submissions and accepted shares.
The legacy `nonce_work_correlations` count means qualified candidates, so zero
does not prove that the ASIC returned nothing. Preserve cumulative counters
across pool resets and use explicit per-Worker baselines. Reinterpreting a
counter must not upgrade an earlier failed evidence record.

Expected-filter matches classify reconstructed results against a software
model; they do not independently prove ASIC register configuration. Below-target
results do not satisfy the normal accepted-share criterion. The optional signed
difficulty hint is sent only after successful pool authorization, and the pool
alone controls the actual target. Never infer granted settings from another
pool's NVS configuration or expose private pool identifiers in diagnostics.

## Freeze evidence and keep reservation states distinct

Seal result samples before late browser-close notifications can append to a
journal. Bind immutable sample and artifact hashes, preserve the earliest typed
failure, and record cleanup failures separately. Historical validation must not
silently change with a future checkout or parser. A compatible terminal suffix
requires the explicit validated recovery path; never rewrite a sealed result.

Issued authorization, admitted Start, durable reservation and measured active
time are separate states. A timed-out or rejected Start alone proves neither
reservation nor absence of work. Confirm exact current-generation recovery and
the durable ledger. Interrupted reservations consume their full charge; there
are no refunds. ADR-0028's exceptional unreserved continuation requires its
complete unchanged-ledger, identity, fresh-possession and exclusive-child proof.
It is not a generic retry mechanism.

Keep the installed firmware, browser bundle and qualification driver identities
separate. A host-only correction may retain runtime qualification only through
the explicit published allowlist and verified artifact lineage. Never replace
the tested package with a newly built driver-stamped image merely because it is
the newest build. Changed runtime pairs require fresh four-cycle qualification.

## Recover before reset and scope conclusions to the evidence

Stale complete device replies before Hello can reject fresh admission without
proving firmware corruption. Release the browser first. Only an active recovery
contract may invoke the bounded receive-only drain; prove cleanup before fresh
admission. It transmits no payload, changes no reset lines and retains no raw
serial output. Ordinary monitoring is not interchangeable with that command.
Collect retained diagnostics and confirm safe state/accounting before another
reset or reflash. Missing transcripts never justify repeating completed writes.

Measure authority closure and shutdown initiation from the last valid advancing
heartbeat, and verify ordered shutdown/cooling separately. Stable terminal
counters and revocation-guard tests support the tested boundary; sparse telemetry
does not independently timestamp every physical operation. Bind safety ranges
to the observed operating phase. Keep sampled extrema distinct from guarantees
about unobserved transients.

The completed acceptance leaves original failures and the exhausted legacy
ledger intact. Fresh-Hello recovery reliability remains tracked in
`task-fixed-usb-hello-resynchronization`. Broader restoration matrices, thermal
soak, Stratum V2 and parity require their own evidence. For recurring macOS
compiler launch delays, use the existing global host-stall lessons: preserve the
exact command/profile and prove the stalled boundary before changing code or
declaring a machine-wide failure. Reap only owned processes; a passing alternate
profile does not explain why another profile stalled.
