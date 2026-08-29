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

## 2026-08-29T20:30:00Z | Worker boot and maintenance-arm boundary

- Clean pushed source `a6fb6328` flashed successfully. A protected
  receive-only reset capture proved the exact firmware boot, factory boot
  validation, inactive ASIC reset-low state, disabled mining, and the expected
  Serial/JTAG-to-TinyUSB transition. Fresh detection then admitted
  `worker_runtime`.
- Automatic cycle-001 stopped before ROM and before writes with
  `handoff_ready_timeout`; cleanup completed and WorkerRuntime remained
  available.
- macOS may assert DTR as part of opening the CDC node. Reasserting DTR before
  applying 1200 baud can therefore deliver the duplicate event that the strict
  firmware reducer correctly disarms.
- The targeted Adapter plan now clears DTR at 115200 baud, waits for those
  callbacks to settle, emits one DTR assertion, waits again, and then applies
  exact 1200-baud line coding. The firmware state machine and its duplicate,
  wrong-order, timeout, disconnect, and unsafe-state rejection remain
  unchanged.
- Verification: the ordered Cargo gates, Bright Builds, focused USB/flash
  tests, all 70 Bazel tests, canonical package, native-USB ownership,
  parity/progress, redaction, reference cleanliness, and whitespace passed.
- No automatic ROM transition or device write occurred in cycle-001. A retry
  requires this host regression to pass every gate, reach a clean pushed
  commit, and produce a new exact clean package.

## 2026-08-29T20:45:00Z | Host-visible PHY disconnect boundary

- Clean pushed package `d9d7fd94` started automatic cycle-002 from admitted
  WorkerRuntime. The corrected macOS control sequence received readiness and
  committed maintenance, proving the prior timeout fixed.
- The same connector remained absent through the 60-second ROM transition
  window and later returned as WorkerRuntime. No ROM admission or device write
  occurred.
- Comparison with the pinned Espressif implementation showed that the reduced
  PHY Adapter cleared and immediately re-enabled the Serial/JTAG USB pad. It
  omitted the host-visible disconnect interval that precedes re-enumeration.
- The targeted fix holds the pad disconnected for 100 ms using the ROM delay
  primitive between the existing clear and set operations. It introduces no
  FreeRTOS owner, callback, descriptor, data I/O, or additional C entrypoint.
- A source-order regression requires disconnect, bounded delay, then reconnect.
  The ordered Cargo gates, Bright Builds, focused USB/firmware tests, all 70
  Bazel tests, actual ESP32-S3 link, canonical package, ownership, parity,
  progress, redaction, reference, and whitespace gates passed.
  A new cycle requires every gate, a clean pushed commit, and a new exact clean
  package.

## 2026-08-29T21:00:00Z | Repeated terminal transition signature

- Clean pushed source `4b165570` and its exact package started automatic
  cycle-003 from WorkerRuntime after all software, firmware-link, ownership,
  parity, privacy, and reference gates passed.
- The maintenance readiness receipt and commit edge again completed, but the
  same physical connector did not appear as an admitted ROM profile within the
  bounded transition window. No ROM admission or device write occurred.
- The authoritative `handoff_transition_timeout` signature exactly repeated
  after its targeted PHY-disconnect fix. The immutable continuation contract
  therefore makes this signature terminal; no further hardware retry or
  speculative fix is eligible in this campaign.
- Host cleanup completed with zero serial holders and zero owned espflash or
  flash-tool processes. The installed device was not restored to recovery-006;
  the automatic route cannot currently reach ROM, and the plan's one-time
  manual BOOT/RESET bootstrap has been consumed.
- Required continuation: create a new explicit recovery contract if another
  built-in-button ROM entry is authorized, restore recovery-006 exactly, and
  separately redesign the ROM transition proof before resuming durability.

## 2026-08-29T21:20:00Z | Software-only root-cause diagnostics

- No hardware action was performed. The repeated transition artifacts and
  host reducer proved that the old `absent` summary conflated true absence with
  a same-physical wrong profile, while the host closed CDC immediately after
  an unacknowledged DTR commit.
- Firmware now emits a fixed committed receipt after accepting DTR falling and
  before touching the PHY. The host keeps CDC open and requires that receipt;
  readiness alone cannot satisfy the new `handoff_commit_timeout` boundary.
- Protected mode-0600 transition traces record at most 512 closed samples:
  `absent`, `same_worker`, `same_serial_jtag`, `same_unknown`, or
  `physical_mismatch`. Ports and all physical/enumeration identity remain
  excluded.
- The one-function C Adapter now mirrors the pinned Espressif behavior without
  Arduino framework ownership: it drives D-/D+ low, reconnects hardware
  USB-Serial-JTAG, polls for an actual BUS_RESET for at most one second, and
  returns an explicit timeout instead of restarting on assumption.
- Unit/source tests cover detach-before-commit, one committed restart,
  committed-versus-ready receipt classification, bounded profile evidence,
  wrong-profile and physical-mismatch categories, BUS_RESET ordering, and
  restart suppression on PHY failure. Hardware validation remains prohibited
  until a new explicit recovery and diagnostic contract is approved.
- Verification passed: ordered Cargo formatting, strict Clippy, all-target
  build, all-feature tests, Bright Builds, focused USB/firmware tests, actual
  normal and rollback ESP32-S3 links, all 70 Bazel tests, canonical package,
  native-USB ownership, parity/progress, redaction, reference cleanliness, and
  whitespace. The first all-target run collided while building both
  non-hermetic firmware variants concurrently; the isolated rollback build and
  required complete rerun passed.
- Delivery: implementation commit
  `ea58797f85478d2a3f7e13cd8042e25dcaf1d28a` is pushed. The canonical package
  rebuilt from that exact clean source with six artifacts, `source_dirty=false`,
  reference `c1915b0a63bfabebdb95a515cedfee05146c1d50`, and manifest SHA-256
  `d10c2668063a3c00b520b58f957452b9cd18b9f3b8fe8f2fa9a63ae9740c45b7`.
