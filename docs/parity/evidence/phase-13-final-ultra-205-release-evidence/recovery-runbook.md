# Phase 13 Recovery Runbook

## Current Package Identity

- package manifest: `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
- factory image: `bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin`
- firmware OTA image: `bazel-bin/firmware/bitaxe/esp-miner.bin`
- expected board: `205` / Ultra 205 BM1366
- current recorded package source commit: `190849539700b8f9a7909fd2b6ebd84142557968`
- current recorded reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

The package manifest is the identity source for recovery evidence. If `just package`
or a wrapper flash rebuilds the manifest, update the evidence summary to cite the
manifest and source commit that were actually used.

## Recovery Procedure

Use the factory image from the current package manifest to recover a board after
an erase, failed update, or interruption:

```bash
just flash board=205 port=<path> image=bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=<out-dir>/large-erase-restore
```

Capture bounded post-restore serial output with:

```bash
scripts/phase13-monitor-capture.sh --port <path> --out <out-dir>/large-erase-post-restore-monitor.log --seconds 35
```

When an explicit reachable `DEVICE_URL` for the just-flashed Ultra 205 is
available, run the HTTP/static recovery check through:

```bash
scripts/phase13-http-static-smoke.sh --device-url "${DEVICE_URL:-}" --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --out-dir <out-dir>/http-static-recovery
```

## Stop Conditions

Stop and write pending or blocked evidence instead of running destructive or
fault-injection commands when any of these conditions apply:

- `just detect-ultra205` has not recorded exactly one selected Ultra 205 port.
- The selected target is not board `205`.
- `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` is missing.
- `bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin` is missing.
- The recovery runbook is missing or does not name the current factory restore path.
- `DEVICE_URL` is missing, ambiguous, unreachable, or not known to point at the
  just-flashed Ultra 205 for live HTTP/OTA evidence.
- A recovery command fails and the next recovery action is not already named here.
- Any log or evidence file contains Wi-Fi credentials, pool credentials, API
  tokens, private endpoints, NVS secret values, worker secrets, or raw terminal secrets.

## Large Erase Procedure

Large erase is destructive and must run only through the Phase 13 recovery helper
after the stop conditions are clear, an explicit reachable `DEVICE_URL` is
available, and `--allow-large-erase` is explicitly provided.

Immediately before erase, the helper must rerun `just detect-ultra205`, require
exactly one detector-reported `port=`, require that port to match `--port`, and
run `espflash board-info --chip esp32s3 --port <path> --non-interactive`.

The only approved erase command is:

```bash
espflash erase-flash --chip esp32s3 --port <path> --non-interactive
```

After erase, restore with the factory command in the Recovery Procedure section,
capture bounded monitor output, and rerun HTTP/static smoke. Do not claim large
erase recovery verified unless the erase result, factory reflash result,
`firmware_commit=`, `reference_commit=`, `safe_state: mining=disabled`,
`spiffs_mount=available`, and `http_static_status: passed` are all recorded.

## Failed Update Procedure

Failed firmware update evidence targets the firmware OTA route:

```text
POST /api/system/OTA
```

The Phase 13 recovery helper creates a fixed invalid firmware image, records its
SHA-256 checksum, uploads it with `curl`, records the public status/body, and
then records post-failure partition/static/API state through the HTTP/static
smoke helper when `DEVICE_URL` is available. It may emit captured failed-update
evidence only for an explicit invalid-image rejection status with a rejection
body marker; a `200` response, curl failure, wrong-route response, or server
error is blocked evidence.

A rejected invalid image is failed-update evidence only. It is not rollback or
boot-validation proof.

## Interrupted Update Procedure

Interrupted firmware OTA evidence targets the firmware OTA route with the current
OTA image:

```bash
curl --max-time 1 --limit-rate 1024 --data-binary @bazel-bin/firmware/bitaxe/esp-miner.bin <DEVICE_URL>/api/system/OTA
```

The Phase 13 recovery helper must record the OTA image checksum, interruption
point, public status/body or curl timeout/error, post-interruption route state,
recovery steps, and final conclusion. A completed `200` OTA response is blocked
evidence, and captured interrupted-update evidence requires post-interruption
`http_static_status: passed`. Without `DEVICE_URL`, keep interrupted update
evidence pending.

## OTAWWW Gap Procedure

`POST /api/system/OTAWWW` remains the REL-03 gap with public response `Wrong API input`
unless a later plan captures whole-`www` update and interrupted-update
hardware-regression evidence. `www.bin` package generation, serial route
registration, and recovery-page availability do not verify OTAWWW parity.

## Evidence Artifacts

The Phase 13 recovery helper writes evidence under:

```text
docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression/
```

Expected artifacts:

- `recovery-regression.log` - top-level failed-update, large-erase, interrupted-update, rollback, boot-validation, OTAWWW, and conclusion fields.
- `large-erase.log` - erase command, `espflash --version`, restore command, restore result, and post-restore checks.
- `large-erase-post-restore-monitor.log` - bounded monitor capture produced by `scripts/phase13-monitor-capture.sh`.
- `interrupted-ota.log` - interrupted firmware upload command shape, checksum, failure point, status/body, and recovery checks.
- `invalid-firmware.bin` - fixed invalid firmware artifact used only for rejected-update evidence.
- `failed-update-http-static/`, `large-erase-http-static/`, or `interrupted-ota-http-static/` - HTTP/static smoke outputs for recovery evidence when `DEVICE_URL` is available; large-erase and interrupted-update captured conclusions require passing smoke.

All recovery/destructive logs must be included in
`docs/parity/evidence/phase-13-final-ultra-205-release-evidence/redaction-review.md`
before citation.
