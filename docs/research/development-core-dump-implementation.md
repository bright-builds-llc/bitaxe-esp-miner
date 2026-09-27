# Development core-dump implementation verification

Date: 2026-09-27. Task: `task-str005-start-panic-diagnosis`.
Outcome: **software and native build verified; no hardware capture claimed**.

The owner's standing authorization is persisted in repo `AGENTS.md`, the evidence
policy and [ADR-0030](../adr/0030-development-core-dumps.md). Current and future
development may collect, persist and inspect full private dumps without another
per-dump approval. The [command guide](../hardware/development-core-dumps.md)
documents private storage and the separate effect/publication controls.

## Implemented behavior

- ESP-IDF flash ELF core dumps with SHA-256, full ELF identity, internal DRAM and
  task capture, dedicated stack and first-dump preservation.
- Core storage expanded to 952 KiB entirely within the reserved flash tail;
  existing NVS/application/filesystem/OTA-data locations remain unchanged.
- Exact optimized debug ELF, linker map, resolved configuration and digest
  sidecar retained as declared canonical outputs.
- Private offline `inspect` and `analyze`, pinned vendor decoder and managed GDB,
  checksum/full-identity enforcement, and bounded process-group cleanup.
- Task-gated `core-dump-read` using the existing physical lease, validated actual
  partition table, ROM admission, guarded return and unconditional release.

The IDF 5.5.4 identity descriptor is 72 bytes, including a 66-byte string field
and ABI padding. The inspector checks all 64 hash characters and the terminating
NUL without treating spare/padding bytes as identity. Tests compile the actual
layout assertion and check the pinned source, rather than relying solely on a
self-consistent synthetic serializer.

The larger native capture support initially exceeded the unchanged 4 MiB image
slot. Applying the existing size-optimization policy to `bitaxe-api` restored
headroom. This changes generated runtime code and cannot inherit old hardware
continuity or safety evidence automatically.

## Clean published candidate

The candidate was built after implementation source was clean and pushed.

| Item                             | Verified value                                                     |
| -------------------------------- | ------------------------------------------------------------------ |
| Source commit                    | `8f69978b2a413173cba2ad7edf77e0f67e8c56d1`                         |
| ELF SHA-256                      | `f44a3927696fb1fb5870bed460e3cc1f915834218dd10a6b5587810603f751a1` |
| Package manifest SHA-256         | `c5061b5ef8c57eb603d32d7b0db2b9bdc638b71e51e374d4e2d916e68cb0264a` |
| Debug sidecar SHA-256            | `5d87fcd7b3ea0c43f89a1476eb101488c5f8568f0caad1dc394d3b7fc47a7424` |
| Retained local inventory SHA-256 | `03ce447a282dab6b5879789bee519d6ec2452192270ad9dad1377206f9fc4adb` |
| OTA image size                   | 4,148,416 bytes                                                    |
| Remaining 4 MiB slot capacity    | 45,888 bytes                                                       |
| Core-dump partition              | offset `0xf12000`, size `0xee000`, flags zero                      |
| Managed decoder / GDB            | esp-coredump 1.17.2 / esp-gdb 16.3_20250913                        |

ELF/map/sdkconfig bytes match their sidecar hashes, and the packaged ELF matches
the debug ELF. The complete package artifacts and debug files were copied into a
protected ignored local snapshot and hash-checked. Source-line resolution of the
known `bitaxe_production_owner_entry` symbol returned
`production_mining_session/owner_loop.rs:8`. That is a symbolization test, not an
observed fault address or a diagnosis of Share002.

## Verification

- 25 local integration tests passed against the real pinned vendor parser and
  process/GDB boundaries, including checksum/identity failures, native ABI,
  unsafe paths, private output, timeout and descendant cleanup.
- 10 acquisition tests passed, including production read failure followed by
  return/release, failed return, identity mismatch and inactive task rejection.
- Canonical automation, flash and partition suites passed, including TypeScript
  compilation, effective-config validation and the process environment boundary.
- Ordered Cargo format, Clippy, build and tests passed: 2,377 passed, three
  existing ignores.
- Clean native packaging, fixed USB ownership/symbol checks, reference,
  redaction, Bright Builds, Markdown and diff checks passed.

The synthetic core successfully exercises container inspection. Its deliberate
lack of crashed-task registers makes full GDB analysis fail after reaching the
vendor register/thread stages; the test verifies that this failure and its output
remain private. It is not evidence of a successful real-device postmortem.

## Remaining limits

No device was opened, reset or flashed, and no real core dump was collected.
Acquisition remains disabled by the current task's independent recovery gate.
Core dumping changes fatal-handler execution time and resource use; compilation
does not establish shutdown timing, real retention, capture completeness or a
hardware-safe reproduction. Internal DRAM/task limits and external-RAM exclusions
remain explicit. Dump erasure is not automatic.

The Share002 and recovery004 sealed inventories were reverified unchanged.
Share002's panic cause and historical resource-proof gap remain unresolved. The
diagnosis task stays unarchived and parity remains 90/95. No mining, grant
issuance, reset, flashing, historical replay or parity promotion was performed.
