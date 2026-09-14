# CPU0 cadence preparation 3 — USB capture failure

Preparation 3 remains unverified. Four continuity cycles passed and the idle
capture initially passed. The USB capture completed all twelve maximum probes,
but failed capture integrity and the fixed interval percentile. Mining was never
armed, issued or started; device accounting remained unchanged.

## Exact execution and sealed evidence

Firmware: `912e7a984f4eeb6da7ee30b579b7f9169e94ce11`.
ELF SHA-256: `1c0ab8efcdd4b6718a5a2001b5638074628d01211f8699b388cd11f4833f6dd0`.
Gate: `ad16c23d2dd1ec6c48f659ef6ebd4a3f7015666a`.
Thirteen-artifact snapshot:
`484c965e499cb46b212c20350839268975d9a5ca9bb5cd10e4f44c1d76b9a5f2`.
Reference: `c1915b0a63bfabebdb95a515cedfee05146c1d50`.

The protected root is
`scratch/qualification-implementation-20260907/iterative/cadence-016-preparation-3`.
Failure inventory SHA-256:
`1936ad2be4fffa59dbc650fd833bc335e280dd31394ab3378e5d2760aa1ab580`.
The independent auditor source SHA-256 is
`66653e866e924d7811acf1b13b3889f20fec2c92c5c31588d00f4f040b171d11`.
It verifies 107 original files, 45 journal rows, thirteen artifacts, five trusted
flash/process receipts, four cycle audits, the prior sibling closure, both phase
observations, twelve probes, observer receipts and final cleanup. The original
failure review at sequence 34 and fresh authenticated diagnostic review at
sequence 42 are separately bound and identical. The final closed baseline is
sequence 45. The finished seal was reverified without modification.

## Measurements and limits

| Measurement                          | Idle, initial review | USB review |
| ------------------------------------ | -------------------: | ---------: |
| Complete intervals                   |                   95 |         91 |
| Intervals at or below 750 ms         |            95 (100%) | 85 (93.4%) |
| Maximum interval                     |           700.282 ms | 876.213 ms |
| Maximum processing                   |           198.279 ms | 369.667 ms |
| Maximum live stage                   |           166.959 ms | 293.860 ms |
| Maximum retained-log stage           |            62.669 ms | 156.905 ms |
| Maximum pruning stage                |            24.293 ms |  22.791 ms |
| Queued / completed send observations |            190 / 190 |  182 / 180 |
| Pending send observations            |                    0 |          2 |
| Global capture-loss indicator        |                    0 |          1 |

USB failed the requirement that at least 95% of intervals be at most 750 ms.
The absolute interval and processing maxima remained below their separate
1,500-ms and 500-ms limits. Both reviews report zero CPU/priority, subscriber,
clock and explicit projection/serialization/queue/send failure counts, but
capture loss prevents treating those counters as a complete success proof.
The global loss value is a saturated indicator, not a reliable event count.

The client stopped at `cadence_supervisor_rejected` during USB review. Replaying
the saved review through the unchanged validator produces `cadence_capture_loss`
as the first rejection. The interval percentile and outstanding send observations
are additional independently visible failures. No successful USB phase receipt
or funded mining result is manufactured.

The later global review changes only the earlier idle summary's derived `passed`
flag to false; its numeric fields are unchanged. The tested recorder recomputes
that flag using campaign-wide loss. Both original representations are preserved,
including the initial passing idle receipt. Neither is rewritten to obtain an
overall qualification pass.

## Recovery and accounting

The passive observer remained connected for 126,253 ms, recorded 194 arrival
events as timing/count metadata and closed with reason `requested`, exit 0 and
actual child cleanup. Host receipt is not browser rendering or complete device
publication evidence. No raw network payloads or endpoint values are published.

Fresh authenticated reconnect confirmed safe baseline and an inactive lease.
It returned the same capture-loss indicator and two pending send observations,
so the pending values were not merely cleared by waiting for collection.
Fresh device accounting was next ordinal 16, last completed 15, total charged
1,200,000 ms, pending false. No allowance was issued or consumed. The browser
closed, the supervisor exited 0, and its listener, owned descendants and serial
holders were confirmed absent. Earlier sealed preparations remain unchanged.

## Supported diagnosis and corrective work

Recorder ownership contention is the source of the loss indicator. Send outcome
callbacks use that same one-attempt guard in the tested implementation and can
discard an outcome on contention, leaving pending counts permanently positive.
The aggregate hardware counters do not identify the exact callbacks lost or
prove every underlying native send completed. Regression tests must reproduce
the lost-accounting mechanism without treating queue acceptance as delivery.

The periodic log path also clones the complete retained-log backing buffer before
checking whether any `/api/ws` log clients exist. This qualification uses only
`/api/ws/live`. Skipping that unused snapshot removes source-proven unnecessary
allocation, copying and lock occupancy. Stage durations include preemption and
waiting; separate maxima cannot be subtracted or assigned to the same iteration.
The measurements do not quantify the copy's individual cost or prove that its
removal alone will satisfy the percentile limit.

Targeted corrections move terminal send outcomes to bounded phase-local atomic
counters, preserve prior phase results independently of later loss, and avoid the
log snapshot when no log subscriber exists. True loss, overflow, failed sends and
incomplete captures must still fail. Sleep, affinity, priority, stacks, safety
deadlines and all acceptance thresholds remain unchanged. These corrections
require new software/native verification, publication and hardware evidence.

A separate exact-seal-bound v2 failure classification may admit a new preparation
only after verified correction, unchanged fresh accounting and complete cleanup.
The original permission and idle-review failure classes retain their guards.
Repeat four exact-pair cycles and all three captures; do not reuse this observer,
its failed capture or its old context. Cadence, USB qualification and migration
remain active. Parity remains 90/95; no mining or shutdown-time proof is claimed
for this preparation.

## Corrective software verification

Before the next attempt, ordered Cargo formatting, Clippy, build and tests pass:
2,216 tests, zero failures and one existing ignored. All 297 supervisor tests and
112 canonical targets pass, including the new recorder and log-stream regressions.
The native package builds; cadence diagnostics occupy 588 static bytes, below
2 KiB. The owner entry remains 4,496 bytes with the unchanged 24,576-byte stack.
Ownership, reference, redaction, standards, formatting and parity checks pass.
Independent recorder and versioned-closure reviews found no blocking issue.

Canonical read-only v1/v2 reviews validate both original failure inventories
through the normal Bazel launcher. A distinct v2 matcher is anchored to this
report's accepted seal and producer; it does not broaden the v1 idle-review
matcher or grant authority on closure. Software success does not change this
failed hardware result or demonstrate the next image's timing. That requires
new exact-pair continuity and cadence captures under the existing contract.
