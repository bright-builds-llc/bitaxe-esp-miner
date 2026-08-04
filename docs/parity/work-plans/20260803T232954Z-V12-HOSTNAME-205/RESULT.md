# V12-HOSTNAME-205 work result

- Parity row: `V12-HOSTNAME-205`
- Final status: `verified`
- Implementation and package commit:
  `cb0fe1f78ad8dd82ec815069739572053fa54c22`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Board: Ultra 205
- Hardware attempts after the typed correction: one

## Evidence and verification

The detector-gated `attempt-003` ran the existing
`just verify-settings-durability --mode capture` interface against the exact
clean package built from the pushed implementation commit. The private
detector admitted exactly one Ultra 205 before the workflow performed an exact
package flash, confirmed the safe disabled boot, changed a non-secret test
hostname, and confirmed immediate readback.

The committed
[`bitaxe-settings-durability-evidence-v2`](../../evidence/v12-hostname-205/durability-projection.json)
projection has SHA-256
`9325d9f02102e8d0fd4f8b0cb887fde4af924ae417e19bae5e0ac6c9bd3c29c5`.
Its closed device-session projection proves:

- macOS observation of board 205 and the same physical USB device;
- three-sample device admission, an armed reader, and pre-restart serial
  delivery before the restart request;
- exactly one complete restart request and an HTTP response;
- service loss followed by recovery of the exact source, reference, and
  application ELF identities;
- a changed boot session, boot ordinal `N+1`, and software reset category;
- the expected post-restart hostname SHA-256, correlated serial delivery, and
  complete holder-free cleanup;
- confirmed restoration of the private original hostname; and
- mining disabled, hardware control disabled, and redaction passed.

The corrected host workflow adds `esp-device-session-reboot-intent-v1` and
`device-session reboot-live`, derives the physical USB identity inside the
same process, and reuses the typed reader-armed restart transaction. It removes
the standalone monitor process, fixed readiness delay, duplicate restart POST,
and dependency on a nonexistent `flash-monitor.log`. Unit tests cover every
non-ready terminal category, malformed or missing projections, timeout and
launch failures, restoration and exact-package recovery, primary-failure
precedence, and public redaction. A real child-process regression proves that
production-shaped monitor behavior cannot reintroduce an invented-file
dependency.

The following gates passed before the hardware attempt and again after the
final projection-validation hardening:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test`
- `just parity`
- `just parity-progress`
- `just verify-redaction`
- `just verify-reference`
- `git diff --check`

## Conclusion

The exact pushed Rust package persisted the hostname across one attributed
normal software restart on the detector-admitted Ultra 205, then restored and
confirmed the original private hostname. This satisfies the narrow
`V12-HOSTNAME-205` observable durability claim with typed workflow and hardware
smoke evidence.

## Privacy, recovery, and non-claims

USB identities and paths, serial and HTTP traces, device origins, hostnames,
network identifiers, and Wi-Fi credentials remain only in ignored mode-`0700`
private roots with mode-`0600` files. The committed projection contains only
closed booleans, bounded counts, category labels, and cryptographic identities.
The credential file was passed as an opaque local input and was never read,
printed, summarized, or copied. Normal restoration passed, so no recovery flash
ran. Earlier failed attempts remain recorded and are not reinterpreted as
successful evidence.

This result does not claim broader configuration parity, network longevity,
mining, ASIC behavior, voltage, fan, thermal or power control, OTA or recovery,
other boards, release readiness, direct UART, or pin manipulation.
