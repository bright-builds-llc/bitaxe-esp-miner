# STR-005 queue workaround install amendment

## Status and precedence

Owner-authorized on 2026-10-04 under `task-str005-start-panic-diagnosis`, as
the follow-up to the stack-realignment correction, together with the standing
autonomous fix-and-retry authorization. This document defines the
successor profile `queue-workaround-install` of the
[noise-serial qualification](str005-noise-serial-qualification.md) and its
[parity-scope amendment](str005-noise-parity-scope-amendment.md). It reuses
their operator flow, journal, judges and cleanup unchanged. It replaces only:

- the namespace;
- the task gate;
- the predecessor;
- the expected ledger.

Parity stays 90/95.

## Why this image

The realignment fix removed every per-request std channel. Nine startup-time
std queues remained as residual risk. The Xtensa LLVM backend realigns their
constructors' frames without `movsp`
([espressif/llvm-project#140](https://github.com/espressif/llvm-project/issues/140),
[esp-rs/rust#284](https://github.com/esp-rs/rust/issues/284)), and no esp
toolchain release fixes it.

This image replaces them with `bitaxe_runtime::queue`: a mutex, two condition
variables and a `VecDeque`. The empty-allowlist realignment audit reports no
realigning function in the image at all. The control-stack diagnostics stay in
the image.

## Profile

| Field        | Value                                                                                                                                                    |
| ------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Namespace    | `scratch/str005-queue-workaround`                                                                                                                        |
| Task line    | `Queue workaround install hardware: enabled.`                                                                                                            |
| Predecessor  | The sealed recovery012: a current safe recovery on `7ca3e29c` after heartbeat007, with its terminal record read and resources released                   |
| Ledger       | Next 27, last 26, 3,180,000 ms, unchanged by the install                                                                                                 |
| Before image | `7ca3e29ce1870396c801f9d8d74ff02aac2ef112` / `227bc380ec2d2171d187f8561390259fae464b92164d4d30c2a80354135eb3f0`                                          |
| Candidate    | The clean pushed HEAD's canonical `just package` output with native readiness, plus `just audit-stack-realignment` on the exact ELF reporting no callers |

The operator sequence is the realignment-fix install's. Install `i` precedes
cycle `i`, with Close and flush before each install and Connect on
`127.0.0.1:48765`.

Installing reboots the board, which clears heartbeat007's retained record.
After a pass, `just str005-lineage advance-install --root <attempt>` records
the new install and clears the latest Start.

## Prohibited and stop conditions

These are the same as the
[realignment-fix install](str005-realignment-fix-amendment.md):

- no Start, grant or mining;
- no factory or NVS reset;
- no core-dump clearing;
- at most five installs;
- stop on any identity, ledger, flash or cleanup failure, or a panic.

A panic during installation is preserved for capture.

## Non-claims

The install proves only that the queue image runs with the ledger preserved.
The workaround is verified separately by the static audit and a bounded
review loop on the installed image.
