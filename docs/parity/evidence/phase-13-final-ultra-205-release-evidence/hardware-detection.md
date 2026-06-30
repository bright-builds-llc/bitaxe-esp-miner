# Phase 13 Hardware Detection Gate

## Command Log

- command: `just detect-ultra205 > docs/parity/evidence/phase-13-final-ultra-205-release-evidence/detect-ultra205.log 2>&1`
- detector log: `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/detect-ultra205.log`
- detector_status: passed
- selected port: `/dev/cu.usbmodem1101`
- board: board `205`
- board-info command: `espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive`
- board-info executor: `scripts/detect-ultra205.sh` via `just detect-ultra205`
- package manifest: `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
- source_commit: `190849539700b8f9a7909fd2b6ebd84142557968`
- reference_commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Detector Evidence

- `just detect-ultra205` exited successfully.
- The detector printed exactly one `port=<path>` result: `port=/dev/cu.usbmodem1101`.
- The detector's board-info command succeeded for `esp32s3`.
- The captured board-info output reports chip type `esp32s3`, flash size `16MB`, and Wi-Fi/BLE features.
- No flash command, monitor command, raw erase, rollback, interrupted-update, voltage/fan/mining stress, or raw write command ran during this detector gate.
- The following wrapper flash refreshed the package manifest to source commit `190849539700b8f9a7909fd2b6ebd84142557968`; serial evidence cites the manifest actually flashed.

## Conclusion

Conclusion: detector_status: passed - exactly one detector-approved ESP32-S3 USB serial port was found for Ultra 205 hardware evidence, and board-info succeeded before any live flash or monitor command.
