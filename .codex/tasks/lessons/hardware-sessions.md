# Lessons: Hardware sessions

Scope: flashing, monitoring, USB and serial transport, device sessions, boots and resets, core dumps, retained device logs, and hardware attempts on the Ultra 205.

## lesson-native-usb-capture-needs-prearmed-observation-or-replay | 2026-07-12 04:00

1. Date: 2026-07-12
2. What went wrong: A lifecycle waited for the operator to report barrel-then-USB restoration before opening the native USB monitor. The ESP32-S3 booted from barrel power before the serial node existed, so correct later ownership still captured zero early boot/listener markers.
3. Preventive rule: For native-USB cold-start evidence, arm the exact-node watcher before instructing physical restoration and start passive ownership automatically on node appearance. When the transport cannot preserve pre-enumeration bytes, validate replayable, session-tagged application proof instead of relying on an arbitrary countdown or post-plug acknowledgment.
4. Trigger signal to catch it earlier: A test requires early boot bytes from a serial device whose node is created only after power-up, or asks the operator to confirm plugging before the monitor process begins waiting.

## lesson-http-liveness-is-not-response-readiness | 2026-07-19 10:31

1. Date: 2026-07-19
2. What went wrong: Route registration, server-start markers, connectivity, and continuing application liveness were treated as if they proved that an HTTP request could deliver a complete parseable response.
3. Preventive rule: Keep connection establishment, request transmission, response status and headers, body receipt, and schema parsing as separate typed boundaries. Do not infer response readiness from route, startup, connectivity, or heartbeat markers.
4. Trigger signal to catch it earlier: Evidence shows a live application and registered route but has no independently observed response status, headers, body bytes, or completed parse.

## lesson-receive-only-serial-still-requires-raw-terminal-configuration | 2026-09-04 15:00

1. Date: 2026-09-04
2. What went wrong: The macOS USB observer opened the CDC callout node read-only and nonblocking but left its terminal line discipline in canonical mode, so binary and partial firmware evidence could remain withheld and healthy application execution appeared silent.
3. Preventive rule: A receive-only serial Adapter must still configure the admitted descriptor as raw at the expected baud, enable local receive, and disable hang-up-on-close while explicitly excluding payload writes, modem-control operations, DTR/RTS changes, and the maintenance baud.
4. Trigger signal to catch it earlier: A serial reader opens successfully but receives no bytes, while a PTY regression with bytes lacking a newline also returns empty or the Adapter never applies raw termios configuration.

## lesson-clear-archived-core-dumps-before-the-next-panic | 2026-10-06 18:00

1. Date: 2026-10-06
2. What went wrong: The firmware stores flash core dumps with `CONFIG_ESP_COREDUMP_FLASH_NO_OVERWRITE=y`. An earlier read returned a stale dump from a previous panic, and later the restart006 panic left no dump at all because the already-analysed queue boot-loop dump still occupied the partition, so its exact panic site was never captured.
3. Preventive rule: After a dump is captured, archived and analysed, clear it under its task contract so the partition is empty before the next effectful run. Before trusting a read, compare the dump's embedded app SHA-256 with the image that actually panicked.
4. Trigger signal to catch it earlier: A core-dump decode fails with `coredump SHA256 != app SHA256`, a read returns a dump whose app identity matches an older image, or an analysed dump has not been cleared when hardware work resumes.

## lesson-retained-log-ring-loses-boot-lines | 2026-10-06 21:30

1. Date: 2026-10-06
2. What went wrong: OTA-002 attempt 001 checked the installed boot's one-time `safe_state` line through `/api/system/logs` more than 20 minutes after boot. The 512 KiB retained log ring held only about 1,770 periodic snapshot and health lines by then, so the line had rotated out and the run stopped at its baseline.
3. Preventive rule: Read a boot's one-time retained lines only shortly after that boot is proven (for example right after a restart session). To prove the state of an older boot, use evidence bound to that boot, such as an authenticated session's Worker lease state, not the retained log.
4. Trigger signal to catch it earlier: A check searches `/api/system/logs` for a boot-time marker minutes after the boot, or the retained log is at its byte cap with only periodic records.

## lesson-fixed-serial-jtag-drops-runtime-log-markers | 2026-10-06 21:30

1. Date: 2026-10-06
2. What went wrong: The OTAWWW supervisor took its device origin, boot session and safe state from `runtime_origin`, `runtime_boot_identity` and `safe_state` lines in the flash-monitor capture. Since the fixed Serial/JTAG migration the USB writer forwards only allowlisted `usb_*` records, so real captures contain none of those lines; host tests passed because the fake device emitted them.
3. Preventive rule: Take the station address from the authenticated Gate endpoint handoff (`just otawww-endpoint`) and identity and safe state from HTTP or session state. Build fake-device fixtures from what a recent real capture actually contains, not from what the firmware logs.
4. Trigger signal to catch it earlier: A host workflow parses `runtime_origin`, `runtime_boot_identity` or `safe_state` from a USB capture, or a fixture emits serial lines absent from recent post-migration captures.

## lesson-start-per-boot-attempts-on-a-fresh-boot | 2026-10-09 17:14

1. Date: 2026-10-09
2. What went wrong: BWG-007 attempt 005 reused attempt 004's boot without a reinstall. The clock stimulus is spent once per boot and the rejection counter is per boot, so the first scenario failed `fact_stimulusCounterConsistent` before testing anything.
3. Preventive rule: When judged facts depend on per-boot one-shot device state, start every attempt on a fresh boot: an install, or the task-permitted `espflash board-info` chip reset. `scripts/bwg-restoration/README.md` states this precondition.
4. Trigger signal: An attempt is about to start on a boot an earlier attempt used, or a first scenario fails a per-boot counter or stimulus fact.

## lesson-flash-the-built-package-manifest | 2026-10-21

1. Date: 2026-10-21
2. What went wrong: The package manifest was copied into the attempt's private root as its record, and the copy was passed to `just flash-monitor --manifest`. The flash tool resolves the image files relative to the manifest, so it failed to read the ELF and stopped before any device write; the endpoint serve already started against the old firmware then had to be stopped.
3. Preventive rule: Pass the built manifest (`bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`) to `just flash-monitor`, and keep the private copy only as the record and for preflights that read its fields. Confirm the two are byte-identical before installing.
4. Trigger signal: `--manifest` points outside `bazel-bin`, or a flash reports `failed to read ... .elf`.
