# Native USB ownership worklog

## 2026-08-29T17:50:47Z | Immutable plan

- Source base: `a0337f013b2b1db1956843d56ccdd2d003f493d7`.
- Immutable plan SHA-256:
  `9568c4eb98386f9a31d5b96079bec35df7d7a635a299ab416413cb7f5c78a807`.
- Action: created the single-PHY ownership, buttonless handoff, one-time manual
  bootstrap, durability, and repository-guardrail contract.
- Hardware/network effects: none.
- Next safe action: verify, commit, and push this plan separately, then implement
  the pre-agreed software seams without hardware effects.

## 2026-08-29T18:20:00Z | Guardrail and profile core

- Added ADR-0020, native USB ownership guidance, the always-loaded AGENTS
  pointer, profile-aware device-session types, and regression tests proving a
  Worker runtime flash selects handoff while monitoring never does.
- Hardware effects: none. Firmware handoff and host adapters remain pending.

## 2026-08-29T19:01:02Z | Complete software handoff

- Moved the maintenance state machine into the pure core and covered the exact
  DTR/1200-baud/safe-stop/readiness/commit sequence plus wrong-order,
  duplicate, disconnect, timeout, active-effect, and failed-safe-stop disarm.
- Made `UsbRuntime` the sole TinyUSB installation and PHY-handoff owner. The C
  shell now uses the minimum pinned Espressif ESP32-S3 Serial/JTAG switch,
  force-download shutdown handler, and explicit error returns.
- Added macOS profile metadata, connector-stable physical identity, bounded CDC
  serial control, retained-lease profile reacquisition, ROM `board-info`
  admission, centralized espflash/esptool routing, and receive-only monitoring.
- Added the Bazel-backed `just verify-native-usb-ownership` source guard, the
  visible-CDC lesson, and detailed profile/recovery guidance. Updated the BWG
  conformance boundary so vendor payloads remain unable to request bootloader
  control.
- Verification: ordered Cargo gates, Bright Builds, focused USB tests, all 69
  Bazel tests, canonical firmware build/package, parity/progress, redaction,
  pinned-reference cleanliness, and whitespace checks passed.
- Hardware/network effects: none. The one-time manual bootstrap, automatic
  handoff validation, 20-cycle durability run, and exact recovery-006 restore
  remain the next stage.

## 2026-08-29T19:37:57Z | Rust runtime Adapter prerequisite

- The stages 1–2 child moved exact descriptors, TinyUSB installation,
  callbacks, vendor/CDC I/O, and bounded write behavior into Rust.
- Repository-owned C is now the intentional PHY/force-download Adapter only.
  The child verifies exact linked callback ownership and absence of retired C
  symbols without touching hardware.
- Hardware/network effects: none. Manual bootstrap and durability remain
  blocked until the child is committed, pushed, archived, and repackaged.

## 2026-08-29T20:00:00Z | Manual bootstrap and ROM-admission regression

- Protected bootstrap-001 recorded the exact clean `632e9603` package and the
  user completed the authorized built-in BOOT/RESET sequence.
- Profile-aware detection admitted exactly one Ultra 205 ROM downloader and
  successful board information. The subsequent exact-package flash stopped
  before any write with `bootloader_sync_failed`.
- Protected child output proved that espflash 4.5.0 reported
  `Chip type: esp32s3`; the admission code compared presentation text against
  only `ESP32-S3`. A red-to-green regression now uses the real output and the
  owner parses the exact chip field case-insensitively.
- Verification: ordered Cargo gates, Bright Builds, focused USB/flash tests,
  all 70 Bazel tests, canonical package, native-USB ownership, parity/progress,
  redaction, and pinned-reference checks passed. One unrelated automation test
  transiently read an incomplete private JSON file; its immediate isolated
  rerun and the following complete Bazel run passed.
- Device effects: ROM detection and board information only; zero flash bytes
  were written. Next action is a clean implementation commit/push, exact clean
  package rebuild, and the progress-backed flash retry without another manual
  button sequence when the board remains in ROM.

## 2026-08-29T20:15:00Z | Fresh-process synchronization boundary

- Clean pushed source `178f2ead` and its exact six-artifact package passed ROM
  admission on the progress-backed retry. The write process then failed before
  transfer with `bootloader_connect_failed`.
- Espflash 4.5.0 documents `no-reset-no-sync` as skipping both reset control
  lines and the serial synchronization command. Because ROM admission and
  flashing are separate supervised processes, the latter must establish its
  own protocol synchronization.
- The targeted correction uses `no-reset`/`no_reset` only after `UsbOwnership`
  has completed profile handoff and ROM admission. It still suppresses DTR/RTS
  reset traffic and cannot send synchronization bytes to Worker CDC.
- The immutable plan remains unchanged. This worklog records the
  hardware-proven command-level deviation and its narrower safety invariant.
- Verification: the ordered Cargo gates, Bright Builds, focused USB/flash
  tests, all 70 Bazel tests, canonical package, native-USB ownership,
  parity/progress, redaction, reference cleanliness, and whitespace passed.
- Device effects: successful board information only; zero image bytes were
  written. A new hardware retry requires this regression fix to pass every
  gate, reach a clean pushed commit, and produce a new exact clean package.
