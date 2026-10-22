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

Install it only state-preserving, with
`just flash-monitor --board 601 --port <port>`, under an active task
contract. The flash tool admits it only to a device registered as board 601
(`just identify-board`), and it refuses a factory install for the 601.

To return to upstream AxeOS, reflash an upstream Gamma release with the
upstream tools.
