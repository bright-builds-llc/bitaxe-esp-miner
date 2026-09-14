# CPU0 cadence ordinal 17 — startup failure

The initial installation of firmware `a261a2d5a33e88d04b01b8ad8eb5c0d20af9c58b`
with Gate `82992de904616cbfc313044b91aa376fef3d91c9` completed its disjoint
state-preserving writes, but startup qualification failed. No browser was
opened, no continuity cycle or cadence phase ran, and no allowance was issued.
This preparation remains unverified.

## Retained evidence

The protected root is
`scratch/qualification-implementation-20260907/iterative/cadence-017`.
The clean ELF SHA-256 is
`48c4689018af1f78a14ce1e2a628b8e4e7e88bf689a2a46d087e14567c9a8e08`;
the thirteen-artifact snapshot is
`702b22236178fc14280eaa45866935707ab934ef290d16d724c0ebb4917e37de`.
The initial flash receipt is
`7924898cd82e593a6d04edcec1d8724bea92fddbc720ba4996cc10b3d04d0b4d`.

The failure-only inventory SHA-256 is
`d65b506e49d271198d16381c30dcf02596d9f7baf2e72198fba322f7a64a2de8`.
It retains the context, original assignment, charged predecessor, exact
artifacts, failed startup observation and actual host cleanup. It grants no
continuation authority and claims neither device restoration nor fresh ledger
observation.

The first observed startup failure was at the statistics stage. Later observed
boots reached runtime-ready before another panic reset. The serial assessment
retains failed startup, reboot history, non-monotonic uptime and mixed boot
identity. The first retained 8192-byte internal-memory allocation failure can
be recoverable; subsequent boots allocated the statistics thread. That receipt
does not identify the panic's cause.

The flash wrapper exited 1; its prearmed process observer exited 0 with no
remaining owned children. The unused supervisor was identified by its original
PID/start-time/process-group snapshot, stopped, and observed exiting 0.
Independent cleanup checks found no listener, owned descendants or serial
holders. No device-safe-state or fresh-accounting proof is fabricated from
those host observations. The last authenticated predecessor remains next 17,
last completed 16, total charged 1380000 ms, pending false; those values are
expectations until fresh admission succeeds.

## Native stack cause

Disassembly of the exact failed ELF proves an emitted telemetry call path
larger than the configured 16384-byte main-task stack:

| Emitted call-path entry          | Frame bytes |
| -------------------------------- | ----------: |
| Main                             |          64 |
| Prepared HTTP runtime            |          48 |
| Telemetry loop                   |          48 |
| Measured iteration               |         224 |
| Live adapter                     |         208 |
| Profiled live projection         |        5552 |
| Snapshot completion closure      |        5968 |
| API view projection              |        1808 |
| System-info wire projection      |        2224 |
| Mining-state projection          |         224 |
| Vector collection                |          96 |
| **Total before further callees** |   **16464** |

The retained native audit binds each frame and actual call edge to the failed
ELF. The final vector call is emitted even for an empty ASIC list. This proves
a stack-budget breach; no complete-callgraph bound is claimed.

The new timing wrappers forwarded large snapshot values through generic
closures, including an additional temporary `Result` around completion. The
correction restores direct large-value flow with small start/finish timing
boundaries. It must preserve snapshot ordering, failure propagation and every
measurement while passing native checks for the affected callers. Increasing
task stacks, changing scheduling or weakening qualification limits is excluded.

Recovery requires the separate prospective recovery-only contract in
[the cadence specification](../../hardware/cpu0-telemetry-cadence-qualification.md).
It must establish stable exact startup and fresh accounting before any new
qualification context can progress. The failed preparation is never restarted,
and no recovery observation replaces the four cycles or three measured phases.

## Software correction verification

Timing now uses direct start/finish calls around the actual operations. Ordinary
HTTP, BAP and statistics paths keep unprofiled implementations; both publication
paths use the same owner and sequence mutex. A mixed-path chronology regression
verifies retained and issued ordering.

The repo-owned `just audit-telemetry-stack` check rejects the frozen failed ELF
at 16464 bytes and accepts the corrected pre-publication main path at 12560.
It verifies actual emitted call edges, rejects unknown paths or dynamic stack
adjustments, and binds the ELF and compiled stack configuration. Broader native
comparisons found no remaining introduced regression in the examined ordinary
callers. These are selected paths, not complete callgraph or hardware-safety
proof. The separate allocation miss still requires healthy startup evidence.

Ordered Cargo checks pass with 2226 tests and one existing ignored test. All
116 canonical targets pass, including the native audit tests and the new
recovery tests. Sixteen focused recovery/supersession regressions cover the
single-install claim, interrupted assignments, evidence changes, fresh ledger
requirements, absent signing routes and historical-task rejection. Ownership,
reference, redaction, standards and parity checks pass; parity remains 90/95.
Clean publication/package admission and actual recovery remain separate gates.
