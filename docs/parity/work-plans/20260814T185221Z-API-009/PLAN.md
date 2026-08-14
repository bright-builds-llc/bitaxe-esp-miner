# Parity work plan

- Run ID: `20260814T185221Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `a4a48db77d33809e77315ba31bef553f32cf1651`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260814T152151Z-API-009/PLAN.md`

## Selection

The clean synchronized selector reports no open plan and ranks API-009 first.
Attempt-019 proved exact-package admission, trusted runtime attestation, the
genuine notification, positive block count, pause, safe stop, local IDENTIFY
activation, and a live rendered checkpoint. Its remaining failure was a host
contract that guessed both a one-hour operator lifetime and an enclosing child
timeout while the physical effect itself remained exactly 30 seconds. Pushed
source `a4a48db7` replaces those guessed human deadlines with typed
`operator-gated` ownership, retains independent automated deadlines, and binds
ready, rendered, and cleared to public checkpoint schema v4. This is verified
progress at the exact failing boundary, not an ordinal-only retry, so this plan
authorizes exactly one attempt-020 and no attempt-021.

## Scope and non-scope

Build and package exact clean pushed HEAD, admit exactly one Ultra 205 through
the protected detector, and run one fresh attempt-020 through
`just api-command-effects-campaign`. Use fresh mode-`0700` attempt, wrapper,
and detector roots plus mode-`0600` files. Keep the device paused and
safe-stopped while an explicitly operator-gated owner waits without a human
response deadline.

Allowed effects are exact-package USB flash/reset; private Wi-Fi and generated
local-fixture NVS seed; conservative BM1366 initialization and local-fixture
mining for at most 600 active seconds after the separately bounded activation
phase; one pause; one resume after both IDENTIFY observations; two IDENTIFY
toggles while paused; one genuine notification dismissal after active recovery;
one canonical software restart; same-device recovery; safe stop; and cleanup.
The operator-ready wait and the later cleared wait are unbounded. The local
`ready` signal starts the exact 30-second IDENTIFY effect and rendered evidence
window. Signal `rendered` only on the live exact blank / `BITAXE IDENTIFY` /
`Hello!` / blank frame, or `declined` if that frame is not visible during the
window. Signal `cleared` only after a live report that the frame is absent.

Do not use an external or owner pool; infer, automate, pre-confirm, reuse, or
expire-forward a physical observation; read prior protected attempts; expose
origins, hostnames, ports, USB/network identities, credentials, workers,
addresses, passwords, tokens, sensor values, or raw traces; or weaken the five-
command quorum. No erase, factory reset, OTA, rollback, power cycle, direct
UART, pin/pad/GPIO manipulation, probe, jumper, soldering, injected signal,
fault injection, voltage/frequency/fan override, non-205 device, attempt-021,
or unchanged retry is in scope.

## Implementation and verification

- [ ] Commit and push this immutable plan/task checkpoint before package,
      detector, credential, USB, network, mining, HTTP, display, or restart
      effects.
- [ ] Re-run focused lifetime, pause/signal/order/timing/checkpoint, campaign,
      CLI, and real-process tests plus every mandatory software, privacy,
      reference, and real-firmware gate.
- [ ] Require ignored `wifi-credentials.json` to be non-empty without reading
      it. Require fresh wrapper, detector, attempt-020, and public projection
      paths before creating private wrapper/detector roots.
- [ ] Run the detector exactly once and continue only on one board-205
      ESP32-S3, ready typed handoff, private modes, and no holder.
- [ ] Invoke exactly once:
      `just api-command-effects-campaign --private-root scratch/api009-command-effects/attempt-020 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api009-command-effects/detector-020/detector.stdout --projection docs/parity/evidence/api009-command-effects/command-effects-projection.json --duration-seconds 600`.
- [ ] Consume each ready/rendered-or-declined/cleared checkpoint exactly once
      with `just signal-api-command-identify --evidence-dir scratch/api009-command-effects/attempt-020/campaign --checkpoint <kind> [--outcome declined]` only after its matching live physical report.
- [ ] Promote only on the complete independently validated command and restart
      quorum; otherwise keep `implemented`, preserve the earliest typed failure,
      withhold evidence, record cleanup/recovery, and stop without attempt-021.

Before plan commit and final source/evidence commit, run in order: Cargo format,
strict Clippy, all-target build, all-feature tests, Bright Builds, `just test`,
`just parity`, and `just parity-progress`. Also run focused tests,
`just verify-redaction`, `just verify-reference`, `just build`, plan digest,
unique task binding, selector closure, exact package validation, private-mode
and holder checks, sensitive-output review, `git diff --check`, and full diff
review.

Campaign start consumes attempt-020. Promote only if the closed projection
proves genuine network-target block notification, pause plus safe stop before
ready, both live matching physical IDENTIFY observations while the pause stays
held, one resume and active recovery after cleared, dismissal with block-count
preservation, exactly one canonical software restart, same physical device,
exact build, changed boot session, ordinal `N+1`, safe stop, cleanup, recovery,
private modes, and redaction. Otherwise close as `blocked`, withhold the
projection, and do not create attempt-021.
