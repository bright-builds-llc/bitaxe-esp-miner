# Chip-Detect Hypothesis Results

Hardware: Ultra 205 board `205`, port `/dev/cu.usbmodem1101`  
Evidence: `.planning/phases/27-live-hardware-asic-and-stratum-bridge/chip-detect-trace-20260705/flash-monitor.log`  
Firmware: Phase 27 package build with upstream-aligned chip-detect fix (uncommitted workspace)

## Summary

| Hypothesis | Result | Evidence |
| --- | --- | --- |
| H1 — single-shot read vs accumulate | **Confirmed helpful; not sole root cause** | Trace shows full 11 bytes in one chunk after fix bundle; accumulate prevents future partial-delivery failures |
| H2 — missing version mask ×3 prelude | **Confirmed required** | Without prelude (prior runs): stable 9/11 partial read. With 3× mask TX: valid 11-byte chip-ID RX |
| H3 — WaitTxDone before read | **Applied; inconclusive alone** | Trace shows `wait_tx_done outcome=Ok(()) elapsed_ms=0` before RX; bundled with H2 |
| H4 — full_init prefix scope | **Not required for chip detect** | Chip detect passes with upstream-aligned prelude only; full_init investigation build available via env |
| H5 — double reset pulse | **Applied (skip second reset)** | Safety bring-up pulse retained; chip-detect plan reset skipped when Phase 27 upstream-aligned options active |

## Trace highlights (post-fix boot)

```text
phase27_safety_bring_up=complete
asic_status=chip_detect_only
asic_uart_trace=tx len=11 hex=55 aa 51 09 00 a4 90 00 ff ff 1c  (×3 version mask)
asic_uart_trace=tx len=7 hex=55 aa 52 05 00 00 0a                 (read chip id)
asic_uart_trace=rx read_index=1 chunk_bytes=11 total_bytes=11
asic_uart_trace=rx_complete hex=aa 55 13 66 00 00 00 00 00 00 05
asic_status=chip_detected chips=1
```

## Fix bundle landed

1. `read_accumulate` in `firmware/bitaxe/src/asic_adapter/uart.rs` (replaces single-shot read)
2. Phase 27 default `ChipDetectPlanOptions::upstream_aligned_after_safety_bring_up()` via `chip_detect_investigation.rs`
3. Investigation compile env `BITAXE_CHIP_DETECT_INVESTIGATION` for isolated H2/H3/H4/H5 experiments
4. Phase 27 UART trace logging (`asic_uart_trace=*`)
5. Host tests: split 9+2 transcript, uart_accumulate unit tests, upstream-aligned init plan test

## Remaining blocker (out of chip-detect scope)

Boot path still hits `asic_status=fail_closed reason=work_result_diagnostic_timeout` ~10s after chip detect. Production bridge retains UART only when diagnostic work result parses. Share proof attempt remains blocked on work-result path, not chip detect.

## Operational note

`just flash-monitor manifest=bazel-bin/...` rebuilds **default** firmware without Phase 27 action env and overwrites `bazel-bin` artifacts. After `./scripts/phase27-live-hardware-bridge-package.sh`, flash with explicit `image=<bazel-out factory.bin>` or re-run the package script immediately before flash.
