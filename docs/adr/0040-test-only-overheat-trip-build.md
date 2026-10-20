# ADR-0040: A test-only build lowers the local overheat trip

## Status

Accepted on 2026-10-20 by the owner ("Test-only lowered trip"), for
`task-local-mining-overheat-check`.

## Context

ADR-0038 decision 2: after an overheat stop, local mining restarts 100 MHz
and 100 mV lower after 30 s of chip readings at 45 C or less, inside the
catalog range; otherwise it stays stopped. That recovery has unit and
fake-device coverage, but no hardware evidence, because the Ultra 205 mines
at about 60 C and never reaches the 75 C trip. Heating the board would be
an uncontrolled electrical and thermal stress. The THR-001 thermal fault
stimulus fakes an invalid reading rather than a hot chip, and its boot
never mines locally.

The 75 C stop is the shared mining-safety verdict, run by the revocation
gate for local runs and leases alike. Lowering that constant would change
every other protection.

## Decision

- **The trip:** `bitaxe-runtime` defines `LOCAL_OVERHEAT_TRIP_C`. It is
  upstream's 75 C, or 55 C under the test-only Cargo feature
  `overheat-trip-test`.
  - In the test build only, the revocation gate stops a local generation
    whose latest fresh chip reading reaches it, with no failing fact, and
    `is_overheat` labels the stop with the same value.
  - A production build compiles no such branch, and its `is_overheat`
    keeps 75 C, so its behaviour is unchanged.
  - A compile-time assertion keeps the trip above the 45 C restart and at
    or below 75 C.
- **The build:** the Bazel flag `--//firmware/bitaxe:firmware_variant`
  (`production` by default, or `overheat-trip-test`) selects the build,
  as the web UI variant does (ADR-0034).
  - The test variant builds in its own Cargo target directory.
  - It retains a boot line carrying the marker
    `firmware_variant=overheat-trip-test`.
  - Its package manifest records `firmware_variant`, checked against the
    marker in the ELF.
  - `build.rs` refuses the feature with a release tag.
- **Installing it:** the flash tool refuses a non-production variant unless
  `--allow-test-firmware <variant>` names it. A test keeps the marker out
  of the production firmware.
- **Checking it:** the soak tool's `overheat-001` runs only on the test
  variant, and every other soak only on production.
  - It confirms the trip's cause from the retained revocation detail,
    which every local overheat stop now logs before its `run_ended` line.
  - It waits until the firmware's recovery has settled before restoring
    settings, because the recovery continues while the device is paused.
  - Each hardware run ends by reinstalling production and confirming its
    settings.
- **Releasing:** the release gate refuses a package whose
  `firmware_variant` is not `production`.

## Consequences

- The recovery, from the overheat label on, runs through the production
  code path. The trip itself is the test build's own branch: it revokes
  with no failing fact, where the shared verdict names the chip.
- In the test build, any unsafe stop with the chip at 55 C or more is
  labelled an overheat; the check rejects a trip that names another
  fact.
- The recovery's log lines are now retained, so the API log shows them in
  every build.
- Leases keep the 75 C stop in both builds; the test variant changes local
  mining only.
- A test-variant image could still be uploaded by hand through the web UI's
  update page or `tools/device-session`'s HTTP update, neither of which
  reads a manifest. It is never published, and its retained boot line
  names it.
