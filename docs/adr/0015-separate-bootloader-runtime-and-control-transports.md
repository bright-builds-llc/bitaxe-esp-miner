# Separate Bootloader, Runtime Observation, And Application Control Transports

## Status

Accepted.

## Decision

Repository-owned ESP device workflows use separate transports for separate
responsibilities:

- pinned `espflash` is the bootloader backend for detection, read-only flash
  probes, factory/NVS writes, and explicit post-flash reset;
- a receive-only OS-native reader observes the running firmware and owns bounded
  same-device USB re-acquisition;
- HTTP owns normal application commands and their postcondition checks; and
- typed evidence joins USB identity, application state, and build provenance
  without treating any one transport as proof of every boundary.

For a normal application reboot, authoritative proof is the conjunction of the
same physical USB device, a changed boot session, a software-reset category,
the exact expected build identity, the next reset-retained boot ordinal, and
the requested persisted postcondition. Serial delivery, sampled service loss,
and USB re-enumeration remain useful independent observations but are not
required events when the authoritative conjunction is complete.

The first production lifecycle adapter is macOS-only. Other platforms must fail
explicitly until their identity, ownership, and re-acquisition adapters have
equivalent tests and hardware evidence.

## Rationale

Attempt 26 proved that a valid HTTP restart request, observed service loss, and
complete cleanup can coexist with a zero-byte `espflash monitor` capture. The
repository already held earlier hardware evidence that the receive-only
OS-native reader delivered application heartbeats while passive `espflash`
delivered none. Keeping `espflash` as Phase 35's runtime observer ignored that
qualified boundary.

ESP32-S3 USB Serial/JTAG may disconnect, become temporarily unresponsive, or
re-enumerate across power-management and reset boundaries. A monitor that owns
one device node and one file descriptor therefore cannot represent the complete
device lifecycle. See the ESP-IDF
[USB Serial/JTAG console guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-guides/usb-serial-jtag-console.html).

`esp_restart()` restarts both CPUs through the bootloader and reports a software
reset on the next boot, which makes the firmware's boot session, reset category,
and RTC-backed ordinal appropriate postcondition evidence. See the ESP-IDF
[miscellaneous system API](https://docs.espressif.com/projects/esp-idf/en/stable/esp32s3/api-reference/system/misc_system_api.html).

Bootloader reset flags remain explicit. Watchdog reset is not an automatic
fallback because Espressif documents platform-specific limitations and possible
re-enumeration; it requires its own diagnosed boundary and verified change.
See the esptool [advanced reset options](https://docs.espressif.com/projects/esptool/en/latest/esp32s3/esptool/advanced-options.html).

## Consequences

ADR-0020 extends this separation to TinyUSB application profiles and the
buttonless transition back to the ROM downloader.

- Runtime workflows must not select `espflash monitor` merely because flashing
  already uses espflash.
- A reboot command is sent once. An ambiguous response is followed by bounded
  observation, never by an automatic second reboot request.
- Stable USB nodes and valid same-physical-device re-enumerations are both
  supported outcomes.
- Node names, enumeration identities, sampled downtime, and serial bytes are
  observations rather than substitutes for application postconditions.
- Re-acquisition is restricted to the already admitted physical device; it does
  not authorize mDNS, ARP, router inspection, or network scanning.
- Protected operational material remains private under the repository evidence
  policy, while committed projections contain only typed shareable facts.
- A completed flash write, USB cleanup, original boot-transcript capture, and
  runtime application verification are separate outcomes. Failure to observe a
  startup-only marker does not retroactively turn a verified write into a
  failed flash.
- The original startup-marker validator remains the sole `boot_transcript`
  trust path. A late receive-only attachment may instead trust the versioned
  `runtime_boot_attestation` path after two same-session, same-ordinal samples
  agree on immutable ready-state facts, increase strictly in uptime, and match
  the admitted manifest's full source commit, reference commit, and app ELF
  digest.
- Runtime attestation proves the exact package currently running in ready safe
  state. It never claims historical delivery of the original boot transcript.
- If neither monitor trust path succeeds, the command remains nonzero but
  reports the completed flash independently and does not recommend an
  automatic reflash.
- Direct UART and electrical interfaces remain outside this decision and retain
  their separate authorization requirement.
