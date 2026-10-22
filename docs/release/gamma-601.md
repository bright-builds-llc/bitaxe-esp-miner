# Gamma 601 safe-boot package

This package is for development of Gamma 601 support (ADR-0041,
`task-gamma-601-bring-up`). It is not a release, and the release gate
accepts Ultra 205 packages only.

## What the firmware does

The firmware boots with the core-voltage output off and the ASIC held in
reset, and it does not mine:

- it holds the ASIC reset (GPIO1) low;
- it commands the TPS546 power controller off and confirms the output is
  off;
- it drives the fan at a fixed duty;
- it keeps Wi-Fi, HTTP and the USB records.

## Installing it

Install it under an active task contract, in one of two ways:
- state-preserving, with `just flash-monitor --board 601 --port <port>`;
- as a factory install, adding `--factory-reset --wifi-credentials <path>`.
  This erases NVS and seeds the 601's upstream defaults
  (`reference/esp-miner/config-601.cvs`) with the Wi-Fi credentials.

The flash tool admits the package only to a device registered as board 601
(`just identify-board`).

To return to upstream AxeOS, reflash an upstream Gamma release with the
upstream tools. A factory install leaves the 601's upstream defaults in
NVS, including `selftest=1`. Upstream firmware that keeps those settings
runs its self-test on boot, which powers the ASIC, as a fresh upstream
factory flash does.
