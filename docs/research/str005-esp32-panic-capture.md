# STR-005: ESP32-S3 panic diagnosis and capture research

Date: 2026-09-27. Scope: software-only research for
`task-str005-start-panic-diagnosis`. No device access occurred for this research.
This is guidance for prospective instrumentation, not an execution contract.

## Finding and immediate recommendation

Prefer an exact-ELF offline investigation followed by small, typed, reset-retained
failure records. Add capture through the existing browser serial owner only after
reviewing console routing and protocol framing. Do not start another monitor,
replay Share002, issue a grant, Start mining, flash, or change device state to obtain
a better trace under this document.

The [Share002 failure report](../parity/evidence/20260927-str005-v2-share-start-unverified.md)
shows a panic reset without a decoded cause. Its startup heap observation and
corrupt preparation receipt do not identify OOM or the failing preparation stage.
[ADR-0029](../adr/0029-piecewise-str005-qualification.md) permits static diagnosis
while recovery remains unresolved. Preserve its seals and the 90/95 parity result.
A prospective capture improvement cannot reconstruct missing historical bytes.

## What native USB can actually capture

ESP32-S3 USB Serial/JTAG is fixed-function hardware, distinct from USB OTG/TinyUSB.
Its console has limited buffering and may stop delivering during sleep. The
nonblocking console flushes on newline; the interrupt-driven driver flushes when
its transmit buffer empties. These facts support prearming the existing reader,
but do not guarantee delivery after late attachment.
[Espressif USB Serial/JTAG, v5.5.4](https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/usb-serial-jtag-console.html)

The actual IDF panic printer is more decisive than normal logging settings:
`panic_print_char_usb_serial_jtag` is compiled only with
`CONFIG_ESP_CONSOLE_USB_SERIAL_JTAG` or
`CONFIG_ESP_CONSOLE_SECONDARY_USB_SERIAL_JTAG`. It polls the hardware FIFO, waits
up to its bounded no-host timeout, and does not write a character when the FIFO
remains unavailable. Native panic printing bypasses ordinary stdio locks. A late
reader cannot recover dropped bytes. The same source prints an ELF digest and
sets panic reset hints for both explicit aborts and faults; the reset category
alone cannot distinguish them.
[IDF v5.5.4 panic.c](https://github.com/espressif/esp-idf/blob/v5.5.4/components/esp_system/panic.c)

Repository defaults currently select UART console, no secondary console, and
print/reboot. The build script enforces the no-secondary choice. The [installed-image audit](str005-start-panic-diagnosis.md) confirms those
settings in Share002's sealed resolved sdkconfig and verifies its ELF identity. Merely connecting a native USB reader does not
route an otherwise UART-only panic to it.
[Local defaults](../../firmware/bitaxe/sdkconfig.defaults),
[build checks](../../firmware/bitaxe/build.rs)

**Design implication:** enabling a secondary console indiscriminately could mix
unframed vendor logs or panic text with authenticated binary control traffic.
Any future change needs byte-level parser tests, secret filtering before writes,
exclusive ownership, bounded receive behavior, and continuity reassessment.
Treat recognizable native panic text as diagnostic data, never authority.

## Rust panic versus IDF exception

The project-patched `esp-idf-sys` panic handler at `f616563a` calls `abort`, but only when `std` is disabled
and its `panic_handler` feature is enabled. It is not the explanation for this
project's std panic-hook behavior.
[esp-idf-sys pinned panic handler](https://github.com/esp-rs/esp-idf-sys/blob/f616563a87595032f06f1fec95b6816b1c11135c/src/panic.rs)

Rust's std hook executes before both aborting and unwinding panic runtimes; its
default writes a message to stderr. The local hook saves a location receipt and
then chains the previous hook. Audit that retained write separately from later
stderr delivery. Native faults and native allocator aborts need separate records;
a Rust hook alone does not establish their cause. Avoid retaining arbitrary panic
payloads, which may contain runtime inputs.
[Rust std hook documentation](https://doc.rust-lang.org/std/panic/fn.set_hook.html),
[local hook](../../firmware/bitaxe/src/panic_evidence.rs)

## Offline decoding before more effects

Retain the original package ELF, map, resolved sdkconfig, compiler identity, source
commit and artifact hashes. Match the installed artifact before resolving any
captured program counters; a newly rebuilt ELF can have different addresses.
IDF Monitor uses the target `addr2line` with the application ELF and can also use
Espressif's matching ROM ELF for ROM frames. The equivalent decoder can run
offline on already admitted addresses without opening a serial port.
[IDF Monitor v5.5.4](https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/tools/idf-monitor.html)

Espflash 4.5.0's monitor accepts an ELF and resolves serial addresses. Its monitor
also has reset behavior and keyboard control; it is not an independent passive
observer to launch while the browser owns control. Use the repository ownership
rules and offline symbolization here, not a second live monitor.
[espflash 4.5.0 monitor implementation](https://github.com/esp-rs/espflash/blob/v4.5.0/espflash/src/cli/monitor/mod.rs)

A valid exception report can distinguish invalid load/store addresses, stack
watchpoints, watchdogs and explicit aborts. A stack watchpoint is useful but cannot
detect every overflow. IDF's compiler stack protector concerns the corresponding
C/C++ compilation; do not assume it instruments Rust. GDB panic mode cannot
resume execution or set normal breakpoints.
[Espressif fatal errors v5.5.4](https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/fatal-errors.html)

## Capture options and tradeoffs

| Option                                       | Diagnostic value                                                                                                  | Constraint and recommendation                                                                                                                                                                                             |
| -------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Typed retained receipt                       | Rust location fingerprint, preparation stage, allocator failure category, build identity and validity after reset | First choice. Fixed-size, allocation-free updates; bind records to boot/build and explicit validity. Test torn/corrupt records and first-failure preservation. Does not prove a native exception without native evidence. |
| Existing browser owner captures native panic | Exception class and address trace can locate failure in exact ELF                                                 | Prospective only. Requires verified native panic routing, bounded demultiplexing, privacy and continuity tests. Transport may lose bytes.                                                                                 |
| Flash core dump                              | Postmortem task stacks survive missing live capture                                                               | Not currently suitable: changes partition/write contract and records uncontrolled memory.                                                                                                                                 |
| Console core dump                            | Can avoid a flash partition                                                                                       | Large raw memory stream, framing and privacy problems; not an allowed log format.                                                                                                                                         |
| GDB stub or native USB JTAG                  | Registers and halted state                                                                                        | Deferred. Halt changes safety behavior and debugger ownership. Not permitted by the current software-only scope.                                                                                                          |

Core dumps include task control blocks and stacks; enabling DRAM capture adds
heap and data sections. Flash dumps require a dedicated partition in a custom
layout, and capture itself needs stack space. Even without full DRAM capture,
stacks may contain credentials. Under this repository's `NeverPersistRaw` policy,
writing a dump into a private file and redacting later is unacceptable. The
preferred alternative is fixed-field metadata that excludes secret bytes by
construction. Do not add a coredump partition or invoke retrieval under this task.
[Espressif core dump v5.5.4](https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/core_dump.html),
[evidence policy](../parity/evidence-policy.md)

IDF's panic GDB path disables watchdogs before entering the stub; runtime GDB
support also takes console input ownership. Consequently, debugger attachment is
not equivalent to harmless observation of a live safety-controlled worker. Its
availability in generic vendor documentation does not satisfy this project's
heartbeat, shutdown or exclusive-owner contracts.
[IDF panic GDB dispatch](https://github.com/espressif/esp-idf/blob/v5.5.4/components/esp_system/panic.c),
[GDB stub Kconfig](https://github.com/espressif/esp-idf/blob/v5.5.4/components/esp_gdbstub/Kconfig)

## Heap and stack instrumentation

Record capability-specific free bytes, largest block and minimum free size at
narrow allocation/preparation boundaries. Total free bytes alone cannot show
fragmentation or availability of a required internal/DMA allocation. Allocation
hooks must avoid allocation and blocking work. Light heap poisoning adds metadata;
comprehensive poisoning adds substantial runtime cost. Use targeted integrity
checks to narrow a corruption interval only after considering their timing cost.
A detected heap corruption site can be later than the corrupting write.
[Espressif heap debugging v5.5.4](https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/system/heap_debug.html)

**Recommendation:** first inspect exact optimized stack frames and existing receipt
validity; then choose the smallest discriminating instrumentation. Do not increase
all stacks or reserve more internal heap based only on a startup reading. Keep
watchdog deadlines intact, measure observer overhead, and regress the failed
boundary. Broad tracing, whole-program sanitizers and verbose logging may move a
memory/timing failure instead of explaining it.

## Bitaxe-specific evidence and limits

The pinned upstream ESP-Miner config targets ESP32-S3, enables PSRAM and external
allocation options, and sets an 8192-byte main stack. Its minimal config has no
explicit panic/core-dump override. This is reference configuration evidence, not
proof that the Rust worker has the same stack, allocation lifetimes or crash.
Absence from a defaults file is not proof that the generated configuration disables
a feature.
[Upstream pinned defaults c1915b0](https://github.com/bitaxeorg/ESP-Miner/blob/c1915b0a63bfabebdb95a515cedfee05146c1d50/sdkconfig.defaults)

No public issue was used as causal evidence. The recommendations above use official
versioned IDF docs/source, esp-rs source, Rust documentation, and pinned Bitaxe
source. None identifies Share002's root cause without matching local evidence.

## Verification and guidance applied

This note follows repo `AGENTS.md`, its Bright Builds sidecar/overrides, standards
index, verification and operability standards, ADR-0029, the Share002 report and
evidence privacy policy. Priority active lessons loaded included panic/runtime
capacity, USB ownership and late capture, redaction, direct-interface authority,
and progress-gated retries, plus all global lessons. The combined active lesson
set exceeded the startup budget; other repository lesson blocks were not loaded
by this focused research worker. Parent orchestration owns the complete loading
inventory and lesson-audit disposition.

Primary URLs were opened during research. No capture capability, hardware fix,
retained historical resource proof, or parity improvement is claimed.
