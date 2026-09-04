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

Transport profile and execution owner are independent. The shared Espressif
USB-Serial-JTAG descriptor can belong to ROM or to the running application;
descriptors alone leave that owner unknown. A current `board-info` exchange
admits ROM for that observation epoch, while a build-bound boot-profile or
runtime-attestation marker admits the application. Worker descriptors admit
the application because only the project firmware exposes that exact tuple.

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
Host control-I/O failures retain the closed maintenance-step label and numeric
errno privately so a retry can target the failed boundary instead of repeating
the same undifferentiated `handoff_unsupported` result.

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
  available. Ordinary Serial/JTAG inspection reports unknown execution
  ownership without sending synchronization traffic. Explicit ROM admission
  uses `just detect-ultra205 --retain-rom`; it is never an implicit consequence
  of observing the shared descriptor. Recovery after a manual BOOT/RESET entry uses
  `just detect-ultra205 --retain-rom`; this runs read-only `board-info` with
  `--before no-reset --after no-reset` so detector admission does not discard
  the already-established ROM profile before restoration.
- Flash and recovery first acquire the physical lease. Worker performs the
  guarded handoff; Serial/JTAG enters ROM directly. Every write requires a
  successful ESP32-S3 `board-info`, then uses `--before no-reset` (or the
  managed esptool equivalent). This prevents DTR/RTS reset traffic while still
  allowing each fresh flashing process to synchronize with the already
  admitted ROM downloader. Synchronization never reaches Worker CDC because
  the profile handoff and ROM admission precede the flashing process.
- Observe selects the receive-only Adapter for Worker or Serial/JTAG and never
  arms maintenance. On macOS the Adapter opens the callout node read-only,
  configures it as raw 115200 serial, enables local receive, and disables
  hang-up-on-close. It does not write payload bytes, issue modem-control
  `ioctl`s, change DTR/RTS, or select the 1200-baud maintenance rate. Raw mode
  is required even for receive-only operation: otherwise the terminal line
  discipline may withhold binary or partial evidence until a newline arrives.
  Profile accessibility probes use this same no-hangup open path; they must not
  perform a separate default-terminal open/close cycle before monitoring.
- After a write, the same lease reacquires the expected application profile
  and remains responsible for process-group cleanup and final holder checks.
- A software handoff that set `RTC_CNTL_FORCE_DOWNLOAD_BOOT` exits ROM through
  the contained managed esptool ESP32-S3 hard-reset path, which clears that
  force bit before reset. `run` with `--after no_reset` is prohibited because
  it intentionally retains the serial bootloader.

Initial production support is macOS. Linux and Windows expose the same
Interface but fail closed until their inventory, identity, serial-control, and
process adapters receive separate qualification.

## Closed failures and evidence

Native ownership adds `runtime_profile_unknown`, `handoff_unsupported`,
`handoff_rejected_unsafe_state`, `handoff_ready_timeout`,
`handoff_commit_timeout`, `bus_reset_timeout`, `same_worker_after_commit`,
`handoff_transition_timeout`, `bootloader_ambiguous`,
`physical_identity_drift`, `bootloader_sync_failed`, `rom_admission_failed`,
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

The task-gated single-transition discriminator is
`just verify-native-usb-transition`. It holds one physical lease across
Worker readiness, acknowledged commit, Serial/JTAG re-enumeration, a
read-only ESP32-S3 `board-info`, hard reset, and Worker reappearance. It never
loads or writes an image. Its candidate result remains protected until
`just native-usb-transition-recovery finalize` joins it to exact primary and
final recovery-006 results; only that finalizer may publish the redacted
projection. Recovery `preflight` validates the historical bundle, public
readiness projection, private validator receipt, managed ESP-IDF tools, exact
current package, and restore admission without leaving a private root or
touching the device.

If recovery writes complete but a late passive monitor misses one-shot boot
evidence, the same `start` Interface may continue only after validating the
protected completed snapshot and NVS receipts. That continuation records a
consume-once private intent, permits no repeated write, performs one admitted
hard reset, and immediately attaches the receive-only observer. Missing or
partial receipts, source-lineage drift, a prior continuation, or a completed
result all fail closed.

`just native-usb-display-recovery` is the recovery-only fallback when the
qualified USB observer carries no application bytes but the running Bitaxe
display shows its private IPv4 address. It accepts that address directly from
the operator, obtains the ESP32-S3 base MAC through admitted USB `board-info`,
and requires the API station MAC digest and complete recovery identity to
match before sending a settings request. Local development UI and console
output may show the address. Public evidence remains redacted. This fallback
does not perform or authorize mDNS, ARP, router lookup, subnet scanning,
hostname discovery, flashing, NVS writes, or a transition diagnostic.

When the display has no station address, the task-gated
`just native-usb-config-ap-recovery` Interface is the only configuration-AP
continuation. Its first stage retains one admitted USB session, reads exactly
the NVS partition range `0x9000..0xefff`, and compares the protected result to
a freshly generated ordinary seed. It performs no device write and no host
network change. Configuration-AP association remains prohibited unless that
sealed discriminator reports `nvs_match`; later recovery must use the same
Interface and the exact USB-derived AP candidate, never general Wi-Fi,
hostname, ARP, mDNS, router, or subnet discovery.

`just native-usb-rom-exit` is the no-write discriminator for the shared
Serial/JTAG transport. It reads only the force-download bit, performs one
contained hard-reset ROM exit, and requires Worker descriptors or exact
application evidence before naming an execution owner.

`just native-usb-owner-recovery` is the passive-first successor when the
shared transport is already visible but explicit ROM admission fails. It first
requires two exact recovery runtime attestations without transmitting serial
data. Only missing or insufficient evidence may advance to one built-in
BOOT/RESET checkpoint, one admitted ROM observation, and one managed hard-reset
application exit. Malformed, inconsistent, or package-mismatched evidence is
terminal and cannot be bypassed with the manual branch.

`just native-usb-boot-chain-integrity` is the read-only successor after both
application observations are byte-empty. It uses one manual ROM entry to
compare bootloader, partition table, OTA selection data, and exactly the
selected application with the immutable recovery snapshot. It never repairs a
range or treats a valid flash comparison as proof that the application ran.

During temporary USB stabilization, `just usb-stability-read` exercises a
known recovery-006 factory slice through the ROM loader without the flasher
stub. It pins the 16 MiB flash size, verifies every bounded chunk, holds the
physical-device lease, and returns to the application without writing flash.
Use a fresh ignored root for each calibration and change only one transport
variable between failed runs.
Use `--pattern repeated` for connection durability against one slice and
`--pattern sequential` for bounded full-partition coverage.
The first accepted no-stub baseline is recorded at
`docs/parity/evidence/native-usb-stability/usb-stability-baseline-v1.json`.

When a Worker node repeatedly disappears before normal session admission,
`just diagnose-usb-reboot-loop --port <worker-port> --timeout-seconds 15`
retains the physical connector and reopens only the receive-only Adapter. Each
Worker mount emits a closed `boot_ordinal`, reset-reason category, and uptime
marker without requiring DTR in firmware. The dedicated macOS observer opens
the Worker CDC read/write only because the driver requires it for class
control, fixes raw 115200, asserts DTR while observing, and clears DTR on drop.
It contains no 1200-baud path and sends no payload, so it cannot arm Worker
maintenance. Increasing ordinals prove a chip reset; one ordinal with
increasing uptime proves a USB-stack-only reset. The diagnostic is bounded to
30 seconds, captures at most 64 KiB, performs no network action, and publishes
no raw identity.

When the closed reset family is `panic`, a boot-installed Rust panic hook
stores only an FNV-1a source-path digest and one-based source line in RTC
no-init memory. The next boot consumes and clears that integrity-checked
receipt and replays it over Worker CDC. A present receipt identifies a Rust
panic location without persisting panic text; a missing receipt across later
panic resets places the failure below Rust. This path never writes flash, so a
rapid reboot loop cannot create coredump wear.
The same Adapter registers ESP-IDF's allocation-failure callback and retains
only the requested byte count and capability mask. It performs one volatile
RTC write and no allocation, logging, lock, or flash operation. This separates
allocator aborts from other ESP-IDF panic paths without enabling repeated
coredump writes.

Manual BOOT/RESET is a one-time bootstrap or last-resort recovery path, never
the normal development workflow. Buttonless flashing is not qualified until
the active task records 20 automatic Worker-to-ROM-to-flash-to-Worker cycles
with stable physical identity, exact package restoration, complete cleanup,
and redacted evidence. OTA may accelerate application updates but cannot be
the only recovery route.
