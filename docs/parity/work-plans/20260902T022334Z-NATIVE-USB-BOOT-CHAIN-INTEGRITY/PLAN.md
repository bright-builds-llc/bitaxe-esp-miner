# Native USB boot-chain integrity discriminator

- Run ID: `20260902T022334Z-NATIVE-USB-BOOT-CHAIN-INTEGRITY`
- Source base: `a2d2b8b8`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-native-usb-boot-chain-integrity-205`
- Predecessor: `task-native-usb-rom-exit-discriminator-205`

## Objective

Determine whether the silent recovery-006 Serial/JTAG runtime is caused by
boot-chain byte drift, OTA selection drift, an invalid selected application,
or an early application/console boundary. Use one manual ROM entry and
read-only flash operations only. Do not repair or flash anything in this child.

## Interfaces and state

- Add `just native-usb-boot-chain-integrity preflight|capture|read|finalize`.
- Use protected root
  `scratch/native-usb-boot-chain-integrity/attempt-001` and public projection
  `docs/parity/evidence/native-usb-boot-chain-integrity/boot-chain-projection-001.json`.
- Seal `prepared -> display_captured -> rom_admitted -> metadata_read ->
  selected_app_read -> complete | mismatch | invalid` without repeating a
  capture, manual checkpoint, ROM admission, range read, or reset.
- `preflight` validates exact clean/pushed source and package, the immutable
  plan/task, recovery-006 bundle and snapshots, the terminal predecessor,
  contained managed esptool, absent outputs, and zero owned processes. It
  creates no root and has no device effect.
- `capture` records one private operator-selected display category:
  `active_ui`, `boot_or_error_text`, `blank_or_dark`, `frozen_or_static`, or
  `unknown`. It also stores the current USB physical-identity digest. No raw
  display text or image is required.

## Read-only hardware discriminator

- `read` opens a no-timeout checkpoint instructing the operator to hold BOOT,
  press and release RESET, release BOOT, and then confirm.
- Reacquire exactly the captured physical connector and require one successful
  ESP32-S3 `board-info --before no-reset --after no-reset` observation.
- Through contained managed esptool, read each range once into mode-`0600`
  files under the mode-`0700` root:
  - exact recovery-006 bootloader range;
  - exact partition-table range;
  - exact OTA-data range;
  - exactly one selected application partition after selection is parsed.
- Compare bootloader, partition table, and OTA data byte-for-byte and by SHA-256
  with their recovery-006 snapshots.
- Parse the partition table with `esp-idf-part`. Parse both 32-byte OTA select
  entries using the pinned ESP-IDF v5.5.4 layout, CRC32, invalid-state rules,
  active-copy selection, and `(ota_seq - 1) % ota_app_count` mapping. Fall back
  to factory only under the pinned bootloader rules.
- Require exactly one selected factory/OTA partition in the recovery bundle,
  read its complete declared range, compare its digest and bytes, and validate
  its ESP32-S3 application header. Bind the selected partition to the
  recovery-006 `running_partition` identity.
- Exit ROM once through the contained managed esptool hard-reset application
  path, perform inspection-only same-device reacquisition, and clean up. Do not
  require serial bytes for acceptance of this readback child.

## Evidence and outcomes

- Public evidence contains only safe source/evaluator/bundle digests, display
  category, fixed range-match booleans, partition-table validity, closed OTA
  entry/selection categories, selected-partition category, selected-app
  digest/header/identity matches, physical-identity match, bounded operation
  counts, no-write/no-network facts, cleanup, terminal category, and redaction.
- Raw flash, addresses, paths, ports, USB identities, board information, OTA
  sequence/state/CRC values, application bytes, process data, and timestamps
  remain protected.
- Any bootloader, partition-table, OTA-data, selected-partition, or application
  mismatch is an accepted terminal discriminator. A later repair must target
  only the demonstrated range under a separate write contract.
- Exact bytes plus a valid selected app closes the flash/OTA hypothesis and
  makes a later early-boot witness or canonical-firmware installation child
  eligible. This child performs neither.
- ROM admission, read, reset, identity, or cleanup failure is terminal. No
  second hardware ordinal is authorized.

## Tests and verification

- Test exact read command/range/output ownership and absence of every write,
  erase, NVS, network, mining, and hardware-control surface.
- Test partition parsing; erased, single-valid, dual-valid, equal-sequence,
  bad-CRC, invalid/aborted, factory-fallback, and OTA-slot selection; truncated
  and oversized data; wrong bundle range; app header/digest mismatch; and
  physical-identity drift.
- Test display acceptance/cancellation, consume-once state, protected modes,
  process groups, runfiles, cleanup, and explicit public allowlisting.
- Before each commit or hardware action, run ordered Cargo gates, Bright
  Builds, all Bazel tests, firmware build/package, native-USB ownership,
  parity/progress, redaction, reference cleanliness, whitespace,
  sensitive-value scan, and final diff review.

## Exclusions

No flash/NVS/OTA/settings/theme write, erase, Wi-Fi or HTTP action, discovery,
mining, ASIC work, fan/voltage/power effect, direct UART/pins/pads/headers/
probes, external pool, other device, durability run, recovery mutation, or
parity promotion is authorized. Manual interaction is limited to the one
built-in BOOT/RESET sequence.
