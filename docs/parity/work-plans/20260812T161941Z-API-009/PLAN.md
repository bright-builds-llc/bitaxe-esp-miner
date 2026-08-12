# Parity work plan

- Run ID: `20260812T161941Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `2fe1388cd5b4587b501109a9cc7924aa620bf51d`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260812T154751Z-API-009/PLAN.md`

## Selection

The clean synchronized selector reports no open plan and ranks `API-009`
first. No row is skipped. Attempt-002 proved the exact pushed package,
detector, board-info, and independent fixture launcher, then stopped before
serial observation as generic `flash_failed`; API-009 therefore remains the
first actionable row.

The ignored mode-protected USB session trace identifies the first boundary
without publishing raw output: the factory `write-bin` child exited while
uploading the RAM flash stub with a `FlashDeflData` command timeout. It emitted
no write, completion, or reset marker, so the durable USB supervisor correctly
classified it as `bootloader_connect_failed` with device effect `none`. The
campaign adapter then erased that typed result by converting it to an untyped
error and mapping every factory child failure to `flash_failed`. No firmware
bytes were proven transferred. The display change is an operator observation,
not trusted runtime or parity evidence.

The pinned local `espflash 4.5.0` command contract supports `write-bin
--no-stub`, which bypasses the exact RAM-stub boundary. A fresh ordinal is
eligible only after both factory and NVS writes use that bounded path and a
real-child regression proves the command construction. Instrumentation alone
does not authorize a retry.

The active lessons total 25,256 bytes with a conservative 8,419-token estimate,
above both loading limits. The unchanged 2026-08-03 audit baseline remains
valid and no new audit trigger exists. Complete relevant safety,
authorization, evidence, retry, redaction, USB-identity, earliest-failure,
real-process, ESP-IDF, and host-stall blocks were loaded. Omitted global blocks
are `lesson-use-source-vtt-for-caption-fixes`,
`lesson-zsh-lowercase-path-mutates-path`, and
`lesson-prefer-exact-row-selection-for-small-dedup`. Omitted repository blocks
include `lesson-gsd-frontmatter-body-separators`,
`lesson-native-usb-capture-needs-prearmed-observation-or-replay`,
`lesson-boot-proof-replay-must-outlive-service-sessions`,
`lesson-heartbeat-cannot-prove-over-silent-transport`,
`lesson-manual-removal-needs-owner-observation`,
`lesson-cold-boot-proof-needs-an-independent-observer`,
`lesson-esp-idf-main-task-runtime-capacity`, and
`lesson-http-liveness-is-not-response-readiness`. This is a flagged budgeted
load, not a new audit trigger.

## Scope and non-scope

Preserve the durable USB supervisor's typed child result through the flash
campaign. Add a closed serializable command diagnostic containing only schema,
terminal category, device-effect state, termination label, attempt count,
connection signature, byte counts, output SHA-256 values, and boolean transfer
facts. It must never contain raw output, a path, port, physical identity,
origin, hostname, network identity, credential, worker, address, password, or
token. The campaign records factory and NVS diagnostics in a mode-`0600`
protected artifact, binds its digest into a versioned protected result, and
keeps the first typed failure primary through cleanup and sealing.

Add `--no-stub` to both admitted factory-image and generated NVS `write-bin`
commands. This changes only the flash transport implementation; package bytes,
addresses, reset policy, exact-device ownership, retry policy, recovery,
cleanup, monitoring, firmware behavior, and public projection rules remain
unchanged.

After a clean pushed implementation, run at most one fresh `attempt-003` using
the existing `just api-command-effects-campaign` interface, an exact clean
package, a fresh mode-`0700` ignored private root, a fresh public projection
destination, and detector admission of exactly one Ultra 205. The campaign may
flash the exact factory package and generated credential seed, run the bounded
local easy-target fixture for at most the existing 600-second lease, request
IDENTIFY on/off, request pause/resume and one canonical software restart,
observe results, safe-stop, recover, and clean up.

No external pool, owner pool credential, diagnostic setter, erase, OTA,
rollback, power cycle, direct UART, pin/header/test-point interaction, fault
injection, voltage/frequency/fan override, control override, or second retry is
allowed. Reference source remains pinned and read-only. Standing task
authorization covers the bounded USB attempt after every software gate passes;
physical IDENTIFY observations are evidence checkpoints, not new permission.

## Implementation

- [ ] Add a redaction-safe typed USB command diagnostic and retain the last
      supervised `espflash` outcome without weakening earliest-failure logic.
- [ ] Preserve factory and NVS diagnostics through `FlashEnvironment`, seal a
      protected campaign flash diagnostic, and version/bind the result schema.
- [ ] Use `write-bin --no-stub` for both exact factory and generated NVS writes.
- [ ] Add behavior-focused unit and real-child regressions for success,
      connection failure, pre-transfer failure, post-transfer failure,
      completed-write recovery failure, primary-failure precedence, private
      modes, schema validation, and absence of sensitive/raw values.
- [ ] Run every focused and mandatory gate, review the simplification and
      sensitive-output surfaces, then commit and push the exact source before
      hardware.
- [ ] Conditionally run the single detector-gated `attempt-003` and publish
      evidence only for the complete API-009 quorum.

## Verification and promotion

Run focused `device-session`, `flash`, and automation tests, including a real
child process whose raw stdout/stderr contain sensitive sentinels while only
closed counts/digests reach the diagnostic. Prove factory and NVS commands both
carry `--no-stub`, exactly once, and that a non-ready typed USB result cannot be
collapsed to generic `flash_failed`. Then run, in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require `just verify-redaction`, `just verify-reference`, generated
contracts, selector and unique task binding, immutable-plan digest, reference
cleanliness, sensitive-output review, fresh attempt/projection paths,
`git diff --check`, and final diff review. Commit and push this plan/task
checkpoint before implementation, and commit and push verified source before
hardware.

For hardware, build and validate the exact pushed package, capture a fresh
private detector result, require exactly one admitted board-205 ESP32-S3 port,
and run the existing command-effects interface with `attempt-003`. Stop after
its first complete terminal result. Recovery is safe-stop/cleanup first and
exact-package recovery only if the typed device-effect state requires it; the
earliest failure remains primary. Accepted non-promotion terminals are
`hardware_blocked`, `evidence_invalid`, `timeout`, and `process_failed`.

Promotion requires one complete five-command device-user quorum: genuine
network-target ASIC notification dismissal, both physical identify
observations, pause/resume, exactly one software restart, same physical device,
exact build, changed boot session, ordinal `N+1`, safe stop, cleanup, recovery
status, and redaction. Otherwise keep API-009 `implemented`, preserve the first
typed terminal category, withhold public evidence, close the plan truthfully,
and do not retry.
