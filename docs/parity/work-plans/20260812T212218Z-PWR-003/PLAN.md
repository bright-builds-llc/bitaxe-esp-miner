# Parity work plan

- Run ID: `20260812T212218Z-PWR-003`
- Parity row: `PWR-003`
- Initial status: `implemented`
- Source commit: `2264f71393949436f1f15306b71f890a6478dc0a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-pwr003-core-voltage-control-evidence-retry`

## Selection

The clean synchronized selector returned no open plan and ranked `API-009`
first, followed by `PWR-003`. `API-009` is temporarily unavailable: its latest
closure requires an explicit pre-effect report that the operator is present,
watching the display, and ready to answer both live prompts. The current report
confirms display observation but not readiness for both time-bounded replies,
so no new API-009 ordinal may be consumed or inferred.

`PWR-003` is the next actionable candidate. Its prior immutable plan closed
after the sole projection attempt failed before candidate creation: the
configured substring `CORE_VOLTAGE_STABILIZATION_MS,` appears once in an
import and once at the intended use site. The accepted PWR-002 projection,
production behavior, and hardware lineage remain intact. This fresh retry is
limited to correcting that host-side semantic matcher and proving the real
production-file boundary before one new software-only projection attempt.

## Scope and non-scope

Replace the ambiguous stabilization substring with a source-shaped fragment
that uniquely identifies the 500 ms sleep immediately before ASIC enable.
Bind the projector to this immutable plan and its fresh active task. Add a
regression that reads the real production
`firmware/bitaxe/src/mining_actuation_adapter.rs` and proves every configured
semantic fragment is admitted exactly once, including the stabilization use
site that the previous fixture failed to model.

This is a software-only evidence audit. It permits reading committed source,
reference, task, plan, and accepted evidence; editing the PWR-003 projector and
its tests; running repository build, test, projection, validation, redaction,
and integrity commands; and atomically publishing one public redacted typed
projection after every gate passes. It does not permit package construction,
device detection, flash, reset, USB or serial access, network requests,
credentials, mining, voltage, fan, power, GPIO, I2C, direct UART, pins, fault
injection, or any other hardware effect. The sealed PWR-002 projection remains
the sole hardware evidence source.

## Implementation

- [ ] Replace only the ambiguous stabilization matcher with an exact
      source-shaped fragment at the intended sleep site.
- [ ] Bind the projector and its fixtures to this plan, task, and immutable
      plan digest without weakening source, task, or clean-tree admission.
- [ ] Add a behavior-focused production-file regression that would reject the
      prior matcher and admits the corrected complete matcher set.
- [ ] Run the focused and mandatory repository gates, commit, and push the
      exact implementation before projection.
- [ ] Run exactly one fresh software-only projection attempt from the accepted
      PWR-002 evidence and independently validate the result.
- [ ] Produce the checklist's required evidence and promote only on the full
      closed quorum.

## Verification and promotion

Focused verification includes the automation TypeScript suite, the Rust
core-voltage evidence contract tests, generated-contract checks, and a
production-file semantic admission regression. Mandatory verification runs in
order: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D
warnings`, `cargo build --all-targets --all-features`, `cargo test
--all-features`, `bun scripts/bright-builds-check.ts all`, `just test`, `just
parity`, and `just parity-progress`. Redaction, pinned-reference cleanliness,
immutable-plan digest, unique task binding, candidate absence, final mode,
source compatibility, selector, and diff checks must also pass.

The sole accepted source is
`docs/parity/evidence/pwr002-asic-power-initialization/power-initialization-projection.json`
with SHA-256
`0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`
and admitted hardware source commit
`3e0966a140edbff1a14d2a48ca63d140649762c0`. The only permitted output is
`docs/parity/evidence/pwr003-core-voltage-control/core-voltage-control-projection.json`;
its candidate must be absent before and after publication, and the final file
must be mode `0644`.

Promotion to `verified` requires independent validation of the exact source
projection and digest, trusted package/runtime identity, the source-bound
DS4432U address/register/code and single-write route, 1100 mV command, complete
500 ms stabilization before active-low ASIC enable, successful downstream
accepted work, active-low safe stop, cleanup, no hardware rerun, passed
redaction, and atomic publication. Any malformed, ambiguous, dirty, missing,
or failed boundary must preserve the earliest typed failure, remove a partial
candidate, withhold public evidence, keep `PWR-003` at `implemented`, record a
terminal closure, and stop without a second projection attempt.
