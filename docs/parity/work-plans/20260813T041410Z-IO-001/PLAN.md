# Parity work plan

- Run ID: `20260813T041410Z-IO-001`
- Parity row: `IO-001`
- Initial status: `implemented`
- Source commit: `3f3358dc720dd77cc3014995c9b6b89bbde9515c`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-io001-i2c-retry-contract`

Continues `docs/parity/work-plans/20260804T135918Z-IO-001/PLAN.md`.

## Selection

The clean synchronized selector returned no open plan and ordered `API-009`,
`THR-001`, then `IO-001`. `API-009` is sealed at
`stop_repeated_boundary`: its exact post-fix command-effect boundary recurred,
attempt-008 is prohibited, and its next eligible action requires new
software-only diagnostic information. `THR-001` requires a distinct bounded
overheat or thermal-fault hardware-regression contract, but the repository
owns no vendor-safe stimulus command and ordinary mining cannot guarantee a
bounded 75 C transition without unsafe stress. Neither row is actionable in
this invocation.

`IO-001` is the first actionable row. Its exact upstream 500 ms bus-acquisition
timeout, three transfer attempts, and 10 ms delay after every failed attempt
are already implemented and covered by focused host tests. The earlier row
closure remained at `implemented` because its SSD1306 hardware breadcrumb
predated the retry owner and shared safety-peripheral behavior had not yet been
reconciled against post-retry evidence. Subsequent independently validated
board-205 artifacts and one sealed current-generation campaign now cover that
gap without another device effect.

This plan follows the repository evidence policy and the Bright Builds
architecture, code-shape, verification, testing, and Rust/TypeScript standards.
It composes existing authoritative boundaries instead of adding a duplicate
capture path or weakening a failed row's claims.

## Scope and admitted evidence

This is a read-only evidence-reconciliation task. It may read committed public
artifacts and only the closed, aggregate, redaction-safe fields and digests of
the ignored protected API-009 attempt-007 campaign. It may run local validators,
source comparisons, tests, builds, parity tooling, and repository checks; write
the row-specific result; and apply one typed checklist transition.

The admitted evidence is fixed:

- `docs/parity/evidence/pwr006-ina260/ina260-projection.json`, SHA-256
  `c9624b3c77e4021137a375de2a70c2bf7425bc947af6ba59c4e42fbceb25634d`,
  mode `0644`, proves post-retry exact-package INA260 reads of registers 1, 2,
  and 3 through matching fresh HTTP and WebSocket samples.
- `docs/parity/evidence/thr001-emc2101-thermal/thermal-projection.json`,
  SHA-256
  `d599357460b8b26431e8e362a3ff4c4f68572856f5cf1960631aa84046a345b5`,
  mode `0644`, proves post-retry exact-package EMC2101 address `0x4c`, register
  `0x00`, and matching fresh HTTP and WebSocket temperature samples.
- `docs/parity/evidence/pwr002-asic-power-initialization/power-initialization-projection.json`,
  SHA-256
  `0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`,
  and `docs/parity/evidence/pwr003-core-voltage-control/core-voltage-control-projection.json`,
  SHA-256
  `11dd1abbf6fda86d203fdcff49b420ab5139e1d29c35f4d17000c61c3112ae68`,
  establish the physical fan/voltage actuation semantics, exact DS4432U
  address/register/code, successful initialized work, accepted submit, and
  safe stop. Their physical run predates the retry implementation, so they are
  admitted only for the device/effect chain and never as post-retry proof.
- The sealed protected attempt rooted at
  `scratch/api009-command-effects/attempt-007/campaign` has mode `0700`,
  mode-`0600` result/diagnostic/seal files, result SHA-256
  `0d17c5980a86536217ec86ba01e50d23e8f6f496d25d937a8003091ac4a1b744`,
  diagnostics SHA-256
  `a6efe832cf16f3cd422e95da9744f0b378cb00ba57f6e39be64f41a248a14164`,
  a matching result seal, and private reboot intent binding source commit
  `ae24565ac3e96290a50dbdc6c137ad8c9c58ea8a` plus the pinned reference.
  Only closed facts may be consumed: board/package/runtime admission, clean
  serial outcome, all 18 accepted preparation events with the terminal
  `retain_production_uart/completed` event, fresh required safety observations,
  accepted work, confirmed safe stop, ready USB cleanup, redaction, and no
  evidence promotion. Its later API-009 network-correlation failure remains
  authoritative for API-009 and is irrelevant to the earlier completed I2C
  preparation subclaim.
- The existing SSD1306 startup hardware-smoke breadcrumb remains the display
  functional proof. The user's fresh report of visible new display information
  is corroborating context only; it is not promoted as a typed artifact and is
  not required to satisfy the result.

No protected origin, hostname, port, USB identity, network identifier,
credential, pool value, address, worker, password, token, raw trace, or sensor
value may enter committed output. No new evidence schema or projector is
permitted.

## Implementation

- [ ] Independently validate all four committed projections, exact digests,
      file modes, source ancestry, and reference identity.
- [ ] Validate the protected attempt seal, modes, exact source/reference
      binding, closed preparation/safety/cleanup facts, and sensitive-output
      exclusion without publishing protected values.
- [ ] Prove the attempt source and current source share the exact retry owner,
      I2C bus owner, DS4432U transaction, mining preparation, display transfer,
      and INA260 transaction semantics; admit the later EMC2101 change only as
      a post-read offset/refactor that preserves address/register/retry routing.
- [ ] Re-run the exact retry/failure and source-ownership regressions plus the
      real normal and rollback firmware builds.
- [ ] Compose a row-specific result with explicit non-claims, then transition
      only `IO-001` if every evidence and mandatory gate passes.

## Verification and promotion

Focused verification must include the I2C retry host tests, sensor/display
source-ownership tests, all four independent Rust evidence validators, the
campaign result-seal/mode/source checks, exact comparison to pinned
`i2c_bitaxe.c`, and normal plus rollback-probe firmware builds. Repository
redaction, reference cleanliness, generated contracts, unique task/plan
binding, sensitive-output absence, and diff checks must pass.

Run the mandatory sequence in order before every commit boundary:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Promotion requires exact source agreement for the timeout/retry/delay contract,
one retained shared I2C owner, all supported transfer shapes routed through the
retry owner, post-retry physical INA260 and EMC2101 reads, post-retry completed
fan/DS4432U preparation on the same typed paths, the existing board-named
SSD1306 smoke boundary, safe stop, cleanup, and a valid row-specific result.
The transition may change only `IO-001` to `verified`, evidence to
`unit,workflow,hardware-smoke,hardware-regression`, and its notes.

Injected electrical I2C faults, forced arbitration loss, live terminal retry
exhaustion, waveform/timing measurement, raw bus probing or scanning,
simultaneous multi-owner access, arbitrary addresses/registers/values, other
devices, and non-205 boards remain explicit non-claims. The unit regressions
remain the authoritative transient/terminal failure proof. Any failed gate
leaves `IO-001` at `implemented`, withholds transition/progress, and records the
earliest blocker without hardware action.
