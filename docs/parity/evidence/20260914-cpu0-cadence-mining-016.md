# CPU0 cadence ordinal 16 — mining percentile failure

The resumed qualification remains unverified. Idle and USB captures passed;
mining completed, but only 90 of 96 intervals were within 750 ms (93.75%), below
the unchanged 95% requirement. Recorder integrity, publication counters,
independent shutdown and final cleanup passed. No failed capture is promoted.

## Exact evidence

Firmware: `73ad71c6c49df54120b03a243e047255115cfb6a`.
ELF SHA-256: `bba818221df20fe1f0427f69b6cede6bf226eb13c3bb9342c786d758f8c1181f`.
Gate: `ad16c23d2dd1ec6c48f659ef6ebd4a3f7015666a`.
Artifact snapshot: `112304018c95c42beef64bcb43ac35cb599d2288c6b278630b947e8f44500d2d`.
Protected root:
`scratch/qualification-implementation-20260907/iterative/cadence-016-preparation-4`.

The canonical unverified result SHA-256 is
`431ffc03d816eba89e73aefaa8f03bbd106b46d269b3309f1b1f6e4751a38616`.
The failure inventory SHA-256 is
`a379f26c3bb2eb8839b872c4a8225e9dd41938ff27b5a99c57eca10e849b11ca`.
The independent failure auditor, SHA-256
`dd0651c5275eb078edee2b79e8c71e49180914ef7e683ea28455d196cf44d8f8`,
revalidated 127 original files, 120 journal rows, thirteen artifacts, five flashes,
four cycles, twelve USB probes, observer coverage, both ledgers and final cleanup.

Cycle 3 retains a separate operator setup error: an observer path traversed an
absent flash directory, so arming failed and the wrapper exited before invoking
flashing. That failed observation, empty command logs and source provenance are
preserved. A corrected direct sibling path and fresh observer logs were armed
before the actual cycle-3 flash. The failed setup is not counted as a hardware
cycle; the successful replacement observation is independently bound.

## Observed results

| Phase  | Intervals within 750 ms | Maximum interval | Maximum processing | Result                          |
| ------ | ----------------------: | ---------------: | -----------------: | ------------------------------- |
| Idle   |                 101/101 |       680.501 ms |         170.693 ms | Passed                          |
| USB    |                   97/98 |       780.440 ms |         278.468 ms | Passed; twelve probes completed |
| Mining |                   90/96 |       770.003 ms |         261.855 ms | Unverified percentile           |

All three summaries report zero dropped observations, pending sends, explicit
projection/serialization/queue/send failures, CPU/priority mismatches,
subscriber mismatches and clock failures. Earlier phase summaries remain stable.
The maximum mining live-stage duration is 260.024 ms. Separate maxima do not
identify the work responsible for each late interval.

The first rejected mining review is journal row 110, review SHA-256
`8c0d4efba1cfba7a09b6d9a7174001c94251849938ec704eeca1aea6dd3469c7`.
Replaying it produces `cadence_phase_unqualified`. The qualified mining receipt
is intentionally absent, so the canonical finalizer reports
`cadence_evidence_missing`. Its `first_failure:null` means no browser failure
enum was recorded; the separate operator receipt preserves the numerical cause.

The final device record shows 73,081 ms of active work, 35 dispatches and three
renewals. Heartbeat-timeout revocation occurred 2,805 ms after the last valid
heartbeat; shutdown initiation occurred at 2,850 ms. Ordered stop and cooling
completed at the qualified fan baseline: 29 C, 3,200 RPM and 7,632 bytes of owner
stack margin. The observer covered the fault plus the required tail and closed
cleanly. Fresh admission confirmed restoration; browser, serial, observer and
supervisor resources were released. Host receipt is not browser rendering.

The complete 180,000-ms reservation is charged. Fresh accounting is next ordinal
17, last completed 16, total 1,380,000 ms, pending false. The original exhausted
ledger remains 240,000 ms with masks 7 and no pending reservation. No refund,
reset or reuse is permitted.

## Bounded continuation

The live path repeatedly reads the mounted asset-version file and response-only
NVS settings, alongside collection, projection, retention and serialization.
The existing measurements do not isolate their costs. Before another allowance,
add bounded exclusive substage timing and a worst-interval witness joined to the
previous iteration; retain all acceptance limits and the 2-KiB storage bound.

The mounted asset version can become a positive-only boot/mount identity fact:
load the actual file until a successful parse, then retain that value while
dynamic platform facts remain fresh. Initial errors remain retryable. Current
WWW replacement is unsupported and application OTA reboots; any future same-boot
replacement or remount must invalidate this fact. This intentionally separates
installed version identity from later transient file-read availability.

After verified correction and exact publication, ordinary cadence preflight may
consume this completed unverified result for fresh ordinal 17. It must repeat
all four exact-pair cycles and all three captures; no supersession or unissued
closure is appropriate for an already charged allowance. No cadence criterion,
safety deadline or parity row changes. Qualification and migration remain active.
