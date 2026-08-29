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

Manual BOOT/RESET is bootstrap or last-resort recovery. OTA is optional, not
the sole recovery path. One canonical image serves development and production.
