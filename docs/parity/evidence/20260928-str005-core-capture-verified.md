# STR-005: controlled core capture verified

Installation007 verifies the measured-capacity correction on published firmware
`361425b902a3e214d6d0b052118c9e7a710458aa`. The ASIC-off controlled self-test
produced a valid dump with its native cutoff receipt in captured memory. This
resolves the observed capture failure; it does not establish the original Start
panic cause or successful mining startup.

| Observation                              | Measured result                                                    |
| ---------------------------------------- | ------------------------------------------------------------------ |
| Full ELF SHA-256                         | `7f3ea3ce75bf3eb8eb4c9a23a5eb7de5110114e124c22341f5cd2a867f97ebf2` |
| Gate                                     | `d3ac37435fbf98af76113fe5b962989e3616f707`                         |
| Self-test                                | ACK matched; explicit panic boot5→6; healthy exact identity        |
| Duration / continuity                    | 9,112 ms; uninterrupted USB; zero port reopens                     |
| Requested / prepared dump                | 88,668 / 88,704 bytes                                              |
| Partition capacity                       | 974,848 bytes                                                      |
| SDK init / prepare / start / end / store | All returned zero                                                  |
| Official decoder                         | Checksum and full ELF identity verified                            |
| Captured cutoff                          | ASIC outputs disabled; generation revoked; self-test marked        |
| Final recovery                           | Boot7; healthy runtime/HTTP/SPIFFS; restoration and release passed |

The profile excludes bulk heap/PSRAM and ordinary DRAM while retaining eligible
task stacks, TCBs, registers and the explicitly selected 28-byte cutoff receipt.
The native v2 audit proves the selected receipt region is in the SDK's actual
memory-section table. The offline verifier reads the receipt from a captured
PT_LOAD segment, not the program ELF's initial bytes. Protected GDB analysis
contains `panic_abort`, `esp_system_abort`, `abort` and
`qualification_restart::core_dump_self_test`, consistent with the controlled
fault. Raw memory, registers and vendor output remain private.

The immediate post-panic store record is valid and source/boot-bound, with
`store_reported_success`. A later post-ROM recovery is explicitly classified
`outside_panic_boot`; it does not replace the immediate receipt. Fresh accounting
remained next18/last17/1,560,000 ms/pending=false, with the old campaign exhausted
at 240,000 ms. A future attempt must measure the next ordinal again.

Private evidence root: `scratch/str005-panic/installation007/attempt`.

| Artifact            | SHA-256                                                            |
| ------------------- | ------------------------------------------------------------------ |
| Sealed inventory    | `7131552c725c17c070b52b4238b2a92a742ccf34691ebadb5df2885ac3d5f925` |
| Full raw partition  | `2f74b8ef1404580a48d3e50514c343fc82e00d8447f80011c651b4d7c86f5b38` |
| Cutoff inspection   | `23ca80a09e5ce43ccdef0f9220b6c4d02b9d0845808173e1062b5f9ada654e3a` |
| Analysis inspection | `7562b76d4c9c4f5d2bb9a5966d68314838a1a0341bf7b29aee196cf5942dd3a7` |
| Final recovery      | `896b2a67d2753ac0ca814a431bb4826b46359bc0f7f3ddd4f3bc394996801a98` |

The generic observer result's `core_capture_verified=false` is preserved. Capture
qualification is established by the actual decoder and cutoff-verifier artifacts,
not that aggregate field. Their `cause_proven=false` correctly leaves the original
Start cause unresolved. The ELF-format header's unused `tasks_declared` field
must not be interpreted as a count of decoded threads.

Browser, server and device ownership were released before sealing. No clearing,
Start, grant, mining, factory reset, external pool or Share002 replay occurred.
The captured self-test remains on device pending a separately published clearing
and startup contract. Earlier seals are unchanged; parity remains 90/95. The
panic task remains active because its original causal question is unresolved.
