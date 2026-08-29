# Native USB Rust Adapter worklog

## 2026-08-29T19:12:51Z | Immutable plan

- Source base: `902108b7fc5d1941b8734732e6ea8dd6a8350a23`.
- Immutable plan SHA-256:
  `e5b4c29fe359a617a6caf12435db4201a0581957437411ad214e1395d7722c50`.
- Plan commit: `88b24d20509f1c993c410c69d02efd1d9896d926`.
- Hardware/network effects: none.

## 2026-08-29T19:37:57Z | Rust ownership implementation

- Added exact host-tested Worker device/configuration descriptors, retained
  strings, and bounded vendor-write progress to the pure core.
- Moved TinyUSB configuration, installation, callbacks, vendor responses, CDC
  evidence, pointer validation, and packed line-coding reads into the private
  Rust `UsbRuntime` Adapter.
- Reduced native C to the ordered ESP32-S3 PHY/force-download handoff and one
  exported restart function.
- Added source-ownership and linked-ELF symbol checks. Focused pure, BWG
  conformance, source-ownership, firmware build, and native-USB guard checks
  pass.
- Hardware/network effects: none. Full repository verification is pending.

## 2026-08-29T19:45:57Z | Software closure

- Ordered Cargo formatting, strict Clippy, all-target/all-feature build, and
  all-feature tests passed.
- Bright Builds, all 70 Bazel tests, the real firmware build, six-artifact
  package, native-USB source and linked-symbol guards, parity/progress,
  redaction, reference cleanliness, and whitespace checks passed.
- `RESULT.md` records the closed software outcome and exact non-claims. The
  completed child task moved atomically to `TASKS.archive.md`.
- Hardware/network effects: none. The clean pushed package identity rebuild is
  the remaining post-commit delivery step.
