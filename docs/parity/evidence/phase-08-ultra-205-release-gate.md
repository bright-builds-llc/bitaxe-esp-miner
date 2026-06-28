# Phase 8 Ultra 205 Release Gate Evidence

This ledger records Phase 8 package, hardware detection, and live release-gate
preconditions for the Ultra 205. It intentionally avoids checklist promotion
language until each live surface has the required evidence class.

## Run Identity

| Field | Value |
| --- | --- |
| board | Ultra 205 |
| port | pending - run `just detect-ultra205` after `just package` |
| source commit | pending - capture before live evidence |
| reference commit | pending - capture before live evidence |
| conclusion | not run - Phase 8 evidence pending |

## Hardware Detection Gate

| Field | Value |
| --- | --- |
| detector result | not run - Phase 8 evidence pending |
| board-info output | not run - Phase 8 evidence pending |
| selected port | not run - Phase 8 evidence pending |
| command | `just detect-ultra205` |
| conclusion | not run - Phase 8 evidence pending |

## Package Manifest

| Field | Value |
| --- | --- |
| package manifest path | pending - `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` |
| factory image path | pending - `bitaxe-ultra205-factory.bin` |
| firmware OTA image path | pending - `esp-miner.bin` |
| www.bin path | pending - `www.bin` |
| otadata path | pending - `otadata-initial.bin` |
| artifact SHA-256 values | not run - Phase 8 evidence pending |
| conclusion | not run - Phase 8 evidence pending |

## DEVICE_URL Discovery

| Field | Value |
| --- | --- |
| DEVICE_URL status | not run - Phase 8 evidence pending |
| DEVICE_URL source | not run - Phase 8 evidence pending |
| sanitized URL evidence | not run - Phase 8 evidence pending |
| exact private URL committed | no |
| conclusion | not run - Phase 8 evidence pending |

## Static And Recovery HTTP Smoke

| Field | Value |
| --- | --- |
| `/` HTTP status | not run - Phase 8 evidence pending |
| `/assets/app.css.gz` HTTP status | not run - Phase 8 evidence pending |
| missing static redirect behavior | not run - Phase 8 evidence pending |
| `/recovery` HTTP status | not run - Phase 8 evidence pending |
| conclusion | not run - Phase 8 evidence pending |

## Firmware OTA Accepted Upload

| Field | Value |
| --- | --- |
| route | `/api/system/OTA` |
| upload artifact | `esp-miner.bin` |
| upload response | not run - Phase 8 evidence pending |
| reboot observed | not run - Phase 8 evidence pending |
| post-reboot identity | not run - Phase 8 evidence pending |
| conclusion | not run - Phase 8 evidence pending |

## Invalid Image Rejection

| Field | Value |
| --- | --- |
| route | `/api/system/OTA` |
| invalid artifact | not run - Phase 8 evidence pending |
| public response | not run - Phase 8 evidence pending |
| device remained operable | not run - Phase 8 evidence pending |
| conclusion | not run - Phase 8 evidence pending |

## Failed Update Recovery

| Field | Value |
| --- | --- |
| failure class | not run - Phase 8 evidence pending |
| running partition after failure | not run - Phase 8 evidence pending |
| recovery procedure | not run - Phase 8 evidence pending |
| conclusion | not run - Phase 8 evidence pending |

## Rollback And Boot Validation

| Field | Value |
| --- | --- |
| pending image state | not run - Phase 8 evidence pending |
| boot-validation output | not run - Phase 8 evidence pending |
| marked-valid observation | not run - Phase 8 evidence pending |
| marked-invalid observation | not run - Phase 8 evidence pending |
| conclusion | not run - Phase 8 evidence pending |

## OTAWWW Gap Response

| Field | Value |
| --- | --- |
| route | `/api/system/OTAWWW` |
| upload artifact | `www.bin` |
| expected public gap response | `Wrong API input` |
| observed response | not run - Phase 8 evidence pending |
| conclusion | not run - Phase 8 evidence pending |

## Large Erase Recovery

| Field | Value |
| --- | --- |
| erase command | not run - Phase 8 evidence pending |
| recovery flash command | not run - Phase 8 evidence pending |
| post-recovery boot | not run - Phase 8 evidence pending |
| conclusion | not run - Phase 8 evidence pending |

## Interrupted Update Recovery

| Field | Value |
| --- | --- |
| interrupted route | not run - Phase 8 evidence pending |
| interruption point | not run - Phase 8 evidence pending |
| post-interruption reachability | not run - Phase 8 evidence pending |
| recovery procedure | not run - Phase 8 evidence pending |
| conclusion | not run - Phase 8 evidence pending |

## Deferred Scope Review

| Field | Value |
| --- | --- |
| non-205 boards | deferred - no Ultra 205 evidence applies to other boards |
| deferred protocols/accessories/UI | deferred - Stratum v2, BAP, all-board images, and Angular UI rewrite remain outside V1 |
| checklist promotion | none in this plan |
| conclusion | not run - Phase 8 evidence pending |

## Secret Redaction Review

| Field | Value |
| --- | --- |
| exact private URLs | not committed |
| Wi-Fi credentials | not committed |
| pool credentials | not committed |
| private endpoints | not committed |
| NVS secret values | not committed |
| conclusion | not run - Phase 8 evidence pending |

## Final Conclusion

not run - Phase 8 evidence pending
