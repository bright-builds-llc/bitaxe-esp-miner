# Known hardware and firmware issues

Open defects that are documented but not yet investigated. Each entry states
what was observed, how to recover, and what is not known.

## USB link stays unusable after a panic reset until a USB replug

- **Observed:** 2026-10-04, Ultra 205, diagnostic image `c634cc20`, during the
  control-stack reproduction (`task-str005-start-panic-diagnosis`).
- **Trigger:** a firmware panic. The CPU reset and rebooted normally (boot
  ordinal advanced, `reset_reason=panic`, the display showed uptime counting
  from the new boot), but the USB-Serial/JTAG link did not recover.
- **Symptoms while stuck:**
  - The detector still admits the device, and macOS still enumerates it.
  - Web Serial opens the port, but the firmware never answers the Gate's
    hello (stage `hello`, serial `timeout`).
  - OpenOCD on the built-in USB-JTAG fails a standard control request
    (`libusb_get_string_descriptor_ascii() failed with -1`).
  - `espflash board-info` cannot enter ROM through the control lines
    ("Failed to connect to the device").
- **Host side ruled out:** no process held the serial node (`lsof`), no agent
  process remained, and Chrome's granted port was not open. The IORegistry
  showed only Apple's ACM driver on the CDC interface and no user client on
  the JTAG interface.
- **Recovery:** unplug and replug only the USB cable, leaving barrel power
  connected. The firmware keeps running (uptime continues), the link
  re-enumerates, and Gate hello, JTAG and recovery all work again. A full
  power cycle is not needed.
- **Also seen:** during a panic boot loop (`board-info` could not connect),
  and once after an ordinary install, where the USB monitor stopped
  mid-bootloader while the application kept running (2026-10-05).
- **Leading hypothesis (2026-10-05, not yet verified on silicon):**
  - The USB-Serial/JTAG peripheral runs on the BBPLL, and neither a CPU
    reset nor the USB-core reset used after flashing resets it.
  - At application start, ESP-IDF's `CONFIG_ESP_SYSTEM_BBPLL_RECALIB`
    powers the BBPLL down and up again whenever the CPU arrived on the PLL,
    which is every reset path except a cold boot.
  - With a host transfer in flight, that clock glitch can wedge the link
    without re-enumeration.
  - The stuck install's stream stopped inside exactly that window (after
    "Disabling RNG early entropy source", before PSRAM init).
- **Mitigation in progress:** `CONFIG_ESP_SYSTEM_BBPLL_RECALIB=n`, which
  the build enforces. ESP-IDF says to disable it for bootloaders built with
  v5.2 or later; ours is v5.5.4. Verification needs a bounded reset loop
  (`task-usb-stuck-link-after-reset`).
- **Unknown:** whether that window is the only trigger; the bootloader's
  own PLL setup is a second, unaddressed candidate.
- **Operator guidance:** after any `reset_reason=panic` or an unexplained
  hello timeout with a ticking display, try a USB-only replug before
  escalating to a power cycle or a debugger.

## Rare chip power-on reset during a requested reset

- **Observed:** 2026-10-05, Ultra 205, image `60e344e2`, during
  `just usb-reset-endurance` (`task-usb-stuck-link-after-reset`). Cycle 64 of
  fixed-001 came back with `reset_reason=power_on` and boot ordinal 1 after a
  normal USB downloader reset. Nobody touched the power. The application
  then ran normally.
- **Rate so far:** 1 in 364 requested resets on that image; 0 in 100 on
  `6f268518`; 0 in the following 300-cycle run.
- **What the reason can mean:** on ESP32-S3, raw reset cause `0x01` covers
  chip power-on, chip brown-out and the super watchdog, and ESP-IDF reports
  all of them as `power_on`. Until ESP-IDF installs its brown-out handler,
  the analog brown-out reset resets the whole chip. A supply dip during the
  ROM, bootloader or early-startup window therefore reads as `power_on`; the
  same dip later would read `brownout`. An EN-pin glitch reads as `power_on`
  too.
- **Unknown:** which of these occurred. Confirming a rail dip or EN glitch
  needs electrical probing, which requires explicit owner authorization.
- **Operator guidance:** a `power_on` reset that nobody caused is
  unexpected; record it with its time. The device recovers by itself, and
  the boot ordinal restarts at 1.
