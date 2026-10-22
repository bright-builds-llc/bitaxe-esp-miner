# ADR-0041: Multi-board support starts with board identity

## Status

Accepted on 2026-10-21 by the owner, who chose:
- the staged Gamma 601 bring-up, from safe boot to mining;
- the same standing hardware authority as the Ultra 205.

Recorded for `task-gamma-601-bring-up`. ADR-0014 still governs what the 601
may claim.

## Context

The owner now has a Bitaxe Gamma 601 beside the Ultra 205. From the pinned
reference (`reference/esp-miner/main/device_config.h`), the 601 has:
- one BM1370;
- a TPS546 PMBus power controller in place of the 205's DS4432U and
  INA260;
- an EMC2101 whose external diode is the chip temperature;
- no ASIC-enable or plug-sense GPIO.

Both boards use an ESP32-S3 with native USB Serial/JTAG, which enumerates
as the generic Espressif device.

Today nothing can tell the two boards apart:
- The detector's physical identity is a digest of the host USB port
  (`tools/device-session`), not of the board.
- ROM admission checks only that the chip is an ESP32-S3.
- A 601 on the 205's usual port would therefore be admitted as the 205, and
  the 205 package could be written to it. That firmware drives GPIO10 as the
  205's ASIC enable and writes the DS4432U; on the 601 it would also run
  BM1366 init.

Mining stays refused there only because the 205's INA260 is absent. That is
not a guard anyone designed.

## Decision

1. **Board identity comes from the device.** A local, ignored registry
   (`boards.local.json`, mode 0600) maps each board id to a SHA-256 digest
   of the ESP32-S3 base MAC that `espflash board-info` reports. Raw MACs are
   never persisted. In `tools/flash`, before any effect that needs the ROM
   downloader (a write, an erase, a flash read), the tool requires the
   admitted device's digest to be registered for the requested board. An
   unknown device and a device registered for another board both refuse,
   before any effect. When no device at all is registered for the requested
   board, the tool refuses even before its reset into the ROM downloader.
2. **The board is chosen at build time.** Each package targets one board.
   The 601 build contains no DS4432U, INA260 or GPIO10 code, and the 205
   build no TPS546 code. A package for one board is refused on another.
   - In `tools/flash`, only `flash`, `flash-monitor`, `monitor` and `detect`
     accept `--board 601`; every other command parses the 205 only.
   - `--board` selects the package to build. Admission requires the
     manifest's `image_metadata.board` and the application's compiled
     marker (`firmware_board=<board>`) to name the requested board, and no
     other board's marker. Packages are admitted only when built from the
     current source, so there is no allowance for older packages.
   - A factory install seeds Ultra 205 defaults into NVS, so it stays 205
     only. The 601 is installed state-preserving.
3. **The firmware checks its board at run time** against the stored
   `boardversion` and refuses hardware control on a mismatch.
   - It reads the value once at boot and reports it, as a closed label
     (`205`, `601`, `missing`, `other` or `unreadable`), with the relation
     `match`, `mismatch` or `unknown`.
   - The report goes in a periodic USB record, `usb_board_identity`, and in
     the API fields `storedBoard` and `boardIdentity`.
   - Only a value naming another known board is a mismatch. It withholds
     the ASIC production handle, which every mining admission requires, and
     the self-test. Holding the board safe stays available.
   - The ASIC diagnostic compile modes (chip detection, work-result
     bootstrap) are not production builds and are outside this guard.
   - A missing, unrecognized or unreadable value is reported but does not
     block. The report is not authoritative: the registry gate above
     decides what may be written.
4. **The 601 does not inherit Ultra 205 verification** (ADR-0014). Its
   identification, safe boot, power controller, ASIC and mining each need
   their own task contract and hardware evidence. Parity rows CFG-002,
   ASIC-008 and PWR-004 move only on that evidence.

## Consequences

- A device absent from the registry cannot be written, so the 205 must be
  registered before its next write. Its digest can be derived from the
  `board-info` output kept privately from earlier runs.
- Detection without the ROM downloader still cannot see the MAC, so a
  detector pass verifies no board. Only ROM admission does.
- **Where the gate sits.** It runs at `tools/flash` ROM admission
  (`ensure_bootloader`), which precedes every flash write, erase, flash read
  and NVS operation, and at the core-dump erase's own probe. ROM admission
  requires the board its USB session was opened for, set once the session
  opens and cleared when it finishes. Sessions opened without a board
  require the 205, and a session cannot change its board. `just
  register-board` records a board from a private `board-info` output.
- **What stays ungated.** Paths outside that admission only reset the chip,
  probe the ROM or read flash, so they cannot change a board's firmware.
  Examples:
  - display recovery's, the mining campaign's and owner recovery's
    `board-info` probes;
  - `detect --retain-rom`;
  - the installed-application, ROM-exit, boot-chain and USB-stability
    resets;
  - the device-session reset endurance;
  - the Stratum v2 restore snapshot's `read-flash`.
- **Identification.** `just identify-board` enters the ROM downloader
  without the gate, because the device may not be registered yet. It reads
  the NVS partition and hard-resets back to the installed application,
  writing nothing. The partition holds Wi-Fi and pool secrets, so it
  passes only through private named pipes and memory, never a stored
  file. It registers a board only when the board's own stored
  `boardversion`, `asicmodel` and `devicemodel` all name it.
- **One requested board for now.** The gate requests board 205 today.
  Phase C threads the requested board through every command.
- The registry holds digests only. A base MAC has little entropy under a
  known vendor prefix, so a digest can be reversed by search: the registry
  stays private (mode 0600, gitignored), and committed evidence names board
  ids and categories, never MACs or digests.
- A registry missing from a worktree or a fresh checkout refuses every
  write until the board is registered again from a private `board-info`
  output.
