# STR-005: controlled panic observed, core capture unverified

Outcome: **blocked — core partition erased after the controlled panic**.
The diagnosis task is not complete and is not archived. Parity remains 90/95.

## Exact execution and observed result

| Item                | Evidence                                                                                 |
| ------------------- | ---------------------------------------------------------------------------------------- |
| Installed firmware  | `9be53f69943ded2c25f9a644c3c4a98558fb3d54`                                               |
| Matching ELF        | `075be768fc0c3789f8577917e2aadbbb28ef62d8ee1388f5cd46a54079135fde`                       |
| Capture host        | `dae7fefb`                                                                               |
| Gate                | `14d0e5b37081b78d0e5c44992010f83a1cf73e95`                                               |
| Controlled effect   | One authenticated ASIC-off self-test; no work grant                                      |
| Reset observation   | Acknowledgement matched; boot 9→10; explicit panic; exact identity and healthy readiness |
| Duration/continuity | 8,420 ms; zero port reopens; uninterrupted native USB                                    |
| Dump read           | Full 974,848-byte partition read; application return and cleanup passed                  |
| Dump contents       | Entirely erased: all bytes `0xff`                                                        |
| Inspection          | Failed before GDB; no valid checksum, ELF note, backtrace or captured cutoff receipt     |
| Final recovery      | Fresh signed identity, accounting, idle status, restoration and release passed           |
| Accounting          | Next 18; last completed 17; total charged 1,560,000 ms; pending false                    |

No Start, mining, grant issuance, dump clearing, factory reset, NVS erase,
external pool or Share002 replay occurred. The native panic wrapper passed its
compiled routing/IRAM audit, but empty flash supplies **no runtime cutoff proof**.
The original Start panic cause remains unknown.

## Corrections made during the staged work

The first diagnostic installation exposed an HTTP task-creation allocation
failure. Adding Rust debug information had unintentionally selected IDF debug
optimization. The correction pins IDF performance optimization and a 2 KiB
default-allocation preference threshold, keeping the internal reserve, HTTP/task
stacks and dedicated dump stack unchanged. Fresh authenticated diagnostics from
the corrected image show startup complete with first failure none; the
post-statistics DMA/internal checkpoint reports 7,575 free bytes and a 4,352-byte
largest block. These observations resolve that startup regression, not Share002.

An installation command initially had conflicting evidence flags; the fix added
real CLI dry-run admission. A later 480-second supervisor expired before its
capture receipt was persisted. That timeout remains a failure, despite the
reported completed write/application return. New read-only recovery established
the installed identity without reflashing. Future supervisors use a separately
recorded 1,200-second cap. No old timeout or installation outcome is rewritten.

Gate diagnostics are latest-value snapshots rather than ordered journals.
Capture admission now retains two exports with increasing ready uptime and a
fresh same-session status confirmation. The actual Gate history producer and
failure/cleanup paths were tested before the self-test.

## Precise blocker and next discriminator

The exact ELF includes the initializer, writer and standard panic path. Its
48-instruction wrapper preserves the native C ABI and delegates normally.
Intentional abort does not skip the writer. The observed complete 8.42-second
cycle is a poor match for the normal 10-second panic watchdog timeout. Static
review does not establish which writer preflight check actually failed.

The earlier assertion that PSRAM heap data was excluded was incorrect. The
pinned [IDF heap writer](https://github.com/espressif/esp-idf/blob/v5.5.4/components/espcoredump/src/core_dump_elf.c)
walks used blocks across all 8-bit heaps. A 512 KiB log and roughly 90 KiB
statistics history make capacity a concrete concern. The
[flash writer](https://github.com/espressif/esp-idf/blob/v5.5.4/components/espcoredump/src/core_dump_flash.c)
checks required length before erasing. This makes capacity rejection compatible
with empty flash, but it is **not proven**; initialization/configuration and
other early-return conditions remain possible.

Before another fault, retain fixed numeric initialization/store status and the
required length/capacity through a panic-safe mechanism, then verify its native
linkage and decoder. Use that measured result to choose the next correction.
Do not increase stack/watchdog limits, resize partitions or retry unchanged on
this evidence alone. Full private development dumps remain authorized.

## Evidence integrity and cleanup

Private root: `scratch/str005-panic/capture-001/attempt`.
Raw files, identity-bearing diagnostics and decoder output remain protected and
ignored. Only closed conclusions and digests are promoted here.

| Artifact                         | SHA-256                                                            |
| -------------------------------- | ------------------------------------------------------------------ |
| Sealed inventory                 | `d7074817eb7ab790b053674d6729d85b293ee98fd239e290b3566feef108d7d9` |
| Self-test observer result        | `e313abe0010143e631f1cc0bad40409f8255212a4bd0ec026c98b1c98dd8800f` |
| Raw core partition               | `94a21164829c644f15d62317c52d9f42a0ef66bd084d5ffdeb007b375e210951` |
| Read/return/cleanup result       | `a61ddec50a8bb0ee036827336b7b18c8dfb0ede6ca4cdf20002a53a7c12460cc` |
| Failed inspection classification | `3a47a4d0bb92b1027f0ca25f7b58d5a183d6b36ebee0701054fd27043bcdabcc` |
| Final current-recovery proof     | `b4ff2f0c10151f7d7e154a76781e8a73144358d10974fb1148d07e7dc8e9dc93` |

Both recovery rounds, native Close, serial-holder absence, owned page/server and
listener cleanup passed. The sealed baseline result's `complete: true` is scoped
to baseline/reset/recovery collection; its `core_capture_verified: false` remains
explicit. This report's capture-qualification outcome is blocked.

Share002 and prior seals remain unchanged. Its missing historical retained
resource proof is not supplied by these current-state observations. The active
task disables further effectful stages pending new verified diagnostics.

Verification included ordered Rust checks (2,403 tests passed, three existing
ignores), 45 host probe tests plus the actual Gate history-producer integration,
40 offline decoder/cutoff tests, affected Bazel/automation checks, canonical
native packages, static cutoff/USB checks, standards, redaction and parity checks.
Passing software checks do not replace the failed hardware capture observation.

Finalization rechecked all nine selected private inventories, including Share002
and recovery004. The disabled acquisition gate rejected a synthetic nonexistent
port before environment/device discovery or output-root creation.
