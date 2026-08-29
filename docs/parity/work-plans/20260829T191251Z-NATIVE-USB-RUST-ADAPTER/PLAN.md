# Native USB Rust Runtime Adapter — Stages 1–2

- Run ID: `20260829T191251Z-NATIVE-USB-RUST-ADAPTER`
- Source base: `902108b7fc5d1941b8734732e6ea8dd6a8350a23`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-native-usb-rust-adapter`
- Blocks: the manual bootstrap and durability portion of
  `task-native-usb-ownership-handoff`

## Objective

Move the repository-owned TinyUSB descriptors, installation, callbacks,
vendor/CDC I/O, and bounded write behavior behind the Rust `UsbRuntime` Module.
Reduce repository-owned C to one internal ESP32-S3 PHY/force-download Adapter.
Preserve the exact Worker USB profile, maintenance protocol, host ownership
contract, and one canonical firmware image.

Underlying pinned ESP-IDF and TinyUSB libraries remain C dependencies. This
child does not remove the final PHY Adapter or perform hardware validation.

## Rust UsbRuntime

Preserve one small firmware Interface:

- `install_worker_runtime()`;
- `send_worker_frame(&[u8])`;
- `emit_evidence(&[u8])`;
- `restart_into_rom_downloader()`.

Use a typed internal failure vocabulary for unavailable transport,
disconnection, partial write, bounded timeout, install failure, and handoff
failure. Keep every raw pointer, packed-structure read, TinyUSB call, and
boot-lifetime descriptor-pointer invariant private to the Adapter.

Add a host-testable pure USB Worker model with the exact current 18-byte device
descriptor, exact 98-byte configuration descriptor, existing strings, and the
existing bounded vendor-write policy with at most 2,000 waits. Build
`tinyusb_config_t` from these static values through the pinned `esp-idf-sys`
bindings without adding a dependency.

Export the TinyUSB mount, unmount, vendor receive, CDC receive, CDC line-coding,
and CDC line-state callbacks directly from Rust. Validate pointer/length input
before slice construction and use unaligned reads for packed callback fields.
Callbacks enqueue ordinary crate-private BWG events. CDC payload input remains
discard-only and class-control events continue through the existing maintenance
reducer.

## Minimal C PHY Adapter

Rename the remaining native source to `usb_phy_handoff.c` and expose only
`int32_t bitaxe_usb_restart_bootloader(void)`. Preserve this ordered operation:

1. register the force-download shutdown handler;
2. uninstall TinyUSB;
3. reset and disable the OTG peripheral;
4. return the internal PHY to USB-Serial-JTAG;
5. restart into ROM.

Keep the pinned Espressif breadcrumb and explicit error returns. Remove all
descriptors, TinyUSB installation, vendor/CDC I/O, callbacks, FreeRTOS delay
logic, BWG delegation, and the obsolete public header from C. Keep only the
existing pinned `espressif/esp_tinyusb` and `esp_hw_support` build dependencies.

## Ownership, tests, and delivery

Update Bazel source sets, BWG conformance checks, and
`just verify-native-usb-ownership` to prove Rust is the sole descriptor,
installation, callback, and data-I/O owner; exactly one TinyUSB install exists;
the C Adapter contains only allowlisted PHY/restart behavior; and vendor payload
handling cannot reach restart or force-download logic.

Use red-to-green vertical slices at the pure descriptor/write Interface, the
Rust callback/source-ownership seam, and the final link-symbol seam. Cover exact
descriptor bytes and fields, string contents, complete/partial/zero/unmounted/
disconnected/timeout writes, pointer validation, CDC discard, callback routing,
one-definition linkage, and absence of removed C symbols.

Before implementation delivery run ordered Cargo formatting, strict Clippy,
all-target/all-feature build, all-feature tests, Bright Builds, focused BWG and
USB tests, all Bazel tests, firmware build/package,
`just verify-native-usb-ownership`, parity/progress, redaction, reference
cleanliness, whitespace, and final diff review. Write `RESULT.md`, archive only
`task-native-usb-rust-adapter`, commit and push one implementation commit, then
rebuild the canonical package from that exact clean pushed commit and verify
source/reference identity.

## Exclusions

Do not run `detect-ultra205`, issue BOOT/RESET instructions, flash, monitor,
recover, open device nodes, or perform any hardware or network effect. Manual
bootstrap, automatic handoff proof, 20-cycle durability, recovery-006
restoration, and final removal of the C PHY Adapter are stages 3 and 4.
