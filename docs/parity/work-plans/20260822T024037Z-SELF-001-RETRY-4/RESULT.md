# Parity work result

- Parity row: `SELF-001`
- Final status: `verified`
- Implementation commit: `a11b579b62cb52a53bbf6072bde209d3eb3f17e2`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempt: detector-admitted Ultra 205 `attempt-005`

## Evidence and verification

The closed public projection at
`docs/parity/evidence/self001-full-lifecycle/self-test-projection.json`
has SHA-256
`1cbc357cb76c51bb354162e48f485442a3ce5eaeb03aead3b71ed50fd3235090`.
It binds board 205, source `a11b579b`, pinned reference `c1915b0a`, exact app
ELF and package digests, attempt ordinal 5, and plan digest
`0c9a03ec490967fc95989d88b91848c7d4ed740a76825822a8107d94e8fd7f84`.
The Rust contract validator and semantic redaction check passed independently.

Phase A proves five stable seconds of deterministic diagnostic work, advancing
watchdog/safety evidence, the closed planned evaluation failure, complete
safe-stop, safe physical-cancel checkpoint, the user's built-in two-second
BOOT hold, lease-bound cancellation receipt, and cancellation restart.

Phase B proves PSRAM, 485 MHz, 1200 mV, difficulty 16, warm-up, the complete
30,101-ms workload, four BM1366 counter-register domain averages, electrical
and fan bounds, watchdog progression, complete safe-stop, pass receipt,
ten-second countdown, automatic exact-build restart, and ordinary runtime.
The private hardware metrics were 1257.913 GH/s total; domain averages
105.993, 121.261, 84.740, and 109.313 GH/s with 21/53/20/53 accepted samples
and zero rejected samples; 5.276 V input, 1186 mV core, 12.300 W, 2188 RPM,
and 61.00 C maximum. These values remain aggregate hardware facts and contain
no network or credential identity.

The supervisor captured restorable settings before the first write, used local
credentials only in memory, restored every confirmed field and theme with
`mineonboot=false`, observed no production mining or pool traffic, released
USB/process ownership, and published the projection only after validation.
Protected attempt files remain ignored under mode-0700 roots with mode-0600
files.

Focused counter, evaluator, firmware ownership, receipt replay, restoration,
real-child recovery, intent, and evidence tests passed. The canonical six-file
ESP32-S3 package, ordered Cargo gates, Bright Builds, all 51 Bazel tests,
parity/progress, redaction, reference cleanliness, file-size, sensitive-value,
and diff checks passed before hardware execution.

## Conclusion

The complete production-safe Ultra 205 self-test lifecycle is verified with
`unit,workflow,hardware-regression` evidence. The controlled failure proves
lifecycle rollback without electrically stressing the board; the passing phase
proves the upstream-compatible operational envelope and both restart paths.

## Non-claims and residual risks

This result does not claim actual overheat, zero-RPM, sensor, power, ASIC, or
communication fault injection; other boards/ASICs; arbitrary profiles;
unbounded load; pool mining; external electrical interfaces; OTA/recovery; or
release readiness. Those require separate authorized evidence.
