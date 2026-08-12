# Parity work plan

- Run ID: `20260812T135813Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `d12f95f09d4f4cb4952595dee3676712c8a2b847`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`

## Selection and objective

The clean synchronized selector reports no open plan and selects `API-009`
first. This row combines five operator command effects: mining pause, mining
resume, response-before-restart, the physical identify display, and dismissal
of an active block-found notification. Current pure plans, typed responses,
effect ordering, retained firmware owners, and handlers are implemented, but
the authoritative checklist explicitly keeps full device-user effects below
verified without claim-specific hardware evidence.

Audit the committed public evidence and current production source for a closed
quorum covering all five effects. Promote only if exact public evidence already
proves every effect on the Ultra 205 from trusted command through observable
device-user postcondition and safe restoration. Otherwise close the row at the
first irreducible missing precondition and stop the standing advance-parity run
without partial hardware effects.

## Scope and non-scope

This is a read-only, software-only eligibility audit. It may inspect committed
public evidence, checklist history, plans/results/closures, current Rust source
and tests, and the pinned reference. It may run repository verification. It may
not inspect protected campaign roots or local credentials.

No detector, package build, flash, reset, USB/network session, HTTP command,
credential input, mining or pool contact, identify toggle, block-state
injection, fan/voltage/power/ASIC actuation, recovery, direct UART, pins, or
physical manipulation is permitted. The user's observation that the display
now contains more information is useful device context but is not admissible
identify-command evidence because it is not causally bound to a trusted
identify request and its 30-second physical render/clear lifecycle.

Do not introduce a diagnostic state injector or synthetic block-found path to
manufacture parity evidence. A genuine active block notification must arise
from the production state owner and be dismissed through the public route.
Likewise, do not run restart or mining commands in isolation when the combined
row cannot be closed; partial effects would add risk without enabling promotion.

## Audit criteria

- [ ] Bind the upstream behavior for all five routes and the current Rust
      response/effect/owner implementations.
- [ ] Inventory committed public hardware evidence for command-correlated
      pause/resume, response-before-restart with same-device recovery, physical
      identify render and clear, and active block-found dismissal.
- [ ] Confirm whether a repo-owned bounded contract can create every missing
      precondition without synthetic evidence, destructive or prohibited
      effects, or user-only visual attestation.
- [ ] Promote only on a complete closed quorum; otherwise write `CLOSURE.md`
      naming the exact terminal evidence blocker and leave `API-009`
      `implemented`.

## Verification and stop rule

Run the mandatory ordered repository gate:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require exact reference cleanliness, immutable-plan digest, task
uniqueness, selector binding, public-evidence-only path checks, redaction,
reference verification, and `git diff --check`.

If trusted public evidence lacks either command-correlated physical identify
rendering or dismissal from a genuinely active production block-found state,
the row cannot be verified and this run ends at a terminal evidence blocker.
Do not fall back to partial hardware execution, synthetic state, weakened
claims, row skipping, or another parity row.
