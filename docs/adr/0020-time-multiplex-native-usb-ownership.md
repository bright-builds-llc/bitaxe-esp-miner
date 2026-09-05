# ADR-0020: Time-multiplex native USB ownership

## Status

Accepted

## Decision

One boot-lifetime `UsbRuntime` Module owns the ESP32-S3 internal USB PHY.
Ordinary safe boots expose the TinyUSB Worker vendor/CDC profile; task-gated
diagnostic and recovery boots retain USB-Serial-JTAG. TinyUSB accepts only a
guarded CDC class-control maintenance gesture and returns the PHY to the ROM
downloader after safe stop. CDC payload input remains non-command data.

One host `UsbOwnership` Module classifies Worker runtime, Serial/JTAG runtime,
and ROM downloader profiles. Flash and recovery retain one physical-device
lease across profile changes and run `espflash` only after ROM admission.

The maintenance commit is acknowledged: after readiness, the host returns
line coding from 1200 to 115200 while DTR remains asserted. Firmware emits one
fixed committed receipt before PHY mutation; the host clears DTR and closes
CDC only after that receipt arrives. The ESP32-S3 PHY Adapter drives D-/D+ low, reconnects
USB-Serial-JTAG, and requires a bounded observed BUS_RESET before restart.
Protected transition evidence records only closed profile categories and never
raw device identity.

ROM exit is a separate capability from flash transfer. The managed ESP32-S3
esptool reset clears the force-download bit before application reset; a serial
line reset alone does not establish application return. Canonical flashing
must use that contained exit after its final write. A task-gated no-write exit
may start an already installed image, but only observed exact runtime identity
establishes which application ran.

Manual BOOT/RESET is bootstrap or last-resort recovery. OTA is optional, not
the sole recovery path. One canonical image serves development and production.
