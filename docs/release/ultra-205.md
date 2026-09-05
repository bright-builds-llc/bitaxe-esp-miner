# Ultra 205 Release and Operator Guide

The active automation interface is `bitaxe-automation`, exposed through fixed
Bazel targets and the recipes below. Flags always use `--kebab-case value`.
Legacy `key=value`, underscore aliases, positional fallbacks, and phase-numbered
commands are intentionally unsupported.

## Prerequisites

Run the read-only dependency and reference checks:

```sh
just doctor
just verify-reference
```

If ESP tooling is missing, install it explicitly:

```sh
just bootstrap-esp
```

The canonical target is Ultra 205 / ESP32-S3. Packaging uses the ESP-IDF tools
managed by the pinned esp-rs build under `.embuild/`.

## Build and package

```sh
just build
just package
```

The package manifest is:

```text
bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
```

Treat that manifest as the authority for the exact ELF, executable image,
factory image, source commit, reference commit, and artifact digests.

## Detect the board

Before any hardware use:

```sh
just detect-ultra205
```

Proceed only when exactly one likely port is reported and ESP32-S3 board-info
passes. Supply the returned path as `--port <path>`; do not infer a target from
mDNS, ARP, network scans, or old logs.

## Flash and monitor

Normal developer commands remain concise Rust-backed recipes:

```sh
just flash --board 205 --port <path> \
  --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json

just monitor --port <path> --capture-timeout-seconds 360

just flash-monitor --board 205 --port <path> \
  --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json \
  --capture-timeout-seconds 360
```

For local Wi-Fi bring-up, the ignored credential file may be passed without
reading or copying its contents:

```sh
just flash-monitor --board 205 --port <path> \
  --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json \
  --wifi-credentials wifi-credentials.json \
  --capture-timeout-seconds 360
```

Never commit Wi-Fi or pool credentials, raw device URLs, IP/MAC addresses,
SSIDs, USB paths, tokens, workers, or NVS secret values.

## Semantic verification commands

The typed interface provides these fixed commands:

```text
doctor                         bootstrap-esp
build-firmware                 package-firmware
verify-reference               verify-redaction
verify-production-session      observe-serial
verify-flash-durability        verify-firmware-ota
verify-web-assets-ota          verify-recovery
verify-http-api                verify-hardware-surface
verify-mining                  capture-operator-evidence
verify-settings-durability     capture-correlated-runtime-evidence
capture-version-evidence
```

OTA, recovery, hardware-control, and mining commands require a task-scoped
typed request. A request records a Rust-owned workflow identity, SHA-256 digest,
structured constraints, authorization, recovery path, retry bounds, and stop
conditions. Missing or unimplemented effect authority exits with policy code 3.

Safety and mining manifests bind a semantic workflow plus request digest, so
unsupported flags cannot be introduced after admission. String-based command
admission has been removed.

## Read-only HTTP verification

Use only an origin obtained from the same current detector/monitor session:

```sh
bazel run //tools/automation:verify_http_api -- \
  --device-url <origin> \
  --route /api/system/info \
  --output scratch/system-info.private.json
```

The adapter rejects redirects, credentials in URLs, non-origin targets, and
cross-origin routes. Raw responses are protected operational files.

## Exact-package version evidence smoke

This is the cutover acceptance workflow. It permits one exact-package flash,
bounded passive boot observation, and one same-origin system-info read. It does
not authorize mining, voltage/frequency/fan actuation, erase, OTA, fault
injection, direct UART, or pin manipulation.

The task contract must name an absent private root and a shareable projection:

```sh
just capture-version-evidence \
  --private-root scratch/automation-refactor/attempt-001 \
  --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json \
  --wifi-credentials wifi-credentials.json \
  --port <path> \
  --projection docs/parity/evidence/automation-refactor/attempt-001/version-evidence.json \
  --capture-timeout-seconds 45
```

The private root is created mode `0700`; private logs and API responses are mode
`0600`. The only shareable output is a Rust-validated
`bitaxe-version-evidence-v1` JSON projection containing digests and categorical
facts, never raw operational identifiers.

There is no automatic retry. After a confirmed effect, only bounded recovery
with the same admitted package is permitted by an explicit task contract.

## Redaction and release checks

```sh
just verify-redaction
just verify-production-session
just parity
```

`verify-redaction` scans active semantic evidence schemas. Historical evidence
is immutable and is not rewritten by the automation cutover. The migration
ledger at `docs/parity/automation-migration.json` records equivalence decisions
and any rows downgraded when an old evidence schema was retired.

## Fixed Serial/JTAG update policy

ADR-0021 makes ordinary `just flash` and `just flash-monitor` state-preserving.
A schema-4 package declares bootloader, binary partition table, application,
web image and boot-selection segments; sector-rounded writes exclude NVS and
unrelated partitions. The merged factory image remains an explicit factory
installation artifact, not the normal update command. Existing Device Identity,
Wi-Fi settings and authorization replay marks survive routine updates.

`--wifi-credentials` requires explicit factory provisioning/reset; normal
updates reject it rather than silently replacing NVS. `--factory-reset` is a
separate destructive choice that resets stored settings and Device Identity;
it is not authorized by the fixed-USB migration's ordinary update or recovery
steps. Provision Wi-Fi through the existing configuration flow when needed.

The browser uses Web Serial directly and must release its streams/port before
flashing. Shared Serial/JTAG descriptors never prove application identity;
verify the exact running source/ELF after every update. Qualification uses the
new safe baseline, not recovery-006 or historical TinyUSB evidence.
