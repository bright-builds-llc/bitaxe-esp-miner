# Parity work result

- Parity row: `STAT-002`
- Final status: `verified`
- Implementation commit: `0fe0c9aa81e3b604b6262c22f74a5e657b28596b`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The single detector-gated `attempt-003` produced the committed aggregate-only
projection at
`docs/parity/evidence/stat002-statistics-history/statistics-history-projection.json`.
The independent Rust validator accepted it as
`bitaxe-statistics-history-evidence-v1`. It binds the exact clean pushed
implementation package, pinned reference, immutable plan, one admitted Ultra
205, a completed factory flash, passive safe state, and one current-session
same-origin API transaction.

The transaction changed only `statsFrequency`, confirmed the enabled readback,
and observed four finite 19-column samples with three strictly increasing
1,000-millisecond intervals. The immediate repeat was unchanged and the later
read demonstrated producer growth. Exact original-setting restoration passed,
including the zero-setting clear behavior. Mining and hardware control remained
disabled; no recovery flash was needed. Cleanup, owner-only private modes, and
redaction all passed.

Commands run included the ordered Rust gates, Bright Builds checks, exact
ESP32-S3 package build, the full Bazel suite, parity report/progress checks,
reference and redaction verification, `just detect-ultra205`, the sole planned
`just capture-statistics-history-evidence` invocation, and the independent
`validate_statistics_history_evidence` Rust binary.

## Conclusion

The live result closes the remaining device cadence/API/restoration gap for
`STAT-002`. It also proves that the corrected 900-second whole-operation owner
allows the flash, setup, 360-second child monitor, and post-monitor result
delivery to complete beyond the former 420-second boundary.

## Non-claims and residual risks

This result does not verify the physical accuracy of voltage, current, power,
temperature, fan, or ASIC telemetry. The legacy statistics `voltage` and
`current` wire fields remain millivolts and milliamps. It also does not verify
browser charts, mining, ASIC work, hardware controls, OTA, recovery behavior,
other boards, release readiness, or full-duration 720-sample retention on live
hardware; exact bounded eviction remains covered by deterministic tests.
