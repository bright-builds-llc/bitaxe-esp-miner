# ADR-0039: Rust atomics avoid S32C1I

## Status

Accepted on 2026-10-20 as the root-cause fix of `task-handover-panic-diagnosis`,
under the owner's standing rule for fixes after a root-cause analysis.

## Context

On the ESP32-S3, the `S32C1I` compare-and-set instruction is not atomic on
external RAM (PSRAM). ESP-IDF handles this for C:
- `CONFIG_STDATOMIC_S32C1I_SPIRAM_WORKAROUND` defaults on for an ESP32-S3
  with PSRAM, and compiles every component with
  `-mdisable-hardware-atomics`;
- its atomic libcalls (`__atomic_*`, `__sync_*`, in IRAM by default) use the
  instruction only for internal RAM, and take a lock for PSRAM addresses;
- `esp_cpu_compare_and_set` does the same.

Rust code is built by rustc, which ignores that flag. On this target it
inlines `s32c1i` for every atomic. Since `92f58abb` (2026-10-05),
`CONFIG_SPIRAM_MALLOC_ALWAYSINTERNAL=0` has placed ordinary Rust heap
allocations in PSRAM, and some task stacks live there too. So every `Arc`
count, channel, and `std::sync::Mutex` lazy initialization could race
between the two cores.

The core dump of boot-resume check attempt 001 (firmware `1dfca479`) is
consistent with that race, and no other explanation fits it. The race
itself is inferred, because the dump holds task stacks, not the PSRAM
slot:
- `POST /api/system/restart` hands its effect to a worker task through a
  one-shot reply slot allocated in PSRAM;
- httpd and the worker both initialized the slot's `OnceBox<pal::Mutex>`
  at once, and both compare-exchanges "won";
- each task locked its own mutex (`uxMutexesHeld` was 1 for both);
- `unlock` re-reads the `OnceBox`, so the worker unlocked httpd's mutex,
  and FreeRTOS asserted in `xTaskPriorityDisinherit`
  (`pxTCB == pxCurrentTCBs[core]`).
In the image, `OnceBox::initialize` performs that compare-exchange with a
raw `wsr.scompare1`/`s32c1i` pair, as do 362 other Rust functions,
including one exported under a C name.

## Decision

Build Rust firmware code without the `S32C1I` feature
(`-C target-feature=-s32c1i` for `xtensa-esp32s3-espidf` in
`.cargo/config.toml`), so every Rust atomic lowers to ESP-IDF's
PSRAM-aware libcalls, as ESP-IDF's C does. Rust keeps every atomic type
(`target_has_atomic` is unchanged).

A device ELF may perform a hardware compare-exchange (`wsr.scompare1`
followed by `s32c1i`) only in an exact allowlist of native functions:
- ESP-IDF's PSRAM-aware primitives and libcall bodies;
- the coprocessor exception handlers;
- the firmware's C fault recorders, which their own audit covers.
`bazel test //scripts:firmware_psram_atomics_test`, part of `just test`,
checks the built firmware. `just audit-psram-atomics <elf> <output>`
checks any other image. The crashing image fails with 363 functions; the
fix passes with none.

## Consequences

- Each Rust atomic, plain loads and stores included, becomes a call. One
  to a PSRAM address masks interrupts and takes a single spinlock shared
  by both cores. This is the cost ESP-IDF already pays in C. It is
  measured on hardware (hashrate and response times) before it is called
  negligible.
- A panic path must not touch a PSRAM atomic: the stalled other core may
  hold that spinlock. The current panic and core-dump wrappers touch only
  internal and RTC memory.
- A `RUSTFLAGS` or `CARGO_ENCODED_RUSTFLAGS` environment variable would
  replace the configured flag. The firmware test catches that for the
  Bazel build; an ELF built any other way needs `just audit-psram-atomics`.
- The memory policy of `92f58abb` stays. Allocation placement no longer
  affects whether atomics are correct.
- Hand-written `s32c1i` in C (the allocation-failure recorder) is unchanged
  and stays audited by the fault-provenance audit.
- Earlier evidence that relied on inline Rust `s32c1i`, such as the
  2026-09-15 USB cadence native audit, describes the older images only.
