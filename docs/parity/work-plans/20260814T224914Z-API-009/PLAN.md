# Parity work plan

- Run ID: `20260814T224914Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `420c22369ad16025b0ce47ba6748144637217d6b`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260814T200547Z-API-009/PLAN.md`

## Selection

The clean synchronized selector reports no open plan and ranks API-009 first.
Attempt-021 reached the safe operator-ready checkpoint and issued one exact
30-second IDENTIFY effect, but the operator was not watching during that
window. Its immutable one-shot contract correctly recorded `declined`, safely
closed, and withheld evidence. Pushed source `420c2236` adds a deterministic,
typed single-replay protocol that queues the optional replay until the prior
effect is inactive, opens one new exact confirmation window, rejects late or
duplicate replay, and binds the optional replayed checkpoint to closed request
counts. This is verified progress at the exact attempt-021 boundary, so this
plan authorizes exactly one attempt-022 and no attempt-023.

## Scope and non-scope

Build and package exact clean pushed HEAD, admit exactly one Ultra 205 through
the protected detector, and run one fresh attempt-022 through
`just api-command-effects-campaign`. Use fresh mode-`0700` attempt and detector
roots plus mode-`0600` files. Keep the device paused and safe-stopped while the
operator-gated owner waits without a human response deadline.

Allowed effects are exact-package USB flash/reset; private Wi-Fi and generated
local-fixture NVS seed; conservative BM1366 initialization and local-fixture
mining for at most 600 active seconds after the separately bounded activation
phase; one pause; one resume after both IDENTIFY observations; one initial
IDENTIFY enable, at most one optional replay enable, one final IDENTIFY clear;
one genuine notification dismissal after active recovery; one canonical
software restart; same-device recovery; terminal safe stop; and cleanup.

The ready, rendered, optional replayed, and cleared operator waits are
unbounded. The local ready signal starts the first exact 30-second IDENTIFY
effect and evidence window. Confirm rendered only on the live exact blank /
`BITAXE IDENTIFY` / `Hello!` / blank frame. If the first frame is missed, signal
exactly one `replay` from the rendered checkpoint; the host must wait until the
first effect is inactive before starting the second exact 30-second window.
Confirm replayed only on that same live exact frame. If the replayed frame is
also missed, explicitly decline. Confirm cleared only after a live report that
the IDENTIFY frame is absent.

Do not use an external or owner pool; infer, automate, pre-confirm, reuse, or
expire-forward a physical observation; read prior protected attempts; expose
origins, hostnames, ports, USB/network identities, credentials, workers,
addresses, passwords, tokens, sensor values, or raw traces; or weaken the
five-command quorum. No erase, factory reset, OTA, rollback, power cycle,
direct UART, pin/pad/GPIO manipulation, probe, jumper, soldering, injected
signal, fault injection, voltage/frequency/fan override, non-205 device,
attempt-023, or unchanged retry is in scope.

## Implementation and verification

- [ ] Commit and push this immutable plan/task checkpoint before package,
      detector, credential, USB, network, mining, HTTP, display, or restart
      effects.
- [ ] Re-run focused replay parser/state-machine/loopback HTTP, late and
      duplicate negative controls, operator-lifetime, pause/signal/order,
      campaign, CLI, evidence-binding, and real-process tests plus every
      mandatory software, privacy, reference, and real-firmware gate.
- [ ] Require ignored `wifi-credentials.json` to be non-empty without reading
      it. Require fresh detector, attempt-022, and public projection paths
      before creating the private detector root.
- [ ] Build and validate the exact package at
      `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` and require its
      source/reference identities to match this plan.
- [ ] Run the detector exactly once using mode-`0700`
      `scratch/api009-command-effects/detector-022` and mode-`0600`
      `detector.stdout`; continue only on one board-205 ESP32-S3, ready typed
      handoff, and no unexpected holder.
- [ ] Invoke exactly once:
      `just api-command-effects-campaign --private-root scratch/api009-command-effects/attempt-022 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api009-command-effects/detector-022/detector.stdout --projection docs/parity/evidence/api009-command-effects/command-effects-projection.json --duration-seconds 600`.
- [ ] Consume each checkpoint at most once with
      `just signal-api-command-identify --evidence-dir scratch/api009-command-effects/attempt-022/campaign --checkpoint <kind> --outcome <confirmed|declined|replay>` only after its matching live operator input. Replay is valid only for the original rendered checkpoint and at most once.
- [ ] Promote only on the complete independently validated command and restart
      quorum; otherwise keep `implemented`, preserve the earliest typed
      failure, withhold evidence, record cleanup/recovery, and stop without
      attempt-023.

Before plan commit and final source/evidence commit, run in order: Cargo format,
strict Clippy, all-target build, all-feature tests, Bright Builds, `just test`,
`just parity`, and `just parity-progress`. Also run the focused tests above,
`just verify-redaction`, `just verify-reference`, `just build`, immutable plan
digest, unique task binding, selector closure, exact package validation,
private-mode and holder checks, sensitive-output review, `git diff --check`,
and full diff review.

Campaign start consumes attempt-022. Promote only if the closed projection
proves genuine network-target block notification, pause plus safe stop before
ready, both live matching physical IDENTIFY observations while the pause stays
held, one resume and active recovery after cleared, dismissal with block-count
preservation, exactly one canonical software restart, same physical device,
exact build, changed boot session, ordinal `N+1`, terminal safe stop, cleanup,
recovery, private modes, and redaction. Otherwise close as `blocked`, withhold
the projection, and do not create attempt-023.
