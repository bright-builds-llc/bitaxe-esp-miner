# Parity work result

- Parity row: `ASIC-09`
- Final status: `verified`
- Implementation commit: `7f8ca3bb9d6e9b7b56d1040b1d6d6eeb2bf2648d`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; committed sealed ASIC-002 through ASIC-005 evidence
  was joined without a hardware rerun

## Evidence and verification

The source-bound summary at
`docs/parity/evidence/asic09-mode-separation/summary.md` joins four already
accepted public projections after independent Rust validation:

- ASIC-002 `eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4`
- ASIC-003 `447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c`
- ASIC-004 `e99c054c4d660155d5c2b1ee38d3f17aed5ae7101e7e4a5fd1c6451d1b48b7c7`
- ASIC-005 `bad828db694ee59c4ef3d77b2e58ef89e0195ef382526b97912d0a71e882ad69`

The same-attempt chain from hardware commit
`3e0966a140edbff1a14d2a48ca63d140649762c0` proves all nine initialization
steps, exactly one chip, mining-ready completion, retained production UART, a
required production-ready gate, typed production work, a qualified parsed and
correlated result, live work TX and result RX, an accepted response, fresh
safety, confirmed safe stop, cleanup, trusted identity, and passed redaction.
Each projection is mode `0644`. No new projector or protected artifact was
used.

Current tests prove diagnostic modes require exact compile-time
acknowledgements and otherwise fail closed, while production commands contain
only production work and result variants. The production executor source
contains no diagnostic-work command. Host `cargo test -p bitaxe-firmware`
cannot compile the ESP-IDF crate on this host; the same source contract was
reviewed directly, and `just package` compiled the current firmware image.

The following focused gates passed on source `7f8ca3bb`:

- independent validation of all four projections
- `cargo test -p bitaxe-asic adapter_gate`
- `cargo test -p bitaxe-asic production`
- production-executor source review
- `just verify-reference`
- `just package`

## Conclusion

`ASIC-09` has a closed proof that live Ultra 205 BM1366 production
initialization, work, result, and UART behavior used the production path, while
current source keeps diagnostic modes fail-closed and unreachable from the
production executor. This supports `ASIC-09` at `verified` with
`unit,golden,workflow,hardware-smoke,hardware-regression` evidence.

## Non-claims and residual risks

This result does not verify arbitrary diagnostic builds, frequency transitions,
voltage/fan/thermal behavior, nonzero version-mask or multi-midstate breadth,
arbitrary-load serial behavior, other ASICs or boards, arbitrary pools or
profiles, unbounded mining, OTA/recovery, or release readiness. It does not
promote ASIC-10, ASIC-11, ASIC-12, STR-08, or STR-09. No credential or
protected-attempt access, detector, device/USB/network runtime, flash,
monitor, mining, restart, recovery, hardware attempt, fault injection, external
UART/BAP, pins, or electrical work occurred during this plan.
