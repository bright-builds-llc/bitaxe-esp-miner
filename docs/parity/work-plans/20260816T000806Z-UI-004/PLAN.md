# Parity work plan

- Run ID: `20260816T000806Z-UI-004`
- Parity row: `UI-004`
- Initial status: `implemented`
- Source commit: `df32a1a248eeb5d749da2ef8bf2a1bcedb1fcb6f`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-ui004-projection-continuation`

Continues the closed attempt at
`docs/parity/work-plans/20260813T045300Z-UI-004/PLAN.md` without repeating its
hardware or browser effects.

## Selection

The clean synchronized selector returned no open plan and ordered `UI-001`,
`UI-002`, `UI-003`, `SELF-001`, `BAP-002`, then `UI-004`. `UI-001` and
`UI-002` still require a trusted physical panel observation, `UI-003` requires
a recorded physical button interaction, `SELF-001` requires qualified
self-test stimulus, and `BAP-002` requires a compatible accessory plus a
separately authorized electrical attachment. None of those evidence inputs is
available through the authorized software and USB surfaces without human or
external-hardware intervention.

`UI-004` is the first actionable row. Its exact-package attempt-001 already
closed the hardware transaction and the read-only browser quorum. The only
failure was the projector wrapper's process-default `0644` redirection files.
The prior closure permits a fresh software-only continuation that proves an
owner-only redirect contract and distinguishes the captured package source
from the later projector source. The protected attempt and browser roots are
present with their required owner-only modes, and the public projection and
candidate remain absent.

## Scope and non-scope

Repair only the UI workflow evidence join. Preserve
`bf5b74f98cdb117ca5682b0118a61743db85856f` as the attempt/package/browser
source identity, record the clean current commit separately as the projector
source, and admit the old observation only when the attempt source is an
ancestor of the projector source and the following captured UI/static-serving
paths are byte-unchanged between them and clean in the worktree:

- `firmware/bitaxe/static/www/index.html`
- `firmware/bitaxe/static/www/assets/app.css`
- `firmware/bitaxe/static/www/assets/ui-core.js`
- `firmware/bitaxe/static/www/assets/api-client.js`
- `firmware/bitaxe/static/www/assets/app.js`
- `firmware/bitaxe/src/static_files.rs`
- `firmware/bitaxe/src/filesystem.rs`
- `crates/bitaxe-api/src/static_plan.rs`
- `tools/automation/src/static-ui.test.ts`
- `tools/automation/src/static-provenance.test.ts`

The projector must independently validate the preserved operator projection,
browser attestation and every browser artifact; bind their digests, the closed
attempt plan and closure digests, the exact package-manifest digest, application
ELF digest, static-image digest, source/reference identities, prior joined
evidence, protected modes, cleanup, safe state and redaction; and publish only
the existing aggregate evidence schema after the Rust validator passes. The
schema may be narrowed to explicit attempt/projector source fields because no
UI-004 public projection exists.

This continuation may inspect the protected attempt only through the repo-owned
projector and validators. It may modify and test repository source, contracts,
generated bindings, task/worklog/result/checklist artifacts, and run one
software-only projection/validation transaction. It must not print, summarize,
copy into Git, or otherwise expose private file contents, paths beyond the
already committed contract, origins, hostnames, addresses, ports, USB/network/
process identities, page values, HTTP or WebSocket bodies, screenshots, traces,
or credentials.

No detector, USB, device, network request, browser session, flash, reset,
restart, settings/theme submission, firmware upload, OTA/OTAWWW, mining, ASIC
work, pool access, display or button claim, direct UART, pin/pad/header/GPIO
interaction, voltage, frequency, fan, thermal, power, recovery, or hardware
attempt is authorized. The preserved attempt remains immutable.

## Implementation

- [ ] Add distinct attempt-source and projector-source identities plus closed
      compatibility-path, prior-plan, prior-closure and protected-artifact
      bindings to the UI workflow projector, schema, generated TypeScript and
      independent Rust validator.
- [ ] Add a production-shaped regression proving shell-created capture files
      pass only under `umask 077`, along with negative tests for source
      ancestry, compatibility-path drift, dirty paths, mismatched protected
      identities, changed closure/plan digests and malformed private evidence.
- [ ] Run focused source/contract/projector/real-child/redaction/static-UI
      tests and every mandatory repository gate, then commit and push the
      implementation before the one projection transaction.
- [ ] With owner-only redirections, run the projector exactly once over the
      preserved attempt, validate the public candidate independently, and
      promote only `UI-004` if the complete closed quorum passes.

## Verification and promotion

Before the projection transaction, run focused UI-workflow Rust and TypeScript
tests, the generated-contract verifier, static UI/provenance tests, redaction,
the real firmware build, and the repository-required sequence:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`
9. `just verify-redaction`
10. `just verify-reference`
11. immutable-plan, task-uniqueness, protected-mode, sensitive-output,
    compatibility-path, no-public-output and diff checks

After the implementation commit is pushed, clean and synchronized, run exactly
one transaction from the repository root:

```bash
(umask 077; just project-ui-workflow-evidence \
  --private-root scratch/ui004-live-workflows/attempt-001 \
  --attempt-source-commit bf5b74f98cdb117ca5682b0118a61743db85856f \
  --operator-snapshot-projection scratch/ui004-live-workflows/wrapper-001/operator-snapshot-projection.private.json \
  --browser-attestation output/playwright/ui004-attempt-001/browser-attestation.private.json \
  --projection docs/parity/evidence/ui004-live-workflows/ui-workflow-projection.json \
  > scratch/ui004-live-workflows/wrapper-001/projector-002.stdout \
  2> scratch/ui004-live-workflows/wrapper-001/projector-002.stderr)
```

The public projection and candidate plus both `projector-002` redirect files
must be absent before launch. Starting this command consumes the sole
software-only projection transaction; do not retry it. Validate the published
projection once with
`just validate-ui-workflow-evidence docs/parity/evidence/ui004-live-workflows/ui-workflow-projection.json`,
redirected beneath the protected wrapper with the same `umask 077`.

Promotion to `verified` requires the projector and independent validator to
pass; exact captured and projector source identities; clean compatible source
paths; all seven desktop/mobile routes; responsive navigation; blank
write-only secrets; update and OTAWWW guards; same-origin API/log traffic; zero
console or unexpected network failures; exact package/static identities;
normal restart; disabled mining and hardware control; protected modes;
complete browser/device cleanup; joined prior evidence; redaction; and all
mandatory gates. Any nonzero command, missing/malformed input, source drift,
mode failure, privacy failure, or incomplete quorum withholds the projection,
leaves `UI-004` `implemented`, records a terminal closure, and stops without a
second projection or hardware/browser attempt.

Physical panel rendering, physical input, settings/theme mutation in this
attempt, firmware upload, OTAWWW behavior beyond its unavailable UI guard,
scoreboard/swarm population, mining telemetry, other boards, and release
readiness remain explicit non-claims.
