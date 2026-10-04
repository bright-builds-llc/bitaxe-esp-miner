# STR-005 idle-review panic capture amendment

## Status and precedence

Owner-authorized on 2026-10-04 under `task-str005-start-panic-diagnosis`: "go
ahead with the core dump and continue the work". The owner also authorized
autonomous, iterative fixes and fresh attempts. This amendment applies the
[development core-dump contract](development-core-dumps.md) and ADR-0030/0031
to one new panic on the installed image. Earlier panic-probe stages, Share001
and every sealed result keep their meaning. Parity stays 90/95.

## Basis

- **Image.** Firmware `654338d0`, ELF `2641c24f…`, installed by the step-5
  reinstall. The exact debug ELF is retained in that root's
  `qualified-artifacts`. The image writes ESP-IDF flash ELF core dumps to the
  `0xf12000` partition with first-dump preservation.
- **Panic.** Heartbeat005's baseline step issued only read-only Gate reviews
  on an idle device (no grant, no mining). One of them reported a serial
  `timeout`. Heartbeat006 then measured boot 16, `reset_reason=panic`, with an
  uptime that places the reset at that step on boot 15. The cause is unknown.
- **Accounting.** The ledger stayed at next 26, last 25, 3,000,000 ms, not
  pending.

## Phase A: one current recovery (`just str005-panic-recovery`)

This is the Share001 failure-only current-recovery collector, bound to a
different profile (`scripts/str005-panic-recovery/profile.mjs`):
- predecessor: heartbeat002's sealed root, which supplies the attempt ID and
  the Gate assets;
- task line: `Idle panic recovery hardware: enabled.`;
- listener: `127.0.0.1:48765`;
- the observed boot must exceed the panicked boot 15.

Every other rule is that collector's
[contract](../../scripts/str005-share-recovery/CONTRACT.md).

```sh
just str005-panic-recovery preflight --private-root <parent>/recovery001 --predecessor-root <heartbeat002/attempt> --gate-root <pinned-gate>
just detect-ultra205   # -> <parent>/detector.stdout.log, <=60 s before serve
just str005-panic-recovery serve --private-root <parent>/recovery001
just str005-panic-recovery finish --private-root <parent>/recovery001   # after final-detector.stdout.log
```

Operator sequence in the persistent Gate tab: Connect Worker, then "Prepare
fresh recovery session". Connect Worker again, then "Collect current recovery
and release". Finally navigate the tab to `about:blank`.
A complete collection writes `current-recovery.json`, timestamped at
collection begin.

## Phase B: one core-dump read (`just core-dump-read`)

Run within the proof's 120-second window, with a fresh detector:

```sh
just core-dump-read --board 205 --port <fresh-port> --expected-physical-sha256 <physical> --expected-installed-source 654338d0101521490d90330c5a4a10e5ec32e5c2 --expected-installed-elf 2641c24fc3f4fc3a80a4bcb8bfd389b2d70d3e048d9e18a5771b14309b588193 --recovery-proof <parent>/recovery001/current-recovery.json --private-root <capture-parent>/capture001
```

The task line `Development core-dump acquisition: enabled (recovery evidence
prerequisite satisfied).` admits exactly this read. Everything else follows the
read contract:
- one physical-device lease and bounded ROM admission (`board-info`);
- a validated partition table, and a read of only the core-dump partition;
- no erase or write;
- the guarded application return, an identity check and release.

The return resets the device, so the next boot is expected to be 17.

## Phase C: offline analysis (no device)

```sh
just core-dump inspect --dump <capture001/core-dump.private.bin> --elf <exact-ELF> --elf-sha256 2641c24f… --private-root <new-child>
just core-dump analyze --dump <same> --elf <exact-ELF> --elf-sha256 2641c24f… --private-root <another-new-child>
```

A dump binds this panic only if its ELF identity note equals `2641c24f…` and
it decodes. An erased region, a foreign ELF identity or a truncated dump is a
failed observation, not a cause. Committed conclusions are redacted: crash
category, faulting task and symbolized frames only, never memory contents.

## Prohibited

- core-dump clearing, erase, write, flash or factory operations;
- NVS reset;
- any Start, grant, renewal, mining, voltage or frequency override, or fault
  injection;
- a second read on the same root, or an implicit retry;
- direct UART or pins, network discovery, or synthesized permission gestures;
- publishing raw dumps, memory or unredacted analysis.

## Recovery, retry and stop

Recovery uses the collector's own Stop/Close path and the read's guarded
application return. Stop on:
- a detector result other than exactly one Ultra 205;
- identity drift;
- a stale or incomplete proof;
- a failed ROM admission (no blind reset retry);
- an unproven return or cleanup;
- invalid dump data.

A further read needs a verified boundary fix, a fresh proof and new roots.

## Non-claims

A decoded dump identifies where this panic occurred. Any correction still
needs its own regression and admitted verification. Heartbeat-loss
qualification stays unverified until a corrected image passes a fresh restart
and heartbeat attempt.
