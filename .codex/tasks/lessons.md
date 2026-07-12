## lesson-gsd-frontmatter-body-separators | 2026-06-28 14:14

1. Date: 2026-06-28
2. What went wrong: A GSD summary used standalone `---` body separators after YAML frontmatter. The GSD frontmatter parser scans all `--- ... ---` blocks and selected the last body pair, so lifecycle validation ignored the real frontmatter and failed.
3. Preventive rule: In GSD artifacts and other frontmatter-parsed Markdown, use standalone `---` only for the opening and closing YAML frontmatter delimiters at the top of the file. Use headings or `***` for body breaks instead. Markdown table separator rows such as `| --- |` remain valid.
4. Trigger signal to catch it earlier: Lifecycle validation reports missing frontmatter fields even though the file visibly has them near the top, or a Markdown artifact has more than two standalone `---` lines.

## lesson-esp-idf-service-ownership-and-redaction | 2026-07-02 23:29

1. Date: 2026-07-02
2. What went wrong: Wi-Fi startup initialized the default ESP-IDF event loop through raw `esp_event_loop_create_default()` before `EspSystemEventLoop::take()`, so esp-idf-svc's ownership tracker returned `ESP_ERR_INVALID_STATE`. The first hardware evidence run also showed that ESP-IDF Wi-Fi driver logs can expose the connected SSID outside JSON or `key=value` fields.
3. Preventive rule: Let esp-idf-svc own managed ESP-IDF service handles such as the default event loop; use raw idempotent init only for services without a wrapper ownership tracker. Redaction tests must include vendor log formats, not only project log formats.
4. Trigger signal to catch it earlier: A managed `take()` API fails with `ESP_ERR_INVALID_STATE` immediately after a raw init call, or sanitized serial evidence still contains natural-language Wi-Fi driver lines such as `wifi:connected with ...`.

## lesson-opaque-handoff-before-fallible-validation | 2026-07-11 14:55

1. Date: 2026-07-11
2. What went wrong: A fresh exact-head attempt created a one-time opaque resume handle, then a fallible handoff assertion rejected the otherwise valid checkpoint before the handle reached the operator. The live private attempt could no longer be addressed through the normal handle-only cleanup path.
3. Preventive rule: Emit or durably escrow a one-time public locator before running fallible post-construction assertions. Provide a narrowly guarded, effect-free cleanup path for a uniquely identifiable pristine orphan without reconstructing or exposing the clear locator.
4. Trigger signal to catch it earlier: A command creates a private active record or capability and then performs validation, formatting, or output transformation before returning its only public locator.

## lesson-cross-process-tests-use-real-boundaries | 2026-07-11 14:55

1. Date: 2026-07-11
2. What went wrong: In-process fixtures passed while the first real lifecycle continuation failed because a Unix-socket receiver mixed buffered line input with unbuffered payload reads. Other live-only failures involved process-group descendants and fresh-process capability parsing.
3. Preventive rule: Test IPC, process ownership, framing, and capabilities through real fresh processes, Unix sockets, coalesced and fragmented writes, process groups, and mode-enforced files. Keep pure tests too, but do not let them substitute for the operating-system boundary that production uses.
4. Trigger signal to catch it earlier: A test injects a function or prebuilt object where production crosses a socket, process, PTY, file-permission, or process-group boundary.

## lesson-espflash-no-reset-is-not-passive | 2026-07-11 14:55

1. Date: 2026-07-11
2. What went wrong: The retained-runtime capture treated `espflash monitor --no-reset` as a passive serial open. In espflash 4.0.1 that flag suppresses the monitor's final application reset, but the default connection still drives reset lines, synchronizes with the bootloader, and may load the flasher stub.
3. Preventive rule: A passive ESP32-S3 monitor must use all three controls together: `--before no-reset-no-sync --after no-reset --no-reset`, with `--chip esp32s3`. Treat bare `--no-reset` as a reset-capable and bootloader-affecting command.
4. Trigger signal to catch it earlier: Any retained-runtime or no-flash capture renders `espflash monitor` with `--no-reset` but omits either explicit `--before no-reset-no-sync` or `--after no-reset`.

## lesson-power-and-usb-session-are-distinct | 2026-07-11 14:55

1. Date: 2026-07-11
2. What went wrong: USB replug, barrel-power retention, both-power cold start, and warm reset were sometimes discussed as interchangeable recovery actions even though they preserve different MCU and USB-peripheral state.
3. Preventive rule: Record barrel/DC state and USB state independently, plus the USB enumeration epoch. Label every action as a USB re-enumeration, warm reset, or true both-power cold start; never infer one from another.
4. Trigger signal to catch it earlier: A hardware checkpoint says only `replug`, `power-cycle`, or `reset` without naming both power paths and the expected USB-session transition.

## lesson-native-usb-capture-needs-prearmed-observation-or-replay | 2026-07-12 04:00

1. Date: 2026-07-12
2. What went wrong: A lifecycle waited for the operator to report barrel-then-USB restoration before opening the native USB monitor. The ESP32-S3 booted from barrel power before the serial node existed, so correct later ownership still captured zero early boot/listener markers.
3. Preventive rule: For native-USB cold-start evidence, arm the exact-node watcher before instructing physical restoration and start passive ownership automatically on node appearance. When the transport cannot preserve pre-enumeration bytes, validate replayable, session-tagged application proof instead of relying on an arbitrary countdown or post-plug acknowledgment.
4. Trigger signal to catch it earlier: A test requires early boot bytes from a serial device whose node is created only after power-up, or asks the operator to confirm plugging before the monitor process begins waiting.
