# Phase 14 Safety Hardware Evidence Packs

## Scope

This directory is the Phase 14 component-pack contract for Ultra 205 board `205`
safety hardware evidence. It defines what each pack may claim, which metadata is
required, how a `safety-allow` manifest gates hardware-facing procedures, and
when a conservative `hardware evidence pending` conclusion is required.

Phase 14 evidence is procedure-scoped. A pack can support only the exact claim
named by its allow manifest, command, artifacts, observed readings or blocker,
conclusion, and redaction review. It must not promote a broad checklist row when
active subclaims still require `hardware-regression`.

## Required Hardware Gate

Every live Phase 14 run starts with:

```bash
just detect-ultra205
```

Continuation is allowed only when the detector finds exactly one likely ESP USB
serial port and the detector-owned board-info check succeeds for board `205`:

```bash
espflash board-info --chip esp32s3 --port <port> --non-interactive
```

If the detector finds zero ports, multiple ports, a non-205 target, board-info
failure, stale package identity, missing recovery instructions, missing
post-action safe-state markers, unavailable route or stimulus, or redaction
uncertainty, the affected pack records `hardware evidence pending`.

## Allow Manifest Contract

Before a Phase 14 wrapper runs an evidence-producing or active hardware-facing
procedure, it must validate a procedure-scoped allow manifest with this command
shape:

```bash
bazel run //tools/parity:report -- safety-allow --manifest <path> --surface <surface> --allowed-command <command>
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
- `evidence_dir`
- `redaction_reviewer`
- `checklist_rows`

Active claim tiers require `evidence_class` exactly `hardware-regression`.
Read-only, safe-unavailable, API/WebSocket projection, unsupported-pending, and
parity-redaction tiers may support only the exact non-active claim recorded by
their manifest and artifacts.

## Required Metadata For Every Pack

Each component pack records:

- Board `205`.
- Selected `port=<path>`.
- `just detect-ultra205` transcript or blocker.
- `espflash board-info --chip esp32s3 --port <port> --non-interactive`
  output or blocker.
- Source commit.
- Reference commit.
- Package manifest path.
- Exact command.
- Allow manifest path.
- Raw artifact paths.
- Observed readings, observed behavior, or failure/blocker.
- Conclusion scoped to the pack claim.
- Redaction review status.

## Component Packs

| Pack | Claim tier boundary | Required pack result |
| --- | --- | --- |
| `safe-baseline` | Safe boot, detector, board-info, package identity, and safe-state markers only. | Passed only for exact board-205 safe baseline observations; otherwise `hardware evidence pending`. |
| `power-telemetry` | INA260 current, voltage, power, freshness, and read status as read-only observations. | Read-only observation or safe-unavailable status only; no actuator or power-sequencing claim. |
| `voltage-control` | DS4432U voltage writes and suppressed voltage effects. | Active voltage-control parity requires bounded `hardware-regression`; otherwise `hardware evidence pending`. |
| `thermal-fan` | Thermal readings, fan RPM observation, fan duty effects, and overheat/fault boundaries. | Read-only thermal/RPM observations can be exact subclaims; fan duty and fault behavior require bounded `hardware-regression`. |
| `self-test-watchdog-load` | Self-test lifecycle, watchdog supervisor behavior, bounded liveness, and load/stress responsiveness. | Pure or safe-unavailable subclaims stay narrow; self-test hardware and load stress require bounded `hardware-regression`. |
| `display-input` | Startup display, runtime display, screen flow, LVGL parity, and input behavior. | Startup display may be exact evidence; runtime display/input stays `hardware evidence pending` without a real route and observation. |
| `live-api-websocket-telemetry` | Live HTTP/API and WebSocket safety telemetry projection, cadence, and freshness. | Requires explicit route target and redaction review; missing `DEVICE_URL` or frame client records `hardware evidence pending`. |
| `parity-redaction` | Checklist guard, evidence-class matching, and artifact redaction governance. | Supports evidence-governance claims only; it does not verify hardware behavior. |

## Prohibited Actions Outside A Phase 14 Allow Manifest

Do not run any of these ad hoc actions outside a Phase 14 allow manifest and
surface wrapper:

- ad hoc voltage writes
- fan duty commands
- overheat or fault stimulus
- self-test hardware submode
- load or stress workload
- runtime input or display actuation
- raw I2C write
- raw flash write
- erase
- rollback
- interrupted update
- mining stress

If a plan lacks a validated allow manifest, exact input bounds, abort
conditions, recovery steps, post-action safe-state checks, evidence destination,
and redaction reviewer, the only acceptable result is `hardware evidence
pending`.

## Checklist Promotion Rules

Use exact-claim promotion. `hardware-smoke` can support only exact board-named
safe baseline, read-only telemetry, safe-unavailable, or startup-only
observations with board, port, commits, command/log, conclusion, and redaction
review. `hardware-regression` is required for active voltage writes, fan duty
effects, ASIC reset/power sequencing, overheat/fault behavior, self-test
hardware submodes, runtime input/display behavior, watchdog/load stress, and
failure-path parity.

Broad rows remain below `verified` while any active subclaim is missing a
matching evidence class. Pending evidence should name the owner, blocker,
required recovery path, and follow-up surface rather than borrowing claims from
older or narrower evidence.
