# AxeOS feature inventory

This inventory maps upstream AxeOS features to the two web UI variants
([ADR-0034](../adr/0034-web-ui-variants-and-build-flag.md)) and to the firmware
API. It is the parity plan for both variants.

Upstream Angular AxeOS is a feature reference only. It is never built, bundled
or copied. The upstream paths below are relative to
`reference/esp-miner/main/http_server/axe-os/src/app`. Route registration in
our firmware is in `firmware/bitaxe/src/http_api.rs` (`register_http_handlers`),
`firmware/bitaxe/src/static_files.rs` and `phase07_routes()` in
`crates/bitaxe-api/src/route_shell.rs`.

Status values:

- **yes**: present.
- **partial**: present with a narrower scope, explained in the notes.
- **no**: absent and planned.
- **excluded**: deliberately absent. The notes give the reason and what would
  lift it.

`current` is the handwritten UI in `firmware/bitaxe/static/www`. `solid` is the
SolidJS port in `firmware/bitaxe/web/solid`. Today `solid` matches `current`
feature for feature, and both share the contract tests in
`firmware/bitaxe/web/solid/test`.

## Pages and workflows

| Feature                               | Upstream source                                                                    | Firmware API                                          | current  | solid    | Notes                                                                                                                                                                                                                                                                            |
| ------------------------------------- | ---------------------------------------------------------------------------------- | ----------------------------------------------------- | -------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Dashboard summary                     | `components/home/`                                                                 | GET `/api/system/info` (yes)                          | partial  | partial  | Shows the current state, hashrate, temperature, power, shares, fan and uptime, refreshed on demand. Upstream uses the `/api/ws/live` stream.                                                                                                                                     |
| Dashboard history chart               | `components/home/` (`chartY1Data`, `chartY2Data`)                                  | GET `/api/system/statistics` (yes)                    | no       | no       | Plan: a dual-axis chart in `solid` first. Watch the gzip budget, and keep chart choices out of browser storage (upstream keeps them in localStorage).                                                                                                                            |
| Live telemetry stream                 | `services/live-data.service.ts`                                                    | WS `/api/ws/live` (yes)                               | no       | no       | Plan: replace the manual Refresh with the live stream. Needs a bounded reconnect policy.                                                                                                                                                                                         |
| Dashboard widget layout and edit mode | `services/dashboard-edit.service.ts`, `layout/app.topbar.component.html`           | none                                                  | excluded | excluded | Upstream stores the layout in localStorage. Both variants keep the no-browser-persistence rule. Reconsider only with a device-side store.                                                                                                                                        |
| Pause, resume and restart             | `layout/app.topbar.component.ts`                                                   | POST `/api/system/pause`, `/resume`, `/restart` (yes) | yes      | yes      | Every command asks for confirmation first.                                                                                                                                                                                                                                       |
| Primary/fallback pool switch          | `components/home/` (`onPoolChange`)                                                | PATCH `/api/system` `useFallbackStratum` (yes)        | no       | no       | Plan: add with the fallback pool form.                                                                                                                                                                                                                                           |
| Block-found notice and dismiss        | `components/confetti/`, `home.component.ts`                                        | POST `/api/system/blockFound/dismiss` (yes)           | no       | no       | Plan: a plain banner without the confetti effect, to save size.                                                                                                                                                                                                                  |
| Share rejection help                  | `services/share-rejection-explanation.service.ts`                                  | GET `/api/system/info` (yes)                          | no       | no       | Plan: static help text keyed by rejection reason.                                                                                                                                                                                                                                |
| Hide sensitive data                   | `services/sensitive-data.service.ts`, `pipes/address.pipe.ts`                      | none                                                  | partial  | partial  | Pool passwords and the Wi-Fi password are write-only and never loaded. Worker names and addresses are not masked. Plan: a masking toggle.                                                                                                                                        |
| Scoreboard (best difficulty)          | `components/scoreboard/`                                                           | GET `/api/system/scoreboard` (yes)                    | yes      | yes      | Bounded, exact, descending wire shape. No sort control: upstream keeps the sort order in localStorage.                                                                                                                                                                           |
| Logs with live stream                 | `components/logs/`, `services/web-socket.service.ts`                               | GET `/api/system/logs`, WS `/api/ws` (yes)            | yes      | yes      | Retained logs, then the live stream. Filter, pause, download and clear. No ANSI colouring yet (`pipes/ansi.pipe.ts`). Plan: colouring within the budget.                                                                                                                         |
| System and ASIC information           | `components/system/`                                                               | GET `/api/system/info`, `/api/system/asic` (yes)      | partial  | partial  | Hostname, board, ASIC, Wi-Fi state, IP addresses and firmware provenance. Plan: the read-only ASIC table.                                                                                                                                                                        |
| Identify (blink)                      | `components/system/` (`identifyDevice`)                                            | POST `/api/system/identify` (yes)                     | no       | no       | Plan: a confirmed Identify button. The firmware behavior has its own parity evidence.                                                                                                                                                                                            |
| Pool settings (primary)               | `components/pool/`                                                                 | PATCH `/api/system` (yes)                             | partial  | partial  | Protocol (SV1 only), host, port, worker and a write-only password. Plan: TLS, certificate, extranonce subscribe, suggested difficulty, coinbase decoding and Stratum V2 fields, once their firmware rows are verified.                                                           |
| Pool settings (fallback)              | `components/pool/` (`fallbackStratum*`)                                            | PATCH `/api/system` (yes)                             | no       | no       | Plan: a mirror of the primary form.                                                                                                                                                                                                                                              |
| Network settings                      | `components/network-edit/`                                                         | PATCH `/api/system` (yes)                             | yes      | yes      | Hostname, SSID and a write-only password, with a status card.                                                                                                                                                                                                                    |
| Wi-Fi scan                            | `components/network/` (`scanWifi`)                                                 | GET `/api/system/wifi/scan` (yes)                     | no       | no       | Plan: a scan dialog. General network scanning stays out of agent workflows (AGENTS.md); this is a user-initiated device scan.                                                                                                                                                    |
| Captive AP mode route                 | `app-routing.module.ts` (`ap`), `guards/ap-mode.guard.ts`                          | static routes (yes)                                   | partial  | partial  | `/ap` opens the network page. There is no AP-mode guard redirect.                                                                                                                                                                                                                |
| Telemetry sample frequency            | `components/edit/` (`statsFrequency`)                                              | PATCH `/api/system` (yes)                             | yes      | yes      | Settings page.                                                                                                                                                                                                                                                                   |
| Frequency and core voltage            | `components/edit/` (`frequency`, `coreVoltage`, `?oc`, `overclockEnabled`)         | PATCH `/api/system`, GET `/api/system/asic` (yes)     | excluded | excluded | Hardware controls stay out of the UI until their live safety evidence is complete (AGENTS.md safety constraint). ASIC-007 and PWR-003 list arbitrary targets as non-claims. The API does not block these fields, so the exclusion is enforced by the UI's patch field list only. |
| Fan, auto-fan and temperature target  | `components/edit/` (`autofanspeed`, `minfanspeed`, `manualFanSpeed`, `temptarget`) | PATCH `/api/system` (yes)                             | excluded | excluded | Same safety exclusion. THR-001 to THR-003 list dynamic retuning as a non-claim.                                                                                                                                                                                                  |
| Overheat mode and its reset           | `components/edit/` (`overheat_mode`)                                               | PATCH `/api/system` (yes)                             | excluded | excluded | Same safety exclusion.                                                                                                                                                                                                                                                           |
| Display settings                      | `components/edit/` (`display`, `rotation`, `invertscreen`, `displayTimeout`)       | PATCH `/api/system` (yes)                             | no       | no       | Not a power or thermal control. Plan: add once UI-001 and UI-002 display behavior covers runtime changes.                                                                                                                                                                        |
| Theme                                 | `components/design/`, `services/theme.service.ts`                                  | GET/POST `/api/theme` (yes)                           | yes      | yes      | Dark by default, with a light scheme and an accent colour, saved on the device.                                                                                                                                                                                                  |
| Firmware update (`esp-miner.bin`)     | `components/update/`                                                               | POST `/api/system/OTA` (yes)                          | yes      | yes      | Exact file name, then a separate confirmation.                                                                                                                                                                                                                                   |
| Web UI update (`www.bin`)             | `components/update/`                                                               | POST `/api/system/OTAWWW` (yes)                       | yes      | yes      | Exact file name, then a confirmation; reloads after success. Either variant's `www.bin` can replace the other.                                                                                                                                                                   |
| GitHub release check and notes        | `services/github-update.service.ts`                                                | external `api.github.com`                             | excluded | excluded | The device UI makes no third-party requests. Plan: point to this repository's releases with a link only.                                                                                                                                                                         |
| Swarm (multi-device)                  | `components/swarm/`                                                                | other devices' `/api/system/*`                        | excluded | excluded | Needs subnet scanning and cross-origin requests to other devices. AGENTS.md prohibits general network discovery in agent workflows, and CORS preflight (OPTIONS) is not implemented. Reconsider with an explicit owner decision.                                                 |
| Whitepaper link                       | `src/bitcoin.pdf`                                                                  | static `/bitcoin.pdf`                                 | excluded | excluded | Would add a large binary to the 3 MiB partition. An external link is acceptable later.                                                                                                                                                                                           |
| Seasonal snow effect                  | `components/snowflakes/`                                                           | none                                                  | excluded | excluded | Cosmetic. Not worth its size.                                                                                                                                                                                                                                                    |
| Firmware provenance in product chrome | none (Bright Builds rule)                                                          | GET `/api/system/info` (yes)                          | yes      | yes      | Version, short commit with a link and build time, showing `Unavailable` when missing.                                                                                                                                                                                            |
| Source link and maintainer credit     | `layout/app.footer.component.ts` (upstream links its own repository)               | none                                                  | yes      | yes      | Footer links this repository and Peter Ryszkiewicz's OpenLinks page.                                                                                                                                                                                                             |
| Recovery page                         | `reference/esp-miner/main/http_server/recovery_page.html`                          | GET `/recovery` (yes)                                 | yes      | yes      | Firmware-embedded page, independent of the variant.                                                                                                                                                                                                                              |
| OpenAPI documentation                 | `reference/esp-miner/main/http_server/openapi.yaml`                                | none                                                  | excluded | excluded | Upstream has no UI page for it either.                                                                                                                                                                                                                                           |

## Firmware API coverage

Every upstream endpoint is registered by our firmware except CORS preflight:

| Endpoint                                      | Registered | Used by current / solid |
| --------------------------------------------- | ---------- | ----------------------- |
| GET `/api/system/info`                        | yes        | yes / yes               |
| GET `/api/system/asic`                        | yes        | no / no                 |
| GET `/api/system/statistics`                  | yes        | no / no                 |
| GET `/api/system/scoreboard`                  | yes        | yes / yes               |
| GET `/api/system/wifi/scan`                   | yes        | no / no                 |
| GET `/api/system/logs`                        | yes        | yes / yes               |
| PATCH `/api/system`                           | yes        | yes / yes               |
| POST `/api/system/pause`, `resume`, `restart` | yes        | yes / yes               |
| POST `/api/system/identify`                   | yes        | no / no                 |
| POST `/api/system/blockFound/dismiss`         | yes        | no / no                 |
| POST `/api/system/OTA`, `/api/system/OTAWWW`  | yes        | yes / yes               |
| GET/POST `/api/theme`                         | yes        | yes / yes               |
| WS `/api/ws` (logs)                           | yes        | yes / yes               |
| WS `/api/ws/live` (telemetry)                 | yes        | no / no                 |
| OPTIONS `/api/*` (CORS preflight)             | no         | not needed              |

Our firmware also serves GET `/api/system/command-status`, which upstream does
not have.

## Parity plan

1. **Keep both variants equal until `solid` has its own evidence.** New
   features land in `solid` first. They are ported to `current` only if
   `current` stays the default. The shared contract tests must keep passing for
   both.
2. **Read-only features first:** the live telemetry stream, the history chart,
   the ASIC table, ANSI log colouring and the block-found banner. They need no
   new safety evidence. Each one must fit the gzip budget in
   `firmware/bitaxe/web-ui-budget.json`. Raise the budget only by an explicit,
   reviewed change backed by a `just web-ui-sizes --append-history` row.
3. **Write features next:** fallback pool, pool switch, identify, Wi-Fi scan and
   display settings. Each needs its firmware row verified and a UI workflow
   test.
4. **Hardware controls last.** Frequency, voltage, fan, thermal and power
   controls stay excluded until their live safety evidence is complete and an
   explicit task authorizes exposing them.
5. **Promotion.** UI-004 evidence binds `current`. Promoting any UI row for
   `solid`, or making `solid` the default, needs its own hardware evidence
   bound to a package whose manifest records `web_ui_variant: solid`.
