# Development core dumps and exact-image analysis

[ADR-0030](../adr/0030-development-core-dumps.md) is the standing authorization
for collecting, persisting and inspecting full private development dumps. This
guide is the command contract for the implementation. STR-005 acquisition remains
disabled until its independent recovery/effect prerequisites are satisfied.

## Diagnostic firmware

`just package` uses the canonical optimized firmware graph. It enables ESP-IDF
5.5.4 flash ELF dumps with SHA-256 checksums, internal DRAM capture, up to 64 tasks,
a dedicated 4096-byte dump stack, full 64-character application ELF identity and
first-dump preservation. Native console routing remains UART0 with no secondary
console; browser Web Serial retains its existing ownership.

Only the core-dump partition grows: offset `0xf12000`, size `0xee000` (952 KiB).
All NVS, application, filesystem and OTA-data locations remain unchanged.
Ordinary update segments continue to exclude the core-dump region. The build
checks the effective sdkconfig and generated binary table, rather than trusting
requested defaults. This configuration captures the memory supported by IDF;
external RAM is excluded from DRAM capture, and task/partition limits remain
finite. A truncated or missing dump is a failed observation.

The output directory includes the exact optimized firmware ELF with line-level
debug information, its `.map`, resolved `.sdkconfig` and `.debug.json` digest
sidecar. The packaged `bitaxe-ultra205.elf` has the same bytes as that debug ELF.
Keep the matching artifacts together. Rebuilding from the same source is not a
substitute for the ELF whose hash appears in the dump.

The first dump is preserved. A nonblank previous dump can prevent later captures;
collect and verify it before any separately authorized clearing operation. These
commands never erase a dump or reset accounting. Enabling core capture changes
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

`just core-dump-read` is reset-capable. Its implementation is available, but the
current task contains a disabled acquisition gate. Privacy authorization alone
does not activate it. Before a future invocation, its active task must satisfy
recovery/safety/evidence prerequisites and publish the exact command, identities,
evidence parent and allowed effects on clean pushed source. The sole active
`task-str005-start-panic-diagnosis` block must then contain:

```text
Development core-dump acquisition: enabled (recovery evidence prerequisite satisfied).
```

Do not add that declaration merely to bypass the unresolved prerequisite. The
command checks active/unique task admission before environment or device discovery
and checks clean pushed tooling plus an unmodified tracked task contract again
before acquisition.

Future admitted command shape:

```sh
just core-dump-read --board 205 --port <fresh-detector-port> --expected-physical-sha256 <fresh-physical-identity> --expected-installed-source <40-hex-commit> --expected-installed-elf <64-hex-elf-sha256> --private-root <absolute-new-private-child>
```

Run `just detect-ultra205` first under the published task contract and preserve
its output privately. Require exactly one known profile and fresh application
identity/baseline proof before browser release. Keep wrapper stdout/stderr as
separate mode-0600 siblings under the mode-0700 parent; leave the child absent.
The expected installed image identifies the application to return to, not an
inferred dump identity. Offline inspection independently checks the dump's ELF.

The acquisition owns one existing repository physical-device lease. It admits
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
retry. The receipt preserves the first failure and separate read/return/cleanup
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
