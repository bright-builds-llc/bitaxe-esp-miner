# Diagnostic self-test captured a usable original panic frame

The installed `ce8f015811b93385c5aab0bac7bcdf6307452b31` image retained a complete,
byte-verified core from one ASIC-off self-test. The exact full ELF SHA-256 was
`453d2fa3bbe2b58bcffcbf2019ab69c7968d1ae90620685c8139a061a3325c31`.
The self-test establishes capture capability on this image. It does not explain
the earlier Share001 crash.

| Boundary           | Observed result                                                                                                                                           |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Before self-test   | Empty974848-byte core region; authenticated boot20, inactive lease, preservation matched                                                                  |
| Fault              | One ASIC-off self-test; acknowledged panic reset and runtime-ready boot21                                                                                 |
| Store              | Previous-boot receipt reported79424 prepared bytes, store result0                                                                                         |
| Core read          | Official same-physical reader returned exact image and released resources; raw SHA-256 `9db6ac9b2cbb52be6493a6e8f397388bac1a69a36ad17e5acf71369fd4de1644` |
| Offline inspection | Checksum, exact full ELF, native cutoff and captured memory verified                                                                                      |
| Provenance         | Original frame meaningful; source and pre-reset boot joined; no SDK fake crashed frame                                                                    |
| Debugger           | Pinned bounded batch analysis completed privately                                                                                                         |
| After read         | Independent authenticated boot22 recovery, Stop/Close and host release                                                                                    |
| Accounting         | Next21/last20/charged2,100,000 ms and original charged240,000 ms, both pending=false                                                                      |
| Capture result     | `complete=true`, `core_capture_verified=true`, no blockers or first failure                                                                               |

The empty-core recovery is privately sealed at
`scratch/str005-share-diagnostic/recovery002/attempt`, inventory SHA-256
`e39d2978de5060f5b41227b12bc02ae7b405d2e1358a77c422ee4412dc0b2e99`.
The self-test and acquired raw partition are privately sealed at
`scratch/str005-share-diagnostic/capture001/attempt`, inventory SHA-256
`8806886244f104ad611f0717932b1f4cbf6dfc56840cb0edd464924fca804a16`.
The full raw dump, allocation history, registers, stacks and debugger text stay
in ignored owner-only storage. Earlier Share001/core seals remain unchanged.

No Start, mining, grant or renewal occurred, and no ordinal was consumed. The
captured on-device core must be cleared only through a fresh, separately published
archive-bound contract before another deliberate panic. The accepted-share task
remains active, Share001's cause and historical resource proof remain unresolved,
and parity remains **90/95**.
