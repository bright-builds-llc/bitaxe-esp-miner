# Parity work plan

- Run ID: `20260802T215555Z-OTA-001`
- Parity row: `OTA-001`
- Initial status: `implemented`
- Source commit: `24f5517bd4072bf2b8b4be5047a93fce9c4cef7c`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-ota-001-reboot-evidence`

## Selection

`bazel run //tools/parity:report -- next-item --format json` reported no open
plan. The previously audited candidates through `LOG-001` remain ineligible:

- `CFG-001`, `PWR-001`, `PWR-002`, `PWR-003`, `PWR-005`, `PWR-006`,
  `THR-001`, `THR-002`, `THR-003`, and `SELF-001` retain safety-critical or
  hardware-regression gaps without an exact promotable row contract.
- `CFG-005`, `API-002`, `API-003`, and `API-009` retain broader live firmware
  response/effect gaps than their existing pure-model evidence closes.
- `NET-001`, `ASIC-002`, `ASIC-003`, `ASIC-004`, `ASIC-005`, `ASIC-007`,
  `STR-001`, `STR-006`, and `STR-007` retain live network, ASIC, coordinator,
  watchdog, or soak gaps. The prior mining-soak authorization is consumed and
  closed at its repeated boundary.
- `LOG-001` retains no-init header, soft-reboot retention, and live lifecycle
  gaps.

`OTA-001` is the first actionable candidate. Its route planning, ESP-IDF
streaming adapter, invalid-image rejection, valid-upload response, and current
package artifact already exist. Prior Phase 18 hardware evidence remained
below verified only because its no-reset serial monitor attached after the OTA
response and captured no reboot identity or boot-validation markers. The fresh
required detector preflight found exactly one Ultra 205 on
`/dev/cu.usbmodem1101`, and the ignored local Wi-Fi credential file is
available without its contents being read.

## Scope and non-scope

Eliminate the post-response monitor race by owning a bounded serial capture
before the valid OTA upload, while preserving exact invalid-image rejection,
manifest checksum admission, response-body, reboot-identity, and ESP-IDF
boot-validation requirements. Generate only a redacted committed result; raw
device URL, Wi-Fi/network values, IP/MAC values, serial transcript, and HTTP
artifacts remain under ignored `target/advance-parity-ota001/` paths.

Do not modify the pinned reference tree, run destructive rollback, force a boot
failure, erase flash, interrupt an update, exercise OTAWWW, access pool
credentials, start mining, actuate voltage/fan/power controls, use direct UART
or manipulate pins, or claim selected-partition internals or rollback parity.

## Implementation

- [ ] Make the existing firmware OTA smoke helper start and own the bounded
      no-reset monitor before submitting the valid OTA image, with deterministic
      cleanup and failure propagation.
- [ ] Add shell-level regression coverage proving monitor-before-upload order,
      marker admission, cleanup, and fail-closed behavior without hardware.
- [ ] Build/package an exact clean implementation commit, flash it through the
      repo-owned wrapper, and derive one origin-only `DEVICE_URL` solely from
      the same trusted flash-monitor evidence session.
- [ ] Run exactly one bounded invalid-plus-valid OTA evidence attempt and
      produce a redacted `RESULT.md` only if the current package identity,
      invalid rejection, HTTP success response, reboot identity, safe-state,
      and `ota_boot_validation=` markers all pass.

## Verification and promotion

Software acceptance commands:

- `bash -n scripts/phase13-firmware-ota-smoke.sh`
- `bazel test //scripts:phase13_firmware_ota_smoke_test`
- `bazel run //scripts:verify_reference_clean`
- mandatory Rust, managed Bright Builds, all-Bazel, parity, progress,
  redaction, and diff checks.

Exact effectful hardware contract after the implementation commit:

1. `just detect-ultra205`
2. `just package`
3. `just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=target/advance-parity-ota001/serial-boot capture-timeout-seconds=60 redact-evidence=false`
4. One invocation of `scripts/phase13-firmware-ota-smoke.sh` using the manifest
   OTA image, `/dev/cu.usbmodem1101`, a 60-second monitor, ignored raw output
   under `target/advance-parity-ota001/firmware-ota`, and the single origin-only
   device URL extracted in-process from that same trusted flash evidence.
5. `just detect-ultra205` as the cleanup/health check.

The evidence destination is private local raw data under ignored `target/`;
only redacted category labels, source/reference commits, artifact digests,
route/status markers, and conclusions may be committed. If the post-attempt
detector fails, one recovery flash of the same admitted current package through
the repo-owned `just flash` wrapper is allowed, followed by one detector check.
There is no second OTA attempt. Stop at `complete` only when every promotion
criterion passes; otherwise record `stop_repeated_boundary`,
`stop_hardware_blocker`, `stop_authority_boundary`, or
`stop_impossible_contract`. Any invalid detector result, manifest mismatch,
ambiguous/missing device URL, HTTP failure, absent identity/boot-validation or
safe-state marker, privacy failure, or cleanup failure stops the attempt.

Promotion to `verified` is limited to `OTA-001`: current manifest image
admission, invalid rejection, valid route completion, scheduled reboot, exact
post-reboot source/reference identity, safe-state boot, and boot validation.
Selected partition, rollback, destructive/fault-injection recovery, OTAWWW,
network longevity, mining, and hardware-control behavior remain non-claims.
