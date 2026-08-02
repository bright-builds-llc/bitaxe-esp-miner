# OTA-001 bounded hardware retry plan

- Run ID: `20260802T223139Z-OTA-001-RETRY`
- Parity row: `OTA-001`
- Starting status: `implemented`
- Authorization source: explicit user authorization on 2026-08-02
- Initial source commit: `d697f44fa47cda56be23bfa6c2c624da7ebebb06`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Attempt budget: one invalid-plus-valid OTA invocation; zero OTA retries
- Current gate: Phase B exact one-attempt contract

## Purpose

Repeat the previously consumed OTA evidence attempt only after the fixed host
timeout and qualified OS-native reader changes. Promote `OTA-001` only if the
current admitted package rejects the invalid image, accepts the valid image,
reboots, reports exact source/reference identity, remains fail-closed in safe
state, reports OTA boot validation, passes privacy checks, and is healthy at
cleanup.

Selected-partition internals, rollback, destructive/fault-injection recovery,
interrupted update, OTAWWW, mining, pool access, network longevity, active
voltage/fan/power control, direct UART, and pin manipulation remain non-claims.

## Phase A — detector-only authorization

The only hardware-interacting command authorized before the Phase B contract
is committed is:

```bash
just detect-ultra205
```

Run it exactly once. Continue only if it reports exactly one likely USB serial
port and `espflash board-info --chip esp32s3 --port <port> --non-interactive`
succeeds for board 205. Record the output only in ignored private evidence,
replace every pending port below with that one qualified path, and commit the
updated contract before package, flash, monitor, HTTP, or OTA work. Any other
result is terminal `stop_hardware_blocker`; do not retry detection in Phase A.

Phase A passed once. Its ignored detector log has SHA-256
`7262c900f315b74744e1bd870eac975f4b3d0e60079d117a216460560a80e176` and
selected `/dev/cu.usbmodem1101` as the sole qualified port.

## Phase B — fresh port bound

The two raw roots were confirmed ignored. After this update is committed, that
clean commit becomes the immutable source/package identity for the attempt and
the following exact commands are authorized in order:

```bash
just package
just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=target/advance-parity-ota001-retry-20260802T223139Z/serial-boot capture-timeout-seconds=60 redact-evidence=false
scripts/phase18-firmware-ota-evidence.sh --device-url-from-flash-evidence target/advance-parity-ota001-retry-20260802T223139Z/serial-boot/flash-command-evidence.json --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin --port /dev/cu.usbmodem1101 --out-dir target/phase18-firmware-ota-and-rollback-evidence-dev-raw/retry-20260802T223139Z --monitor-seconds 360
just detect-ultra205
```

The Phase 18 wrapper must derive exactly one origin-only device URL from the
same fresh flash-monitor evidence. It must not print or commit the raw origin.
The OTA helper's valid request is bounded at 120 seconds and its prearmed
OS-native capture spans that upload budget plus the 360-second post-upload
observation budget.

## Evidence policy

- Keep raw detector, USB, serial, HTTP, origin, IP/MAC, Wi-Fi, and device output
  only in the ignored roots named above.
- Pass `wifi-credentials.json` only to the repo-owned flash wrapper. Do not read,
  print, summarize, copy, or commit its contents.
- Committed evidence may contain only public source/reference commits, admitted
  artifact digests, HTTP status/body categories, redacted marker categories,
  and terminal conclusions.
- Run `just verify-redaction` before any result commit.

## Recovery, retries, and stop conditions

There is no second OTA invocation and no second valid upload. If the final
detector succeeds, recovery is prohibited. If and only if it fails after the
single OTA invocation, one recovery command using the same admitted package is
allowed after recording the failure:

```bash
just flash board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
just detect-ultra205
```

Stop without further hardware interaction on a detector or board-info failure,
target ambiguity, wrong board, missing credential input, unqualified reader,
package/manifest/source/reference mismatch, missing or ambiguous same-session
origin, HTTP failure, absent response/reboot/identity/safe-state/boot-validation
marker, privacy failure, or final cleanup failure. Classify the outcome as
`complete`, `stop_repeated_boundary`, `stop_hardware_blocker`,
`stop_authority_boundary`, or `stop_impossible_contract`.

## Promotion gate

Only `OTA-001` may transition, and only when every current-package admission,
invalid rejection, valid upload, scheduled reboot, exact post-reboot identity,
safe-state, OTA boot-validation, cleanup, and privacy criterion passes. Otherwise
leave it `implemented` and record the terminal blocker without progress sync.
