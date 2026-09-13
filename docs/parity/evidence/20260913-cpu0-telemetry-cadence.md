# CPU0 telemetry cadence qualification — 2026-09-13

Qualification remains **unverified**. Four fresh state-preserving update/reconnect
cycles passed, but the idle capture could not be collected because Gate rejected
its review with `probe_admission`. USB-load and mining captures did not start.
This is a controller admission failure, not a measured device-cadence failure.
Cadence, USB qualification and USB migration remain active; parity is unchanged.

## Exact tested pair and preserved evidence

The tested firmware is `1d60fb1edbe741ce0ca743bb695476fab73042f5`, with ELF SHA-256
`612cffedb9de89bbf14ce7b5cce53e4af8cc2c3164618571df7607e0a1f12f07`.
The tested Gate is `687021821affa33ca0afbdd5bf21f93e4adfc83b`, with archive SHA-256
`8d5a400f5c1a83e04d9c26e08aa714cc29d144548eb51b69fa07b88a61945645`.
The thirteen-file artifact snapshot SHA-256 is
`92c8e211d688e7559ef7c8bd283257700cbe1fb499586165e7f52807a2b3f73f`.
The pinned reference is `c1915b0a63bfabebdb95a515cedfee05146c1d50`.

Protected preparation 2 is retained under
`scratch/qualification-implementation-20260907/iterative/cadence-016-preparation-2`.
Its original context, journal, observer receipts, five flash observations,
four cycle records and operator failure/cleanup observations are preserved.
No passing result or mining-accounting receipt is manufactured.

The failure-only inventory seal is
`6895f2fcfe161a0d7617b51b02b9f12fcdf49454c9ee540295c31f40b463a5a7`.
Its independent verifier, SHA-256
`53849ff5e24230bb3ff7181fda40bc9ab61b2722ea96152b32824e95846673fe`,
checks all 102 original files, 42 journal rows, thirteen artifacts, five trusted
flash/process receipts and four rederived cycle audits. It also verifies the
unchanged sealed predecessor, observer closure, absent issuance/mining artifacts,
parent-observed unchanged ledger and final cleanup. Read-only verification of
the finished seal passed. This inventory records failure and grants no authority
to continue the preparation or claim qualification.

Preparation 1 remains separately sealed as an unissued permission-stage failure.
It installed firmware `4b66d61c49e7abe0107986dd0dd51fe3230bf510` but performed no
cadence capture or mining. A native foreground click resolved the browser gesture
problem; authenticated read-only recovery established unchanged accounting and
safe closure. The published preparation-supersession change then admitted a fresh
preparation identity for still-unused device ordinal 16. Neither preparation
rewrites the other, and the original ordinal marker remains intact.

## Observed hardware outcome

| Obligation                  | Observation                                                                                                                                                                                                       | Conclusion                                                                           |
| --------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| Exact image and startup     | Initial installation and four ordinary updates each returned to the exact image, healthy stable startup and safe baseline. No NVS seed was supplied.                                                              | Satisfied for this tested pair.                                                      |
| Four continuity cycles      | Every cycle independently passed fresh possession, original identity/settings preservation and 65,536-byte request and response payloads.                                                                         | Satisfied; no cadence inference.                                                     |
| Passive subscriber          | Authenticated endpoint admission and handshake succeeded. The observer retained 94 arrival records / 78,794 payload bytes as counts only, then closed with reason `requested`, exit 0 and actual process cleanup. | Observer operated; arrival times do not prove device intervals or browser rendering. |
| Idle cadence                | Device arm succeeded. After the scheduled observation period, Gate rejected review before refreshing possession. A fresh diagnostic reconnect also rejected review, this time after possession refresh.           | Unresolved: no frozen summary was collected.                                         |
| USB-load and mining cadence | Neither phase started. No allowance was issued, loaded or reserved.                                                                                                                                               | Unresolved; no live-mining or shutdown measurement on this pair.                     |
| Accounting                  | Fresh authenticated device review reports next ordinal 16, last completed 15, charged 1,200,000 ms, pending false.                                                                                                | No additional reservation or refund.                                                 |
| Final cleanup               | Fresh recovery confirmed closed safe baseline, inactive lease and USB release. Owned browser tab closed; supervisor exited 0; listener, owned descendants and serial holders were absent.                         | Confirmed by actual observations.                                                    |

The observer ran for about 63 seconds, within its 360-second lifetime. It stores
allowlisted timing/count/outcome metadata only. Its counts do not establish
that every projection, queue operation or asynchronous send succeeded. The
uncollected device summary cannot support any timing, CPU/priority, subscriber
continuity or publication-integrity pass claim.

September 8 accepted-share and heartbeat-loss evidence and September 11 recovery
evidence remain bound to their original pairs in the
[USB reconciliation](20260913-usb-qualification-reconciliation.md).
They do not fill these three new cadence captures.

## Failure and correction

The actual controller regression holds native settlement of a complete, credited
ordinary heartbeat while requesting cadence review. The original Gate rejects
the request because its generic `unfinishedRecord` flag also describes this
normal serialized write. That flag does not by itself mean a cancelled partial
control record. The same guard runs before and after possession refresh, matching
the two observed failure boundaries; possession expiry is not the demonstrated
cause.

The correction uses the existing bounded serial send queue for normal heartbeat
settlement. Concurrent pending controls and failed or cancelled records still
reject admission, and fresh possession remains required. Device heartbeat,
write, receive-credit and shutdown deadlines are unchanged. Corrected-source
software verification is separate from this failed pair's hardware evidence.

Published corrective Gate is `ad16c23d2dd1ec6c48f659ef6ebd4a3f7015666a`, archive
SHA-256 `304e6fc6663740820c38f6d11a02beaf54dbf26e1aa67c54c68bd02bb3ef4bda`,
and exact-HEAD bundle SHA-256
`3ccc6bae10fa084030ab59f8bd44cdcdec3350bb172056c9aa18148a7ee200cc`.
Its full verification passes: 352 Rust tests (two existing ignored), 549 web tests
(1,511 assertions), browser conformance, types, packaging and standards. Five new
actual-controller regressions cover both heartbeat races, bounded timeout,
cancelled partial-record recovery and a full simulated idle interval. The root
pins this correction for subsequent work; it was not used in the hardware
preparation described above.

## Instrumentation and software limits

The implemented recorder retains three fixed summaries and measures the actual
main-loop intervals, total processing, live publication, retained-log handling
and pruning. It distinguishes suppression, missing subscribers, projection,
serialization, queue and asynchronous completion outcomes. Boundary intervals,
first-dispatch generation binding, lost observations, contention, clock errors
and incomplete capture are tested through the production loop with fake clocks
and transports. Recording adds no allocation, logging, durable write or waiting
lock. The existing 500-ms sleep and CPU0/priority-5 configuration remain intact.

Native symbols total 540 additional static bytes, within the 2-KiB target. The
owner entry audit reports 4,496 bytes against its 8,192-byte budget, with the
unchanged 24,576-byte stack and 4,096-byte required measured-free threshold.
That threshold is a qualification requirement, not a newly observed mining
stack margin from this failed preparation.

Before effects, ordered root Cargo gates passed with 2,209 tests and one existing
ignored test; all 272 supervisor tests and 109 canonical targets passed. Native
packaging, ownership, reference, redaction and standards checks passed. The
tested Gate passed 352 Rust tests with two existing ignored and 544 web tests.

## Remaining bounded work

The existing permission-only `cadence-close-unissued` command must reject this
preparation because the observer ran and idle was armed. The funded cadence judge
must also reject it because no allowance was issued or consumed. Neither guard
is bypassed to obtain a closure or another attempt.

A subsequent attempt needs an evidence-preserving pre-mining failure closure and
separate supersession admission after the tested controller correction. Require
unchanged authenticated accounting, no issuance/work/renewal, complete cleanup,
preserved first failure and a changed clean published pair. Retain the unused
device ordinal, allocate a fresh preparation/allowance identity, and repeat four
exact-pair cycles before all three captures and the independent shutdown test.
Do not restart this observer or reuse its failed capture.

All prospective timing/publication limits remain fixed in the
[contract](../../hardware/cpu0-telemetry-cadence-qualification.md). BWG restoration
and STR-005 still require completed qualification and their separate successor
contracts. No task is archived and no parity row is promoted by this outcome.

## Final publication verification

The corrective pin passes the final ordered root Cargo format, Clippy, build and
test sequence: 2,209 passed, zero failed, one existing ignored. All 272 supervisor
tests and 109 canonical Bazel targets pass. The native package builds, and USB
ownership/symbols, reference integrity, redaction, standards, report formatting
and parity/progress checks pass. Cargo and Bazel verification did not overlap.

An independent review found no actionable evidence/privacy or task-scope issue.
The unchanged archive and checklist and all 230 unique active/archive task IDs
were checked. The three unresolved task records remain active. Historical
evidence documents, checklist and progress history receive no transition;
parity remains **90/95 active rows verified, 99 total, four deferred (94.7%)**.
