# Phase 7 OTA, Filesystem, And Release Evidence

This record separates package, live firmware, hardware, gap, and compliance
evidence so release-readiness language does not outrun the available proof.

## package evidence

- Conclusion: package evidence record is initialized.
- Current state: package commands and manifest checks are not recorded in this
  file yet.
- Evidence status: not run - hardware verification pending for live device
  package/flash confirmation.

## static/SPIFFS evidence

- Conclusion: static/SPIFFS evidence record is initialized.
- Current state: `/`, representative static files, `.gz` preference, cache
  behavior, missing-file redirect, and `/recovery` live behavior are not
  recorded here yet.
- Evidence status: not run - hardware verification pending.

## firmware OTA evidence

- Conclusion: firmware OTA runtime upload evidence remains pending, while
  rollback-capable boot validation is now implemented in firmware.
- Current state: `/api/system/OTA` success, rejection, AP/APSTA rejection,
  progress/status, validation/activation errors, and reboot scheduling are not
  recorded here yet. Boot validation now logs `ota_boot_validation=not_pending`,
  `ota_boot_validation=marked_valid`, or
  `ota_boot_validation=marked_invalid_reboot` from the ESP-IDF OTA state path.
- Evidence status: implemented/compiled path planned for host verification;
  live OTA and hardware rollback verification pending.

## OTAWWW REL-03 gap

- Conclusion: OTAWWW remains an explicit REL-03 V1 parity gap owned by
  `phase-07-release`.
- Current state: `/api/system/OTAWWW` preserves the update access gate and
  AP/APSTA rejection path, then fail-closes with status 400 body `Wrong API
  input`. Firmware logs `otawww_update=gap
  reason=interruption_evidence_missing owner=phase-07-release` for accepted
  private-network attempts.
- Release impact: owners can package and flash `www.bin`, but in-firmware
  whole-`www` static asset upload is not claimed for V1 release parity.
- Follow-up path: implement whole-partition SPIFFS erase/write with size
  checks, recovery access, successful update evidence, and interrupted-update
  recovery evidence before moving REL-03 above explicit gap status.
- D-15 status: not claimed in this plan because interruption/recovery hardware
  evidence is not scheduled in this plan set.
- Evidence status: explicit gap documented; hardware interruption/recovery
  verification pending.

## rollback/recovery evidence

- Conclusion: rollback boot validation is implemented but not hardware
  verified.
- Current state: firmware calls `esp_ota_get_state_partition` on the running
  partition at startup, marks pending images valid after startup diagnostics,
  and marks pending images invalid with reboot when diagnostics fail. Recovery
  page access, rollback observation, and return-to-operable-state proof are not
  recorded here yet.
- Evidence status: implemented/compiled path planned for host verification;
  live rollback and recovery verification pending.

## interrupted-update evidence

- Conclusion: interrupted-update evidence record is initialized.
- Current state: interrupted firmware update and interrupted static update cases
  are not recorded here yet.
- Evidence status: not run - hardware verification pending.

## license/provenance evidence

- Conclusion: license/provenance evidence record is initialized.
- Current state: Cargo report, non-Cargo license inventory, provenance manifest,
  GPL review status, and release artifact review structure exist or are planned;
  release-gate acceptance is not recorded here yet.
- Evidence status: documentation evidence initialized; live artifact review is
  pending.

## final release-readiness status

- Conclusion: release-readiness status is not established.
- Current state: package, live firmware, recovery, rollback, interrupted-update,
  license, and provenance evidence must be filled by later Phase 7 plans before
  final release claims are made.
- Evidence status: not run - hardware verification pending.
