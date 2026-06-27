# Phase 03 Ultra 205 BM1366 Chip-Detect Evidence

**Date:** 2026-06-27
**Scope:** Human-gated Ultra 205 BM1366 chip-detect diagnostic smoke
**Reference:** `reference/esp-miner` at `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Command

```bash
bazel build --action_env=BITAXE_ASIC_DIAGNOSTIC=chip-detect --action_env=BITAXE_HARDWARE_EVIDENCE_ACK=ultra205-chip-detect-safe-bench //firmware/bitaxe:firmware_image
bazel run //tools/flash:flash -- flash-monitor --board 205 --port <port> --image bazel-bin/firmware/bitaxe/bitaxe-ultra205.elf --evidence-dir docs/parity/evidence/phase-03-ultra-205-bm1366-chip-detect
```

## Board

Pending live run. Required value before hardware evidence can be accepted: `Ultra 205`, board version `205`, ASIC `BM1366`.

## Port

Pending live run. A visible serial device alone is not evidence of board identity or safe bench setup.

## Firmware commit

Pending live run. The fail-closed firmware adapter baseline before this evidence record was `525f1b8`.

## Reference commit

`c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Relevant logs

No live chip-detect logs captured yet. Required logs before promotion to `hardware-smoke` evidence:

- `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`
- `asic_status=...` lines showing diagnostic gate, chip-detect status, observed chip count, and fail-closed/no-mining conclusion
- `flash-monitor.log` captured by the flash tool evidence directory

## Observed result

not run - hardware verification pending

## Skipped gates

- Live flashing and monitoring were not run during automated execution.
- No Ultra 205 board identity, safe bench approval, serial port, or chip-detect log evidence has been captured for this file yet.
- Production mining, Stratum pool connection, voltage control, fan control, thermal control, power enable, and production work submission remain skipped.

## Conclusion

not run - hardware verification pending

Until a human approves the safe bench setup and live chip-detect run, ASIC initialization, serial transport, frequency transition, diagnostic work-send, and result-receive rows must remain below `verified`.
