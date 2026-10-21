# Lessons: Firmware runtime

Scope: firmware code under `firmware/` and the crates it runs, ESP-IDF configuration, task stacks and heap, telemetry units and ranges, and safety-listed sources.

## lesson-esp-idf-main-task-runtime-capacity | 2026-07-19 10:31

1. Date: 2026-07-19
2. What went wrong: Host checks passed while the ESP-IDF main task lacked the runtime capacity required by the composed firmware startup and service stack.
3. Preventive rule: Treat the ESP-IDF main-task stack setting as an explicit runtime contract, keep one authoritative assignment, and regression-test its minimum capacity alongside the code paths that depend on it.
4. Trigger signal to catch it earlier: Firmware adds startup, parsing, service, or orchestration work without checking the configured main-task stack, or multiple stack assignments make the effective capacity ambiguous.

## lesson-bind-telemetry-ranges-to-operating-state | 2026-08-15 17:18

1. Date: 2026-08-15 17:18 CDT
2. What went wrong: The ADC evidence validator rejected fresh `0 mV` readings using an unconditional 400–2,000 mV range even though the same evidence workflow deliberately kept the ASIC rail disabled and the typed acquisition path treats a successful zero as fresh truth.
3. Preventive rule: Validate telemetry units against the producer's real wire domain, and bind narrower expected operating ranges to independently validated device state. Never apply an energized-state range to disabled-state evidence.
4. Trigger signal: A telemetry validator has a fixed positive lower bound but does not consume power, enable, mode, or lifecycle state, or its accepted range contradicts a typed zero-value regression in the producer.

## lesson-trace-legacy-wire-units-through-the-ui | 2026-08-16 04:02

1. Date: 2026-08-16
2. What went wrong: SI-typed internal INA260 values in volts and amps were serialized directly into legacy API fields whose upstream contract transports millivolts and milliamps, so type-safe internals still produced reference-incompatible wire values.
3. Preventive rule: Keep internal engineering units explicit, then verify every compatibility boundary from sensor conversion through API serialization, statistics history, and reference UI normalization before claiming parity.
4. Trigger signal: The reference UI divides an API field by 1,000, the reference driver documents milli-units, or an internal `*_volts`/`*_amps` field is assigned directly to an unqualified legacy wire name such as `voltage` or `current`.

## lesson-native-usb-and-wifi-share-internal-dma-heap | 2026-09-04 16:30

1. Date: 2026-09-04
2. What went wrong: Starting the optional TinyUSB Worker before Wi-Fi consumed and fragmented shared internal/DMA-capable heap, leaving the ESP32-S3 unable to satisfy a later 852-byte `DMA | 8BIT | INTERNAL` allocation; the allocator aborted directly and the board entered a panic reboot loop that looked like unstable USB enumeration. An attempted fix changed an unknown `TINYUSB_VENDOR_RX_BUFSIZE` symbol, and a later real TinyUSB stack reduction only moved the failure: after Wi-Fi became stable, the 12 KiB Worker pthread failed with `ENOMEM` because ordinary allocations had consumed the internal pool reserved by default at only 32 KiB.
3. Preventive rule: Treat USB, Wi-Fi, and forced-internal pthread stacks as one ordered budget. Allocate large required pthread stacks while internal memory is contiguous, defer optional USB installation until after Wi-Fi's fixed DMA resources, size the internal reserve from measured post-stage headroom plus the largest remaining forced-internal allocation, assert resolved Kconfig values at compile/build time, and retain stage-specific heap plus previous-boot failure evidence for field diagnosis.
4. Trigger signal to catch it earlier: An optional USB task starts before Wi-Fi, a forced-internal pthread is added without increasing or measuring the internal reserve, generated sdkconfig omits a requested setting, Worker enumerates briefly before repeated `panic` resets, an allocation receipt reports capability mask `0x0000080c`, or a deferred Worker spawn returns `ENOMEM` after Wi-Fi connects.

## lesson-diagnose-heap-loss-from-a-passive-series | 2026-10-06 18:00

1. Date: 2026-10-06
2. What went wrong: Two internal-heap readings, 9,687 and 607 bytes free, suggested an idle leak, but both were sampled during Worker connections. A once-a-minute idle sample read passively over serial showed persistent exhaustion with periodic 6 KB dips, not a leak. Separately, the retained `allocation_failure` row is the boot's first failed allocation, which was not necessarily the one that caused the panic.
3. Preventive rule: Before attributing internal-heap loss to a leak or to idle time, capture a passive time series (`internal_heap_sample` via receive-only `just monitor`, judged with `just internal-heap-series`) both idle and after the suspect session, without opening the connection under test. Treat a retained allocation-failure row as the first failure of that boot, not as the panic cause. This extends `lesson-native-usb-and-wifi-share-internal-dma-heap`.
4. Trigger signal to catch it earlier: A leak hypothesis rests on samples taken inside connections, the largest free internal block is below the size of the next required allocation, or a panic is explained by the retained allocation-failure row alone.

## lesson-update-the-host-mirror-with-the-production-adapter | 2026-10-17

1. Date: 2026-10-17
2. What went wrong: Adding a field to the production session adapter broke `//firmware/bitaxe:production_worker_admission_tests`, whose host test mirrors the adapter struct and compiles the real `production_mining_session/bwg.rs`.
3. Preventive rule: After changing `OrdinaryEspProductionSessionAdapter` fields or what `bwg.rs` reads, update the host mirror (`firmware/bitaxe/src/production_worker_admission_host_test.rs` and its submodules) and run that target before the full gate.
4. Trigger signal: A `self.<field>` is added in `bwg.rs` or the adapter struct.

## lesson-keep-safe10-pinned-fragments-byte-identical | 2026-10-17

1. Date: 2026-10-17
2. What went wrong: Rewriting the readiness safety expression failed `//tools/automation:automation_test` because `tools/automation/src/safe10-source-inventory.ts` pins exact source fragments of SAFE-10 files.
3. Preventive rule: Before editing a file listed in `safe10ProductionFragments`, keep each pinned fragment byte-identical and add new behavior as a separate term.
4. Trigger signal: Editing `readiness.rs`, `recovery_policy.rs`, `cooling.rs`, `mining_actuation*.rs` or another SAFE-10-listed file.
