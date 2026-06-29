# Phase 10 Route Manifest And API Compare Unification Evidence

This ledger records Phase 10 manifest and tooling evidence only. It proves that
firmware route reporting and API compare route policy consume the Phase 7 typed
route manifest, while live HTTP, static, recovery, OTA, rollback, erase,
failed-update recovery, and interrupted-update recovery remain outside the Phase
10 claim.

## Run Identity

| Field | Value |
| --- | --- |
| phase | Phase 10 - Route Manifest And API Compare Unification |
| plan | 10-03 - checklist/evidence claim boundaries and final verification |
| source commit at plan start | `bfdc9e7c4ab3c3c99918756fe872fd48ba17509b` |
| source commit for final verification | `a65439415e3291700fbd86e86abb75a9d20a9c40` |
| reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| dependency evidence | `.planning/phases/10-route-manifest-and-api-compare-unification/10-01-SUMMARY.md`, `.planning/phases/10-route-manifest-and-api-compare-unification/10-02-SUMMARY.md` |
| evidence classes used | `unit`, `workflow`, `api-compare` |
| evidence classes not introduced | no `manifest-compare` taxonomy |
| live hardware/network probes | not run by Phase 10 |
| conclusion | manifest/tooling alignment evidence only; live release behavior remains Phase 13-owned |

## Manifest And Tooling Claims

| Claim | Evidence class | Evidence |
| --- | --- | --- |
| `phase07_routes()` is the Phase 7 route source for firmware OTA, OTAWWW gap, recovery, and static wildcard ownership. | `unit` | `crates/bitaxe-api/src/route_shell.rs` contains `/api/system/OTA` as `RouteKind::FirmwareUpdate`, `/api/system/OTAWWW` as `RouteKind::AxeOsStaticUpdateGap`, `/recovery` as `RouteKind::Recovery`, and `/*` as `RouteKind::StaticFiles`; Plan 10-01 added route-report unit coverage. |
| Firmware startup route reporting consumes manifest-derived route metadata. | `workflow` | `firmware/bitaxe/src/http_api.rs` calls `phase07_route_report()` and logs `manifest_routes`, `firmware_update_routes`, `otawww_gap_routes`, `recovery_routes`, and `static_file_routes`; handler registration remains explicit and ordered. |
| API compare route presence and kind policy consumes the Phase 7 typed route manifest. | `api-compare` | `tools/parity/src/api_compare.rs` delegates production compare checks through `run_api_compare_with_routes(..., phase07_routes())`; Plan 10-02 added missing-route, kind-downgrade, and weak verified-claim regressions. |
| Phase 5 schema and captured-response evidence remains active. | `api-compare` | Plan 10-02 preserved `phase05-required-routes.json` schema and captured-response checks while moving Phase 7 route presence and kind policy to typed routes. |
| Checklist rows cite Phase 10 without promoting unsupported live or release behavior. | `workflow` | `docs/parity/checklist.md` cites this ledger on API/static/OTA/release-sensitive rows and keeps those rows below unsupported live or release `verified` status. |

## Claim Boundary Matrix

| Surface | Phase 10 proves | Phase 10 does not prove | Owner for live/release proof |
| --- | --- | --- | --- |
| `phase07_routes()` manifest entries | `unit` evidence that `/api/system/OTA`, `/api/system/OTAWWW`, `/recovery`, and `/*` keep expected methods and `RouteKind` values. | Live handler reachability, live response bodies, route ordering under ESP-IDF, or network behavior. | Phase 13-owned |
| Firmware route reporting | `workflow` evidence that startup reporting uses manifest-derived route metadata instead of the old Phase 5 count. | A live serial run from this exact Phase 10 commit or proof that all reported routes were reachable over HTTP. | Phase 13-owned |
| `tools/parity api-compare` route policy | `api-compare` evidence that missing Phase 7 routes, route-kind downgrades, and weak verified-claim overclaims fail. | Live API responses from a device or proof that static/recovery/OTA requests succeed. | Phase 13-owned |
| `/` static root | Manifest/static planning remains visible through Phase 7 route policy. | Live `/` HTTP response. | Phase 13-owned |
| `/assets/app.css.gz` | Manifest/static planning remains visible through Phase 7 route policy. | Live gzip asset response or headers. | Phase 13-owned |
| Missing static redirect | Not proven by Phase 10. | Live redirect or fallback behavior. | Phase 13-owned |
| `/recovery` | Route remains classified as `RouteKind::Recovery`. | Live recovery page reachability or content parity. | Phase 13-owned |
| Valid firmware OTA | `/api/system/OTA` remains classified as `RouteKind::FirmwareUpdate`. | Accepted upload, reboot, boot-validation, post-update identity, or rollback behavior. | Phase 13-owned |
| Invalid OTA rejection | Route policy remains present for firmware OTA. | Invalid image rejection response or device operability after rejection. | Phase 13-owned |
| OTAWWW whole-`www` behavior | `/api/system/OTAWWW` remains classified as `RouteKind::AxeOsStaticUpdateGap`. | Whole-`www` write, interruption safety, recovery, or static update success. | Phase 13-owned |
| Rollback | Not proven by Phase 10. | Pending-image handling, valid/invalid marking, or rollback reboot behavior. | Phase 13-owned |
| Erase | Not proven by Phase 10. | Large erase recovery or post-erase factory flash recovery. | Phase 13-owned |
| Failed-update recovery | Not proven by Phase 10. | Recovery after failed update or return to operable state. | Phase 13-owned |
| Interrupted-update recovery | Not proven by Phase 10. | Interruption point, post-interruption reachability, or recovery procedure outcome. | Phase 13-owned |

## Command Evidence

| Command | Evidence class | Result | Notes |
| --- | --- | --- | --- |
| `bazel test //crates/bitaxe-api:tests //tools/parity:tests` | `unit` | passed | `//crates/bitaxe-api:tests` and `//tools/parity:tests` passed. |
| `bazel run //tools/parity:report -- api-compare` | `api-compare` | passed with `validation_errors: none` | Checked `schema=99`, `captured-response=47`, `static-route=36`, and left `firmware-smoke` as `not-run`. |
| `just parity` | `workflow` | passed with `validation_errors: none` | Ran after checklist edits and again during final verification to prove no unsupported verified claims were introduced. |
| `just test` | `workflow` | passed | `bazel test //...` passed all 13 test targets and rebuilt firmware/package targets at source commit `a65439415e32`. |
| `cargo fmt --all` | `workflow` | passed | Required by repo Rust pre-commit policy. |
| `cargo clippy --all-targets --all-features -- -D warnings` | `workflow` | passed | Required by repo Rust pre-commit policy. |
| `cargo build --all-targets --all-features` | `workflow` | passed | Required by repo Rust pre-commit policy. |
| `cargo test --all-features` | `workflow` | passed | 362 unit tests and doc tests passed across workspace crates. |
| `git diff -- reference/esp-miner --exit-code` | `workflow` | passed | The pinned upstream reference stayed read-only. |

## Secret Redaction Review

| Secret-bearing surface | Result |
| --- | --- |
| Wi-Fi credentials | not committed |
| pool credentials | not committed |
| private endpoints | not committed |
| private URLs | not committed |
| request bodies | not committed |
| NVS secret values | not committed |

Phase 10 records file paths, command names, route paths, route kinds, source
commits, and reference commits only. It does not record Wi-Fi settings, pool
credentials, private endpoints, private URLs, OTA request bodies, or NVS secret
values.

## Final Conclusion

Phase 10 records manifest/tooling evidence only. `phase07_routes()` and
firmware route reporting are proven by unit and workflow evidence, and
`tools/parity api-compare` route presence/kind policy is proven by tests and API
compare. Live `/`, `/assets/app.css.gz`, missing static redirect, `/recovery`,
valid firmware OTA, invalid OTA rejection, OTAWWW whole-`www` behavior,
rollback, erase, failed-update recovery, and interrupted-update recovery are
Phase 13-owned and not proven by Phase 10.
