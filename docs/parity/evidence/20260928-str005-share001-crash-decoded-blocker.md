# Share001 crash preserved and decoded; repair blocked by missing fault frame

Failure-only collection is repaired and verified on hardware. The original crash
partition was acquired, independently archived, checksum-verified and matched to
the exact installed ELF. Post-acquisition recovery passed. The captured native
panic state confirms generation revocation and disabled ASIC outputs.

The crash does **not** provide a repairable call chain: ESP-IDF substituted its
synthetic stack for the faulting task. No firmware repair, new Start or heartbeat
fault is justified by this evidence. The accepted-share task remains blocked and
unarchived; parity stays **90/95**.

## Acquisition and current recovery

All effects used published host `e827baaf6b8b89a3d83e954b9201e2c3ee5ba73c`,
installed source `f000872f2e436aa7cdaa8cbfa41eee965a27731e`, full ELF
`a3e257418d625aef5e9423fe28f598377f92b092b634264d3cc1e86b4e83e3c2`,
and Gate `9643e87664397a321c715a3a1b1bb6c1183b83ea`.

Fresh recovery004 passed on boot16. The official read consumed its one-use proof,
retained the physical lease, passed ROM admission, read the partition table and
entire974848-byte core region, returned to the exact application and released.
The offline verifier independently checked the table checksum/current geometry,
raw length, producer identity/source and return/release flags, then sealed the
original files. No clearing or firmware/NVS write occurred.

Post-return recovery005 passed on boot17 with authenticated idle status, current
restoration/preservation/inactive authority and actual host/serial release. Both
recoveries measured next21/last20/2100000ms, pending=false, and unchanged exhausted
original campaign240000ms. Historical retained resources remain unavailable.

| Private evidence root                               | Sealed inventory SHA-256                                           |
| --------------------------------------------------- | ------------------------------------------------------------------ |
| `scratch/str005-share-recovery/recovery004/attempt` | `5a2c707c640bd23287641a547ac968f0b830e87c71a0d55c29101e4fbfb063b6` |
| `scratch/str005-share-crash/acquisition001/attempt` | `3ec746b255da28290d8f1944e1cc375f7deea559f8d936bd49dea8e5ed6ad39b` |
| `scratch/str005-share-recovery/recovery005/attempt` | `49abc244d0efe12af30a7265cd5661dea0e735fa78baa1a7ff30b279cf3f39a6` |

Full-region raw SHA-256:
`b665c154d35f43bf7a0ab9acfe0aab207ac9e36c6a1a10ce649f3863f880f29c`.
Acquisition verification SHA-256:
`e1dfc48e049ce361d1debf0a60d40d4af71498cb003a2aa29af45150643939b1`.
Original Share001 and recovery001/002 seals remain unchanged.

## What the decode establishes

The pinned esp-coredump1.17.2 loader verifies checksum, ESP32-S3 format and full
ELF identity. `verify-cutoff` independently reports captured memory verified,
native cutoff verified, generation revoked, ASIC outputs disabled and
self_test_marked=false. Generic cause_proven remains false.

The high-level `analyze` command timed out at120s. Its partial output is retained
in `analysis001`; SDK reporting can spend repeated waits on unsupported task
queries, and its debugger cleanup contains an unbounded communicate call. This
is a host-reporting limitation, not evidence of a bad dump checksum.

A separate bounded batch of the same pinned managed GDB16.3_20250913 loaded the
officially extracted core ELF promptly. `gdb-probe001` and `thread-analysis001`
retain private output. Arguments were suppressed; no remote debugger or device
access occurred. Independent inspection confirmed26 task register notes with the
SDK's expected layout. The faulting task has the SDK's112-byte synthetic stack;
its exception cause is unavailable, and its original panic-frame pointer/stack
is not present. The saved TCB is readable, but its saved context does not recover
the rejected exception frame. A separate exact-ELF disassembly check verified
that the native panic wrapper preserves its incoming panic-info argument; this
does not recover the missing frame or identify the rejected stack predicate.

ESP-IDF deliberately substitutes this frame when its stack-sanity checks fail;
that substitution alone does not identify which predicate failed or prove stack
overflow. See the exact pinned
[ESP-IDF5.5.4 Xtensa core-dump implementation](https://raw.githubusercontent.com/espressif/esp-idf/v5.5.4/components/espcoredump/src/port/xtensa/core_dump_port.c).
Missing ROM symbols explained a different thread's delay frame, not the synthetic
fault frame. Other stacks show ordinary waits in transport, ASIC and USB owners;
none supplies the missing allocator/status call chain. The faulting task's static
name is the SDK default `pthread`, which cannot identify a particular Rust owner.

Offline outputs are also sealed, including the timed-out partial analysis:

| Root below `scratch/str005-share-crash/` | Inventory SHA-256                                                  |
| ---------------------------------------- | ------------------------------------------------------------------ |
| `inspection001`                          | `37e3464995a051853a1d1cff4b3052b93412558f8004d8f6ca40f86cbcbf3ef8` |
| `analysis001`                            | `4b158a07f2e5dc8d63d0b6306b1483f2a324893b378351ab5bec7acd55f94dbc` |
| `cutoff001`                              | `b2e88c7e6ab5fc8b8929d399dbed6f0215e788e235cc74303d842b97ca72fb02` |
| `gdb-probe001`                           | `5cd75962909710e55821b0769f272ef0acd510286ed783b90da26c584b2a6775` |
| `thread-analysis001`                     | `1aa942665408f4d27710e1d386cc01da7327b3fd2db39d4d8b678824c911bbb8` |
| `register-validity001`                   | `2c48cad5b0bc8101df098332415fffd4d8d656473a20f58644254393c79a2580` |
| `register-abi-gdb002`                    | `db6bf7be6b35d4ad705be30962a065cd39f2a2d9a725b5a22c63d87d3a208bac` |
| `panic-wrapper-abi001`                   | `b78713080842721aef0151b4379db9b779bf8641972211b0fe29eb3713019944` |

## Allocation lead and non-claims

A local diagnostic reported an8192-byte INTERNAL|8BIT allocation failure while
the global boot stage was runtime_ready. Its source hash identifies the full
firmware revision, not a call site. The callback discards its allocator-function
argument, records only the first failure across tasks, and also records
recoverable failures. It therefore does not prove fatal allocation failure,
which task requested it, or its causal connection to this panic.

V2 status construction snapshots/clones retained state and serializes a response;
that is a candidate for constrained-allocation testing. Concurrent internal-memory
allocations and the lost panic-frame boundary remain separate hypotheses. No
captured call chain selects one, so no speculative allocation, stack-size, timing
or safety-policy change was made.

## Stop and follow-up boundary

Outcome: `stop_hardware_blocker`; diagnostic boundary
`crashed_task_stack_unavailable_cause_unproven`. Recovery and acquisition succeeded;
accepted-share and heartbeat qualification remain unproved. The consumed recovery
and acquisition gates are disabled. No new Start, grant, renewal, self-test,
clearing, flashing, factory reset or parity promotion followed acquisition.

The next diagnostic needs bounded provenance before the failing frame is lost:
allocator identity/call location and task identity, plus panic-frame and stack-bound
validation facts, using fixed-size private diagnostic records. First reproduce
and test those software boundaries; then publish a separate observation/capture
contract before any new image installation or reproduction. Do not reuse this
core as proof of a specific panic cause or infer historical resource release.
