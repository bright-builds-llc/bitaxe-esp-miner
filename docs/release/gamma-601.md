# Gamma 601 package

This package is for development of Gamma 601 support (ADR-0041,
`task-gamma-601-bring-up`). It is not a release, and the release gate
accepts Ultra 205 packages only; a 601 release is a later decision.

## What the firmware does

The firmware boots with the core-voltage output off and the ASIC held in
reset:

- it holds the ASIC reset (GPIO1) low;
- it commands the TPS546 power controller off and confirms the output is
  off;
- it drives the fan at a fixed duty until the fan controller starts;
- it keeps Wi-Fi, HTTP and the USB records.

Since the promotion (ADR-0041 item 15) it then mines like the Ultra 205: it
powers the BM1370 through the TPS546 at the saved profile (upstream's
default is 525 MHz / 1150 mV) when mining starts, on boot by the saved
start-on-boot preference, from a local resume, or under a Worker lease.
Live voltage and frequency changes are off on the 601; a changed profile
applies at the next mining start.

## Installing it

Install it under an active task contract, in one of two ways:
- state-preserving, with `just flash-monitor --board 601 --port <port>`;
- as a factory install, adding `--factory-reset --wifi-credentials <path>`.
  This erases NVS and seeds the 601's upstream defaults
  (`reference/esp-miner/config-601.cvs`) with the Wi-Fi credentials, and
  saves start-on-boot off, so the device stays idle until mining is started.

The flash tool admits the package only to a device registered as board 601
(`just identify-board`).

To return to upstream AxeOS, reflash an upstream Gamma release with the
upstream tools. A factory install leaves the 601's upstream defaults in
NVS, including `selftest=1`. Upstream firmware that keeps those settings
runs its self-test on boot, which powers the ASIC, as a fresh upstream
factory flash does.
