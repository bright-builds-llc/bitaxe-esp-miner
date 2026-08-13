# Parity work plan

- Run ID: `20260813T163921Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `f9add8e29e47806baa79bef398d1a951437e1dad`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260813T160905Z-API-009/PLAN.md`

## Selection

The clean synchronized selector has no open plan and ranks API-009 first, so
no candidate is skipped. Attempt-011 admitted the exact package and runtime,
proved active mining, a genuine block, and four accepted shares, then sent one
pause request. The pause was lost because requested operator intent shared the
replaceable session-derived mining projection. The host correctly timed out
the missing pause/safe-stop join and recovered without issuing IDENTIFY.

Pushed commit `f9add8e2` fixes that exact production ownership race. Requested
intent now has a distinct typed boot-lifetime owner that boot preference and
pause/resume commands update, authoritative readiness reads, and session
publication cannot replace. The red interleaving regression, ownership tests,
focused session/campaign/real-process tests, all mandatory gates, and the real
ESP firmware build pass. This is new objectively verified production-boundary
information, not an ordinal-only retry, so this plan authorizes exactly one
attempt-012 and no attempt-013.

## Scope and non-scope

Build and package exact clean pushed HEAD, admit exactly one Ultra 205 through
the protected detector, and run one fresh `attempt-012` through
`just api-command-effects-campaign`. Use fresh mode-`0700` attempt, wrapper,
and detector roots plus mode-`0600` files. Keep the campaign attached to its
closed checkpoint stream.

Allowed effects are exact-package USB flash/reset; private Wi-Fi and generated
local-fixture NVS seed; conservative BM1366 initialization and local-fixture
mining for at most 600 seconds; one pause; one resume; two IDENTIFY toggles;
one genuine notification dismissal; one canonical software restart;
same-device recovery; safe stop; and cleanup. Consume `ready` only after the
user confirms they are watching. Consume `rendered` only when they report the
exact blank / `BITAXE IDENTIFY` / `Hello!` / blank frame while its 30-second
checkpoint is live. Consume `cleared` only when they report that frame absent.

Do not use an external or owner pool; infer, automate, pre-confirm, or reuse a
physical observation; read prior protected attempts; expose origins,
hostnames, ports, USB/network identities, credentials, workers, addresses,
passwords, tokens, paths, sensor values, or raw traces; or weaken the complete
five-command quorum. No erase, OTA, rollback, power cycle, direct UART,
pin/pad/GPIO manipulation, probe, jumper, soldering, injected signal, fault
injection, voltage/frequency/fan override, non-205 device, attempt-013, or
unchanged retry is in scope.

## Implementation and verification

- [ ] Commit and push this immutable plan/task checkpoint before package,
      detector, credential, USB, network, mining, HTTP, display, or restart
      effects.
- [ ] Re-run focused intent/checkpoint/campaign/CLI and real-process tests,
      every mandatory software/privacy/reference gate, real firmware build,
      package validation, and exact clean-pushed identity admission.
- [ ] Require ignored `wifi-credentials.json` to be non-empty without reading
      it. Require the following fresh paths to be absent, then create only the
      private wrapper/detector roots:
      `scratch/api009-command-effects/wrapper-012`,
      `scratch/api009-command-effects/detector-012`,
      `scratch/api009-command-effects/attempt-012`, and
      `docs/parity/evidence/api009-command-effects/command-effects-projection.json`.
- [ ] Run the detector exactly once into
      `scratch/api009-command-effects/detector-012/detector.stdout` and
      `.stderr`; continue only on one board-205 ESP32-S3, ready typed handoff,
      private modes, and no holder.
- [ ] Invoke exactly once:
      `just api-command-effects-campaign --private-root scratch/api009-command-effects/attempt-012 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api009-command-effects/detector-012/detector.stdout --projection docs/parity/evidence/api009-command-effects/command-effects-projection.json --duration-seconds 600`.
- [ ] Consume `ready`, `rendered`, and `cleared` exactly once each only after
      their matching live user reports, using
      `just confirm-api-command-identify --evidence-dir scratch/api009-command-effects/attempt-012/campaign --checkpoint <kind>`.
- [ ] Promote API-009 only on the complete independently validated command and
      restart quorum; otherwise keep `implemented`, preserve the earliest typed
      failure, withhold evidence, prove cleanup/recovery, and stop without
      attempt-013.

Before the immutable plan commit and again before final source/evidence commit,
run in order: `cargo fmt --all`, strict all-target/all-feature Clippy, all-target
build, all-feature tests, Bright Builds, `just test`, `just parity`, and
`just parity-progress`. Also run focused tests, `just verify-redaction`,
`just verify-reference`, `just build`, immutable-plan digest, unique task
binding, selector closure, reference cleanliness, sensitive-output review,
fresh-path/projection checks, `git diff --check`, and full diff review.

Campaign start consumes attempt-012. Promote only if the closed projection
proves genuine network-target block notification, active pause/resume with the
same-session safe-stop join, operator readiness before IDENTIFY, both matching
physical IDENTIFY observations, dismissal with block-count preservation,
exactly one canonical software restart, same physical device, exact build,
changed boot session, ordinal `N+1`, safe stop, cleanup, recovery, private
modes, and redaction. Otherwise close as `blocked`, withhold the projection,
and do not create attempt-013.
