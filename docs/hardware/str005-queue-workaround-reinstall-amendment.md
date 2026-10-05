# STR-005 queue workaround reinstall amendment

## Status and precedence

Owner-authorized on 2026-10-04 under `task-str005-start-panic-diagnosis`, as
the verified-fix continuation of the
[queue workaround install](str005-queue-workaround-amendment.md), together
with the standing autonomous fix-and-retry authorization. This document
defines the successor profile `queue-workaround-reinstall` of the
[noise-serial qualification](str005-noise-serial-qualification.md) and its
[parity-scope amendment](str005-noise-parity-scope-amendment.md). It reuses
their operator flow, journal, judges and cleanup unchanged. It replaces only:

- the namespace;
- the task gate;
- the predecessor;
- the expected ledger.

Parity stays 90/95.

## Why a reinstall

Queue-workaround attempt-001 installed `a2052ab0`, which panicked on every
boot. A one-time reproduction stored its core dump. Decoded, the dump shows
the 16 KiB main task overflowing during the Worker trust parse's curve25519
key checks:

- the queue change let `production_mining_session::start` (4,400-byte
  frame) inline into `run_startup`;
- `run_startup`'s frame grew from 1,776 to 4,896 bytes and stayed live under
  that parse.

The fix keeps `start` out of line (`run_startup`: 1,072 bytes).
`just audit-startup-frames` blocks the failing image and passes the previous
and fixed ones.

## Profile

| Field        | Value                                                                                                                                                                                                                            |
| ------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Namespace    | `scratch/str005-queue-workaround-reinstall`                                                                                                                                                                                      |
| Task line    | `Queue workaround reinstall hardware: enabled.`                                                                                                                                                                                  |
| Predecessor  | The sealed recovery015: a current safe recovery with idle V2 on the restored `7ca3e29c`                                                                                                                                          |
| Ledger       | Next 27, last 26, 3,180,000 ms, unchanged by the install                                                                                                                                                                         |
| Before image | `7ca3e29ce1870396c801f9d8d74ff02aac2ef112` / `227bc380ec2d2171d187f8561390259fae464b92164d4d30c2a80354135eb3f0`                                                                                                                  |
| Candidate    | The clean pushed HEAD's canonical `just package` output with native readiness. `just audit-stack-realignment` must report no callers and `just audit-startup-frames` must report `startup_frames_within_budget` on the exact ELF |

The operator sequence is the queue-workaround install's. After a pass,
`just str005-lineage advance-install --root <attempt>` records the new
install.

## Prohibited and stop conditions

These are the same as the
[queue workaround install](str005-queue-workaround-amendment.md):

- no Start, grant or mining;
- no factory or NVS reset;
- no core-dump clearing;
- at most five installs;
- stop on any identity, ledger, flash or cleanup failure, or a panic.

The core-dump partition is empty, so a recurrence is stored. Recovery from a
boot loop uses the restore path recorded in `task-str005-queue-bootloop-recovery`.

## Non-claims

The install proves only that the fixed queue image runs with the ledger
preserved. The workaround is verified separately by the two static audits
and a bounded review loop on the installed image.
