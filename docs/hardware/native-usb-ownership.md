# Native USB ownership

Read this before changing TinyUSB, USB descriptors/sdkconfig, firmware startup,
detection, flashing, monitoring, or recovery.

Profiles are `worker_runtime`, `serial_jtag_runtime`, and `rom_downloader`.
VID/PID and product describe profile identity. Stable connector location plus a
common serial describe physical identity. Device nodes and registry epochs are
enumeration identity.

The buttonless flow is Worker runtime → guarded 1200-baud/DTR maintenance
gesture → safe stop → ROM downloader → admitted flash → expected runtime.
Monitoring never arms handoff, and CDC payload bytes never request it.

Run `just verify-native-usb-ownership` for software changes. Hardware changes
remain incomplete until the active task records 20 automatic cycles with
stable physical identity, exact package recovery, cleanup, and redaction.
