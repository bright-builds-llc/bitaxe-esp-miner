# Parity work result

- Parity row: `CFG-07`
- Final status: `verified`
- Implementation commit: `04ecfab523bbeacead9871f4107e0d79426fe385`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The accepted projection is
`docs/parity/evidence/cfg07-runtime-credentials/runtime-credentials-projection.json`
(SHA-256
`7840b62bf8aef9104e254202dbe007e00c54510ca30e30e1d0949f5ac437d206`).
It was created exactly once from committed public inputs by the frozen plan
command and passed `just validate-cfg07-evidence` independently.

The projection joins detector-admitted scoreboard attempt-003, accepted SAFE-10
live mining evidence, the immutable attempt command, exact attempt/current
credential semantics, current evaluator identity, and pinned reference. It
proves that local-owner-supplied Wi-Fi and pool inputs were both required,
forwarded, validated, and consumed by the accepted live mining campaign; an
accepted submit was observed; safe stop and cleanup completed; and protected
source semantics plus redaction remained valid.

The projector accepts no credential or protected-attempt path and records that
it read no credential contents. Its public output contains no credential paths
or values and records `committed_credential_values: none` plus
`raw_artifacts_committed: no`. Projection mode, generic redaction, and a direct
sensitive-pattern review pass.

The canonical Phase 30 conclusion now has explicit promoted disposition and the
four exact CFG-07 proof fields required by parity admission. Focused Rust,
TypeScript, real-process, source-drift, predecessor, and Phase 30 tests pass,
along with the real firmware package, ordered Cargo gates, Bright Builds, all
Bazel targets, reference verification, parity/progress, source inventory,
file-size, and diff checks.

## Conclusion

The accepted same-chain evidence directly closes the checklist gap: runtime
credential inputs were consumed during detector-gated accepted live mining,
while every committed artifact retained closed category labels and no credential
values. This supports `CFG-07` at `verified` with
`unit,workflow,hardware-smoke,hardware-regression` evidence.

## Non-claims and residual risks

This result does not expose or independently validate credential contents or
verify rotation/persistence beyond the accepted campaign. It does not promote
STR-09 or ASIC-11; verify arbitrary profiles/pools; inject faults; verify active
controls, self-test, BAP/UART, other boards/ASICs, unbounded mining,
OTA/recovery, or release readiness.
