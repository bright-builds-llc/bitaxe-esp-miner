# STR-005 failure-only recovery execution contract

Owner: `task-str005-failure-recovery-accounting`. Policy: ADR-0029.
This independently published successor observes the installed Share002 pair:
firmware `cf7a3f038dabdd5383083734336aeb64a22f837c`, ELF
`66a77d2cb699e2064469c7b124f482a57a7fd6f6b13ef737554f53f721e78590`, Gate
`e20c0fd52d2216596f904992ffa54fda33be9025`. It does not replay Share002,
repair its sealed evidence, grant a mining ordinal, or promote parity (90/95).

## Objective and admission

Collect fresh authenticated qualification and original campaign ledgers, retained
Share002 status/resource facts, bounded boot diagnostics, normal restoration and
actual resource release. Read the current next/last/charged/pending values; neither
ordinal18 reuse nor next19 is an input assumption. An absent retained record is a
blocker even when a fresh baseline is safe. Diagnostic observations are not
authenticated accounting or historical resource proof.

Publish the implementation and this contract with applicable tests passing before
any detection or connection. `preflight` requires exact clean pushed firmware-host
and Gate repositories, active task, unchanged Share002 seal/context and pinned
page/bundle/trust bytes. The host source commit binds the complete evaluator and
transitive validation sources. Runtime firmware remains the installed old pair;
host-only collector changes do not relabel it or require flashing.

## Commands and protected evidence

Create a fresh mode-0700 ignored parent below `scratch/str005-recovery/` and leave
its `attempt` child absent. Create separate mode-0600 sibling wrapper stdout/stderr
and detector stdout/stderr files. Use the following repository command sequence
with absolute paths, inspecting each exit before continuing:

1. `just str005-failure-recovery preflight --private-root <parent>/attempt`
1. `just detect-ultra205` with output directed to the protected sibling detector
   logs, named `detector.stdout.log` and `detector.stderr.log`.
1. `just str005-failure-recovery serve --private-root <parent>/attempt` with
   separate protected sibling output logs. Serve verifies fresh detector output
   (within 60 seconds), the same physical device/node as Share002's final install,
   Serial/JTAG runtime profile and no existing serial holders.
1. Open the emitted loopback recovery page in desktop Chrome, keep it visible,
   and use native Connect to obtain fresh exact identity and possession. No
   deadline applies while waiting safely for permission. Click **Collect failure
   recovery and release** once. Stop and Close remain available on failure.
1. Close the owned page after confirmed serial release; terminate the supervisor
   with SIGTERM and wait for exit. Run
   `just str005-failure-recovery finish --private-root <parent>/attempt`.

The supervisor has no signer, pool fixture, flashing adapter or Start route. It
serves only pinned Gate assets, public trust/configuration, a random possession
scope, and validated recovery collection routes. Work controls are removed from
the page. No credentials or authority directory are read. Possession's internal
authorization-context operation is a non-issuing identity operation, never a
Work Lease or grant.

Only closed, validated projections cross the persistence boundary. Network/socket
tuples and raw proofs remain in memory; diagnostics retain only boot categories
and omission counts. Evidence is mode-0600 under the ignored mode-0700 child.
Partial parts are preserved independently; failed collection cannot suppress Stop
or Close. The original failed test remains failed. Finish checks actual supervisor
absence, listener closure and both serial-node holder inventories, then writes a
separate result and immutable inventory. It never writes into Share002.

## Effects, bounds and stopping

Allowed: one detector invocation; native connection/Hello/possession; authenticated
ledger, retained-status and diagnostic reads; ordinary Stop/restoration/cooling;
Close and actual host cleanup. Existing settings, Device Identity, replay marks
and both ledgers are preserved. No Start, mining, grant issuance/renewal, pool
contact, flash, reset, ROM recovery, factory provisioning, electrical manipulation,
heartbeat suppression or Share002 replay is authorized.

One collection per fresh root/page. Reads have 30-second bounds; restoration and
Close each have 150-second bounds including cooling. Existing two-second serial
record and 2.8-second device heartbeat deadlines remain. No blind retries. A new
attempt requires a regression-tested boundary fix, renewed clean publication and
a fresh root; no ordinal is selected by this command. The server waits safely for
the operator and supports SIGTERM; finite protocol operations retain their bounds.

Stop on identity/profile mismatch, stale session, missing device, unexpected owner,
partial evidence, pending accounting, absent retained resources, restoration failure
or unproved release. Preserve all successful independent observations, report the
precise blocker and keep the task active. Never expand authority to obtain a pass.
Complete/archive only when authenticated accounting, retained diagnostics/resource
proof, restoration and actual host release all pass. Browser-reported closure alone
does not replace the host resource checks.
