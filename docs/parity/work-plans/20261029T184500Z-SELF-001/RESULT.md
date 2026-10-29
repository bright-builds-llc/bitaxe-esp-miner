# Parity work result

- Parity row: `SELF-001`
- Final status: `verified`
- Implementation commits: `b5940eba` (the 205's attempt 007 package) and
  `bb9f2ae7` (the 601's attempt 009 package)
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

- **Ultra 205:** the redacted summary
  `docs/parity/evidence/20261029-ultra205-f46-self-test.md`, with its
  committed projection
  `docs/parity/evidence/self001-full-lifecycle/attempt-007/self-test-projection.json`.
  The run used `just self-test-campaign` on the exact production 205
  package at `b5940eba`, on attempt 005's unchanged contract, under the
  rerun plan.
- **Gamma 601:** the redacted summary
  `docs/parity/evidence/20261028-gamma601-production-self-test.md`, with
  its committed projection
  `docs/parity/evidence/self001-gamma601/attempt-009/self-test-projection.json`.
  The run used the same campaign on the exact installed production 601
  package, under the 601's immutable plan. The test build's earlier pass
  (`docs/parity/evidence/20261027-gamma601-self-test.md`, attempt 006)
  agrees.

Both completed every phase and were independently checked against their
private captures.

## Revision

The notes gain this passage:

Later evidence reruns this lifecycle at the final Gamma 601 bring-up commit and narrows the other-boards non-claim for the Gamma 601 only. On the Ultra 205, production b5940eba completed attempt 005's unchanged contract on attempt 007, under docs/parity/work-plans/20261029T050000Z-SELF-001-ULTRA205-RERUN/PLAN.md: the controlled failure, its safe stop, the owner's BOOT cancellation and restart, then a pass at 387.3 GH/s over 30.16 s against at least 368.6 GH/s, four BM1366 domains at 104.1 to 116.3 GH/s, 12.47 W, 1198 mV, the chip at most 60 C, the receipt, the automatic restart and the settings restored; see docs/parity/evidence/20261029-ultra205-f46-self-test.md and docs/parity/evidence/self001-full-lifecycle/attempt-007/self-test-projection.json. Attempt 005's hashrate window also counted the warm-up's nonces and repeated one identical job; since cd9aee8e only the measured window counts and since 22ee4801 every dispatch sends new work, and attempt 007 is the 205's pass under that measurement. On the Gamma 601, under its board-aware contract (`gamma601_full_v1`, 525 MHz / 1150 mV, a 22 W ceiling, the TPS546 fault and regulator-temperature checks, and the regulator on under a 240 s bound), the promoted production 601 build at bb9f2ae7 completed the same lifecycle on attempt 009: the controlled failure, its safe stop, the owner's BOOT cancellation and restart, then a pass at 1028.4 GH/s over 30.07 s with four BM1370 domains at 260.3 to 272.1 GH/s, 18.8 W, the chip at most 65.62 C and the regulator at most 54.75 C, the receipt, the automatic restart and the settings restored; see docs/parity/evidence/20261028-gamma601-production-self-test.md and docs/parity/evidence/self001-gamma601/attempt-009/self-test-projection.json. The 601 also checks its input voltage from the TPS546, where upstream skips that check without an INA260. Other boards beyond the 205 and the 601, and the remaining non-claims above, still apply.

## Non-claims

Nothing beyond the redacted summaries is claimed. The status, the evidence
tokens and the owned target are unchanged. The 601's own seam,
`firmware/bitaxe/src/self_test_runtime/board.rs`, is not added to the owned
target, because a verified revision cannot change it. The board-aware
contract lives in the already owned
`crates/bitaxe-safety/src/self_test/hardware.rs`.
