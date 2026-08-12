# Parity work plan

- Run ID: `20260812T170039Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `f595fa6f97441c1bc44975f90b4b23891292169a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260812T161941Z-API-009/PLAN.md`

## Selection

The clean synchronized selector again ranks API-009 first, so no row is
skipped. Attempt-003 materially resolved the prior factory/NVS flash boundary:
both no-stub writes completed on their first supervised attempt, trusted
runtime identity followed, safe stop and cleanup passed, and no public evidence
was emitted. The campaign then stopped before network activity with the closed
terminal reason `stratum_v1_unsupported`.

The generated campaign NVS contract explicitly writes primary `SV1`, disables
TLS and fallback preference, and treats an absent fallback selector as the
project `SV1` default. The production gate therefore should have admitted this
image. Static inspection instead finds repeated
`EspDefaultNvsPartition::take()` calls across concurrently scheduled settings,
production campaign, protocol-gate, and scoreboard adapters. The pinned
`esp-idf-svc` implementation makes that call process-exclusive until every
clone and open namespace drops. A transient or overlapping owner is currently
collapsed to the same false boolean as a genuinely unsupported selector, and
the campaign marker preserves only `stratum_v1_unsupported`. This is a firmware
ownership and observability contract mismatch, not evidence that the seeded
protocol value is unsupported.

The active lessons remain above the deterministic loading budget with the
unchanged 2026-08-03 audit baseline and no new trigger. Complete relevant
safety, authorization, evidence, retry, redaction, USB-identity,
earliest-failure, real-process, ESP-IDF, NVS-confirmation, and host-stall blocks
were loaded. The omitted lesson set is unchanged from the immediately preceding
API-009 plan; this remains a flagged budgeted load rather than a new audit
trigger.

## Scope and non-scope

Acquire the default ESP-IDF NVS partition exactly once during ordered boot,
retain it for the process lifetime, and provide clones to the settings,
production campaign, protocol-gate, and scoreboard adapters. Fail startup
closed if that sole acquisition fails. Preserve namespace-level transaction
locks and writable/read-only scopes; do not broaden access to pool secrets or
weaken commit/reload confirmation.

Replace the boolean protocol read with a closed decision that distinguishes
ready, partition-owner unavailable, namespace unavailable, primary selector
invalid/unsupported, and fallback selector invalid/unsupported. Carry only the
closed category into the production snapshot and campaign marker so protected
evidence can distinguish ownership failure from a real protocol mismatch.
Never expose selector values, endpoints, users, workers, credentials, network
identifiers, ports, paths, USB identities, or raw logs.

After a clean pushed implementation, run at most one fresh `attempt-004` using
the existing `just api-command-effects-campaign` interface, exact-package
admission, a fresh mode-`0700` ignored private root, a fresh public projection,
and fresh detection of exactly one Ultra 205. The effects and 600-second local
fixture lease remain identical to attempt-003.

No external pool, owner pool credential, diagnostic setter, erase, OTA,
rollback, power cycle, direct UART, pin/header/test-point interaction, fault
injection, voltage/frequency/fan override, control override, or second retry is
allowed. Reference source remains pinned and read-only. Instrumentation alone
does not authorize attempt-004; the shared-owner change and production-shaped
regressions must prove a material correction first.

## Implementation

- [ ] Add one boot-lifetime default-NVS partition owner and route every firmware
      settings, production, protocol, and scoreboard consumer through clones.
- [ ] Add a pure closed protocol-gate decision and carry it through the
      production snapshot and versioned campaign marker/evidence parser.
- [ ] Prove the exact generated campaign NVS selector/default contract, the
      exclusive owner seam, startup ordering, transient failure recovery,
      redaction, and schema validation with focused tests.
- [ ] Run every focused and mandatory gate, review ownership and sensitive
      surfaces, then commit and push the exact source before hardware.
- [ ] Conditionally run the sole detector-gated attempt-004 and publish only a
      complete API-009 quorum.

## Verification and promotion

Focused tests must cover one process owner, clone reuse across all NVS
consumers, no surviving firmware `EspDefaultNvsPartition::take()` caller beyond
the owner, exact primary/fallback `SV1` decisions from the generated campaign
NVS contract, each closed failure category, recovery after an unavailable
read, campaign parser/schema binding, and absence of sensitive or raw values.
The firmware target must compile against pinned ESP-IDF. Then run, in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Also require `just verify-redaction`, `just verify-reference`, generated
contracts, selector and unique-task binding, immutable-plan digest, reference
cleanliness, sensitive-output review, fresh attempt/projection paths,
`git diff --check`, and final diff review. Commit and push this plan/task
checkpoint before implementation, and commit and push verified source before
hardware.

Promotion still requires one complete five-command device-user quorum: genuine
network-target ASIC notification dismissal, both physical identify
observations, pause/resume, exactly one software restart, same physical device,
exact build, changed boot session, ordinal `N+1`, safe stop, cleanup, recovery,
and redaction. Otherwise retain API-009 at `implemented`, preserve the first
typed terminal category, withhold public evidence, close truthfully, and do not
retry.
