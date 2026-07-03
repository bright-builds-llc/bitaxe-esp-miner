# Phase 19 Recovery Regression And OTAWWW Evidence Contract

## Scope

Phase 19 evidence closes or explicitly bounds the recovery-regression and
OTAWWW whole-www update evidence gap for the current Ultra 205 chain. This
contract is created before live artifacts are cited, so every artifact class
starts as `pending` or `absent - not cited`.

network_scan: disabled

Board: `205`

Manifest path:
`docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json`

Recovery factory image: pending

Recovery OTA image: pending

Source commit: pending

Reference commit: pending

`DEVICE_URL` provenance: pending; must come from an explicit origin-only input
or trusted board `205` flash-monitor evidence. Network scans, mDNS, ARP, router
state, and inferred serial-log guesses are out of scope.

## Required Artifact Classes

| Artifact | Initial status | Claim boundary |
| --- | --- | --- |
| `package-release-gate.md` | pending | Package command, release-gate command, source commit, reference commit, manifest path, artifact checksum ledger, and recovery image identity. |
| `package-release-gate/bitaxe-ultra205-package.json` | pending | Copied manifest for board `205`; source/reference commits and artifact metadata only. |
| `serial-boot.md` | pending | Detector and flash-monitor identity summary only. |
| `serial-boot/detect-ultra205.log` | pending | Detector output and board-info gate for the selected USB port. |
| `serial-boot/flash-command-evidence.json` | pending | Trusted flash-monitor command metadata, selected port, source commit, reference commit, and commit-redacted status. |
| `serial-boot/flash-monitor.log` | pending | Redacted serial boot, Wi-Fi join, route registration, recovery access, and boot markers when present. |
| `target-lock.json` | pending | Sanitized target provenance with `network_scan: disabled` and no raw target. |
| `recovery-regression.md` | pending | Recovery regression ledger separating failed-update, large-erase, interrupted-update, restore, safe-state, rollback, and non-claim fields. |
| `recovery-regression/recovery-regression.log` | pending | Phase 19 wrapper log plus delegated Phase 16 helper provenance. |
| `recovery-regression/failed-update.log` | pending | Failed-update evidence or pending status; must not imply rollback proof by itself. |
| `recovery-regression/large-erase.log` | pending | Large-erase and factory-restore evidence only when explicitly allowed and gated. |
| `recovery-regression/large-erase-post-restore-monitor.log` | pending | Post-restore serial markers for firmware identity, reference identity, safe-state, and SPIFFS availability. |
| `recovery-regression/interrupted-ota.log` | pending | Bounded interrupted-update evidence only when explicitly allowed and gated. |
| `otawww.md` | pending | OTAWWW ledger separating route response, gap status, whole-www proof requirements, and non-claim fields. |
| `otawww/otawww-gap.log` | pending | OTAWWW REL-03 gap evidence; may include selected redacted response fields but never whole-www update proof by itself. |
| `summary.md` | absent - not cited | Final Phase 19 evidence ledger after live artifacts and redaction review exist. |
| `redaction-review.md` | pending | Redaction gate for every cited Phase 19 artifact. |

## Required Evidence Fields

| Field | Required value or source | Initial status |
| --- | --- | --- |
| `network_scan` | `disabled` | pending |
| `board` | `205` | pending |
| `source_commit` | Current package manifest source commit | pending |
| `reference_commit` | Current package manifest reference commit | pending |
| `manifest` | Phase 19 copied package manifest path | pending |
| `selected_port` | Detector-approved Ultra 205 USB port | pending |
| `board_info` | `espflash board-info --chip esp32s3 --port <port> --non-interactive` output | pending |
| `factory_image` | Manifest-listed factory image used for restore | pending |
| `ota_image` | Manifest-listed firmware OTA image used for failed or interrupted update flows | pending |
| `restore_command` | Exact repo-owned factory restore command | pending |
| `post_action_safe_state_markers` | Firmware commit, reference commit, safe-state disabled mining/work/control, and SPIFFS availability when applicable | pending |
| `failed_update_status` | Pending, blocked, or captured from Phase 16 delegated helper | pending |
| `large_erase_status` | Pending, blocked, or captured from Phase 16 delegated helper | pending |
| `interrupted_update_status` | Pending, blocked, or captured from Phase 16 delegated helper | pending |
| `otawww_claim` | `REL-03 gap` unless whole-www proof requirements are present | pending |
| `redaction_status` | `pending` until every cited artifact is reviewed | pending |

## Claim Classes

| Claim class | Required proof before citation | Initial status |
| --- | --- | --- |
| failed-update recovery | Explicit allow flag, detector and board-info gates, current package manifest, invalid or failed upload evidence, post-failure operability proof, and redaction review. | pending |
| large erase recovery | Explicit allow flag, detector and board-info gates, factory image, restore command, post-restore monitor markers, optional HTTP/static safe-state proof, and redaction review. | pending |
| interrupted update recovery | Explicit allow flag, detector and board-info gates, bounded interruption point, post-interruption operability proof, and redaction review. | pending |
| OTAWWW whole-www update | Size checks, chunked erase/write behavior, recovery access, and interrupted-update hardware-regression evidence for the `www` partition. | absent - not cited |
| OTAWWW REL-03 gap | Route response, current implementation boundary, missing whole-www proof, owner, blocker, operator impact, and follow-up path. | pending |
| rollback and boot validation | Captured bootloader or ESP-IDF rollback state plus post-action boot-validation markers from a plan-documented recovery procedure. | absent - not cited |

## Promotion Rules

- `www.bin`, static serving, route presence, and `Wrong API input` do not prove
  whole-www OTAWWW update behavior.
- OTAWWW remains a REL-03 gap unless size checks, chunked erase/write behavior,
  recovery access, and interrupted-update hardware-regression evidence all
  exist.
- Failed-update evidence does not prove rollback or whole-www update behavior.
- Large-erase and interrupted-update actions require explicit Phase 19 allow
  flags and delegated Phase 16 detector and board-info gates before live action.
- `redaction-review.md` must remain `redaction_status: pending` until every
  cited artifact is scanned and reviewed.
