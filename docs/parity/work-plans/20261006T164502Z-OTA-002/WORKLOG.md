# Parity work log

## 20261006T171500Z | software implementation

- Source commit: plan `90231b06`; implementation is the commit that adds
  this entry.
- Actions:
  - Added `crates/bitaxe-api/src/www_update.rs`, the pure OTAWWW core:
    exact-size admission (missing partition 500, oversized 400 upstream copy,
    undersized 400 `File provided is too small for device`), 64 KiB erase
    ranges, 1000-byte receive cursor with upstream offsets, timeout retry,
    `Protocol Error` on a failed or closed stream, upstream progress
    computed before each chunk is counted, yields every 16 chunks, upstream
    status labels and responses.
  - `update_plan.rs` now accepts private, non-AP OTAWWW requests
    (`AcceptOtaWww`) instead of returning the REL-03 gap.
  - The Phase 7 route kind and startup report name the update owner
    (`AxeOsStaticUpdate`, `static_update_routes`); the API-compare policy
    requires that kind.
  - Added `firmware/bitaxe/src/www_update.rs` over `esp_partition_find_first`,
    `esp_partition_erase_range` and `esp_partition_write`; the
    `/api/system/OTAWWW` handler holds the same mutation guard as firmware
    OTA and logs `www_update_status=` lines.
  - The bundled UI replaces the "unavailable" card with a `www.bin` upload
    that reloads two seconds after success, as upstream does.
  - `docs/release/ultra-205.md` documents the operator behavior and the
    interrupted-upload recovery paths.
  - The UI-004 projector's pinned `static-ui.test.ts` digest is updated, as
    STAT-003 did in `a337babc`; the committed UI-004 projection, its
    `otawww_unavailable` attestation and its validator are unchanged.
- Verification: `cargo fmt --all`; `cargo clippy --all-targets
  --all-features -- -D warnings`; `cargo build --all-targets --all-features`;
  `cargo test --all-features` (2,683 passed); `bun
  scripts/bright-builds-check.ts all` (zero findings); `just build`; `just
  test` (296 passed); `just verify-redaction`; `just parity`
  (`validation_errors: none`). A one-off `#![deny(warnings)]` firmware build
  showed no warnings from the new files.
- Evidence: `unit` (16 `www_update` tests, updated `update_plan`,
  `route_shell`, `api_compare` and static UI tests).
- Outcome: software implementation complete; supports `implemented`.
- Blocker or next safe action: no device effect has run. `verified` needs a
  committed hardware contract for one exact-package OTAWWW update with reboot
  and served-asset identity proof plus one interrupted upload and its
  NVS-preserving recovery, then an independent evidence review.
  Residual risks: the mounted SPIFFS is rewritten in place and may serve
  stale or corrupt reads until reboot, like upstream; the mutation guard is
  not exclusive, so an already queued restart could still fire mid-erase; the
  HTTP server task is blocked for the whole erase and write; a client that
  stalls without closing is retried indefinitely, as upstream does.
