# Developer-Raw USB Evidence And Wi-Fi Flash Permission

## Summary

- Added developer-raw flash evidence as the default and `redact-evidence=true` for commit-ready evidence.
- Added trusted USB flash-monitor `device_url` extraction for Phase 17 HTTP and WebSocket helpers.
- Updated repo-local hardware rules and Ultra 205 operator docs for local Wi-Fi credential use and raw evidence boundaries.

## Verification

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bazel test //tools/flash:tests //scripts:phase17_live_http_api_smoke_test`
- `bazel build //firmware/bitaxe:firmware_image`
- Hardware UAT with `just detect-ultra205`, developer-raw `just flash-monitor`, HTTP smoke via `--use-flash-log-device-url`, and WebSocket captures via `--device-url-from-flash-evidence`.

## Hardware UAT Notes

- Flash evidence under `target/phase17-dev-raw-usb-target/` is gitignored and not commit-ready.
- USB flash-log target extraction passed and created a sanitized target lock.
- HTTP route probes reached the device but remain blocked by route expectation/status mismatches.
- WebSocket captures accepted the USB flash evidence target and opened, then reported pending connection-error frame status.
