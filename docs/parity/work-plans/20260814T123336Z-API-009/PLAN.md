# Parity work plan

- Run ID: `20260814T123336Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `0e9ade815f2f0eb15415fb4d7cf4047503f6d3a1`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260814T120645Z-API-009/PLAN.md`

## Selection

The clean synchronized selector has no open plan and ranks API-009 first, so no
candidate is skipped. Attempt-016 admitted the exact package and trusted
runtime, observed a genuine notification, completed pause/safe-stop and resume,
then reached readiness while active and later failed closed on stale safety
before a timely operator report arrived.

Pushed source `fe0995fd` converts that unsafe-to-wait ordering into a tested
paused signal boundary: readiness is armed only after pause plus safe stop, the
device remains safe-stopped for a checked one-hour window, one private signal
issues the sole resume, and IDENTIFY waits for same-session active recovery.
This is new objectively verified orchestration progress, not an ordinal-only
retry, so this plan authorizes exactly one attempt-017 and no attempt-018.

## Scope and non-scope

Build and package exact clean pushed HEAD, admit exactly one Ultra 205 through
the protected detector, and run one fresh attempt-017 through
`just api-command-effects-campaign`. Use fresh mode-`0700` attempt, wrapper,
and detector roots plus mode-`0600` files. Keep the campaign attached while it
waits safe-stopped for the private ready signal.

Allowed effects are exact-package USB flash/reset; private Wi-Fi and generated
local-fixture NVS seed; conservative BM1366 initialization and local-fixture
mining for at most 600 active seconds after the separately bounded activation
phase; one pause; one resume; two IDENTIFY toggles; one genuine notification
dismissal; one canonical software restart; same-device recovery; safe stop;
and cleanup. Signal `ready` only after the user reports they are watching.
Signal `rendered` only when they report the exact blank / `BITAXE IDENTIFY` /
`Hello!` / blank frame while its 30-second checkpoint is live. Signal `cleared`
only when they report that frame absent.

Do not use an external or owner pool; infer, automate, pre-confirm, reuse, or
expire-forward a physical observation; read prior protected attempts; expose
origins, hostnames, ports, USB/network identities, credentials, workers,
addresses, passwords, tokens, sensor values, or raw traces; or weaken the five-
command quorum. No erase, OTA, rollback, power cycle, direct UART,
pin/pad/GPIO manipulation, probe, jumper, soldering, injected signal, fault
injection, voltage/frequency/fan override, non-205 device, attempt-018, or
unchanged retry is in scope.

## Implementation and verification

- [ ] Commit and push this immutable plan/task checkpoint before package,
      detector, credential, USB, network, mining, HTTP, display, or restart
      effects.
- [ ] Re-run focused pause/signal/order/timing/checkpoint/campaign/CLI/real-
      process tests, every mandatory software/privacy/reference gate, the real
      firmware build, package validation, and exact clean-pushed identity
      admission.
- [ ] Require ignored `wifi-credentials.json` to be non-empty without reading
      it. Require fresh wrapper, detector, attempt-017, and public projection
      paths before creating private wrapper/detector roots.
- [ ] Run the detector exactly once and continue only on one board-205
      ESP32-S3, ready typed handoff, private modes, and no holder.
- [ ] Invoke exactly once:
      `just api-command-effects-campaign --private-root scratch/api009-command-effects/attempt-017 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api009-command-effects/detector-017/detector.stdout --projection docs/parity/evidence/api009-command-effects/command-effects-projection.json --duration-seconds 600`.
- [ ] Send each ready/rendered/cleared checkpoint exactly once with
      `just signal-api-command-identify --evidence-dir scratch/api009-command-effects/attempt-017/campaign --checkpoint <kind>` only after its matching live user report.
- [ ] Promote only on the complete independently validated command and restart
      quorum; otherwise keep `implemented`, preserve the earliest typed failure,
      withhold evidence, record cleanup/recovery, and stop without attempt-018.

Before plan commit and final source/evidence commit, run in order: Cargo format,
strict Clippy, all-target build, all-feature tests, Bright Builds, `just test`,
`just parity`, and `just parity-progress`. Also run focused tests,
`just verify-redaction`, `just verify-reference`, `just build`, plan digest,
unique task binding, selector closure, exact package validation, private-mode
and holder checks, sensitive-output review, `git diff --check`, and full diff
review.

Campaign start consumes attempt-017. Promote only if the closed projection
proves genuine network-target block notification, pause plus safe stop before
ready, one live user-ready signal, resume and active recovery before IDENTIFY,
both matching physical IDENTIFY observations, dismissal with block-count
preservation, exactly one canonical software restart, same physical device,
exact build, changed boot session, ordinal `N+1`, safe stop, cleanup, recovery,
private modes, and redaction. Otherwise close as `blocked`, withhold the
projection, and do not create attempt-018.
