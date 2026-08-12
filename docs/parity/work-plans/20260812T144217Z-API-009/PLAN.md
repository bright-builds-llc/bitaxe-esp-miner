# Parity work plan

- Run ID: `20260812T144217Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `401352c0e5fe9c3b6c888234ee1d860ccf1d0542`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260812T141252Z-API-009/PLAN.md`

## Selection

The clean synchronized selector reports no open plan and selects `API-009`
first. No candidate is skipped. The preceding continuation added the missing
production block-notification writer and closed with one concrete next action:
prove the five command effects together on a detector-admitted Ultra 205.

The connected board is reporting additional display content after the earlier
apparent reset loop. That observation makes a fresh bounded campaign eligible,
but it is not evidence by itself and does not weaken any admission gate.

## Scope and non-scope

Add a `command-effects` stage to the existing typed mining campaign. Reuse its
exact-package flash, physical USB admission, private NVS seed, conservative
mining profile, trusted runtime-origin derivation, serial/network correlation,
lease, safe-stop, cleanup, and redaction boundaries. Extend the shared strict
HTTP client only with the remaining fixed command routes.

Run a repository-owned local Stratum fixture that admits normal Stratum
configure/subscribe/authorize/notify/submit traffic and uses compact target
`207fffff`. A valid current-generation ASIC result may therefore raise the
production notification through the real network-target decision without a
diagnostic state setter. The fixture and public projection expose only typed
counts, booleans, hashes, and categories.

The live sequence is conjunctive: exact-package active mining; genuine
notification raised; pause observed; resume observed; identify enabled;
operator-visible IDENTIFY rendering acknowledged through a fresh private
one-time checkpoint; identify disabled and visible clearing acknowledged;
notification dismissed while preserving its count; safe stop; then one
reader-armed software restart through the canonical device-session transaction.
Promotion requires every effect and same-device/build/session invariant.

No external pool, owner pool credential, raw origin, hostname, port, USB or
network identity, Wi-Fi value, worker, address, password, token, raw trace, or
operator checkpoint secret may enter public evidence. No diagnostic state
injection, direct UART, pin manipulation, erase, OTA, rollback, fault injection,
voltage/frequency/fan override, power cycling, or unbounded mining is allowed.
Reference source remains pinned and read-only.

## Implementation

- [ ] Add the typed command-effects campaign state machine and fixed-route HTTP
      operations, preserving request-once semantics and the earliest failure.
- [ ] Add a production local Stratum fixture with an admitted easy network
      target and private generated campaign credential handoff.
- [ ] Add private one-time identify-rendered and identify-cleared checkpoints;
      absence, replay, timeout, or malformed confirmation fails closed.
- [ ] Reuse `device-session reboot-live` for the final restart and require the
      closed same-device, exact-build, ordinal `N+1`, software-reset projection.
- [ ] Emit `bitaxe-api-command-effects-evidence-v1` only after the complete
      quorum, safe stop, recovery, cleanup, and redaction checks pass.
- [ ] Add behavior-focused unit, real-process, failure-category, recovery,
      request-order/count, private-mode, and sensitive-output regressions.

## Verification and promotion

Run focused HTTP transport, flash campaign, device-session, automation, and
evidence-contract tests, then the mandatory ordered gate:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require `just verify-redaction`, `just verify-reference`, generated
contract checks, selector and unique task binding, immutable-plan digest,
reference cleanliness, sensitive-output review, `git diff --check`, and final
diff review. Commit and push this plan/task checkpoint before implementation,
then commit and push verified source before any hardware effect.

After a clean pushed source, run at most one hardware attempt:

1. Build/package and record the exact package identity.
2. Run `just detect-ultra205` into the fresh private attempt wrapper and admit
   exactly one ESP32-S3 board-205 port.
3. Start the repository local fixture and invoke the new repo-owned
   `just api-command-effects-campaign` interface with board `205`, detector
   output, exact manifest, opaque Wi-Fi input, `attempt-001` private root,
   bounded 600-second lease, and a fresh public projection path.
4. When the command reports the private identify checkpoint, ask only for the
   user's physical observation of IDENTIFY-on and IDENTIFY-cleared; this is an
   evidence observation, not renewed authorization. Submit each observation
   once through the repo-owned confirmation command.

Recovery always attempts pause/safe-stop first, then restores the exact package
only if the campaign disturbed it and safe stop cannot otherwise be confirmed.
Recovery booleans are secondary; the first typed failure remains primary.
Accepted terminal categories are `hardware_blocked`, `evidence_invalid`,
`timeout`, and `process_failed`; no projection is published on failure. Stop
after this one attempt without retry.

Promote `API-009` to `verified` only if the public projection and private seal
prove the complete five-command quorum, real network-qualified ASIC result,
physical identify on/off observation, same physical device, exactly one
software restart, exact build identity, boot session change, ordinal `N+1`,
safe stop, cleanup, recovery status, and redaction. Otherwise keep
`implemented`, create a truthful closure with the typed terminal category and
next safe action, and do not claim any partial command effect.
