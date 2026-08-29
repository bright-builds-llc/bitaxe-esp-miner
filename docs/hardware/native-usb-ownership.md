# Native USB ownership

Read this before changing TinyUSB, USB descriptors or sdkconfig, firmware
startup, detection, flashing, monitoring, or recovery. The ESP32-S3 has one
internal USB PHY, so the repository time-multiplexes that PHY instead of
maintaining separate development and production firmware.

## Profiles and identity

| Profile | Owner | Allowed host behavior |
| --- | --- | --- |
| `worker_runtime` | Boot-lifetime `UsbRuntime` using TinyUSB vendor control plus CDC evidence | Inspect, receive-only monitor, or guarded maintenance handoff |
| `serial_jtag_runtime` | ESP-IDF USB-Serial-JTAG retained for consume-once diagnostics, recovery, and unconfirmed safe baselines | Inspect, receive-only monitor, or direct automatic reset into ROM |
| `rom_downloader` | ESP32-S3 ROM | `board-info`, admitted flash, and recovery writes |
| `unknown` | Unqualified | Fail closed |

Profile identity comes from exact USB descriptors and, for `rom_downloader`, a
successful ESP32-S3 `board-info` exchange. Physical identity is the stable
macOS connector location, falling back to a stable USB serial only when the
location is unavailable. VID/PID, product strings, device nodes, inode data,
and registry epochs are profile or enumeration identity and never enter the
physical digest. One `UsbSession` lease remains held while those fields change.

## Firmware handoff

Ordinary confirmed-safe boots select `worker_runtime`. A consume-once TCP,
Noise, or self-test admission selects `serial_jtag_runtime`; an unconfirmed
boot-safe baseline also retains Serial/JTAG by withholding TinyUSB startup.

The only Worker-to-ROM command channel is this CDC class-control sequence:

1. The host opens the admitted Worker CDC node, primes DTR low at 115200 baud,
   and allows those callbacks to settle. It then asserts one DTR arm edge,
   allows that callback to settle, and sets exact 1200-baud line coding. This
   avoids duplicate DTR assertions from macOS open semantics.
2. TinyUSB callbacks enqueue line-coding and line-state events. They perform no
   safe stop, PHY mutation, or restart.
3. The Rust owner closes Worker ingress, rejects an already-active Worker
   effect, and requires Production Mining Session safe-stop success.
4. The owner emits `usb_maintenance={"status":"ready"}` on CDC evidence.
5. After observing that exact bounded receipt, the host clears DTR but keeps
   CDC open. The owner accepts the falling edge and emits
   `usb_maintenance={"status":"committed"}` before PHY mutation. Failure to
   emit that receipt disarms without rebooting.
6. The host requires the committed receipt before closing CDC. A missing
   receipt is `handoff_commit_timeout`, distinct from readiness and ROM-profile
   transition failures.
7. The owner uninstalls TinyUSB, drives the ESP32-S3 D-/D+ pads low, returns
   the internal PHY to USB-Serial-JTAG, and waits up to one second for an
   observed BUS_RESET. Only then does the registered force-download shutdown
   handler restart into ROM; timeout returns an explicit handoff failure.

Wrong ordering, a duplicate event, timeout, disconnect, active effect, failed
safe stop, missing receipt, or control I/O failure disarms without rebooting.
CDC payload bytes remain non-command data under ADR-0018.

## Firmware implementation ownership

The pure `bitaxe-core` USB Worker model owns the exact device/configuration
descriptor bytes and bounded vendor-write progress. The firmware `UsbRuntime`
Module owns descriptor lifetimes, TinyUSB installation, exported TinyUSB
callbacks, vendor responses, CDC evidence, and typed failures through private
`esp-idf-sys` calls. The BWG Worker is an Adapter that receives ordinary Rust
events and never owns USB FFI.

Repository-owned C is intentionally limited to `usb_phy_handoff.c`. That
Adapter registers the force-download shutdown handler, uninstalls TinyUSB,
switches the ESP32-S3 internal PHY to USB-Serial-JTAG, and restarts. It owns no
descriptor, callback, vendor/CDC data path, or Worker delegation. Removal of
this final C Adapter is a later stage after hardware durability evidence.

The PHY and force-download sequence is the minimum ESP32-S3 behavior adapted
from Espressif's pinned Arduino implementation at commit
`bb0bb3ec57fbcf7efb8409f727fb792e3d28fe79`. See
`cores/esp32/esp32-hal-tinyusb.c` and `cores/esp32/USBCDC.cpp` in
`espressif/arduino-esp32`.

## Host operations

All profile crossings happen behind the existing repo-owned commands:
`just detect-ultra205`, `just flash`, `just monitor`, `just flash-monitor`, and
the recovery commands.

Monitoring never arms handoff.

- Inspect reports a known Worker profile without pretending `board-info` is
  available. Serial/JTAG inspection may use `board-info` to distinguish ROM.
- Flash and recovery first acquire the physical lease. Worker performs the
  guarded handoff; Serial/JTAG enters ROM directly. Every write requires a
  successful ESP32-S3 `board-info`, then uses `--before no-reset` (or the
  managed esptool equivalent). This prevents DTR/RTS reset traffic while still
  allowing each fresh flashing process to synchronize with the already
  admitted ROM downloader. Synchronization never reaches Worker CDC because
  the profile handoff and ROM admission precede the flashing process.
- Observe selects the receive-only Adapter for Worker or Serial/JTAG and never
  arms maintenance.
- After a write, the same lease reacquires the expected application profile
  and remains responsible for process-group cleanup and final holder checks.

Initial production support is macOS. Linux and Windows expose the same
Interface but fail closed until their inventory, identity, serial-control, and
process adapters receive separate qualification.

## Closed failures and evidence

Native ownership adds `runtime_profile_unknown`, `handoff_unsupported`,
`handoff_rejected_unsafe_state`, `handoff_ready_timeout`,
`handoff_commit_timeout`, `handoff_transition_timeout`, `bootloader_ambiguous`,
`physical_identity_drift`, `bootloader_sync_failed`,
`application_reappearance_timeout`, and `recovery_required`. Existing
flash/recovery/cleanup failures remain available, and the earliest failure is
never replaced by a later cleanup error.

Raw addresses, ports, descriptors, identities, serial data, process data, and
timestamps remain in ignored mode-`0600` artifacts under mode-`0700` roots.
Public output contains only closed categories, booleans, bounded counts and
timings, and safe digests.

Each Worker-to-ROM transition also writes a bounded protected profile trace
using only `absent`, `same_worker`, `same_serial_jtag`, `same_unknown`, and
`physical_mismatch`. This trace distinguishes a wrong-profile same-device
observation from true absence without recording ports, descriptors, location
IDs, serial numbers, or physical digests.

## Development and recovery

Run `just verify-native-usb-ownership` after every software change in this
surface. It checks the sole TinyUSB owner, diagnostic Serial/JTAG retention,
the maintenance reducer, centralized host routing, linked callback/PHY symbols,
and agent/ADR/docs guardrails through Bazel.

Manual BOOT/RESET is a one-time bootstrap or last-resort recovery path, never
the normal development workflow. Buttonless flashing is not qualified until
the active task records 20 automatic Worker-to-ROM-to-flash-to-Worker cycles
with stable physical identity, exact package restoration, complete cleanup,
and redacted evidence. OTA may accelerate application updates but cannot be
the only recovery route.
