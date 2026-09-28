# Development core dumps and exact-image analysis

[ADR-0030](../adr/0030-development-core-dumps.md) is the standing authorization
for collecting, persisting and inspecting full private development dumps. This
guide is the command contract for the implementation. [ADR-0031](../adr/0031-prospective-panic-diagnostics.md) and the
[staged panic probe](str005-panic-probe.md) admit prospective acquisition only
with a fresh current-state recovery proof and the published effect contract.

## Diagnostic firmware

`just package` uses the canonical optimized firmware graph. It enables ESP-IDF
5.5.4 flash ELF dumps with SHA-256 checksums, eligible task stacks/registers and
explicitly selected diagnostic memory, up to 64 tasks,
a dedicated 4096-byte dump stack, full 64-character application ELF identity and
first-dump preservation. Native console routing remains UART0 with no secondary
console; browser Web Serial retains its existing ownership.

The core-dump partition remains at offset `0xf12000`, size `0xee000` (952 KiB).
All NVS, application, filesystem and OTA-data locations remain unchanged.
Ordinary update segments continue to exclude the core-dump region. The build
checks the effective sdkconfig and generated binary table, rather than trusting
requested defaults. After measured capacity rejection in installation006, the
current profile disables `CONFIG_ESP_COREDUMP_CAPTURE_DRAM`: bulk heap/PSRAM and
ordinary DRAM data capture are excluded. The 28-byte panic-cutoff receipt uses the
official `.dram2.coredump.*` selected-user region and must be proven present by
native audit and actual dump decoding. TCBs, registers and eligible task stacks
remain captured under the SDK's task-count and sanity limits.

The earlier full-DRAM profile walked used blocks from all 8-bit heaps, including
PSRAM, and requested more space than the partition. Full private development
capture remains authorized; the current bounded profile does not claim complete
physical memory. A truncated or missing dump is still a failed observation.

The output directory includes the exact optimized firmware ELF with line-level
debug information, its `.map`, resolved `.sdkconfig` and `.debug.json` digest
sidecar. The packaged `bitaxe-ultra205.elf` has the same bytes as that debug ELF.
Keep the matching artifacts together. Rebuilding from the same source is not a
substitute for the ELF whose hash appears in the dump.

The first dump is preserved. A nonblank previous dump can prevent later captures;
collect and verify it before any separately authorized clearing operation. The read and offline
commands never erase a dump or reset accounting. Explicit `core-dump-clear` is
a separate effect requiring a verified private archive and fresh recovery proof. Enabling core capture changes
native code, RAM use and fatal-handler timing; the build proves compilation and
resource limits, not the device's safety or retention behavior under a real fault.
Mining/fault qualification must separately account for that timing before use.

## Offline commands

Prepare the official debugger selected by the pinned IDF tools manifest:

```sh
just core-dump-tools
```

The command uses the existing managed IDF Python environment and installs only
`xtensa-esp-elf-gdb` through `idf_tools.py`. It does not open a device. The decoder
requires managed `esp-coredump` 1.17.2 and the exact recommended GDB version.
Missing dependencies fail explicitly; analysis never falls back to reading flash.

Store the raw dump as mode 0600 below a mode-0700 gitignored development parent.
Use absolute paths and a nonexistent output child under another protected ignored
parent. The ELF may come from retained canonical build artifacts.
Resolve Bazel output aliases with `realpath`, or use a byte-identical retained
regular-file copy; the inspector rejects symlink components and hardlinked input.

```sh
just core-dump inspect --dump <absolute-private-raw-dump> --elf <absolute-matching-elf> --elf-sha256 <full-64-hex-sha256> --private-root <absolute-new-private-child>
just core-dump analyze --dump <absolute-private-raw-dump> --elf <absolute-matching-elf> --elf-sha256 <full-64-hex-sha256> --private-root <another-absolute-new-private-child>
```

`inspect` validates the vendor raw format/checksum and requires exactly one full
matching ELF identity note. Prefix-only matches, missing notes, duplicates and
wrong ELFs fail before GDB. `analyze` performs that inspection first, then invokes
the official decoder with explicit `--core` and `--core-format raw`; GDB startup
files are disabled. It records debugger identity and sends all vendor analysis,
including registers, task names and memory-derived data, to protected files.

The CLI prints only closed status/digests. Private copies, inputs, decoder output
and temporary ELF/memory files remain in the output child, including on failure.
It rejects existing outputs, symlink/hardlink inputs and unsafe permissions.
Each decoder process group is bounded to 120 seconds; interruption/timeout kills
the group and retains partial artifacts. Cleanup never deletes the original.
Local inspection is authorized; sharing raw artifacts still requires explicit
permission. Redacted conclusions remain subject to `just verify-redaction`.

Run `just test-core-dump` for the local integration suite using the managed
Espressif decoder. Synthetic fixtures contain no device memory. Missing tooling
fails the suite rather than silently skipping the real decoder boundary.

## Device acquisition contract

`just core-dump-read` is reset-capable. Privacy authorization alone does not
activate it. Before invocation, its active task must satisfy
recovery/safety/evidence prerequisites and publish the exact command, identities,
evidence parent and allowed effects on clean pushed source. The sole active
`task-str005-start-panic-diagnosis` block must then contain:

```text
Development core-dump acquisition: enabled (recovery evidence prerequisite satisfied).
```

The prospective prerequisite is ADR-0031 current-state proof, not a rewrite of
Share002 historical evidence. The
command checks active/unique task admission before environment or device discovery
and checks clean pushed tooling plus an unmodified tracked task contract again
before acquisition.

Admitted command shape:

```sh
just core-dump-read --board 205 --port <fresh-detector-port> --expected-physical-sha256 <fresh-physical-identity> --expected-installed-source <40-hex-commit> --expected-installed-elf <64-hex-elf-sha256> --recovery-proof <absolute-fresh-current-proof> --private-root <absolute-new-private-child>
```

Run `just detect-ultra205` first under the published task contract and preserve
its output privately. Require exactly one known profile and fresh application
identity/baseline proof before browser release. Keep wrapper stdout/stderr as
separate mode-0600 siblings under the mode-0700 parent; leave the child absent.
The expected installed image identifies the application to return to, not an
inferred dump identity. Offline inspection independently checks the dump's ELF.

The recovery proof is protected, exact-image/device-bound, nonpending and at
most 120 seconds old. Future timestamps fail. The acquisition owns one existing
repository physical-device lease. It admits
ESP32-S3 ROM with bounded board-info before reading. It reads the actual partition
table at `0x8000`, validates its integrity and nonoverlapping ranges, and reads
only the unique unencrypted core-dump partition at `0xf12000`. Historical 64 KiB
and current 952 KiB layouts are supported; other ranges, flags and ambiguous
tables fail closed. Read commands use the session-owned port with no additional
reset and never erase or write flash.

After successful ROM admission, success or read failure attempts the existing
guarded application return on the same device, then checks the expected runtime
identity. Resource release runs even when return fails. A physical mismatch
prohibits further reads/reset; failed ROM admission does not trigger a blind reset
retry. Admission can fail after a reset: the device may remain in ROM, restoration
is unproven, and further effects stop until bounded same-device recovery is admitted. The receipt preserves the first failure and separate read/return/cleanup
outcomes. Application identity is not a hardware-baseline proof: a future effect
owner must additionally obtain fresh authenticated safe-baseline confirmation.

Bounds are inherited from the owner: board-info and return operations use their
existing finite limits, each read is at most 360 seconds, and runtime observation
is bounded. Keep the command timeout larger than the composed operation budget.
One invocation per new evidence child, no implicit retry, reset-loop capture,
fault injection, Start, grants, mining, installation or factory operation.
Stop on identity drift, invalid data, missing return/baseline proof or incomplete
cleanup. Continuation requires a verified boundary fix and a fresh contract/root.

## Completion and historical limits

Software verification can establish that the firmware contains the capture path,
debug artifacts are usable and the host tools reject unsafe or mismatched input.
It cannot manufacture Share002's missing panic bytes or retained-resource record.
A future real crash must produce a valid, fully bound dump before a root cause
or capture success can be claimed. Preserve all older seals and parity at 90/95.

## Captured cutoff verification and explicit clearing

`just core-dump verify-cutoff` accepts the same arguments as `inspect`. It checks
the native checksum and full ELF identity, then reads the cutoff receipt only
from captured core PT_LOAD bytes at the matching ELF symbol address. Safe pin
configuration/output state and actual generation revocation are required. An
off-only self-test additionally requires `self_test_marked: true`. Static image
contents and default BSS cannot supply missing captured state.

`just core-dump-clear` uses the read arguments plus `--preserved-dump` and
`--preserved-sha256`. The private full-partition archive must match a fresh
read byte-for-byte before erasure. It re-admits the held device without reset,
erases only the validated core partition and reads back all erased bytes. The
active task must explicitly enable clearing; no retry or NVS/accounting reset is
implicit. See the staged probe contract for ordered commands and failure cleanup.

## Current hardware qualification result

The first controlled off-only panic produced no stored dump: the entire 952 KiB
region remained erased. The [sealed outcome](../parity/evidence/20260927-str005-core-self-test-no-dump.md)
records recovery and the remaining blocker. Current task effect gates are
disabled; command examples above describe implementation interfaces, not new
admission. Full private development capture remains authorized by ADR-0030.

The pinned [ELF heap writer](https://github.com/espressif/esp-idf/blob/v5.5.4/components/espcoredump/src/core_dump_elf.c)
captures used blocks from all 8-bit heaps. Its
[flash writer](https://github.com/espressif/esp-idf/blob/v5.5.4/components/espcoredump/src/core_dump_flash.c)
rejects excess length before erasing. Retain the actual store result and required
length before changing scope or repeating the test. Empty flash does not prove
which preflight check failed.
