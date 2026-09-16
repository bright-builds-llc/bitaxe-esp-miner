# STR-005 Noise fixture and evidence preparation

## Partial software result

The frozen [Serial Noise contract](str005-noise-serial-qualification.md), digest
`0da417fc198a89042eb62902999ac822be5365a5f0333df8220f15afe3447a62`, remains
unchanged. Runtime readiness is
[blocked on bounded cryptographic cancellation](str005-noise-runtime-readiness.md).
This slice implements independently useful local fixture and schema work. It
does not complete the fixture/evidence task or qualify connected hardware.

The new `just stratum-v2-noise-serial` family recognizes the frozen argument
shapes but rejects every action with `noise_runtime_readiness_unverified`.
Rejection happens before input-path reads, child/assignment creation, listening,
USB admission, signing or publication. There is no override. Historical Noise
effect entrypoints are retired separately; their parsers, projections and
read-only validators keep their historical meanings.

## Implemented boundaries

- The canonical Rust fixture has a separate `noise-serial` mode. It preserves
  the old `noise-auth` and pool modes and emits the new closed `ready.json` and
  `terminal.json` shapes. Authority material is generated locally and held in
  zeroizing buffers; no private key or raw handshake is written.
- The fixture uses one private literal IPv4 listener and expected peer,
  canonical attempt/key encoding, at most three stored candidates, exact act-one
  and encrypted proof sizes, and explicit peer EOF. Unexpected or duplicate
  peers, extra bytes and absent EOF fail. The final candidate observation ends
  at the last nonblocking inventory check immediately before listener closure;
  it does not claim observation of connections after that boundary.
- Absolute accept/read/write/EOF bounds cover partial progress. A selected
  timely act one receives the full 500-ms additional-peer observation even when
  that tail extends beyond its read deadline. An independent host watchdog uses
  deadlines fixed before thread scheduling for the five-second ready and
  150-second lifetime limits. Watchdog termination emits a closed failure code;
  it supplies no terminal-success evidence.
- New pure JavaScript parsers cover Start, device status/retained progress,
  fixture receipts, cleanup shape and public projection shape. Protocol metadata
  consistency checks bind attempt/input digest, source boot, fixture authority,
  socket tuple, sizes and deadlines. They explicitly return
  `hardware_qualified: false`: caller-supplied matching objects are not an
  independent hardware judgment, artifact admission or authenticity proof.
- Retained job checks preserve first failure and immutable terminal results.
  Late actual cleanup can add observations without promoting an incomplete
  result. Shape validation preserves failed exit codes rather than pretending
  that parsing a cleanup receipt proves actual cleanup.

## Verification and limits

Focused Node tests exercise closed/private-field rejection, inconsistent joins,
retained-history mutation, timeout classification, late cleanup and the real CLI
from a fresh process and unrelated working directory. Every blocked action
leaves the requested evidence directory absent. Fixture Rust tests use real
loopback sockets and the production Noise initiator/responder, including exact
proof plus EOF, extra/partial bytes, duplicates, unexpected peers, overflow,
absolute deadlines, and fresh-process watchdog termination. The two ignored
watchdog helper tests are invoked explicitly by their corresponding parent
tests.

Canonical test targets are `//scripts:str005_noise_serial_contract_test`,
`//scripts:str005_noise_serial_cli_test`, and
`//tools/stratum-v2-fixture:noise_serial_test`. These software fixtures are not
application-identity, resource-fit or hardware measurements. The host watchdog
may terminate its own process; that mechanism does not provide safe
firmware-thread cancellation.

## Remaining integration

After runtime readiness is resolved without weakening the frozen contract, the
fixture/evidence task still must implement and verify:

1. Protected source/task/predecessor/artifact admission, exclusive assignments,
   installation claims and four fresh continuity/probe cycles.
1. The real Gate page integration, independent child/process supervision,
   bounded stream sanitizers, same-session network observation and sticky
   consume-once Start admission. There is no runnable supervisor in this slice.
1. Authenticated preservation/accounting journals and same-page recovery with
   original baseline retention, actual host cleanup witnesses and sealed inputs.
1. The independent evidence evaluator and immutable finalizer/review/publication
   path, including semantic redaction registration for the admitted projection.
   The current projection parser validates shape only and writes nothing.
1. Full production Gate/supervisor/fixture integration, exact published source
   bindings and native combined-owner resource proof before any live admission.

No new hardware result, recovered baseline, mining allowance, parity transition
or passing readiness receipt is created by these changes. All three Noise tasks
remain open; live qualification remains blocked.
