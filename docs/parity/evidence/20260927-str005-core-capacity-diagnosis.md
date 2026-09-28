# STR-005: core-store capacity rejection measured

Installation006's instrumented ASIC-off self-test reproduced the erased core
partition and identified its cause: the requested dump exceeds the partition.
This diagnoses capture failure, not Share002's original mining Start panic.

| Observation                                  | Measured value                                                     |
| -------------------------------------------- | ------------------------------------------------------------------ |
| Firmware                                     | `bfc2cbb0122af5d79c73e64a57028fd2284a7eab`                         |
| Full ELF SHA-256                             | `065c301890135571dcfb4363b3fda9597b7315c790892cf01bea06c6f392f17a` |
| Gate                                         | `d3ac37435fbf98af76113fe5b962989e3616f707`                         |
| Self-test                                    | ACK matched, explicit panic, boot1→2, exact healthy identity       |
| Duration / continuity                        | 8,355 ms; uninterrupted USB; zero port reopens                     |
| Retained record                              | valid integrity/source/boot binding; self-test marked              |
| Store initialization result                  | 0 (success)                                                        |
| Raw requested / post-prepare length          | 1,195,632 / 1,195,632 bytes                                        |
| Prepare / store result                       | 257 / 257 (`ESP_ERR_NO_MEM`)                                       |
| Partition capacity                           | 974,848 bytes                                                      |
| Required including pinned alignment/checksum | 1,195,680 bytes; 220,832 over capacity                             |
| Write start / end                            | not observed                                                       |
| Full partition read                          | 974,848 bytes, entirely erased                                     |

The pinned SDK checks aligned size plus its checksum before erasing or writing.
The unchanged prepare length, NO_MEM result and measured size establish that
boundary. A later erase failure would occur after the SDK increased the length.
The typed classifier preserves initialization rejection, later failure and
incomplete progress as separate outcomes; it does not infer success from a return
code alone.

The 80-byte RTC record and five wrappers were software/native tested before the
fault. Native inspection caught a compiler-generated DROM jump table; a table-free
arithmetic correction passed the same audit. The clean native audit proved SDK
routes, original argument/results, IRAM accesses and bounded receipt writes, with
240 added stack bytes under the 256-byte gate. The normal initializer's linkage
is static evidence; `init_result` specifically measures SDK write initialization.

Fresh recovery saved the immediate post-panic receipt before ROM acquisition.
The read restored the exact application and released ownership; a second recovery
at boot3 confirmed healthy current state and actual Stop/Close/resource release.
That later record is explicitly outside the panic boot and cannot replace the
first receipt. Both ledgers stayed unchanged: next18, last17, 1,560,000 ms charged,
pending=false; the original campaign remains exhausted at 240,000 ms.

Installation006 inventory SHA-256:
`b1034a75f3927844fd98960b691064618a1850da866b3f91c5b0b045bc07c25f`.
Private root: `scratch/str005-panic/installation006/attempt`. Raw partition digest:
`94a21164829c644f15d62317c52d9f42a0ef66bd084d5ffdeb007b375e210951`.
All earlier seals remain unchanged. The host result's complete flag covers
installation/observation/recovery; `core_capture_verified` remains false.

The targeted correction selects SDK task-stack/register capture plus its explicit
user-memory region for the cutoff receipt, excluding the oversized bulk heap.
Actual dump fit, checksum/full ELF binding and captured cutoff still require a
fresh self-test. No partition or stack increase, Start, grant, mining, clearing,
factory reset or Share002 replay occurred. The task remains unarchived and parity
90/95 until its remaining criteria pass.
