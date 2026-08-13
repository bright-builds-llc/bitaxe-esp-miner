# Parity work plan

- Run ID: `20260813T100428Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `66f9cd78169696b17dcce5538e31eb5fd5c818a0`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260813T085022Z-API-009/PLAN.md`

## Selection

The clean synchronized selector has no open plan and ranks API-009 first, so
no candidate is skipped. Attempt-009 reached the prior closure's exact
software-diagnosis condition: one pause request was accepted, but the host's
130-second logical-pause/hardware-safe-stop join expired before confirmation.
The terminal readiness projection showed stale safety, an unchanged
observation epoch, and no recovered pending observation. No later command was
issued and no public evidence was published.

The production source explains that boundary without reading raw protected
traces. `ProductionSessionEffect::SafeStopHardware` has no stop purpose, so an
operator-resumable pause executes the same eight-step shutdown used for
terminal and fault recovery. That synchronous adapter call waits up to 120
seconds for a fresh temperature at or below 45 C and then lowers the fan. The
production owner cannot consume sensor wakeups or publish its safe-stop marker
while that call is running, leaving only ten seconds of host-deadline margin.
The pinned reference's ordinary pause calls `mining_stop`, whose prompt path
reduces frequency, resets nonce state, removes core power, holds reset, and
returns; the 45 C cooling loop belongs to overheat recovery, not user pause.

The active lesson set remains above its deterministic loading budget with the
current 2026-08-03 audit baseline and no new audit trigger. Complete safety,
authorization, evidence, retry, redaction, real-process, hardware-boundary,
host-stall, and task-relevant blocks are loaded. Caption/VTT, small-table
deduplication, legacy GSD separator, and manual-removal blocks remain disclosed
unrelated omissions. Repo-local guidance and the architecture, code-shape,
verification, testing, Rust, and TypeScript standards govern this plan.

## Scope and non-scope

Separate the hardware-stop purpose in the pure production-session contract.
An operator-resumable pause must select a prompt fail-closed plan that blocks
dispatch, reduces frequency and nonce state, holds reset, removes ASIC/core
power, and commands full fan duty without entering the terminal cooling wait
or lowering the fan. Terminal, fault, shutdown, lease-consumption, and failed-
preparation rollback paths retain the complete cooling proof and paused fan
settlement. The firmware adapter remains the only hardware shell and earliest
typed failure remains authoritative.

Prove the regression at the production boundaries: pure session selection,
exact actuation plans, operator sensor/wakeup ownership, campaign status
confirmation, and host pause join. Include a production-shaped composed test
that fails if resumable pause invokes the 120-second cooling step, loses the
confirming marker, or permits resume without both logical pause and the same-
lease stopped-hardware fact. Keep terminal shutdown behavior unchanged.

This is software-only authority. Do not access credentials, protected attempt
contents, detector, package effects, device/USB/network/HTTP sessions, flash,
reset, mining, ASIC traffic, command effects, OTA, recovery, direct UART,
pins/pads/GPIO, or attempt-010. This plan cannot promote API-009 or authorize a
hardware retry. A later separately selected immutable plan may consider one
fresh hardware attempt only after this exact production boundary is fixed,
the real-boundary regression and every mandatory gate pass, and the closure
records objective retry progress.

## Implementation

- [ ] Add a closed resumable-versus-terminal hardware-stop purpose to the pure
      production-session effect and select it from retained session state.
- [ ] Split the pure actuation plan so resumable pause completes after the
      immediate safe effects with full fan duty, while terminal/fault/rollback
      keeps the existing 45 C cooling proof and 30-percent settlement.
- [ ] Bind the firmware adapter to the typed plan without weakening failure
      precedence, safe-state confirmation, lease identity, or retry guards.
- [ ] Add focused unit, production-source ownership, campaign-marker, host
      pause-join, and production-shaped composed regressions for the exact
      attempt-009 boundary.
- [ ] Keep API-009 `implemented`, publish no parity evidence, and close this
      software plan with the exact later hardware-eligibility condition.

## Verification and promotion

Run the focused Rust/Bazel tests for production-session recovery, mining
actuation, campaign status, sensor source ownership, flash campaign markers,
and pause joining, plus the real firmware build. Require proof that:

1. Operator pause emits exactly one resumable stop purpose for the retained
   lease; terminal/fault/shutdown paths emit the terminal purpose.
2. The resumable plan contains every immediate safe effect in order, excludes
   both the 120-second cooling wait and 30-percent fan command, retains full fan
   duty, and preserves earliest failure while attempting all of its steps.
3. Terminal shutdown and failed-preparation rollback retain the complete
   eight-step cooling plan unchanged.
4. The composed pause path reaches stopped hardware and the firmware-owned
   same-lease confirmation without waiting on a cooling observation; the host
   still requires its conjunction with API-visible logical pause before one
   resume.
5. Sensor observation publication and pending-wakeup ownership remain intact,
   public output contains no sensitive values, and no hardware/evidence path is
   touched.

Then run, in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also run `just firmware`, generated/source-ownership checks affected by the
change, `just verify-redaction`, `just verify-reference`, immutable-plan digest,
unique task binding, selector closure, reference cleanliness, sensitive-output
review, `git diff --check`, and full diff review. Do not request a no-op
checklist transition or synchronize progress because no checklist field or
accepted parity evidence changes. Create `CLOSURE.md`, commit and push the
verified software fix, and leave attempt-010 prohibited for this invocation.
