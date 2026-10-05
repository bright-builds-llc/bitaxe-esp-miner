# USB BBPLL recalibration install amendment

## Status and precedence

Owner-authorized on 2026-10-05 under `task-usb-stuck-link-after-reset`,
together with the standing autonomous fix-and-retry authorization. This
document defines the successor profile `usb-bbpll-install` of the
[noise-serial qualification](str005-noise-serial-qualification.md) and its
[parity-scope amendment](str005-noise-parity-scope-amendment.md). It reuses
their operator flow, journal, judges and cleanup unchanged. It replaces only:

- the namespace;
- the task gate;
- the predecessor;
- the expected ledger.

Parity stays 90/95.

## Why this image

The USB-Serial/JTAG link has needed a USB replug after resets
([known issues](known-issues.md)). The peripheral runs on the BBPLL, which
CPU and USB-core resets do not reset. ESP-IDF's startup BBPLL recalibration
powers that PLL down after every non-cold reset, inside the window where a
stuck install's monitor stream stopped. This image sets
`CONFIG_ESP_SYSTEM_BBPLL_RECALIB=n`, which the build enforces. Our
v5.5.4 bootloader is written by every install. There is no controller, PHY,
descriptor or ownership change, so ADR-0021 is unaffected.

## Profile

| Field        | Value                                                                                                                                                                                                                              |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Namespace    | `scratch/usb-bbpll-install`                                                                                                                                                                                                        |
| Task line    | `USB BBPLL install hardware: enabled.`                                                                                                                                                                                             |
| Predecessor  | The sealed recovery016: a current safe recovery with idle V2 on `6f268518` after loop007                                                                                                                                           |
| Ledger       | Next 27, last 26, 3,180,000 ms, unchanged by the install                                                                                                                                                                           |
| Before image | `6f268518b86ee264911e033acad769fac176f375` / `b6908f6dda85b5e4d133401dcf3dc84f940d4f41121216c40e93f2ff4f6cbe3b`                                                                                                                    |
| Candidate    | The clean pushed HEAD's canonical `just package` output with native readiness. `just audit-stack-realignment` and `just audit-startup-frames` must pass on the exact ELF, and `recalib_bbpll` must be absent from its symbol table |

The operator sequence is the queue-workaround reinstall's. After a pass,
`just str005-lineage advance-install --root <attempt>` records the new
install.

## Prohibited and stop conditions

These are the same as the
[queue workaround reinstall](str005-queue-workaround-reinstall-amendment.md):

- no Start, grant or mining;
- no factory or NVS reset;
- no core-dump clearing;
- at most five installs;
- stop on any identity, ledger, flash or cleanup failure, or a panic.

A stuck USB link during an install is recorded as evidence for the task;
recovery is the owner's USB-only replug and an owner-confirmed continuation.

## Non-claims

The install proves only that the image runs with the ledger preserved. Whether
the link survives resets is measured separately with
`just usb-reset-endurance`.
