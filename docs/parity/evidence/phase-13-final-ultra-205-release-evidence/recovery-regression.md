# Phase 13 Recovery Regression Evidence

## Command Log

- command: `scripts/phase13-recovery-regression.sh --device-url "${DEVICE_URL:-}" --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --factory-image bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin --port /dev/cu.usbmodem1101 --out-dir docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression`
- recovery_regression_status: pending
- board: `205`
- selected port from Plan 13-02: `/dev/cu.usbmodem1101`
- package manifest: `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
- factory image: `bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin`
- firmware OTA image: `bazel-bin/firmware/bitaxe/esp-miner.bin`
- DEVICE_URL status: blocked - missing DEVICE_URL
- recovery runbook: `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-runbook.md`
- top-level log: `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression/recovery-regression.log`

## Gate Decision

Destructive and fault-injection actions were not run. `DEVICE_URL` was missing,
and the helper was invoked without `--allow-failed-update`,
`--allow-large-erase`, or `--allow-interrupted-ota`.

This satisfies D-16 for this run: missing prerequisites produced pending
evidence, and no large erase, failed-update live HTTP action, interrupted upload,
rollback action, raw write, voltage/fan/mining stress, or ad hoc recovery
experiment was performed.

## Failed Update Evidence

| Field | Evidence |
| --- | --- |
| failed update route | `POST /api/system/OTA` |
| failed update status | `pending - allow flag not provided` |
| invalid artifact checksum | pending - invalid artifact was not created because failed-update live HTTP action was not allowed and `DEVICE_URL` was missing |
| failure point | pending - no invalid firmware upload was attempted |
| public status/body | pending - no live HTTP request was sent |
| post-failure partition/static/API state | pending - no post-failure HTTP/static/API state exists because no failed update was attempted |
| recovery steps | explicitly not needed for this run because the failed-update action was not executed |
| conclusion | pending - failed-update recovery remains below verified |

## Large Erase Evidence

| Field | Evidence |
| --- | --- |
| large erase exact command | `espflash erase-flash --chip esp32s3 --port <path> --non-interactive` |
| large erase selected command | `espflash erase-flash --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` |
| large erase result | `pending - allow flag not provided` |
| factory reflash command | `just flash board=205 port=/dev/cu.usbmodem1101 image=bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression/large-erase-restore` |
| factory reflash result | pending - large erase was not run, so restore was not needed |
| post-restore monitor markers | `capture_status=pending` in `recovery-regression/large-erase-post-restore-monitor.log` |
| recovery route/API state | pending - no post-restore HTTP/static smoke because large erase was not run and `DEVICE_URL` was missing |
| conclusion | pending - large erase recovery remains below verified |

## Interrupted Update Evidence

| Field | Evidence |
| --- | --- |
| interrupted-update route | `POST /api/system/OTA` |
| interrupted-update artifact | `bazel-bin/firmware/bitaxe/esp-miner.bin` |
| interrupted-update artifact checksum | pending - OTA image checksum was not recorded because interrupted upload was not allowed |
| interrupted-update failure point | pending - no bounded upload interruption was attempted |
| interrupted-update public status/body | pending - no live HTTP request was sent |
| interrupted-update post-failure state | pending - no post-interruption HTTP/static recovery check exists because `DEVICE_URL` was missing |
| recovery steps | explicitly not needed for this run because the interrupted upload action was not executed |
| conclusion | pending - interrupted-update recovery remains below verified |

## Rollback And Boot Validation

- rollback status: pending - Plan 04 OTA evidence not run yet
- boot-validation status: pending - Plan 04 OTA evidence not run yet

A rejected invalid upload would not be rollback proof. Rollback and
boot-validation parity require post-update bootloader or boot-validation state
from a valid OTA attempt.

## OTAWWW Gap

OTAWWW remains the REL-03 gap. The expected public response is `Wrong API input`,
but this run did not observe it because `DEVICE_URL` was missing and no live
HTTP request was sent.

Do not mark OTAWWW verified unless whole-`www` interrupted-update
hardware-regression evidence is captured in a later plan.

## Redaction

redaction: passed for `recovery-runbook.md`, `recovery-regression.md`, and the
generated recovery logs:

- `recovery-regression/recovery-regression.log`
- `recovery-regression/large-erase.log`
- `recovery-regression/large-erase-post-restore-monitor.log`
- `recovery-regression/interrupted-ota.log`

No private `DEVICE_URL`, route headers, route body snippets, Wi-Fi credentials,
pool credentials, API tokens, NVS secret values, private endpoints, or raw
terminal secrets were generated because all live recovery and fault-injection
actions remained pending.

## Conclusion

Conclusion: recovery_regression_status: pending - the recovery runbook and
repo-owned helper exist, but failed update, large erase, interrupted-update,
rollback, boot-validation, and OTAWWW live evidence remain below verified until
`DEVICE_URL` and the corresponding explicit allow flags are available.
