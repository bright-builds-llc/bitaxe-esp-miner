# Parity work plan

- Run ID: `20260815T041103Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `f39892d0619f882010155357de38e1aacc642cc4`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260815T034210Z-API-009/PLAN.md`

## Selection

Clean synchronized HEAD has no open plan and the deterministic selector ranks
API-009 first. Attempt-025 proved trusted identity, genuine notification,
pause and stopped hardware, the complete latency-tolerant IDENTIFY sequence,
resumable active recovery, and an accepted share. Its sealed command record
then isolated the one post-reactivation dismissal readback as the remaining
failure.

Pushed source `a90a0405` repairs that exact host boundary. The sole dismissal
now occurs after the joined HTTP and serial safe-stopped pause, and exact
notification-clear plus block-count-preservation readback gates IDENTIFY.
After IDENTIFY clears, one resume request and active confirmation advance
directly to terminal validation. Live-shaped regressions and all repository
gates pass. This is material progress at the consumed attempt-025 boundary,
so this plan authorizes exactly one attempt-026 and no attempt-027.

## Scope and non-scope

Build and package exact clean pushed HEAD, admit exactly one Ultra 205 through
the protected detector, and run one fresh attempt-026 through
`just api-command-effects-campaign`. Use fresh mode-`0700` attempt and detector
roots plus mode-`0600` files. Keep the device paused and safe-stopped throughout
every unbounded operator wait.

Allowed effects are exact-package USB flash/reset; private Wi-Fi and generated
local-fixture NVS seed; conservative BM1366 initialization and local-fixture
mining for at most 600 accumulated active seconds after bounded activation;
one pause; one notification dismissal during the joined safe-stopped pause;
one initial IDENTIFY enable; at most one optional replay enable after the prior
effect is inactive; one resume after the IDENTIFY observation and natural
clear; one canonical software restart; same-device recovery; terminal safe
stop; and cleanup. Resume intent has a 15-second bound, production reactivation
has a 180-second bound, and failure recovery has a 130-second correlated safe-
stop bound. A stale sample during eligible pre-active reactivation may cause a
resumable safe stop and later reprepare within that same bounded reactivation
phase. Those automated bounds do not limit an operator checkpoint.

The ready, rendered, optional replayed, and cleared operator waits are
unbounded. The host may open ready only after it has confirmed dismissal
cleared the notification without changing the positive block count. The local
ready signal starts the first exact 30-second IDENTIFY effect. Confirm rendered
if the exact blank / `BITAXE IDENTIFY` / `Hello!` / blank frame was observed
during that effect; the report may arrive later. If the first frame was missed,
signal exactly one `replay` from the rendered checkpoint; the host must wait
until the first effect is inactive before starting the second exact 30-second
effect. Confirm replayed if the same exact frame was observed during that
replay effect, even if the report arrives later. If the replayed frame was also
missed, explicitly decline. The host opens the cleared checkpoint only after
conservative natural expiry. Confirm cleared only after a live report that the
IDENTIFY frame is absent.

Do not use an external or owner pool; infer, automate, pre-confirm, reuse, or
expire-forward a physical observation; read prior protected attempts; expose
origins, hostnames, ports, USB/network identities, credentials, workers,
addresses, passwords, tokens, sensor values, or raw traces; or weaken the
five-command quorum. No erase, factory reset, OTA, rollback, power cycle,
direct UART, pin/pad/GPIO manipulation, probe, jumper, soldering, injected
signal, fault injection, voltage/frequency/fan override, non-205 device,
attempt-027, or unchanged retry is in scope.

## Implementation and verification

- [ ] Commit and push this immutable plan/task checkpoint before package,
      detector, credential, USB, network, mining, HTTP, display, or restart
      effects.
- [ ] Re-run focused paused-dismissal, reactivation-safety, active-budget,
      resume-intent/reactivation, recovery-correlation, delayed-attestation,
      natural-expiry, replay, parser/state-machine/loopback HTTP, duplicate
      negative control, operator-lifetime, campaign, CLI, evidence-binding, and
      real-process tests plus every mandatory software, privacy, reference,
      and real-firmware gate.
- [ ] Require ignored `wifi-credentials.json` to be non-empty without reading
      it. Require fresh detector, attempt-026, and public projection paths
      before creating the private detector root.
- [ ] Build and validate the exact package at
      `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` and require its
      source/reference identities to match this plan and pushed HEAD.
- [ ] Run the detector exactly once using mode-`0700`
      `scratch/api009-command-effects/detector-026` and mode-`0600`
      `detector.stdout`; continue only on one board-205 ESP32-S3, ready typed
      handoff, and no unexpected holder.
- [ ] Invoke exactly once:
      `just api-command-effects-campaign --private-root scratch/api009-command-effects/attempt-026 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api009-command-effects/detector-026/detector.stdout --projection docs/parity/evidence/api009-command-effects/command-effects-projection.json --duration-seconds 600`.
- [ ] Consume each checkpoint at most once with
      `just signal-api-command-identify --evidence-dir scratch/api009-command-effects/attempt-026/campaign --checkpoint <kind> --outcome <confirmed|declined|replay>` only after its matching live operator input. Replay is valid only for the original rendered checkpoint and at most once.
- [ ] Promote only on the complete independently validated command and restart
      quorum; otherwise keep `implemented`, preserve the earliest typed
      failure, withhold evidence, record cleanup/recovery, and stop without
      attempt-027.

Before plan commit and final source/evidence commit, run in order: Cargo format,
strict Clippy, all-target build, all-feature tests, Bright Builds, `just test`,
`just parity`, and `just parity-progress`. Also run the focused tests above,
`just verify-redaction`, `just verify-reference`, `just build`, immutable plan
digest, unique task binding, selector closure, exact package validation,
private-mode and holder checks, sensitive-output review, `git diff --check`,
and full diff review.

Campaign start consumes attempt-026. Promote only if the closed projection
proves genuine network-target block notification, pause plus safe stop before
dismissal, exactly one count-preserving notification dismissal before ready,
one live matching physical IDENTIFY observation while the pause stays held,
natural expiry and live absence before resume, one resume-intent confirmation
and active recovery, exactly one canonical software restart, same physical
device, exact build, changed boot session, ordinal `N+1`, terminal safe stop,
cleanup, recovery, private modes, and redaction. Otherwise close as `blocked`,
withhold the projection, and do not create attempt-027.
