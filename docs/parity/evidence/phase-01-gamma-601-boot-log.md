# Phase 01 Gamma 601 Boot/Log Evidence

## Command

Command template:

```bash
just flash-monitor board=601 port=<port> evidence-dir=docs/parity/evidence/phase-01-gamma-601-boot-log
```

Precheck command:

```bash
espflash list-ports
```

Precheck output:

```text
No known serial ports found.
```

## Board

Gamma 601 with BM1370 ASIC.

## Port

No compatible Gamma 601 serial port was visible during the Phase 01 Plan 09 precheck.

## Firmware Commit

firmware_commit=7657b54

## Reference Commit

reference_commit=c1915b0a63bfabebdb95a515cedfee05146c1d50

## Manifest Path

`bazel-bin/firmware/bitaxe/bitaxe-gamma601-package.json`

## Default Flash Image

The package manifest default flash image is `bitaxe-gamma601.elf`. `bitaxe-gamma601-factory.bin` is an additional package artifact only, not the default `espflash flash` input.

## Log Path

Not created. No Gamma 601 serial port was visible, so `just flash-monitor` was not run.

## Observed Required Log Lines

No hardware boot log was captured. These exact required patterns must be observed before boot/log rows are marked `verified`:

- `bitaxe-rust boot: board=Gamma 601 asic=BM1370`
- `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`
- `reset_reason=`
- `partition=` or `image_partition=`
- `platform_status=` or `psram_status=`
- `firmware_commit=7657b54` or `firmware_commit=Unavailable`
- `reference_commit=c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Conclusion

Conclusion: missing hardware-smoke evidence - no Gamma 601 serial port visible.
