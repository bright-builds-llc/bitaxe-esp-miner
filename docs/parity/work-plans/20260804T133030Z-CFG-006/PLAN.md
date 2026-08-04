# Parity work plan

- Run ID: `20260804T133030Z-CFG-006`
- Parity row: `CFG-006`
- Initial status: `in-progress`
- Source commit: `0bc16ca966d91ed3f87b22ed6d88534dd3c47851`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-cfg006-defaults-matrix`

## Selection

The deterministic selector reported no open plan after `STR-012` closed. All
earlier implemented candidates retain their previously audited live hardware,
safety, network, broad API, logging, release, mining, or consumed-attempt gaps.
`CFG-006` is the first remaining actionable candidate: its hardware catalog is
already comprehensive, and the pinned reference contains a bounded set of 20
board CSV seeds plus one custom seed whose exact board-specific defaults can be
modeled and tested without hardware effects.

## Scope and non-scope

Add one typed board-defaults matrix covering every numbered upstream config
seed and the custom seed. Bind board version, device model, ASIC model,
frequency, voltage, rotation, automatic/manual fan defaults, self-test,
overheat mode, and the one custom pool-port exception to a provenance-bearing
golden fixture. Cross-check each numbered seed against the existing board
catalog and keep Ultra 205 as the sole `ActiveUltra205` entry.

Do not change runtime board selection, enable non-205 firmware builds, claim
non-205 hardware behavior, actuate any control, read credentials, modify the
pinned reference, or promote any ASIC, power, thermal, release, or other-board
row. Public pool seed values remain outside this matrix because they are common
configuration defaults rather than board-profile discriminators.

## Implementation

- [ ] Add a typed `BoardProfileDefaults` API and exact 21-entry seed matrix.
- [ ] Add a provenance-bearing golden fixture generated from the pinned CSV
      discriminators and tests that require exact order and field equality.
- [ ] Cross-check all numbered seeds against catalog ASIC/family/default values
      and require custom seed overrides to remain explicit and non-selectable.
- [ ] Record verification and create `RESULT.md` only if the full pure matrix
      passes while every non-205 hardware claim remains withheld.

## Verification and promotion

Run focused `bitaxe-config` Cargo/Bazel tests, then the mandatory ordered Rust
checks, Bright Builds, `just test`, parity/progress, redaction, reference
cleanliness, and diff checks. Promote only `CFG-006` to `implemented` with
`unit,golden` evidence when all 21 exact seed rows and catalog cross-checks pass.
Do not mark it `verified`: live device behavior for every non-205 profile has
not been exercised. No hardware, recovery, or authorization surface exists.
