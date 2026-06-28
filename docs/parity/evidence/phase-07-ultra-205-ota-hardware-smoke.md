# Phase 7 Ultra 205 OTA And Recovery Hardware Smoke

This document is the manual evidence template for Plan 07-09 hardware
execution. Filling it in does not by itself mark checklist rows verified; each
row needs captured command output, logs, HTTP responses, and a concrete
conclusion.

## Run Identity

| Field | Value |
| --- | --- |
| board | Ultra 205 |
| port | not provided |
| firmware commit | `e32c47f167b5d695d84fe1a8cb22dc227e008b50` |
| reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| package manifest path | `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` |
| app OTA image path | `bazel-bin/firmware/bitaxe/esp-miner.bin` |
| www.bin path | `bazel-bin/firmware/bitaxe/www.bin` |
| factory image path | `bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin` |
| conclusion | not run - hardware verification pending |

## commands run

Record the exact commands, timestamps, operator, shell cwd, and exit status.

```bash
just build
just package
just flash board=205 port=<port>
just monitor port=<port>
just flash-monitor board=205 port=<port> evidence-dir=docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke
```

Automated checkpoint commands run on 2026-06-28:

- `just test` - passed.
- `just package` - passed and produced `bitaxe-ultra205.elf`,
  `esp-miner.bin`, `www.bin`, `otadata-initial.bin`,
  `bitaxe-ultra205-factory.bin`, and `bitaxe-ultra205-package.json`.
- `just parity` - passed with `validation_errors: none`.
- `bazel run //tools/parity:report -- release-gate` - passed.

Hardware commands were not run. No connected Ultra 205 serial port or hardware
approval was provided, so `just flash`, `just monitor`, `just flash-monitor`,
`/api/system/OTA`, `/api/system/OTAWWW`, large erase, rollback injection, and
interrupted-update tests remain pending.

Command conclusion: not run - hardware verification pending.

## Package Manifest

Record values from `bitaxe-ultra205-package.json`:

| Artifact | Path | SHA-256 | Offset | Notes |
| --- | --- | --- | --- | --- |
| `bitaxe-ultra205.elf` | `bazel-bin/firmware/bitaxe/bitaxe-ultra205.elf` | `b817f9463aa962baa1d475be093a48f6fad5e1dca6308c81d30d57271036c4f5` | `Unavailable` | Default flash image. |
| `esp-miner.bin` | `bazel-bin/firmware/bitaxe/esp-miner.bin` | `6c494c8c2b9b426940502843e923b25c4e40d70b8aa6b02a2ffbfbc9c0bc5079` | `0x10000` | App OTA image. |
| `www.bin` | `bazel-bin/firmware/bitaxe/www.bin` | `0dbb0eba0cc4198186d0175557ec134d7829f3426faf35d8baf263ee0a7c65a0` | `0x410000` | Static filesystem image. |
| `otadata-initial.bin` | `bazel-bin/firmware/bitaxe/otadata-initial.bin` | `7d2c7ac4888bfd75cd5f56e8d61f69595121183afc81556c876732fd3782c62f` | `0xf10000` | OTA data image. |
| `bitaxe-ultra205-factory.bin` | `bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin` | `0129c5ae2cabfeb3479d168b6c6c49f114d87db4d5e292e11725514032984ea8` | `0x0` | Merged factory image. |

Manifest conclusion: not run - hardware verification pending.

## Firmware OTA Accepted Upload

Capture `/api/system/OTA` behavior using `esp-miner.bin`.

| Field | Value |
| --- | --- |
| request route | `/api/system/OTA` |
| upload file | `esp-miner.bin` |
| upload checksum | TBD |
| public response | TBD |
| expected success response | `Firmware update complete, rebooting now!` |
| reboot observed | TBD |
| post-reboot identity | TBD |
| running partition | TBD |
| conclusion | not run - hardware verification pending |

## invalid image rejection

Capture a controlled invalid-image or wrong-file-name attempt.

| Field | Value |
| --- | --- |
| route | `/api/system/OTA` |
| invalid artifact | TBD |
| public response | TBD |
| firmware logs | TBD |
| device remained operable | TBD |
| conclusion | not run - hardware verification pending |

## rollback and boot-validation

Record boot-validation logs and rollback observations after OTA success or
failure injection.

| Field | Value |
| --- | --- |
| pending image state before reboot | TBD |
| boot-validation log lines | TBD |
| marked valid log observed | TBD |
| marked invalid/reboot log observed | TBD |
| rollback observed | TBD |
| running partition after rollback decision | TBD |
| conclusion | not run - hardware verification pending |

## Static Filesystem Smoke

Capture packaged static route behavior from the running firmware.

| Surface | Expected | Observed |
| --- | --- | --- |
| `/` | HTTP success for static entry point | TBD |
| gzip asset | `/assets/app.css.gz` served with gzip behavior | TBD |
| `/assets/app.css.gz` | representative gzip smoke path reachable | TBD |
| missing static redirect | missing static path redirects to `/` | TBD |
| API coexistence | `/api/*` routes are not captured by static wildcard | TBD |

Static route conclusion: not run - hardware verification pending.

## Recovery Page

Capture recovery behavior.

| Field | Value |
| --- | --- |
| route | `/recovery` |
| recovery page reachable | TBD |
| upload file | `www.bin` |
| upload response | TBD |
| restart response | TBD |
| post-restart `/` result | TBD |
| conclusion | not run - hardware verification pending |

## OTAWWW Gap Response

Capture the current explicit REL-03 gap behavior.

| Field | Value |
| --- | --- |
| route | `/api/system/OTAWWW` |
| upload file | `www.bin` |
| expected public response | `Wrong API input` |
| firmware gap log | `otawww_update=gap reason=interruption_evidence_missing owner=phase-07-release` |
| conclusion | not run - hardware verification pending |

## large erase and recovery steps

Use this section only when a destructive erase is intentionally run.

| Field | Value |
| --- | --- |
| erase command | TBD |
| port | TBD |
| package manifest path | `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` |
| factory image used | `bitaxe-ultra205-factory.bin` |
| flash command after erase | TBD |
| boot logs after erase | TBD |
| `/recovery` after erase | TBD |
| conclusion | not run - hardware verification pending |

## interrupted-update

Capture interrupted firmware or static update behavior only when intentionally
tested.

| Field | Value |
| --- | --- |
| route interrupted | TBD |
| artifact | TBD |
| interruption point | TBD |
| post-interruption `/` result | TBD |
| post-interruption `/assets/app.css.gz` result | TBD |
| post-interruption `/recovery` result | TBD |
| recovery action | TBD |
| conclusion | not run - hardware verification pending |

## final conclusion

not run - hardware verification pending
