---
status: resolved
trigger: "Phase 33 receives no fresh post-restart device origin because native USB resumes delivering application bytes only after the new boot's one-shot markers have passed."
created: 2026-07-14T23:47:25Z
updated: 2026-07-15T00:43:46Z
---

## Current Focus

hypothesis: Confirmed and resolved — the durable proof surface outlives the native USB enumeration gap through RTC-backed boot ordinals and bounded typed replay.
test: The exact `a630455` package passed the sole detector-gated Ultra 205 attempt with typed A/N to B/N+1 replay, software CPU reset, one fresh B/N+1 origin binding, stable physical identity, durable hostname-digest equality, cleanup, and restoration.
expecting: Satisfied by the redacted hardware proof at `docs/evidence/phase-33/hardware-summary.md`.
next_action: None for this diagnosis. Preserve the proof as Phase 33 durability evidence only; Phase 35 still owns admission and parity promotion.

## Resolution

resolved_at: 2026-07-15T00:43:46Z
proof: `docs/evidence/phase-33/hardware-summary.md`
result: The one eligible fresh hardware attempt passed without retry. The classifier proved the reset-surviving ordinal transition and bounded replay facts that were absent from the earlier lossy native-USB capture. Process/holder cleanup and original-hostname restoration also passed.
non_claims: No raw identifiers or settings values were promoted, no parity row changed, and no Phase 35 admission occurred.

## Symptoms

expected: After the sole restart response, one protected passive monitor observes exactly one approved application restart, the new boot, exactly one fresh origin, stable physical identity, and the durable hostname.
actual: Detector, exact-package setup, PATCH confirmation, immediate readback, passive ownership, restart response-before-effect, service loss, cleanup, and restoration passed. The 360-second protected capture contained one new boot session's later heartbeats but no boot, restart-effect, or origin marker, so origin extraction failed closed.
errors: `fresh_origin_not_unique`, with zero candidates rather than multiple candidates. No shareable hardware evidence was emitted; CFG-12, Plan 03, and Phase 33 remain pending.
reproduction: Only the repo-owned Phase 33 wrapper may reproduce this. Diagnosis did not invoke hardware.
started: First observed on 2026-07-14 after the earlier macOS identity, system-info, and restart-response defects were resolved. Hardware source commit was `7f213d9`; diagnosis ran on clean, synced `1f2404a`.

## Eliminated

- Multiple or malformed post-restart origins: protected derivation found zero origin candidates.
- A failed application restart: service loss/recovery occurred and all observed heartbeats belong to one new boot session with monotonic sequence and uptime.
- A firmware stall until approximately 130 seconds: the heartbeat sequence had already advanced through the complete early-cadence window before the first delivered heartbeat. The observer was running; delivery was absent.
- A visible panic, watchdog, or later reset after byte delivery resumed: the protected aggregate has no hazard marker, one heartbeat session, monotonic sequence/uptime, and successful cleanup/restoration. A fully hidden intermediate reset inside the initial delivery blackout remained possible, which is why the fix adds a reset-surviving boot ordinal instead of trusting session count alone.
- An extraction offset that excludes the whole new boot: later heartbeats from the new boot are inside the exact post-offset range, proving the range is readable and post-restart.
- An intentional reset by the monitor: the wrapper passes the complete `--before no-reset-no-sync --after no-reset --no-reset --non-interactive` contract. Those flags prevent espflash-initiated reset; they cannot make the application's separate `esp_restart()` preserve the native USB transport.
- A delayed or repeated `device_url` producer: local firmware emits it once when Wi-Fi connects. The later successful HTTP recovery/restoration proves the application service recovered, but there is no periodic serial replay of that origin.

## Evidence

- timestamp: 2026-07-14T23:47:25Z
  checked: Protected Phase 33 trace through derived values only.
  found: Trace SHA-256 `8203de4a52c09a6474a747a5617e88e1ca29f0c9077344a4bf8eb5853dfb5ba9`; 23 heartbeats from one session; first sequence 119 at uptime 130181 ms; last sequence 141 at uptime 350281 ms; zero boot markers; zero restart-effect markers; zero unique origins; zero hazard markers.
  implication: The new boot's observer ran for roughly 130 seconds before the monitor received its first application line. All missing facts are one-shot lines emitted before that point; the periodic heartbeat is the only required marker still being produced afterward.
- timestamp: 2026-07-14T23:47:25Z
  checked: `scripts/phase33-confirmed-settings-durability.sh` and `scripts/phase13-monitor-capture.sh`.
  found: Phase 33 opens one espflash passive monitor before the POST, records the post-arming byte offset, performs the application restart, waits the full capture, and only then extracts post-offset origin/boot/restart markers. The capture wrapper starts one monitor process and never observes USB disappearance/re-enumeration or opens a replacement enumeration epoch.
  implication: The evidence design assumes a single native-USB serial handle is continuous across an application reset, although the local serial-session rules explicitly separate physical identity, enumeration identity, and delivered bytes.
- timestamp: 2026-07-14T23:47:25Z
  checked: espflash v4.0.1 `espflash/src/cli/monitor/mod.rs` and `espflash/src/connection/mod.rs`, matching the installed `espflash 4.0.1`.
  found: The monitor receives one Unix `serialport::TTYPort`, sets a 5 ms timeout, and loops on `read`; timeout and interrupt are the only retry cases. There is no port rediscovery, reopen, physical-identity match, or enumeration-epoch transition in the monitor loop.
  implication: `--no-reset` does not create a re-enumeration-aware observer. Once an ESP32-S3 USB Serial/JTAG reset interrupts that TTY epoch, espflash has no code that can deliberately reacquire the new epoch in time for early boot output.
- timestamp: 2026-07-14T23:47:25Z
  checked: Pinned ESP-IDF v5.5.4 USB Serial/JTAG documentation and source under `.embuild/espressif/esp-idf/v5.5.4`, plus firmware console configuration.
  found: Firmware selects UART as primary console and USB Serial/JTAG as secondary. ESP-IDF duplicates console characters to `esp_rom_usb_serial_putc`. Its documentation describes only a small internal device-to-host buffer and a one-time 50 ms wait when the PC is not requesting data; it does not provide a durable boot backlog. Espflash's own v4.0.1 documentation states that resetting a chip using built-in USB Serial/JTAG also resets that peripheral and disconnects the chip.
  implication: Output produced while the host's usable serial epoch is absent is not recoverable later from native USB. A later heartbeat can prove the firmware kept running, but cannot reconstruct earlier boot, restart, or origin lines.
- timestamp: 2026-07-14T23:47:25Z
  checked: `firmware/bitaxe/src/main.rs`, `firmware/bitaxe/src/http_api.rs`, `firmware/bitaxe/src/wifi_adapter.rs`, and `firmware/bitaxe/src/boot_evidence.rs`.
  found: The restart-effect line is emitted immediately before `esp_restart()`, the boot line is emitted near the beginning of `main`, and the origin line is emitted once on Wi-Fi connection. Only the heartbeat repeats for the boot lifetime, at 1-second cadence through 120 seconds and 10-second cadence afterward.
  implication: First delivery at sequence 119 and uptime 130181 ms precisely selects the repeating fact while missing every one-shot fact the wrapper requires.

## Root Cause

root_cause: Phase 33's proof architecture incorrectly equates a pre-restart espflash process and stable physical USB identity with continuous post-restart byte observation. On the ESP32-S3 native USB Serial/JTAG path, `esp_restart()` resets/disconnects the USB peripheral and creates an enumeration discontinuity. Espflash 4.0.1 retains one `TTYPort` and does not rediscover or reopen the device. The firmware then emits restart, boot, and fresh-origin evidence only once while that transport is not delivering to the protected reader. By the time macOS begins delivering from the new boot, only periodic heartbeats remain. `extract_one_origin` correctly fails closed on zero values; it is reporting the transport/evidence gap, not causing it.
confidence: high
files_involved: [`scripts/phase33-confirmed-settings-durability.sh`, `scripts/phase13-monitor-capture.sh`, `firmware/bitaxe/src/http_api.rs`, `firmware/bitaxe/src/main.rs`, `firmware/bitaxe/src/wifi_adapter.rs`, `firmware/bitaxe/src/boot_evidence.rs`]

## Suggested Fix Strategies

1. Make the Phase 33 observer explicitly own native-USB disappearance, a new enumeration epoch with the same stable physical identity, readiness, reattachment, cleanup, and bounded timing. Do not rely on espflash's single pre-reset TTY handle to span the reset.
2. Add a bounded post-attachment replay/proof surface for the non-secret boot identity and fresh origin, because reattachment alone cannot recover bytes emitted before enumeration. The replay must remain owned by boot lifetime rather than Wi-Fi/HTTP progress and must not expose secrets.
3. Replace the lossy pre-reset restart log as sole restart-effect proof with a post-boot, session-bound restart provenance fact or other reset-surviving evidence that composes with the already-proved HTTP response-before-effect and service-loss ordering.
4. Add simulation/source guards proving re-enumeration is handled and proving zero early native-USB bytes cannot qualify without replay. Preserve the current zero/multiple-origin fail-closed behavior.

## Implementation

- Added a dependency-free RTC boot record with magic, schema version, ordinal, complement, and checksum. Zero state initializes to ordinal 1; valid state increments exactly once; corruption, torn state, and overflow reinitialize instead of trusting continuity.
- Isolated firmware unsafe access in `rtc_boot_ordinal.rs` using volatile reads/writes to `.rtc_noinit`, initialized before worker threads.
- Added immediate and 10-second boot-lifetime `runtime_boot_identity` replay without changing the existing `runtime_heartbeat` formatter.
- Added immediate and 10-second `runtime_origin` replay, bound to session and ordinal for 360 seconds after Wi-Fi connection.
- Replaced Phase 33's one-shot marker/origin extraction with a typed classifier. The wrapper proves passive byte delivery before its sole POST, then requires a changed session, exactly N+1 ordinal, software CPU reset, monotonic identity replay, and exactly one unique bound origin. Identical replay duplicates are accepted.
- Added regressions for late attachment at heartbeat sequence 119, N+2, unchanged/multiple sessions, wrong reset reason, wrong-session origin, zero/multiple origins, duplicate acceptance, RTC corruption/torn state/overflow, and source-level wiring constraints.

## Verification Pending

- Mandatory Rust sequence passed: `cargo fmt --all`, warnings-as-errors Clippy for all targets/features, all-target/all-feature build, and all-feature tests. `bitaxe-api` passed 160 tests, including all RTC and typed Phase 33 classifier regressions.
- Sequential direct shell tests passed for serial-session tracing, Ultra 205 detection, and Phase 33 durability simulation. `shellcheck`, `shfmt -d`, Bash syntax, and `git diff --check` passed for the changed shell surfaces.
- Targeted Bazel tests passed for serial-session tracing, detection, Phase 33 simulation, and parity. All 10 Phase 33 source guards passed.
- Canonical `just build`, `just package`, and `just verify-reference` passed. The packaged firmware ELF contains a writable 32-byte `.rtc_noinit` section.
- Real hardware evidence remains intentionally pending. No hardware action, commit, push, or parity promotion has occurred yet.
