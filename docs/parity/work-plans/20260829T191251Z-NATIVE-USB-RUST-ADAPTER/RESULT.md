# Native USB Rust Runtime Adapter result

## Outcome

Stages 1–2 are complete. The deep Rust `UsbRuntime` Module owns the exact
Worker descriptors, boot-lifetime descriptor pointers, TinyUSB installation,
all TinyUSB callbacks, bounded vendor responses, CDC evidence, and typed
failure mapping. The BWG Worker receives ordinary Rust events and contains no
USB FFI exports.

Repository-owned C is reduced to `usb_phy_handoff.c`, one minimal Adapter that
exposes `bitaxe_usb_restart_bootloader` and retains the ordered uninstall,
ESP32-S3 PHY switch, force-download shutdown handler, and restart sequence.

## Verification

- Seven pure descriptor/write tests prove exact 18-byte and 98-byte descriptor
  compatibility, strings, endpoint/profile fields, partial completion,
  unavailable transport, zero-progress timeout, partial timeout, and impossible
  progress rejection.
- BWG conformance and source-ownership tests prove Rust ownership, CDC payload
  discard, callback pointer checks, packed unaligned line-coding reads, and the
  absence of descriptor/callback/data ownership from C.
- The linked-ELF test requires each TinyUSB callback and
  `bitaxe_usb_restart_bootloader` exactly once and rejects every retired C ABI
  symbol.
- Ordered Cargo formatting, strict Clippy, all-target/all-feature build, and
  all-feature tests passed. Bright Builds and all 70 Bazel tests passed.
- The canonical ESP32-S3 firmware and six-artifact package, native-USB guard,
  parity/progress, redaction, pinned-reference cleanliness, and whitespace
  checks passed.

## Non-claims

No detector, USB device node, BOOT/RESET action, flash, monitor, recovery,
credential, network, mining, ASIC, fan, voltage, or other hardware effect
occurred. This result does not remove the final C PHY Adapter or qualify the
buttonless route. Manual bootstrap, automatic handoff proof, 20-cycle
durability, recovery-006 restoration, and final zero-repository-C conversion
remain stages 3 and 4.
