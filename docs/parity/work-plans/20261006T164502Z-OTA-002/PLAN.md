# Parity work plan

- Run ID: `20261006T164502Z-OTA-002`
- Parity row: `OTA-002`
- Initial status: `deferred`
- Source commit: `2ce494c053c0107387a9041a1bd5c8c218cc4fcf`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-ota002-www-partition-update`
- Activation authority: explicit user request on 2026-10-06

## Selection

`OTA-002` is `deferred`, so automatic ranking excludes it. The owner reviewed
the queue on 2026-10-06 and explicitly selected it: it is the only remaining
gap an ordinary Ultra 205 owner meets, because the stock AxeOS update page
posts `www.bin` to `/api/system/OTAWWW`
(`reference/esp-miner/main/http_server/axe-os/src/app/services/system.service.ts:380`).

Higher automatic candidates are not substituted for this request. `BAP-001`
and `BAP-002` need a live accessory over direct UART, which stays owner-gated
under `task-parity-bap-live-accessory-verification`. `ASIC-009` and `ASIC-010`
need unsupported board families. This plan does not modify those rows.

Following the STR-005 activation precedent, the row moves `deferred` to
`in-progress` and then, with software evidence only, to `implemented`.

## Scope and non-scope

Implement `POST /api/system/OTAWWW` per
`reference/esp-miner/main/http_server/http_server.c:POST_WWW_update`:
private-network gate (401), AP or APSTA refusal (500 `Not allowed in AP mode`),
`www` SPIFFS partition lookup (500 `WWW partition not found`), oversized-body
refusal (400 `File provided is too large for device`), whole-partition erase
in 64 KiB steps with yields, 1000-byte receive chunks written in order,
timeouts retried, `Protocol Error` and `Write Error` failures, upstream status
labels and the `WWW update complete` response.

Owner-approved divergences:

- A body shorter than the partition is refused with 400 `File provided is too
  small for device` before any erase. Upstream writes it at the end of the
  erased partition, leaving the UI unmountable; release images are always
  exactly partition-sized.
- An erase failure returns `Write Error` instead of aborting through
  `ESP_ERROR_CHECK`.

The bundled operator UI gains a matching `www.bin` upload. Route
classification and the API-compare policy change from the REL-03 gap kind to
an update owner. Historical evidence contracts, including UI-004's committed
`otawww_unavailable` projection, stay unchanged.

Non-scope: any device effect, flash, live OTAWWW request, interrupted-update
test, SPIFFS remount, or verification claim. Hardware work needs its own
contract under the Effectful Hardware Task Gate.

## Implementation

- [ ] Pure `crates/bitaxe-api` core for admission, erase ranges, chunk
      offsets, progress, statuses and responses, with Arrange/Act/Assert tests.
- [ ] Route decision accepts OTAWWW; route manifest, startup report and
      API-compare policy name the update owner.
- [ ] Firmware adapter over `esp_partition_*`, holding the same mutation guard
      as firmware OTA.
- [ ] Bundled UI upload, static UI test and release operator copy.

## Verification and promotion

Commands: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D
warnings`, `cargo build --all-targets --all-features`, `cargo test
--all-features`, `bun scripts/bright-builds-check.ts all`, `just build`,
`just test`, `just parity`, `just parity-progress`.

`implemented` requires the route in firmware, a passing firmware build, and
`unit` evidence for every refusal, error, offset and status path.
`verified` additionally requires `hardware-regression` evidence with an
`interrupted-update` case, as the parity validator demands for OTA-002, under
a separate committed hardware contract and an independent evidence review.
