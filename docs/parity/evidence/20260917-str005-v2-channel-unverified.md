# Ultra 205 V2 channel attempt: unverified

## Disposition

Channel002 completed its network protocol exchange but is **unverified** under
the original published validator. It is not an accepted Channel predecessor and
cannot admit Share. No V2 mining attempt, Work Lease authority signing, grant or reservation
occurred. STR-005 and parity remain unchanged at 90/95 active rows verified.

The [base contract](../../hardware/str005-v2-serial-qualification.md),
[clock amendment](../../hardware/str005-v2-serial-clock-amendment.md), and
[permission amendment](../../hardware/str005-v2-serial-permission-amendment.md)
remain unchanged. Channel001's permission failure and separately reviewed
closure remain immutable.

| Identity           | Value                                                              |
| ------------------ | ------------------------------------------------------------------ |
| Board              | Ultra 205 / BM1366                                                 |
| Firmware           | `097050c09e9def6a9bf57df254a597a0f287bfa1`                         |
| Gate               | `e20c0fd52d2216596f904992ffa54fda33be9025`                         |
| Application ELF    | `76639dc29251dcc3001164e159fd2ca8db142796df46c88b07b25345ae9d0498` |
| Context            | `b2c889e617a1e1f976c9dfc1fd25a889edb62a515fbcbdc88c7ccf3df2fdb179` |
| Private result     | `675230a92e4f6ebd78ca0cf1748019c364772b5f31742075dda3c3c38af7b4b0` |
| Sealed inventory   | `a12a3c25cc89c137d12ece4bc80c4896546f7f2d37bfaabf36109004b2b26c80` |
| Parent observation | `0e903f9798f7846c1a386d2a44d718c588e45e297623381b0572101468073744` |

Private artifacts reside under `scratch/str005-v2-serial/channel-002`. The seal
binds 961 files, including separately attributed parent observations. The
original finalizer and read-only reviewer agree on `unverified`, outcome
`stop_evidence_incomplete`, judge code `v2_baseline`, and no producer cause.
No accepted public projection was generated.

## Supported observations

Five state-preserving installations and four fresh update/reconnect cycles
completed, with exact candidate identity and maximum 65,536-byte request/response
exchanges. The page retained its private preservation baseline and connections
used native foreground browser gestures. These observations remain subject to
the full qualification failure below; they confer no successor cycle credit.

Independent execution and protocol inspection accepted one correlated Standard
channel/job exchange: seven device records, eleven device events and eight
fixture events. Submissions, device acknowledgements and unsubmitted work were
zero. Device-local preparation took 143865 microseconds, maximum read duration
714083 microseconds, maximum write duration 2154 microseconds, TCP connection
454266 microseconds, and reported network resource release 2013173 microseconds.
These are device-local measurements, not host/device timestamp differences.

Fresh authenticated restoration and accounting observations were collected.
Execution and accounting review pass. Device Identity, settings and authorization
comparisons match; mine-on-boot is false, leases are inactive and the candidate
remains installed. Accounting before and after the channel exchange is next
ordinal 18, last completed 17, total charged 1560000 ms and pending false.
The original exhausted campaign remains unchanged. No new work or shares were
observed.

## Two independent host failures

The first parent-observed failure was `v2_listener_inventory_shape` during
cleanup collection. macOS `lsof -Fpn` emitted numeric `f` file-descriptor records;
the published parser understood only `p` process and `n` name records. It rejected
the inventory before writing the resource or cleanup receipt. The raw listener
inventory and private pool port were not persisted. The private port existed
only in the prepared collector's memory and was lost when that parent exited.

Actual browser closure and supervisor zero-exit witnesses are preserved. The
fixture reported closed peer/socket/listener resources and exited. These facts
do not replace the independently required fresh pool-port absence observation.
Complete host cleanup therefore remains unproved, even though no owned browser,
supervisor or fixture remains running. The parser correction cannot reconstruct
that missing observation or retroactively qualify this attempt.

Canonical judgment rejects earlier at a separate baseline-selection bug. The
continuity judge chooses the first before-phase journal entry, which is the
page's configured, unconnected state. Authenticated ready entries and initial
accounting exist later in that same journal. Future evaluation must use the
already joined initial-accounting observation rather than assume that the first
page event is a device baseline. Historical review must retain the published
validator's behavior and original `v2_baseline` verdict.

The parent observation and canonical judgment are separate provenance. Neither
has been rewritten as a browser failure or device failure. Supplemental exact
copies of operator observations were moved out of the finalizer's reserved
staging-directory name before sealing; original operational artifacts were
unchanged. The canonical finalizer alone created its final-input snapshot.

## Software correction

The host parser now accepts numeric file-descriptor groups and rejects malformed,
mixed, duplicate or incomplete records. All 27 focused tests pass, including a
real macOS listener presence/absence regression that failed before the fix.

Future continuity review uses the authenticated initial-accounting row with
exact journal, context, phase, state and ordering joins. A realistic
configured-to-ready regression exercises the complete simulated installation
and cycle workflow. The historical selection rule is retained only for the
exact frozen evaluator digest and byte length, without a caller override.

Read-only review after these corrections reproduces the original Channel002
result digest, seal digest and `v2_baseline` verdict. The corrections do not
create the missing cleanup receipt or an accepted Channel projection.

Verification passed: ordered Cargo format/Clippy/build and 2346 tests (three
existing ignored), all 248 V2 host tests, all 191 canonical Bazel targets, Bright
Builds, native USB ownership/symbols, reference, redaction, and read-only
parity/progress checks. One initial aggregate test failure is retained locally;
the fake readiness producer now matches the real fixture's atomic publication,
and the final complete suite passes. No firmware or Gate runtime change was
needed for these host corrections.

## Continuation boundary

The current permission amendment admits only Channel002 after Channel001; it
explicitly rejects Channel003 and generic retries. Share requires a separately
accepted Channel result. A fresh attempt needs prospectively published, guarded
admission that preserves this unverified result, verifies targeted corrections,
establishes safe fresh ownership and the actual installed baseline, and assigns
a new root/ordinal exclusively. It cannot reuse this consumed context, treat
missing cleanup as a pass, import these cycles, or start mining directly.

The Noise-only accepted evidence remains bound to its own earlier pair. This
attempt provides no accepted V2 share or heartbeat-loss evidence and makes no
claim of universal cancellation inside opaque cryptography.
