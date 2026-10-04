# STR-005 stack-realignment correction install amendment

## Status and precedence

Owner-authorized on 2026-10-04 under `task-str005-start-panic-diagnosis`, as
the correction step of the owner-requested panic diagnosis, together with the
standing autonomous fix-and-retry authorization. This document defines the
successor profile `realignment-fix-install` of the
[noise-serial qualification](str005-noise-serial-qualification.md) and its
[parity-scope amendment](str005-noise-parity-scope-amendment.md). It reuses
their operator flow, journal, judges and cleanup unchanged. It replaces only:
- the namespace;
- the task gate;
- the predecessor;
- the expected ledger.

Parity stays 90/95.

## Why this image

The reproduced idle-review panic decodes to a toolchain hazard:
`std::sync::mpmc` channel constructors realign their frame to 64 bytes with a
plain write to `a1` instead of `movsp`. An interrupt in that prologue makes
the caller restore its registers from stale memory.

The corrected source avoids it:
- every per-request reply site uses `bitaxe_runtime::reply`;
- `just audit-stack-realignment` proves that only startup code can reach a
  realigning constructor.

The control-stack diagnostics (end-of-stack watchpoint, per-command heap
checks, the captured stack trace) remain in this image, so a recurrence still
decodes.

## Profile

| Field         | Value                                                                       |
| ------------- | --------------------------------------------------------------------------- |
| Namespace     | `scratch/str005-realignment-fix`                                            |
| Task line     | `Realignment fix install hardware: enabled.`                                |
| Predecessor   | The sealed control-diagnostic `recovery007` result: current safe recovery on `c634cc20` |
| Ledger        | Next 26, last 25, 3,000,000 ms, unchanged by the install                     |
| Before image  | `c634cc206979fd4179eb32478d20feab1825e31c` / `d986b2ead04672f42dbab9eb8c17e52f63cf1877881cf4c7ddcdb276cf8b5770` |
| Candidate     | The clean pushed HEAD's canonical `just package` output with native readiness, plus a passing stack-realignment audit on the exact ELF |

The operator sequence is the control-stack diagnostic install's. Install `i`
precedes cycle `i`, with Close and flush before each install and Connect on
`127.0.0.1:48765`.

## Prohibited and stop conditions

These are the same as the
[control-stack diagnostic install](str005-control-stack-diagnostic-amendment.md):
- no Start, grant or mining;
- no factory or NVS reset;
- no core-dump clearing;
- at most five installs;
- stop on any identity, ledger, flash or cleanup failure, or a panic.

A panic during installation is preserved for capture.

## Non-claims

The install proves only that the corrected image runs with the ledger
preserved. The panic correction is verified separately by the batched
review loops and the static audit.
