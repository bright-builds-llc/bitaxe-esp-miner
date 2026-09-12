# Hello recovery with live mining

The active owner is `task-fixed-usb-hello-resynchronization`. The owner authorized
this successor on 2026-09-11, including true mining, targeted fixes and
progress-backed iteration. The fixed USB ownership and hardware-attempt policies
remain authoritative. This contract changes future evidence requirements;
attempts 001–003 and their original judgments remain immutable and unverified.

## Diagnosed evidence defect and correction

The production browser channel can receive an entire response in a native read
while asynchronous integrity validation keeps the response promise pending.
The reproduced coalesced credit/reply case disproves the old inference that a
consumed request plus pending response necessarily leaves a complete reply for
the next Hello. It does not identify what happened on physical attempt 003.

The successor receipt describes observed browser processing stages explicitly.
Software regression tests prove stale complete replies, fragmented/coalesced
records, integrity/deadline rejection and revoked-session isolation. Hardware
proves actual work, loss, device-local shutdown, fresh identity/possession,
preservation and separately authorized resumed work. Hardware reports the actual
stale-reply count; zero does not claim stale-reply hardware coverage. The original
`no-mining-judge` continues requiring its original positive stale-reply evidence.

Firmware and browser retain bounded metadata traces: temporary numeric epochs,
request/frame sequence numbers, closed stages, monotonic timestamps and byte
counts. They never retain payloads, raw session identifiers, credentials, proofs
or pool data. Writer queue acceptance is not host receipt; host frame assembly
is not successful integrity validation. A pending native TX can hit its existing
two-second bound and revoke the link before heartbeat expiry; `link_closed`
requires both TX-abandonment and epoch-revocation observations in the retained
interrupted epoch. Neither event order is assumed, since owner cancellation can
precede the writer noticing abandonment. Overflow, contention and unavailable
snapshots remain explicit. Trace export is read-only and grants no work or
liveness authority. Firmware and browser clocks and epoch counters are independent;
correlate ordered session/request events, never subtract their timestamps as a
shared clock. Diagnostics must not block the heartbeat deadline.

Accepted signed Start/Renew legitimately advances the durable authorization
high-water map. The original page baseline remains immutable and may therefore
report an authorization mismatch after work. At the qualified work checkpoint,
the browser retains the latest authenticated status digest privately with a
fresh checkpoint identifier and generation. Fresh reconnect compares its new
status against that checkpoint. Only the identifier, generation and pending/match
result enter evidence; the digest never leaves browser memory. Loss acceptance
requires a matching post-work checkpoint across reconnect. Resume clears that
completed checkpoint only after the loss result is sealed; it never resets the
original baseline or device counters.

## Exact command and source contract

Use `just fixed-usb-qualification recovery-preflight` with absolute paths:

```text
--recovery-phase loss|resume
--firmware-root <firmware repository>
--gate-root <Gate repository>
--firmware-commit <clean pushed HEAD>
--gate-commit <exact pinned published Gate commit>
--manifest <canonical native package manifest>
--private-root <absent child in existing iterative ledger parent>
--authority-directory <existing protected authority directory>
--previous-receipt <previous sealed result.json>
--input <protected verified-progress receipt>
--suggested-difficulty 1000
```

The version-5 context uses the existing signed `diagnostic` purpose and its
30000-ms maximum. It never creates a legacy campaign or resets either ledger.
The loss phase requires a changed verified runtime pair and software-correction
progress. Resume requires a passed loss result on that exact pair, another
fresh ordinal, and a fresh signed allowance. The preceding independently
validated result currently selects ordinal 14 and 1140000 ms already charged;
fresh device accounting must agree before issuance. Ordinal 15 is conditional
on passing loss and is charged separately. Reservations are never refunded.

The supervisor runs through the existing `serve` command with the context,
protected authority directory and ignored owner pool credential path. It keeps
signing payloads only in memory. The browser is the sole application transport.
There are no new credential readers, raw-record injectors or network discovery.

Create a mode-0700 parent, prove the child absent immediately before preflight,
and use separate mode-0600 sibling stdout/stderr logs. Use `umask 077` for every
caller. Bind all source, package, browser, context and evidence hashes. Copy and
verify the 13 exact runtime artifacts before a later build can replace them.
Never reuse or modify a sealed attempt root. For detector/flash execution,
pre-arm the metadata-only process observer against the exact PID/start-time/group
before effects (15-second machine setup bound); it observes at most 600 seconds
and never signals a device owner. Revalidate the frozen package/artifact snapshot
before the command. An issued or completed context cannot authorize another
flash. Require root/descendant completion and zero serial holders before reuse.
These machine bounds do not constrain a human checkpoint.

## Continuity before mining

1. Complete all relevant software regressions, ordered Cargo checks, browser
   checks, canonical tests, native package/USB ownership, reference, redaction,
   progress, standards and scoped formatting checks. Publish Gate and the exact
   root archive pin before hardware effects.
1. Run `just detect-ultra205`; require exactly one admitted Ultra 205 and a known
   fixed Serial/JTAG profile. Retain the physical lease. Each ROM write must
   pass same-device `board-info` through the repo-owned flash command.
1. Install the exact package using state-preserving
   `just flash-monitor --board 205 --port <admitted port> --manifest <manifest> --evidence-dir <fresh child> --capture-timeout-seconds 30 --redact-evidence`.
   This contract explicitly permits a 30-second safe-baseline startup capture.
   Flash completion, runtime identity and cleanup are separate observations.
1. Complete four browser-release/flash/reconnect cycles of that exact pair.
   Each requires a fresh maximum-size exchange, same Device Identity/settings,
   unchanged authorization marks before mining, disabled mine-on-boot and
   complete host/USB cleanup. Seal each with `record-cycle` and its protected
   actual-observation input. Reports validate observations; they do not execute
   flashes. A changed runtime pair requires four new cycles.

The initial installation plus four updates write only admitted disjoint package
segments. NVS, Device Identity, replay marks and unrelated partitions remain
preserved. No factory installation, erase, provision, direct UART, pins,
electrical injection, voltage/fan stress or broader network actions are allowed.

## Loss and resume sequence

Configure the page from the supervisor context while retaining its continuity
baseline. Prepare the observer before signing. Human waits have no deadline;
protocol, signing, active work, shutdown and cleanup retain their finite bounds.

1. Export the idle `before` trace, prove cooling/fan operation, review the fresh
   qualification ledger, obtain fresh possession and sign/start contiguously.
   Use the existing Conservative profile: 400 MHz, 1100 mV and fan 100 percent.
1. In loss mode, the one-use qualification hook waits for an actual parsed
   current-generation observation with work dispatched and more than 3000 ms
   remaining at the work gate. It flushes the observed checkpoint and rechecks
   admission before closing the native channel. It sends no Restore or Close
   control record. Failure to reach the checkpoint is a failed observation,
   never permission to extend the allowance or cut at a guessed time.
1. Save the exact before/released journal binding and browser `loss` trace.
   Allow the device-local 2.8-second heartbeat expiry and require gate revocation
   and shutdown initiation within three seconds of its last valid heartbeat.
   Cooling is a separate bounded postcondition. Before reconnect, prove native
   ownership release. The observed 15550-ms budget covers pre-reset ASIC
   shutdown; cooling separately permits 120000 ms. Before the single reconnect,
   wait 145000 ms after the recorded cut, covering heartbeat expiry, that
   pre-reset plan, cooling and scheduling margin. Elapsed time is not stop proof.
1. Reconnect explicitly with fresh Hello and possession, without a CLI drain,
   reset or flash. Require exact source/ELF, retained interrupted-generation
   shutdown evidence, heartbeat-timeout reason or trace-supported link closure,
   unchanged settings/identity and
   inactive lease. Export `recovered` traces before maximum probes or unrelated
   requests can displace evidence.
1. Review both ledgers, close and flush the journal, and submit completion.
   The loss result requires the matching post-work authorization checkpoint,
   actual work, safe cooling, resource margins,
   timely local shutdown, fresh admission and required trace artifacts. Seal
   and independently review it before the resume context can issue authority.
1. With the same runtime and continuity baseline, create the next resume
   context/ordinal from that passed result. Export `before`, repeat cooling,
   ledger/possession/signing admission and start a fresh diagnostic allowance.
   Require new-generation actual work. The ordinary diagnostic stop ends work;
   export `resumed`, review accounting, close, flush and seal completion.

Fresh energized observations require 4.5–5.5 V input, at most 15 W, below 75 C,
nonzero RPM and live watchdog. Actual owner stack headroom must remain at least
4096 bytes, with qualified shutdown resources. Ordered shutdown and cooling to
45 C precede fan 30 percent. No accepted share within these short diagnostic
windows is required or implied; normal mining acceptance remains separate.

## Recovery, cleanup and stop outcomes

Finish with actual safe shutdown/cooling, no active lease or volatile pool
secrets, mine-on-boot false, browser streams/locks released, the supervisor
reaped and no unexpected holder on either admitted serial node. Preserve the
earliest failure even if recovery later succeeds. If reconnect fails, preserve
browser traces and stop before any drain, reset or reflash; a distinct
regression-backed recovery correction must be published before another effect.

Select exactly one repository outcome: `complete`,
`continue_after_verified_fix`, `continue_after_manual_remediation`,
`stop_repeated_boundary`, `stop_hardware_blocker`, `stop_authority_boundary` or
`stop_impossible_contract`. A new ordinal alone, added logging alone or a renamed
failure is not progress. One recurrence at the same authoritative boundary after
its targeted verified fix stops further attempts. Unavailable physical access,
ambiguous identity, unproved cleanup or contradictory evidence is a hard blocker.

Private evidence contains only permitted operational observations under protected
modes. Never persist raw credentials or signing payloads. Public reports contain
closed categories, counts, durations, booleans and hashes after redaction review.
Completion never automatically promotes unrelated parity or prior failed runs.
