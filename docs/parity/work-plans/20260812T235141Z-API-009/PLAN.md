# Parity work plan

- Run ID: `20260812T235141Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `89d78f2869880e5fcf52f67c31e8013a63fdfe24`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260812T182405Z-API-009/PLAN.md`

## Selection

The clean synchronized selector again ranks API-009 first, so no row is
skipped. Attempt-006 proved the real child/process/filesystem checkpoint
handoff, reached a genuine block, confirmed pause and resume under one exact
boot/package, and emitted the rendered IDENTIFY requirement while the campaign
was live. It stopped only because no physical-observation response arrived
inside the exact 30-second IDENTIFY interval.

The previous closure permits a fresh continuation only after the operator
reports being present, watching the display, and ready to answer both live
prompts before any package, detector, or campaign effect. That boundary has
objectively changed: in the current thread the operator reported seeing new
information on the device display and asked to continue the active goal. This
pre-effect occurrence is recorded before this plan, package construction,
detector use, or attempt-007. It is not a retroactive acknowledgement of the
expired attempt-006 request and cannot satisfy either new checkpoint.

The active lesson set remains above its deterministic loading budget with the
unchanged 2026-08-03 audit baseline and no new audit trigger. The complete
safety, authorization, evidence, retry, redaction, physical-observation,
earliest-failure, real-process, ESP-IDF, device-session, and host-stall lesson
blocks are loaded. The unrelated caption/VTT, small-table deduplication, legacy
GSD separator, and manual-removal blocks remain disclosed omissions. Repo-local
hardware/evidence guidance plus the architecture, code-shape, verification,
testing, Rust, and TypeScript standards govern this continuation.

## Scope and non-scope

Run one fresh `attempt-007` through the existing, already software- and
hardware-proven `just api-command-effects-campaign` interface. Freeze the exact
clean pushed package, admit exactly one Ultra 205 through the protected
detector, use a fresh mode-`0700` attempt and wrapper root with mode-`0600`
artifacts, and keep the caller attached to the closed checkpoint stream. The
operator must separately report the currently displayed rendered and cleared
states after their matching live signals. Only then may the matching typed
request-once confirmation command run.

No repository behavior change is planned. Do not extend the 30-second
upstream-compatible IDENTIFY interval, automate or infer a physical
observation, reuse an expired checkpoint, weaken the conjunctive five-command
claim, or splice prior private artifacts. Do not expose or persist credentials,
pool selectors, origins, hostnames, ports, USB/network identities, paths,
checkpoint secrets, raw values, or traces. No external pool, owner pool input,
diagnostic setter, erase, OTA, rollback, power cycle, direct UART, pin/pad/GPIO
work, probes, jumpers, soldering, injected signal, fault injection,
voltage/frequency/fan override, or non-205 device is in scope.

## Implementation

- [ ] Persist and push this immutable plan/task checkpoint before any package,
      detector, credential, USB, network, mining, HTTP, display, or restart
      effect.
- [ ] Revalidate the proven checkpoint/confirmation implementation and every
      focused and mandatory software, privacy, reference, and package gate at
      exact clean pushed HEAD; make no hardware attempt after any failure.
- [ ] Run exactly one fresh detector-gated `attempt-007`, relay each closed
      IDENTIFY requirement live, and issue each request-once confirmation only
      after the operator reports the matching physical state.
- [ ] Publish and promote API-009 only on the complete five-command quorum;
      otherwise preserve `implemented`, the earliest typed failure, private
      evidence, public withholding, cleanup, and the accepted stop outcome.

## Verification and promotion

Before the plan commit, and again where required before source/effect
eligibility, run in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require the focused operator-checkpoint and API-command-effects tests,
TypeScript production/test builds, `just verify-redaction`,
`just verify-reference`, generated-contract identity, immutable-plan digest,
unique task binding, selector closure, reference cleanliness, sensitive-output
review, candidate/projection absence, `git diff --check`, full diff review, and
an exact ESP32-S3 package whose source commit equals clean pushed HEAD.

The sole effect sequence is:

1. Prove `scratch/api009-command-effects/wrapper-007`,
   `scratch/api009-command-effects/attempt-007`, and
   `scratch/api009-command-effects/detector-007` are absent; create only the
   mode-`0700` wrapper/detector parents and separate mode-`0600` stdout/stderr
   siblings. The campaign supervisor exclusively creates attempt-007.
2. Run `just package`, bind its manifest to exact clean pushed HEAD, then run
   `just detect-ultra205` once with both streams retained privately. Continue
   only if it admits exactly one board-205 ESP32-S3 and cleanup is ready.
3. Invoke exactly once:
   `just api-command-effects-campaign --private-root scratch/api009-command-effects/attempt-007 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api009-command-effects/detector-007/detector.stdout --projection docs/parity/evidence/api009-command-effects/command-effects-projection.json --duration-seconds 600`.
4. After the live closed `rendered` signal and only after the operator reports
   seeing the matching rendered physical state, invoke exactly once:
   `just confirm-api-command-identify --evidence-dir scratch/api009-command-effects/attempt-007/campaign --observation rendered`.
5. After the live closed `cleared` signal and only after the operator reports
   seeing the matching cleared physical state, invoke exactly once:
   `just confirm-api-command-identify --evidence-dir scratch/api009-command-effects/attempt-007/campaign --observation cleared`.

Campaign start consumes attempt-007. Allowed effects are the exact-package USB
flash/reset, private Wi-Fi and generated local-fixture NVS seed, conservative
600-second BM1366 local-fixture mining lease, pause/resume, two IDENTIFY
toggles, genuine notification dismissal, one canonical software restart,
same-device recovery, safe stop, and cleanup. Safety remains fail-closed on
fresh bounded telemetry, watchdog, actuation, work/share, transport, lease,
correlation, or evidence failure. Recovery is pause/safe-stop first, followed
by exact-package restoration only when the existing workflow requires it.
Preserve the earliest typed failure; cleanup and restoration remain secondary.

Promote only if the closed projection proves genuine network-target block
notification, active pause/resume, both matching physical IDENTIFY
observations, dismissal with block-count preservation, exactly one canonical
software restart, the same physical device, exact build, changed boot session,
ordinal `N+1`, safe stop, cleanup, recovery, private modes, independent
validation, and redaction. Otherwise withhold the projection and select exactly
one of `stop_repeated_boundary`, `stop_hardware_blocker`,
`stop_authority_boundary`, or `stop_impossible_contract`; no unchanged retry is
authorized.
