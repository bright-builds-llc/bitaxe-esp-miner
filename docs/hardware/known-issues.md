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
- **Unknown:** whether the stuck state is in the chip's USB-Serial/JTAG
  peripheral (not reset by a software CPU reset) or in host-side USB state
  for the device, and whether every panic triggers it. Earlier panics on this
  board were followed by working sessions, so it may be intermittent.
- **Operator guidance:** after any `reset_reason=panic` or an unexplained
  hello timeout with a ticking display, try a USB-only replug before
  escalating to a power cycle or a debugger.
