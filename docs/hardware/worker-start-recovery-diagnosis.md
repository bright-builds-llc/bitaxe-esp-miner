# Worker Start and recovery admission diagnosis

Software investigation of attempt-010, 2026-09-06. The original hardware
observations and consumed campaign are unchanged; see the
[attempt report](../parity/evidence/20260906-fixed-usb-attempt-010.md).

## Reproduced failure

The production Worker adapter registers an owner session before asking the
Production Mining Session to prepare hardware. When network, safety freshness
or actuation readiness blocks that request, the core returns before adopting
the lease. On a fresh boot its state remains `Unavailable / Unprepared`.

The adapter formerly waited for either `Active` or `Consumed` to settle Start.
Neither state could occur on this path. The browser's 30000-ms response bound
then expired while the firmware caller could wait up to 65000 ms. Revocation
could not finish the unadopted lease: the core had no lease or started hardware
to stop, while adapter cleanup required `Consumed / Stopped`. The revoked
generation therefore remained owned, and `begin_link` rejected fresh Hello
admission. Increasing a response timeout cannot repair that missing transition.

A prior consumed campaign produced different behavior: its old terminal
snapshot could settle the new blocked candidate. This demonstrated why cleanup
must distinguish whether this candidate actually reached preparation.

The deterministic reproduction includes the actual firmware `bwg.rs`, actual
generation gate and actual Production Mining Session core. Only readiness,
time, notifications and durable-storage effects are replaced with synthetic
host adapters. It does not contact hardware, a pool or private inputs.

```sh
bazel test //firmware/bitaxe:production_worker_admission_tests --test_output=errors
```

The first run observed an empty Start reply where rejection was required, and
failed fresh-link admission after revocation. A third initial assertion forbade
logical generation activation; review found that stronger than the required
behavior. Logical ownership is needed to retain exclusion during durable
cleanup and is separate from ASIC dispatch. The final regression checks no
hardware preparation/work, prompt rejection and correct reservation retirement
instead. The original failing run remains retained.

## Correction and verification boundary

The adapter now records whether the current candidate crossed the actual
`PrepareHardware` effect boundary. Completion rejects a candidate that never
reached that boundary, revokes its generation and finalizes its existing
reservation before releasing ownership. Finalization consumes the reservation;
it does not refund it or allocate another campaign.

The flag is set before the first fallible preparation operation. Preparation
failure, cancellation and late completion therefore retain the existing
ordered shutdown requirement. Such a generation is released only after the
core confirms `Consumed / Stopped` and durable finalization succeeds. SafeStop
acknowledgment now follows that finalization rather than preceding it. A
storage failure retains ownership and leaves restoration unacknowledged until
a later cleanup pass succeeds.

The dedicated suite contains ten admission/cleanup regressions and 22 existing
generation/shutdown tests. It covers fresh and previously consumed core state,
network/safety/actuation rejection, failed durable cleanup, restoration
acknowledgment, cancellation during preparation and late preparation results.
The tests complete in under a second once built. The host storage adapter tests
ordering and failure handling; existing budget tests cover the durable ledger.
No test here claims real NVS, ASIC, cooling or three-second hardware evidence.

Full verification passed: ordered Cargo format/lint/build/tests (2083 passed,
one preexisting ignored), all 81 Bazel tests, real ESP32-S3 packaging, native
USB ownership/symbols, reference integrity, redaction, formatting and standards.
The formatted admission harness was rerun successfully. Read-only review found
no introduced regression and identified the separate pre-owner gap below.

The simplification review retained the existing generation state machine,
timeouts and wire contracts. No browser transport change, new USB mode or
compatibility layer is involved.

## Remaining uncertainty

This proves and corrects a software path consistent with Start timeout and
blocked recovery. Attempt-010 captured no running/qualification sample or
readiness discriminator, so its exact initiating condition remains unknown.
The earlier manifest-fields recovery failure is not explained by these tests.
Pool activation, blocking peripheral operations and command timing beyond
initial readiness remain separate possible causes of a Start timeout.

A separate preexisting cleanup gap also remains: errors after durable budget
reservation but before `OwnerSession` installation can bypass owner-managed
finalization. The correction above covers registered candidates. Queue
rejection, pre-owner admission failure and reservation/connection races need
their own regression before a comprehensive Start-cleanup claim or further
live acceptance. No deadline should be widened to hide these boundaries.

No hardware effect or campaign mutation was performed during this diagnosis.
The previously verified serial-0.2 firmware remains installed with mining
disabled. The original consumed window and all four hardware cycle receipts
remain historical evidence of their exact source/ELF pair. This correction
requires its own hardware qualification before a hardware-success claim.

## Pre-owner follow-up

ADR-0024 addresses the earlier reservation gap: a reservation fence precedes
NVS work, cleanup waits for in-flight writes and owner acknowledgment, and
competing finalizers claim retirement before updating counters. Ambiguous
writes preserve the intended full charge. Actual owner queue rejection,
revoked admission, durable failure and read-only ledger review have dedicated
host regressions. Bounded correlated errors and independent closed diagnostics
make rejection distinguishable from transport timeout. The original hardware
failure remains its original evidence; only a new qualified attempt can
establish hardware behavior after these corrections.
