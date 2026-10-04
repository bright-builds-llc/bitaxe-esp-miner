# STR-005 control-stack reproduction amendment

## Status and precedence

Owner-authorized on 2026-10-04 under `task-str005-start-panic-diagnosis` ("yes,
go ahead with the diagnostic image"), together with the standing autonomous
fix-and-retry authorization.

This amendment covers what happens after the
[control-stack diagnostic install](str005-control-stack-diagnostic-amendment.md)
put `c634cc20`/`d986b2ea` on the Ultra 205: it reproduces the idle-review panic
on that image and captures it. It applies the
[development core-dump contract](development-core-dumps.md) and the
[idle-panic capture amendment](str005-idle-panic-capture-amendment.md) rules
unchanged. Parity stays 90/95.

## Why a clear comes first

The image keeps the first stored dump (`CONFIG_ESP_COREDUMP_FLASH_NO_OVERWRITE`).
The partition still holds the boot-15 dump. Capture001 archived it privately
(full 952 KiB, SHA-256 `71b18ec37b5ad07bdc32349ed7f79c2d0c1d0102142634b3e0e8e9730d9ac09a`).
A new panic leaves nothing readable until that archived dump is cleared.

## Phase A: current recovery (`just str005-control-diagnostic-recovery`)

This is the failure-only current-recovery collector with profile
`CONTROL_DIAGNOSTIC_RECOVERY` (`scripts/str005-panic-recovery/control-diagnostic.mjs`):
- predecessor: the sealed install attempt-001 (result `923b9c6a…`, seal
  `91144bd8…`), which supplies the image identity, ledger and Gate assets;
- recovery001, which supplies the board's physical identity and the last
  attempt ID;
- listener `127.0.0.1:48765`; any boot after 15 is accepted;
- task line `Control diagnostic recovery hardware: enabled.`

Use a fresh root for every proof:

```sh
just str005-control-diagnostic-recovery preflight --private-root <parent>/<recoveryNNN> --predecessor-root <install attempt-001> --gate-root <pinned-gate>
```

The detector, serve, page steps and finish are the same as recovery001's.

## Phase B: one archive-bound clear (`just core-dump-clear`)

Run within the 120-second window of a phase-A proof:

```sh
just core-dump-clear --board 205 --port <fresh-port> --expected-physical-sha256 <physical> --expected-installed-source c634cc206979fd4179eb32478d20feab1825e31c --expected-installed-elf d986b2ead04672f42dbab9eb8c17e52f63cf1877881cf4c7ddcdb276cf8b5770 --recovery-proof <proof> --private-root <clear-parent>/clear001 --preserved-dump <capture001>/core-dump.private.bin --preserved-sha256 71b18ec37b5ad07bdc32349ed7f79c2d0c1d0102142634b3e0e8e9730d9ac09a
```

The task line is `Development core-dump clearing: enabled (private archive verified).`

The command:
- re-reads the partition and requires a byte-for-byte match with the archive;
- erases only the validated core partition, then reads it back as erased;
- performs the guarded application return (one reset) and releases.

## Phase C: bounded read-only review loop (`just str005-review-loop`)

```sh
just str005-review-loop preflight --private-root <loop-parent>/loop001 --predecessor-root <install attempt-001> --gate-root <pinned-gate>
just detect-ultra205   # -> <loop-parent>/detector.stdout.log, <=60 s before serve
just str005-review-loop serve --private-root <loop-parent>/loop001
just str005-review-loop finish --private-root <loop-parent>/loop001   # after final-detector.stdout.log
```

Task line: `Control review loop hardware: enabled.`

In the persistent Gate tab:
1. Click Connect Worker, then "Run bounded read-only review loop".
2. The page requires an idle, ready session with the lease inactive.
3. It runs up to 300 rounds of the reviews the panicked baseline issued: the
   ledger, budget, V2 possession and V2 status. Each review is bounded at 5 s,
   and the loop stops at the first failure.
4. Rows carry only counts, the operation and a closed Gate category.
5. Close, then send the tab to `about:blank`.

No Start, grant, signer, fixture, mining, suppression or write command is
reachable from the page.

## Phase D: capture after any panic

A device panic is judged by a fresh phase-A recovery. Its signs are a new boot
with `reset_reason=panic`, or a loop failure category such as `timeout` or
`closed`.

On a panic:
1. Within the proof window, make one `core-dump-read`. This needs the task
   line `Development core-dump acquisition: enabled (recovery evidence
   prerequisite satisfied).`
2. Run offline `inspect`/`analyze` against the retained exact ELF.
3. Read `BITAXE_PANIC_FRAME_RECORD`, `BITAXE_CONTROL_STACK_TRACE` and any
   panic-details note. A watchpoint or named-abort cause is decisive.

Without a panic, the outcome is "not reproduced in N rounds". The captured
high-water mark is then unavailable, and that is not proof of headroom.

## Prohibited

- any Start, grant, renewal, mining or fault injection;
- external pools, Wi-Fi provisioning, a factory or NVS reset, any flash write
  other than the phase-B core-partition erase;
- a second clear or read on the same root, or an implicit retry;
- direct UART or pins, network discovery, or synthesized permission gestures;
- publishing raw dumps, memory or unredacted analysis.

## Stop conditions

Stop on:
- a detector result other than exactly one Ultra 205;
- identity or ledger drift;
- a stale proof;
- an archive mismatch (never erase on mismatch);
- a failed ROM admission (no blind reset retry);
- an unproven return or cleanup.

Each further phase needs a fresh root and proof. Further loops follow the
progress-gated attempt policy.
