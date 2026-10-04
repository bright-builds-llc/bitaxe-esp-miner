# STR-005 control-stack diagnostic install amendment

## Status and precedence

Owner-authorized on 2026-10-04 under `task-str005-start-panic-diagnosis`: "yes,
go ahead with the diagnostic image". This document defines the
successor profile `control-stack-diagnostic-install` of the
[noise-serial qualification](str005-noise-serial-qualification.md) and its
[parity-scope amendment](str005-noise-parity-scope-amendment.md). It reuses
their operator flow, journal, judges and cleanup unchanged. It replaces only:
- the namespace;
- the task gate;
- the predecessor;
- the expected ledger.

Earlier results keep their meaning. Parity stays 90/95.

## Why this image

The [idle-review panic capture](../parity/evidence/20261004-str005-idle-panic-capture.md)
found a `StoreProhibited` on the Worker control owner. Before the fault, the
owner's register spill area had been overwritten with heap-like data. The
leading hypothesis is an earlier silent overflow of the owner's 16 KiB stack.
This image only adds observability; it changes no control behavior:
- `CONFIG_FREERTOS_WATCHPOINT_END_OF_STACK=y`. Any write into the last 32
  bytes of a running task's stack faults at the writing instruction. The build
  requires this line.
- Before and after each control command, the owner checks internal-heap
  integrity. On a failed check it aborts with
  `bitaxe_control_heap_corrupt_before_command` or `..._after_command`, which
  the dump's panic-details note keeps.
- After each command, the owner's stack high-water mark is folded into
  `BITAXE_CONTROL_STACK_TRACE`. That record sits in the captured
  `.dram2.coredump` region and holds the command count, minimum free bytes,
  the command at the minimum, the last free bytes and the check count.

## Profile

| Field         | Value                                                                       |
| ------------- | --------------------------------------------------------------------------- |
| Namespace     | `scratch/str005-control-stack-diagnostic`                                    |
| Task line     | `Control stack diagnostic install hardware: enabled.`                        |
| Predecessor   | The sealed idle-panic `recovery001` result: current safe recovery on `654338d0` |
| Ledger        | Next 26, last 25, 3,000,000 ms, unchanged by the install                     |
| Before image  | `654338d0101521490d90330c5a4a10e5ec32e5c2` / `2641c24fc3f4fc3a80a4bcb8bfd389b2d70d3e048d9e18a5771b14309b588193` |
| Candidate     | The clean pushed HEAD's canonical `just package` output, with native readiness |

The operator sequence is the base contract's, with ordinal 1, the
state-preserving update segments only, and Connect on `127.0.0.1:48765`:
1. Before-install accounting.
2. Install 0, configure the candidate, then installs 1–4 with one recorded
   cycle each.
3. The network-only Noise diagnostic.
4. Restore and record, then cleanup, finalize and review.

## Prohibited

- any Start, grant, renewal or mining;
- external pools or credentials, Wi-Fi provisioning;
- an NVS or factory reset, `--factory-reset`, erase or rollback;
- core-dump clearing;
- more than five installs, or a retry without a reviewed continuation;
- direct UART or pins, network discovery, or synthesized permission gestures;
- publishing raw private evidence.

## Stop conditions

Stop on:
- a detector result other than exactly one Ultra 205;
- identity, ledger or baseline drift;
- a failed flash or untrusted output;
- a panic during install (preserve the dump, do not clear it);
- unproven cleanup.

Any panic on this image is the intended diagnostic signal. Capture it under a
fresh current recovery and read contract before any further effect.

## Non-claims

The install proves only that the diagnostic image runs with the ledger
preserved. A reproduction of the panic, and any correction, need their own
contracts.
