# Parity work plan

- Run ID: `20260812T182405Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `cde304c6a7a0bae0af89164f36394ab451e2c370`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260812T173427Z-API-009/PLAN.md`

## Selection

The clean synchronized selector again ranks API-009 first, so no candidate is
skipped. Attempt-005 materially proves the production readiness remediation:
the same exact-package boot returned active after confirmed pause and resume,
where attempt-004 stopped. It also reached the first physical IDENTIFY
checkpoint. The complete row remained blocked because the orchestration
contract did not make that checkpoint observable to its caller while the
campaign was running, so neither rendered nor cleared confirmation completed.

Source inspection confirms the boundary. The Rust campaign writes one private
mode-`0600` `required` document immediately after each request and already
provides a typed request-once confirmation command. The TypeScript automation
parent instead awaits `ProcessPort.run`; the local process port buffers child
stdout and stderr until settlement, and `captureApiCommandEffects` neither
watches the required documents nor emits an intermediate closed signal. The
only usable notification is therefore an internal private path that an
external caller must guess and poll. This is an orchestration contract gap,
not evidence that the display failed to render.

The active lesson set remains above its deterministic loading budget with the
unchanged 2026-08-03 audit baseline and no new audit trigger. The complete
safety, authorization, evidence, retry, redaction, physical-observation,
earliest-failure, real-process, ESP-IDF, and host-stall lesson blocks remain
loaded; the previously disclosed unrelated omitted set remains unchanged.

## Scope and non-scope

Add a typed, redaction-safe operator-checkpoint sink to the API-009 automation
shell. While the existing campaign child promise remains pending, supervise
the two exact private required documents in order. Admit only regular
mode-`0600` files with schema `bitaxe-identify-checkpoint-v1`, the expected
closed observation (`rendered` then `cleared`), and status `required`. Emit
each accepted signal exactly once through stderr with only schema, command,
observation, and status. Preserve stdout as one final
`bitaxe-automation-result-v1` envelope.

Do not reintroduce a general process-start API. Race the already running child
settlement against a bounded, cancellation-safe private checkpoint watcher so
no poll survives child completion. The watcher must never create a
confirmation, claim a physical observation, expose a path, or replace the
Rust campaign's typed validation and consume-once ownership. Missing,
malformed, wrongly ordered, non-private, duplicate, or post-settlement
checkpoints remain fail-closed with the earliest campaign or evidence failure
primary.

No firmware behavior, 30-second upstream IDENTIFY duration, display renderer,
HTTP route, mining safety bound, protocol behavior, package format, device
session, or public evidence schema changes are in scope. No synthetic display
state, automatic confirmation, inferred physical observation, preconfirmation,
camera, raw trace, origin, hostname, port, USB/network identity, credential,
endpoint, checkpoint secret, or private path may enter the signal or public
output.

## Implementation

- [ ] Add the closed operator-checkpoint model, injected sink, and concurrent
      campaign-settlement supervisor without broadening `ProcessPort`.
- [ ] Add behavior-focused fake and real-child regressions for ordered prompt
      publication, acknowledgement, settlement cancellation, malformed and
      missing checkpoints, stdout singularity, and sensitive-output absence.
- [ ] Run every focused and mandatory gate, review the process/privacy
      boundary, then commit and push the exact source before hardware.
- [ ] Conditionally run the sole detector-gated attempt-006 and publish only a
      complete five-command API-009 quorum.

## Verification and promotion

Focused tests must first reproduce that a production-shaped child can publish
a required checkpoint while the current parent remains silent until child
settlement. After the fix they must prove immediate, ordered, exactly-once
rendered/cleared signals; no automated confirmation; typed confirmation
consumed once; campaign settlement cancels outstanding polling; malformed,
wrong-mode, wrong-order, duplicate, and missing documents fail closed; a child
failure remains primary; stderr contains only the closed checkpoint vocabulary;
stdout remains one final JSON envelope; and no sensitive input or private path
appears in either public channel. At least one regression must cross a real
child-process and filesystem boundary.

Then run, in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require `just verify-redaction`, `just verify-reference`, exact generated
contracts, selector and unique-task binding, immutable-plan digest, reference
cleanliness, sensitive-output review, `git diff --check`, full diff review, and
an exact ESP32-S3 firmware/package build. Commit and push this plan/task
checkpoint before implementation. Commit and push verified source before any
package or detector action.

Only after all software gates pass may one fresh `attempt-006` use the existing
`just api-command-effects-campaign` interface, an exact pushed package, fresh
mode-`0700` ignored private root
`scratch/api009-command-effects/attempt-006`, fresh detector root
`scratch/api009-command-effects/detector-006`, and the still-absent public
projection `docs/parity/evidence/api009-command-effects/command-effects-projection.json`.
The caller must remain attached to the safe checkpoint stderr stream. After
the user explicitly reports seeing the rendered or cleared physical state for
the currently required checkpoint, invoke exactly one matching typed command:

1. `just confirm-api-command-identify --evidence-dir scratch/api009-command-effects/attempt-006/campaign --observation rendered`
2. `just confirm-api-command-identify --evidence-dir scratch/api009-command-effects/attempt-006/campaign --observation cleared`

Neither command may run before its signal and matching physical observation.
Campaign start consumes the ordinal. No retry, unchanged attempt, or inferred
acknowledgement is allowed. Effects remain limited to the prior conservative
600-second local-fixture lease, exact-package USB flash/reset, conservative
BM1366 mining, pause/resume, two IDENTIFY toggles, dismissal, one canonical
software restart, safe stop, cleanup, and exact-package recovery. No external
pool, owner credentials, diagnostic setter, erase, OTA, rollback, power cycle,
direct UART, pins, fault injection, voltage/frequency/fan override, or control
override is allowed.

Promote API-009 only if the closed evidence proves genuine network-target
notification, confirmed pause/resume, both matching physical IDENTIFY
observations, dismissal with block-count preservation, exactly one software
restart, same physical device, exact build, changed boot session, ordinal
`N+1`, safe stop, cleanup, recovery, private modes, and redaction. Otherwise
retain `implemented`, preserve the earliest typed category, withhold the public
projection, close truthfully, and do not retry.
