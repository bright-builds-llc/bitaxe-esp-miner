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
   - A factory install seeds the board's own upstream defaults into NVS
     (`board_default_seed_values`), golden-tested against
     `config-<board>.cvs`. On the 601 it must seed, with the Wi-Fi
     credentials, and only the ordinary settings. The self-test, thermal and
     diagnostic seeds stay 205 only.
     - *Amended 2026-10-26 (phase F4.3, item 14):* the self-test seed now
       exists on the 601 too, with its own kind (`gamma601_full_v1`). The
       thermal and diagnostic seeds stay 205 only.
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
5. **The 601's first firmware is a safe boot.** The `board-gamma-601`
   Cargo feature (`//firmware/bitaxe:firmware_image_gamma601`, `just
   package --board 601`) builds firmware that:
   - drives the ASIC reset (GPIO1) low as its first act and never releases
     it;
   - commands the TPS546 off before anything else on the I2C bus, only
     after its identity reads as a TPS546. It keeps re-commanding off until
     the controller reports off and its output measures below 0.2 V;
   - can write nothing else to the TPS546. Its bus capability expresses
     only the off writes, in upstream's order (`OPERATION` off, then
     `ON_OFF_CONFIG`);
   - runs the fan at 100 % until the output is confirmed off, then at
     30 %, and sets the EMC2101's external-diode configuration;
   - starts none of the Ultra 205's safety, sensor, fan or mining owners,
     so it has no Worker channel;
     - *Amended 2026-10-24 (phase F2):* the 601 now starts the safety
       supervisor, the production mining owner and the fan controller. Its
       sensors and actuation stay the safe-boot thread's, so it still
       starts no sensor owner, and the 205-only self-test and diagnostic
       owners never start. Only `gamma601-mining` admits dispatch (item
       7); a production 601 refuses every run before its fan or regulator
       moves, and keeps the safe boot's idle fan. An owner that fails to
       start is logged and boot continues to HTTP. Its Worker channel
       follows item 10;
     - *Amended 2026-10-26 (phase F4.3, item 14):* an admitted self-test
       tuple starts the self-test owner in place of the mining owner and
       fan controller, and no Worker starts beside it. A build that cannot
       power the BM1370 refuses the tuple. The diagnostic owners still
       never start;
   - refuses an HTTP firmware update whose image carries another board's
     marker. Both builds do this.

   It logs its `usb_gamma601_safe_boot` and `usb_board_identity` records
   every 5 s. Wi-Fi, HTTP and the startup screen run as on the 205.
   Residual risk: until the firmware's first act, and while a package is
   written in the ROM downloader, the ASIC reset and the TPS546 output are
   whatever the previous firmware left. The safe boot marks its OTA image
   valid even when a reading fails; rolling back could restore firmware
   that mines. The record also carries `STATUS_WORD` and `STATUS_CML` as
   read before the identity read, the boot's first transfers to the
   controller.
6. **The core voltage is first turned on by a test-only variant.**
   `gamma601-vout-check` (`just package-gamma601-vout-check`, Bazel flag
   `--//firmware/bitaxe:firmware_variant`) is the safe boot plus one
   check per boot, with the ASIC held in reset throughout:
   - It starts 30 s after boot, and only from a confirmed-off output with
     the ASIC reset held.
   - It runs only after a power-on, software, USB or external reset. It
     also refuses in two other cases:
     - a check an earlier boot left unfinished, recorded in an RTC latch
       that only removing all power (barrel and USB) clears;
     - a controller that already holds the check's `VOUT_MAX` from earlier
       in its power-up. A rebuilt image cannot move that anchor.
   - It commands the output off again, as upstream does, then configures
     the TPS546 with upstream's Gamma writes, still off:
     - the plan is built only for a read `VOUT_MODE` of `0x97`;
     - the target is clamped to 1.00-1.20 V, and the check uses 1.15 V;
     - every register must read back as written.
   - It records the status the writes left, re-confirms off, sends
     `CLEAR_FAULTS`, and refuses a baseline without `OFF` or with any fault
     bit but CML.
   - Only an armed token, built from the matched readback, the
     re-confirmed off state, the judge and a held reset, can turn the
     output on. Before the enable, the ASIC reset pad is latched low
     (`gpio_hold_en`) until the output is confirmed off again. The token
     is a structural guard; the readings it is built from are the
     caller's.
   - It turns the output on once for 60 s, sampled every 250 ms by a pure
     judge. Any of these ends the window and commands off:
     - a reading outside its limit or implausible;
     - a new fault bit;
     - a missed sample;
     - a clock that does not advance;
     - the sample cap;
     - the ASIC reset found released.
   - It logs a `usb_gamma601_vout_check` record.

   Two deviations from upstream bound the voltage: `VOUT_COMMAND` is the
   target, not the interim 1.2 V, and `VOUT_MAX` is 1.25 V, not 2.0 V.

   Only this variant's build can express the configuration writes,
   `CLEAR_FAULTS` or `OPERATION` on. Its bus refuses any command outside
   the plan, and any NVM command, before the bus.

   The variant is board 601 only, in every place:
   - the 205 targets refuse it;
   - the packager, manifest and flash tool tie it to board 601;
   - the flash tool requires `--allow-test-firmware gamma601-vout-check`;
   - it is never released.

   Residual risks:
   - the judge samples at about 4 Hz, so a fault faster than that is
     bounded only by the controller's own limits;
   - the configuration is volatile, but the controller keeps it until its
     power is removed;
   - a reset during the window leaves `OPERATION` on until the next boot
     commands off, a few seconds later. The latched reset pad is meant to
     keep the ASIC in reset meanwhile, but no hardware evidence yet shows
     that the hold survives a panic or watchdog reset. The latch and the
     `VOUT_MAX` anchor refuse a second check until all power is removed.
7. **The 601 mines first in a test-only variant.** `gamma601-mining`
   (`just package-gamma601-mining`) is the only 601 build that may power
   the BM1370 and admit mining dispatch, for the phase F3 hardware
   contracts:
   - Only it and `gamma601-vout-check` compile the TPS546 configuration
     writes, `CLEAR_FAULTS` and `OPERATION` on. A production 601 build
     cannot turn the output on, and the variant-isolation gate refuses a
     production ELF that carries the enable.
   - It is tied to board 601 like the check, needs
     `--allow-test-firmware gamma601-mining`, and is never released.
   - Production 601 dispatch is promoted only on F3's hardware evidence
     (phase F4).
   - Until its power path and preparation land (phase F2), it boots the
     safe boot with its own marker.
   - Residual risk, a panic while mining: the panic handler's latch write
     drives the ASIC reset low at once (GPIO1 is an output once the
     executor owns it; its GPIO10 write drives nothing on the 601). The
     TPS546 keeps its output on through the reboot until the next boot's
     safe boot commands it off, and between the reboot and the next
     boot's first act the reset line is not driven. Upstream has the same
     window. Holding the pad low through the panic needs register-level
     writes in the panic path and fault-injection evidence on hardware,
     under its own authorization.
8. **The BM1370 is first detected by a test-only check.**
   `gamma601-chip-check` (`just package-gamma601-chip-check`) is the safe
   boot plus one check per power-on, 30 s after boot from a confirmed-off
   output, on the safe-boot thread:
   - it refuses after any reset but a power-on, software, USB or external
     one, or when its RTC latch shows an earlier check unfinished or
     already run since all power was last removed;
   - it sets the fan to full and requires a nonzero tach reading;
   - it latches the reset pad low (`gpio_hold_en`), turns the TPS546 on at
     1.15 V through the production power path (configured and read back, a
     clean off baseline, an armed enable with the reset held), waits
     500 ms and judges the settled output;
   - it frees the pad and runs the BM1370's chip detection through the
     executor (the reset pulse, the `CHIP_ID` read and the count), which
     must find exactly one chip; the count starts only within 5 s of the
     power-on sequence;
   - it always holds the reset low again, latches the pad and turns the
     output off, confirmed.
   The output is on for about 6 s on a healthy bus (the settle, then up to
   5.4 s of detection) and about 8 s to a confirmed off; the steps' own
   I2C timeouts bound it at about 16 s on a degraded bus. The pad latch
   keeps the reset low through any reset while the output may be on,
   except during the count. There, a software restart's shutdown handler
   drives the reset low and latches it, and the panic cutoff drives it low
   until the reboot. After a panic, a host USB reset, a brown-out or a
   hardware watchdog in the count window, the line is undriven from the
   reboot to the next boot's first act (item 7's residual risk).
   - Residual risk, the latch: it lives in `.rtc_noinit`. A rebuilt image
     whose layout moves that word can read it as clear without all power
     removed, so a second check after a rebuild still needs a power
     removal by contract.
   It sends no work and never sets a frequency. It logs and replays a
   value-free `usb_gamma601_chip_check` record. It is tied to board 601
   like the other test variants, needs `--allow-test-firmware
   gamma601-chip-check`, and is never released; dispatch stays refused.
9. **The BM1370 first hashes in a test-only check.**
   `gamma601-hash-check` (`just package-gamma601-hash-check`) runs item 8's
   powered check (the shared `gamma601/check_power.rs`) and, once exactly
   one chip answers:
   - initializes the BM1370 through the executor
     (`bm1370::plan::mining_ready_actions`), ramping to 300 MHz at
     1.15 V, below upstream's 525 MHz for this first run (owner decision,
     2026-10-24), then waits upstream's 2 s;
   - requires two agreeing diode readings between 10 and 65 C, or ends
     `diode_unusable` without sending work;
   - sends the Bitcoin genesis header as one job and verifies every result
     in firmware: the header rebuilt with the result's nonce and rolled
     version bits, its double SHA-256 against the chip's ticket. It passes
     with at least 3 verified results and invalid ones at most a tenth;
     the genesis nonce itself is recorded but not expected, since the
     chip rolls versions;
   - stops on any telemetry limit (VOUT, VIN, power, IOUT, regulator and
     chip temperature, fan, readings stale past 1.5 s) and enforces a 45 s
     output-on cap; the init runs in segments ending at each delay, so a
     stop acts within one ramp step, and the hashing window is at most
     20 s.
   - Residual risk, the stack: the build's audit bounds the check's
     `run_check` frame, not the frames below it (the init, the result
     reads, the double SHA-256). The thread has 16 KiB, and the record
     reports the least free stack after the check.
   It logs and replays a value-free `usb_gamma601_hash_check` record, needs
   `--allow-test-firmware gamma601-hash-check`, and is never released;
   dispatch stays refused and no pool is involved.
10. **Every 601 build runs the Worker.** The owner signed the 601's Worker
    capability (`firmware/bitaxe/bwg/gamma601-capability.json`, board
    `bitaxe-gamma`/`601`) with the development update authority's rotated
    key, which the compiled deployment trust already lists. The Gate admits
    the pair from `b775145` (its ADR-0104).
    - **The boot baseline:** the 601 confirms it once the executor's reset
      owner holds the reset with the UART retained, the safe boot reports
      the TPS546 output confirmed off within 3 s, and its thread runs.
      Otherwise it stays unconfirmed and no Worker starts, as before.
      Worker recovery after a reboot then runs on the 601 as on the 205:
      NVS ledger bookkeeping only. A recovery failure leaves the Worker
      unavailable and boot continues to HTTP, on both boards.
    - **The mining variant:** with a confirmed baseline, `gamma601-mining`
      can now be driven by a signed Worker lease to power and mine. Its
      install stays under a phase F3 contract.
    - **Board facts:** the capability, the model it names and the pool
      subscription model (`bitaxe-gamma`) follow the built board, and the
      local pool settings read the board's own upstream defaults.
    - **Starts:** a board whose mining is not admitted refuses every Worker
      Start before any budget is charged
      (`worker_start=refused category=board reason=board_mining_unadmitted`;
      the Gate sees `session_failed`). So a production 601 refuses, and
      only `gamma601-mining` admits.
    - **The firmware's copy** is checked against the Gate's exported
      capability on every field but the board, and its signature against
      the compiled trust. The Worker threads are the 205's, with the same
      stacks.
11. **The 601's chip temperature counts only once the BM1370 is
    initialized.** Its EMC2101 external channel reads the BM1370's diode,
    which reads 127.75 C (raw `0x3FE`) until the init's analog-mux write and
    37.5 to 40 C after it (phase F3.2). Upstream reads it only after its
    init (`thermal.c`).
    - **Stages** (`bitaxe_safety::gamma601_diode`): with the output
      confirmed off the chip temperature is `Unavailable(AsicUnpowered)`;
      powered before this run's init completes it is
      `Unavailable(AsicNotInitialized)`; once the executor reports the init
      for the generation the output is on for, the reading counts, latched
      until the off is confirmed. While initialized, an exact 127.75 C is a
      fault (a chip that lost its configuration), and a read failure or a
      stale reading revokes as before. The executor's init flag is an
      atomic, so the sweep never waits on the init ramp's lock.
    - **The verdict** skips the chip temperature only for those two reasons
      and only on a board whose chip temperature comes from the ASIC's
      diode (`emc_internal_temp` false in the catalog, as upstream's
      `device_config.h`). The 205's board-sensor reading is unchanged.
    - **The window cap:** powered but uninitialized for more than 20 s is a
      fault (`bm1370_init_overdue`).
    - **Cooling after a stop and overheat recovery** cannot use the diode
      with the chip unpowered. On the 601 they need the chip unpowered and
      the regulator at or below 95 C, for at least 30 s: upstream's own
      600-series rule (`power_management_task.c`). Run admission on the 601
      also needs the chip unpowered.
    - **No mining at boot:** `gamma601-mining` ignores the saved
      start-on-boot preference (which defaults to true, with the factory
      install's default pool saved), so a run starts only from the soak's
      explicit resume after its pool and profile are saved.
    - **The reduced overheat profile** keeps upstream's rule: 100 MHz and
      100 mV off the pair, checked against the built board's catalog range
      (ADR-0038), not the phase F plan's "nearest lower catalog pair". From
      500/1150 the 601 restarts at 400/1050; from 400 MHz it stays stopped.
    - **My calls:** every 601 stop proof takes the 30 s, not only after an
      overheat; a regulator overheat still ends the run stopped rather than
      recovering as upstream does; the overheat-recovery reduced profile
      follows the built board's catalog (it used the 205's).
12. **The 601's overheat is first checked with a lowered trip.**
    `gamma601-overheat-trip-test` (`just
    package-gamma601-overheat-trip-test`) is `gamma601-mining` with
    ADR-0040's 55 C local overheat trip, for phase F3.4's check on the 601
    only; it carries one boot marker of its own and is never released. The
    local-mining soak's overheat kind follows the board: the 601's 22 W stop
    and idle threshold, and its cooling rule (the chip unpowered, the
    regulator at or below 95 C) where the 205 waits for its chip at 45 C.
13. **A 601 soak lease needs no original acceptance campaign** (owner
    decision, 2026-10-25). ADR-0033's soak allowance follows the 205's
    original 240-second campaign, complete at its full ceiling. The 601
    never ran that campaign, so phase F3.4's soak Start was refused
    (`soak_budget=original_incomplete`). Each board now states its
    prerequisite (`SoakPrerequisite`):
    - the 205 keeps the complete original campaign;
    - the 601 needs none, but a stored campaign left unfinished still
      refuses.
    The soak's own ledger, its 600-second budget, heartbeat revocation, the
    ordered shutdown and the board's power and thermal judges are unchanged.
    The staged short-campaign proof is given up on the 601: its lease path
    is board-independent code proven on the 205, and its power and thermal
    risk is judged against the 601's own limits. Because a 601 soak no
    longer implies a finished campaign, a campaign window now refuses while
    a soak awaits finalization (`acceptance_budget=soak_pending`).
    Qualification attempts keep the original-campaign requirement, so they
    stay refused on the 601. A refused Start now keeps its value-free reason
    in the retained log (`worker_start=refused category=... reason=...`),
    which a soak reads, because the page owns the Worker's console during a
    lease.

14. **The self-test runs on the 601** (owner decision 2026-10-26: port it
    before promotion). One pure contract
    (`HardwareSelfTestContract::for_board`) derives each board's profile,
    expected hashrate, core voltage window and power ceiling from its
    catalog entry, as upstream's `self_test.c` derives them from
    `DEVICE_CONFIG`. The 205's values equal its verified constants (a golden
    test pins them). The 601 runs 525 MHz / 1150 mV, expects about
    910 GH/s, admits 1035 to 1265 mV and at most 22 W.
    - **The regulator:** a TPS546 power fault fails the run, as upstream's
      `test_vreg_faults`. So does a regulator temperature above its 105 C
      throttle, or none read.
    - **Authority:** the TPS546 turns on only under a generation's permit
      (item 10). On the 601 the self-test runs as a local generation with a
      240 s budget armed at its start. The gate revokes it at the bound or
      on an unsafe observation, and the regulator's owner turns the output
      off within 100 ms, independent of the self-test thread. Its fan
      commands go as that generation's controller commands. The 205's
      self-test runs exactly as verified.
    - **Deviation:** the 601 keeps the input-voltage check (4.5 to 5.5 V,
      from the TPS546 VIN), which upstream skips without an INA260.
    - **Tooling:** the flash tool and `just self-test-campaign` take the
      board (`--board 601`), each with its own attempt roots, immutable plan
      (`docs/parity/work-plans/20261026T000000Z-SELF-001-GAMMA601/PLAN.md`)
      and evidence projection. The 205's attempt-005 constants are
      unchanged.
    - **USB records** (owner decision 2026-10-26, amending ADR-0021's closed
      allow-list): since ADR-0021 the single Serial/JTAG writer forwards no
      ordinary log line, so SELF-001's campaign, which reads the self-test's
      stages, checkpoint, metrics, terminal and receipt and the connected
      origin from the console, could not run on either board. The writer now
      forwards those records, each in its exact closed shape
      (`usb_diagnostics/self_test.rs`): JSON records of lowercase tokens,
      digits and punctuation only, a receipt with a closed outcome and a hex
      lease, and the origin as a dotted IPv4 `http` URL. The origin is the
      private USB-observer marker it was built as; committed evidence still
      redacts it.
    - **Non-claims:** upstream's factory-seed and BOOT-held starts stay
      unported on both boards. The 601 claims SELF-001 only after its
      promoted production build repeats the lifecycle.


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
- **Board selection.** `flash`, `flash-monitor`, `monitor` and `detect`
  request the board their `--board` names; every other command requests the
  205.
- The registry holds digests only. A base MAC has little entropy under a
  known vendor prefix, so a digest can be reversed by search: the registry
  stays private (mode 0600, gitignored), and committed evidence names board
  ids and categories, never MACs or digests.
- A registry missing from a worktree or a fresh checkout refuses every
  write until the board is registered again from a private `board-info`
  output.
