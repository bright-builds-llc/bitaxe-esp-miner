# STAT-003 scoreboard evidence summary

- Parity row: `STAT-003`
- Board: Ultra 205 (`205`)
- Attempt: `5`
- Capture source: `a31af2873e6b2d41fe47aa18a57626f33aaf099b`
- Evaluation source: `cbc5fa7f`
- Reference: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Projection: `scoreboard-projection.json`
- Projection SHA-256: `e8054e9176154f154a82b4c9f5301f9d87f64ca558e2ad117be7c37fc4efe920`
- Redaction status: `passed`

## Evidence

The independently validated `bitaxe-scoreboard-evidence-v2` projection binds
the immutable capture and evaluation plans, old capture closure, exact retained
capture package identity commitment, protected input commitment, current
32-path evaluator inventory, campaign result/network/diagnostic seals, and both
capture/evaluator source identities.

The accepted conservative 600-second campaign completed 20/20 continuity
windows and observed qualified nonce plus submit outcome evidence. The public
projection records trusted runtime identity, detector admission, confirmed
safe stop, complete cleanup, valid private modes, no hardware rerun, and passed
redaction. The attempt closure records 19 accepted shares and zero rejects.

The retained scoreboard contains 20 finite-positive, exact-shape, bounded,
uppercase-hex entries in descending order. Both pre-restart reads match. The
live scoreboard SPA was served. One normal restart changed the boot session,
incremented the ordinal once, reported `software_cpu`, recovered the same
capture package identity, and kept boot mining disabled. Both post-restart
reads match. Every post-restart entry equals the pre-restart entry in order and
all non-difficulty fields, while difficulty equals the pinned one-decimal NVS
durable projection.

The old capture stopped only because its v1 verifier incorrectly required full
runtime-difficulty equality. V2 represents the original manifest admission
truthfully through the retained capture package identity and exact old terminal
boundary; it does not claim an unavailable original manifest-byte digest.

## Verification

- Independent Rust v1/v2 validator: passed.
- Semantic redaction scanner: `checked=1`, passed.
- Ordered Cargo format/clippy/build/test and doctests: passed.
- Bright Builds, focused automation, all 48 Bazel tests, firmware build/package,
  reference, parity, progress, selector, file-size, sensitive-value, and diff
  gates: passed before protected evaluation.
- Public projection mode: `0644`; protected v2 wrapper streams: `0600`.

## Conclusion

This evidence supports transitioning only `STAT-003` from `implemented` to
`verified` with `unit,workflow,api-compare,static-route,hardware-smoke,hardware-regression`.

## Non-claims

This result does not prove absolute laboratory difficulty calibration,
profitability, arbitrary pools or profiles, other ASICs/boards, unbounded
mining, OTA/recovery, release readiness, UART/BAP, pins, or electrical behavior.
It publishes no scoreboard rows, pool/owner/worker values, credentials,
endpoints, device/network/USB identity, sensor values, raw logs, or NVS secrets.
