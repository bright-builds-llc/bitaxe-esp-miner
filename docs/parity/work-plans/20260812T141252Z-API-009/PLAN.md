# Parity work plan

- Run ID: `20260812T141252Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `2acb77c7ea9d5785b6a93fa6748f6a9af97f1141`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260812T135813Z-API-009/PLAN.md`

## Selection

The clean synchronized selector reports no open plan and selects `API-009`
first. No candidate is skipped. The preceding API-009 audit closed truthfully
after proving that production code never raises the block-found notification;
its first explicit unblock condition is therefore actionable software work.

Implement the missing production transition from a valid current-generation
ASIC nonce whose computed difficulty meets or exceeds the active job's compact
network target. This continuation is limited to that producer and its owner
seams. It does not weaken the row's conjunctive command-effect evidence
requirement or claim that the remaining physical effects are verified.

## Scope and non-scope

Use the existing production correlation receipt, which already binds a parsed
ASIC result to current-generation pool work and computes nonce difficulty.
Derive network difficulty from the same admitted work target, expose only a
redaction-safe typed effect, and let the sole firmware runtime-snapshot owner
atomically increment the found-block count and show the notification. Preserve
upstream per-valid-result behavior, including a repeated valid result, while
rejecting stale, uncorrelated, or malformed target contexts.

Add pure boundary and state-transition tests, production-session effect tests,
and a firmware source-ownership regression. Keep the business decision in the
pure Stratum/API core and the state mutation in the thin firmware shell.

No detector, package capture, USB, flash, reset, HTTP request, network session,
credentials, mining campaign, pool connection, ASIC interaction, identify
command, diagnostic setter, voltage/frequency/fan/power effect, OTA, recovery,
direct UART, pins, or physical manipulation is authorized by this plan.
Reference source remains pinned and read-only.

## Implementation

- [ ] Reuse one compact-target network-difficulty function for coinbase and
      qualified-result decisions, with invalid targets failing closed.
- [ ] Carry network-target qualification through the current-generation
      correlation receipt and emit one redacted `RecordBlockFound` effect
      before submit/scoreboard side effects.
- [ ] Add the pure count/visibility transition and wire the effect into the
      sole firmware runtime-snapshot owner.
- [ ] Cover below/equal/above target, duplicate, stale, saturation, dismiss,
      effect ordering, redaction, and source ownership without synthetic
      firmware state injection.

## Verification and promotion

Run focused API, Stratum production-session, and firmware ownership tests,
then the mandatory ordered gate:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require `just verify-redaction`, `just verify-reference`, selector and
task binding, immutable-plan digest, reference cleanliness, sensitive-output
review, `git diff --check`, and a final diff review. Commit and push this plan
checkpoint before implementation, then commit and push the verified source.

Update only API-009's Rust-owned target when the new production owners are
present. Keep status `implemented` and evidence `unit,workflow`; no hardware or
device-user command-effect promotion is allowed. Close this retry truthfully
with the remaining physical identify, active-mining pause/resume, restart, and
live active-notification dismissal evidence requirements.
