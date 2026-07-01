# Phase 15 BM1366 Mining Evidence Packs

## Scope

This directory is the Phase 15 component-pack contract for Ultra 205 board
`205` BM1366 and mining evidence. It defines the hardware gate, mining allow
manifest contract, evidence ladder, required metadata, stop conditions, and
checklist promotion rules that apply before any generated logs are cited.

Phase 15 evidence is procedure-scoped. A pack can support only the exact claim
named by its `mining-allow` manifest, command, artifacts, observed behavior or
blocker, conclusion, and redaction review. It must not promote broad ASIC,
Stratum, API, WebSocket, statistics, or mining rows when the evidence proves
only a narrower diagnostic or controlled no-share condition.

## Required Hardware Gate

Every live Phase 15 run starts with:

```bash
just detect-ultra205
```

Continuation is allowed only for board `205` after the detector finds exactly
one likely ESP USB serial port and this board-info command succeeds:

```bash
espflash board-info --chip esp32s3 --port <port> --non-interactive
```

If the detector finds zero ports, multiple ports, a non-205 target, board-info
failure, stale package identity, missing recovery instructions, missing
telemetry prerequisites, missing post-action safe-state markers, or redaction
uncertainty, the affected pack records `hardware evidence pending`.

Later tiers may run only when earlier tiers pass. A failed detector, package,
safe-state, chip-detect, work/result, telemetry, watchdog, or redaction
prerequisite stops the current pack and prevents checklist promotion.

## Mining Allow Manifest Contract

Before a Phase 15 wrapper runs an evidence-producing BM1366 diagnostic, mining
smoke, bounded soak, or redaction-governance command, it must validate a
procedure-scoped allow manifest with this command shape:

```bash
bazel run //tools/parity:report -- mining-allow --manifest <path> --surface <surface> --allowed-command <command>
```

The allow manifest fields are:

- `board`
- `port`
- `detector_command`
- `detector_port`
- `board_info_command`
- `board_info_status`
- `package_manifest`
- `source_commit`
- `reference_commit`
- `surface`
- `claim_tier`
- `evidence_class`
- `allowed_command`
- `allowed_inputs`
- `abort_conditions`
- `recovery_steps`
- `post_action_safe_state_markers`
- `prerequisite_artifacts`
- `evidence_dir`
- `redaction_reviewer`
- `checklist_rows`

Accepted surfaces are exactly `bm1366-chip-detect`, `bm1366-work-result`,
`mining-smoke`, `bounded-soak`, and `parity-redaction`.

Accepted claim tiers are exactly `diagnostic-chip-detect`,
`diagnostic-work-result`, `controlled-no-share`, `live-pool-smoke`,
`bounded-soak`, `unsupported-pending`, and `parity-redaction`.

Diagnostic and smoke claim tiers require `evidence_class` exactly
`hardware-smoke`. The `bounded-soak` tier requires `evidence_class` exactly
`soak`. The `unsupported-pending` and `parity-redaction` tiers require
`evidence_class` exactly `workflow`.

`live-pool-smoke` may run only when `allowed_inputs.pool_config` is
`disposable-or-non-secret` and `allowed_inputs.device_url` is `explicit`.
`bounded-soak` must set `allowed_inputs.duration_seconds` between `60` and
`600` inclusive and include abort conditions, recovery steps, and post-action
safe-state markers.

## Evidence Ladder

| Tier | Required prerequisites | Supported conclusion |
| --- | --- | --- |
| `detector/package/safe boot` | `just detect-ultra205`, board `205`, successful board-info, package identity, source commit, reference commit, safe-state markers, and redaction scope. | Exact detector, package, and safe boot claims only; otherwise `hardware evidence pending`. |
| `trusted BM1366 chip-detect or staged init` | Detector/package/safe boot plus a trusted wrapper path, no-mining scope, and chip-detect or staged-init command. | Chip-detect, staged-init, UART timeout, chip-count mismatch, partial read, or fail-closed status only. |
| `typed diagnostic work/result` | Prior tier plus a repo-owned typed diagnostic work/result command and bounded result-or-timeout handling. | Diagnostic work dispatch, result frame parsing, invalid-job rejection, bounded timeout, or no-result observation only. |
| `controlled mining smoke` | Prior tier plus safety gates, controlled no-share or disposable pool setup, watchdog/status observations, telemetry prerequisites, safe-stop, and redaction review. | Pool lifecycle, job construction, work pipeline, accepted/rejected share, or controlled no-share behavior only. |
| `bounded mining soak` | Prior tier plus duration between `60` and `600` seconds, abort conditions, thermal/power/watchdog observations, telemetry snapshots when available, reconnect/fallback scope when exercised, and final safe stop. | Bounded soak completion or safe abort with exact observed share/no-share, telemetry, watchdog, and safe-state evidence. |
| `checklist promotion` | Relevant tier evidence, redaction review complete, checklist notes updated, and `just parity` passes. | Promote only exact rows whose evidence class and artifact match the claim; otherwise keep rows below `verified`. |

## Required Metadata For Every Pack

Each component pack records:

- Board `205`.
- Selected `port=<path>`.
- `just detect-ultra205` transcript or blocker.
- `espflash board-info --chip esp32s3 --port <port> --non-interactive`
  output or blocker.
- Source commit.
- Reference commit.
- Package manifest path and package identity conclusion.
- Exact command.
- Mining allow manifest path.
- Prerequisite artifact paths.
- Raw artifact paths.
- Observed ASIC, mining, telemetry, watchdog, safe-stop, or failure behavior.
- Conclusion scoped to the pack claim.
- Redaction review status.

## Component Packs

| Pack | Claim boundary | Required pack result |
| --- | --- | --- |
| `bm1366-chip-detect` | Trusted BM1366 chip-detect or staged initialization without production mining or work submission. | Exact chip-detect, staged-init, UART, chip-count, timeout, or fail-closed observation; otherwise `hardware evidence pending`. |
| `bm1366-work-result` | Typed diagnostic work-send and result-receive behavior through repo-owned BM1366 abstractions. | Exact diagnostic dispatch, result, invalid-job, timeout, or no-result observation; otherwise `hardware evidence pending`. |
| `mining-smoke` | Controlled mining smoke, including controlled no-share or live-pool micro-smoke only with disposable/non-secret inputs. | Exact pool lifecycle, work pipeline, share/no-share, watchdog, telemetry, and safe-stop outcome; otherwise `hardware evidence pending`. |
| `bounded-soak` | Bounded mining soak with explicit duration, abort conditions, thermal/power/watchdog observations, telemetry snapshots when available, reconnect/fallback scope when exercised, and safe stop. | Exact bounded run or safe abort result with no unbounded stress or leaked secrets; otherwise `hardware evidence pending`. |
| `parity-redaction` | Checklist guard, evidence-class matching, and artifact redaction governance. | Supports evidence-governance claims only; it does not verify BM1366, Stratum, API, WebSocket, statistics, or mining behavior. |

## Stop Conditions

Stop immediately and record pending evidence when any of these occur:

- `just detect-ultra205` finds zero likely ports.
- `just detect-ultra205` finds multiple likely ports.
- The target is not board `205`.
- Board-info fails.
- Package manifest identity does not match the manifest source or reference
  commit.
- Trusted wrapper markers are missing.
- Recovery steps are missing or unsafe.
- Required telemetry prerequisites are missing.
- Abort conditions are missing.
- Post-action safe-state markers are missing.
- Temperature, power, watchdog, or serial responsiveness is unsafe.
- Pool credentials, worker secrets, Wi-Fi credentials, private endpoints,
  private `DEVICE_URL`, API tokens, NVS secret values, local terminal secrets,
  or pasted raw secrets appear in generated artifacts.
- Redaction uncertainty remains for logs, JSON, Markdown evidence, API
  responses, WebSocket captures, pasted output, terminal output, or manual
  observations.

## Prohibited Actions

Do not run any of these ad hoc actions outside a Phase 15 mining allow manifest
and surface wrapper:

- Raw BM1366 serial writes.
- Hand-built BM1366 frames in scripts.
- Direct pool scripts with committed credentials.
- Inferred or scanned `DEVICE_URL` targets.
- Unbounded mining stress.
- Voltage stress or fan stress.
- Raw I2C writes.
- Direct `espflash` outside repo-owned wrapper evidence flow.
- Direct `esptool.py` outside repo-owned package or recovery flow.
- Erase, rollback, interrupted update, failed update, or raw flash writes.
- Any command that commits pool credentials, worker secrets, Wi-Fi credentials,
  private endpoints, private `DEVICE_URL`, NVS secret values, API tokens, or
  local terminal secrets.

If a plan lacks a validated mining allow manifest, exact input bounds, abort
conditions, recovery steps, post-action safe-state checks, evidence
destination, telemetry prerequisite decision, and redaction reviewer, the only
acceptable result is `hardware evidence pending`.

## Checklist Promotion Rules

Use exact-claim promotion. `hardware-smoke` can support only board-named
diagnostic and smoke claims with board, port, commits, command/log, observed
behavior, conclusion, and redaction review. `soak` is required for bounded soak
claims. `workflow` supports only criteria, pending, and redaction-governance
claims.

Do not verify full BM1366 initialization from chip-detect alone. Do not verify
production work submission from typed diagnostic work. Do not verify live mining
from fake-pool, host-only, safe boot, or package evidence. Do not verify live
API/WebSocket/statistics behavior unless the evidence captures the live route or
stream for the exact claim.

Broad rows remain below `verified` while any required subclaim is missing the
matching evidence class. Pending evidence should name the owner, blocker,
required recovery path, telemetry prerequisite, and follow-up surface instead
of borrowing claims from older or narrower evidence.
