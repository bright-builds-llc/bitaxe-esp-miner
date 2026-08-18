# Parity work plan

- Run ID: `20260818T150603Z-CFG-07`
- Parity row: `CFG-07`
- Initial status: `implemented`
- Source commit: `e6a6b041673576862326c8ac861ea883a1756c38`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-cfg07-runtime-credentials`

## Selection

The clean `main` worktree equals `origin/main`, the pinned reference is clean,
and the selector reports no open plan. Candidate order is `SELF-001`,
`BAP-002`, `STAT-003`, then `CFG-07`.

`SELF-001` remains unavailable because no production-safe full self-test route
exists for its hardware regression. `BAP-002` remains dependency- and safety-
blocked by unfinished `BAP-001` firmware/UART lifecycle plus unauthorized
external electrical UART work. `STAT-003` remains environment-blocked after
attempt-004's distinct `network_unavailable` result; its active task prohibits
an unchanged retry without an objective protected pool/network recovery signal.
`CFG-07` is therefore first actionable.

Fresh eligible evidence now exists without another device effect. The accepted
SAFE-10 projection is derived from detector-admitted scoreboard attempt-003,
whose immutable command contract requires local Wi-Fi and pool credential paths.
That projection proves accepted live mining, current prerequisite semantics,
safe stop, cleanup, protected modes, and redaction while exposing no credential
values. A public-only projector can join this same-chain proof to the attempt
and current credential-flow source without opening credentials or protected
attempt artifacts.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
managed architecture/code-shape/verification/testing/Rust standards, the
active tracker/checklist, and bounded lessons for protected evidence,
private-first classification, earliest failure, source identity, standing
authorization, and agent-runtime timing. Active lesson inputs total 31,758
bytes, so headings were inventoried and relevant complete blocks loaded;
less-relevant blocks were omitted under the deterministic budget. The August
lesson-audit baseline remains current and no new audit trigger is due.

## Scope and non-scope

Advance only `CFG-07`. Add a typed `bitaxe-cfg07-evidence-v1` contract,
independent Rust validator, public-only TypeScript projector, source inventory,
real-validator tests, CLI/Just/Bazel wiring, and one closed public projection.
The projector may read only committed SAFE-10/plan/closure/source/history inputs.
It must never read credential files or protected attempt contents.

Bind the fixed inputs:

- `docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json`
  with SHA-256
  `4e9b91bd29629aec098b9967b9bb27b9c1358f64c11819a77f8c8da4c212a20e`;
- `docs/parity/work-plans/20260818T102038Z-STAT-003/PLAN.md` with SHA-256
  `41ca445088dcf15c4c1c46e504a754c61260e7575eb16ccf68e0edb0fc742879`;
- `docs/parity/work-plans/20260818T102038Z-STAT-003/CLOSURE.md` with SHA-256
  `350a56d6eaab1ea066f71a24d5a964a27e37d5472aca733fe912218afa87a79d`;
  and
- attempt source `60a56d4935ced15eeb5ec6950b1ad4ea35fdf223`, current
  source/reference identities, and exact credential-flow source membership.

After focused/full gates, commit/push the exact implementation and execute once:

`test ! -e docs/parity/evidence/cfg07-runtime-credentials/runtime-credentials-projection.json && just project-cfg07-evidence --safe10-projection docs/parity/evidence/safe10-prerequisite-readiness/safe10-projection.json --attempt-plan docs/parity/work-plans/20260818T102038Z-STAT-003/PLAN.md --attempt-closure docs/parity/work-plans/20260818T102038Z-STAT-003/CLOSURE.md --projection docs/parity/evidence/cfg07-runtime-credentials/runtime-credentials-projection.json`

The projection may contain only schema/workflow identity, commits, digests,
path counts, closed category labels, booleans, and bounded counts/duration. It
must contain no credential paths or contents, owner/pool/worker data, endpoints,
ports, USB/network identity, NVS values, telemetry, raw logs/payloads, commands,
PIDs, traces, or protected identifiers.

On accepted projection, update the shared Phase 30 conclusion from the obsolete
global no-promotion disposition to an explicit CFG-07-only promotion artifact.
Keep STR-09 and ASIC-11 conservative and unchanged in meaning.

This plan authorizes local source, tests, committed-public evidence reads, one
public projection, documentation, build/package, Git commit, and push only. It
authorizes no credential-file or protected-attempt access, detector,
device/USB/network runtime, flash, monitor, mining, restart, recovery, new
hardware attempt, fault injection, external UART/BAP, pins, or electrical work.

## Implementation

- [ ] Add the typed CFG-07 evidence contract, independent validator, public-only
      projector, transitive source inventory, CLI/Just/Bazel wiring, and
      real-validator failure regressions.
- [ ] Bind exact same-chain live mining, required/forwarded runtime inputs,
      attempt/current source compatibility, safe stop, cleanup, and redaction
      without reading protected inputs.
- [ ] Run focused/full gates, commit/push, execute the sole projection command,
      independently validate it, and pass direct redaction review.
- [ ] Update the Phase 30 artifact for CFG-07 only, commit projection/evidence/
      `RESULT.md` as `SOURCE_COMMIT`, transition only CFG-07, sync progress,
      archive this task, final-gate, and push.

## Verification and promotion

Focused tests cover complete evidence, immutable path/digest/plan lineage,
real validator execution, missing or malformed public inputs, source membership
and fragment drift, attempt/current incompatibility, incomplete mining/safe-stop
facts, atomic no-clobber publication, and credential/private-field rejection.

The mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`, plus firmware/package,
independent CFG-07 and SAFE-10 validation, redaction, reference, source inventory,
file-size, selector, sensitive-value, and diff checks.

Promotion requires independently validated `bitaxe-cfg07-evidence-v1` proving
board 205, detector/source/reference/plan lineage, local-owner-supplied runtime
credential inputs required and forwarded into the accepted live mining command,
accepted submit/work, safe stop, cleanup, protected source semantics, exact
attempt/current credential-flow compatibility, no committed credential values,
no raw artifacts, and passed redaction.

The Phase 30 conclusion must then contain exactly `phase30_disposition:
promoted`, `new_evidence_input: explicit`, `eligible_share_outcome: accepted`,
`hardware_accessed: true`, `credentials_accessed: true`,
`raw_artifacts_committed: no`, all five common gates `passed`, and:

- `CFG-07.runtime_credentials_input: local-owner-supplied`
- `CFG-07.live_mining_credentials_consumed: true`
- `CFG-07.committed_credential_values: none`
- `CFG-07.safe_stop_status: complete`

On success create `RESULT.md`, commit accepted evidence without checklist
change and save that full commit as `SOURCE_COMMIT`; transition only `CFG-07`
to `verified` with `unit,workflow,hardware-smoke,hardware-regression`, sync
progress, archive only this task, run final gates, and push. On failure create
`CLOSURE.md`, leave CFG-07 `implemented`, and do not sync unchanged progress.

## Non-claims

This plan does not expose or independently validate credential contents; verify
credential rotation/persistence beyond the accepted campaign; verify arbitrary
profiles/pools; promote STR-09 or ASIC-11; inject faults; verify individual
active controls, self-test, BAP/UART, other boards/ASICs, unbounded mining,
OTA/recovery, or release readiness.
