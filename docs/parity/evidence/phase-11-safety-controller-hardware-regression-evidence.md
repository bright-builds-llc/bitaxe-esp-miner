# Phase 11 Safety Controller Hardware Regression Evidence

## Scope

This ledger is the Phase 11 Wave 0 runbook and evidence contract for Ultra 205 board `205` safety-controller hardware regression work. It exists before any live Phase 11 evidence run so later executors have a bounded command surface, explicit recovery prerequisites, component-scoped evidence packs, and checklist promotion rules.

This plan records documentation and artifact contracts only. It does not run live hardware commands, does not authorize active hardware control, and does not promote any checklist row to `verified`.

The ledger covers SAFE-01 through SAFE-09 and EVD-05 for these surfaces:

- ASIC reset and power sequencing.
- DS4432U voltage-control support.
- INA260 power, current, and voltage telemetry.
- EMC2101 thermal and fan observation.
- PID logic and fan hardware boundaries.
- Self-test, watchdog, load responsiveness, API/log/WebSocket status, display, and input evidence boundaries.
- Evidence-governance and parity-check promotion rules.

Every Phase 11 evidence artifact must record board `205`, selected port, source commit, reference commit, exact command or probe, package manifest or firmware identity when applicable, logs, observed behavior, conclusion, and secret-redaction review.

## Hardware Gate

The first live command for any Phase 11 hardware evidence run is:

```bash
just detect-ultra205
```

Continuation is allowed only when `just detect-ultra205` finds exactly one likely ESP USB serial port and the detector's board-info check succeeds:

```bash
espflash board-info --chip esp32s3 --port <port> --non-interactive
```

Executors must stop before any flash, monitor, probe, actuation, stress, destructive, or fault-injection command when any stop case is present:

- Zero likely ESP USB serial ports.
- Multiple likely ESP USB serial ports.
- `espflash board-info --chip esp32s3 --port <port> --non-interactive` failure.
- Target other than board `205`.
- Missing recovery path for the intended evidence claim.
- Missing evidence procedure for the intended claim.
- Redaction uncertainty for captured logs, JSON, command output, API responses, WebSocket frames, or pasted terminal text.

When a stop case is present, record the affected rows as `hardware evidence pending` with the owner or follow-up surface instead of running a workaround command.

## Recovery Protocol

This plan set permits only observe-only and wrapper-owned flash-monitor capture unless a later executor adds a bounded probe with explicit input limits, stop conditions, recovery steps, and fail-closed output. The bounded probe must say exactly what values may be sent, how the run aborts, how the board is returned to a safe state, and which artifact proves the output failed closed.

Recovery and diagnostic commands are limited to these repo-owned non-destructive commands:

- `just detect-ultra205`
- `just flash-monitor board=205 port=<port> evidence-dir=docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence`
- `just monitor port=<port>`
- `just parity`
- `just test`

Executors must stop and record `hardware evidence pending` when recovery prerequisites are incomplete. Incomplete prerequisites include no detector success, no explicit board `205` target, no selected port, no artifact destination, no redaction reviewer, no rollback to a known safe firmware image, no bounded actuation inputs, missing abort conditions, or missing evidence ownership for the affected row.

`just monitor port=<port>` is diagnostic only. It can help inspect safe logs after detector success, but it does not create wrapper-owned JSON evidence and cannot replace `just flash-monitor board=205 port=<port> evidence-dir=docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence` for claim promotion.

## Allowed Command Set

The allowed command set for Phase 11 Plan 01 evidence documentation is documentation-only verification:

- `rg` checks named in the plan.
- Python body-separator checks named in the plan.
- `just parity` to confirm documentation does not introduce invalid checklist claims.
- `git diff --check` scoped to the Phase 11 evidence files.

Later live evidence tasks may use only the non-destructive repo-owned commands listed in the Recovery Protocol after the Hardware Gate succeeds. No live command is allowed to infer active control parity from an observe-only log.

## Prohibited Commands Unless A Later Task Adds Bounded Recovery

The following blocked surfaces are prohibited unless a later task adds bounded recovery:

- active DS4432U voltage writes
- fan duty actuation
- overheat stimulus
- fault injection
- self-test hardware submodes that touch voltage/fan/ASIC work
- mining stress
- raw I2C writes
- raw flash writes
- erase
- rollback
- interrupted-update procedures

Observe-only evidence cannot verify these active behaviors. A boot log, safe-unavailable status, suppressed effect, wrapper-owned serial capture, unit test, or pure model can support implementation or a narrow observed subclaim, but it cannot prove active voltage, fan, overheat/fault, self-test hardware, mining stress, raw-write, erase, rollback, or interrupted-update parity.

## Surface Evidence Matrix

| Requirement | Checklist Row | Component | Claim Type | Allowed Probe | Required Metadata | Pass Criteria | Failure Criteria | Evidence Artifact | Allowed Evidence Token | Promotion Boundary |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| SAFE-01, SAFE-04, SAFE-07 | PWR-001 | ASIC reset behavior | active ASIC reset and fail-closed reset observation require `hardware-regression`; unavailable or held-low safe status can be an exact observe-only subclaim | Detector-gated `just flash-monitor board=205 port=<port> evidence-dir=docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence` for safe logs only; no reset toggling in this plan | board `205`, selected port, source commit, reference commit, command, package manifest or firmware identity, logs, observed reset status, conclusion, redaction review | Safe log proves reset effects are unavailable, held safe, or explicitly blocked without enabling ASIC work | Missing board gate, missing reset status, reset active without bounded recovery, ASIC work starts, or redaction uncertainty | `safe-baseline/flash-command-evidence.json`, `safe-baseline/flash-monitor.log`, redaction review | `hardware-smoke` only for exact board-named safe-unavailable or held-safe observation; `hardware-regression` required for active reset parity | Broad PWR-001 remains below `verified` until ASIC reset sequencing has bounded hardware-regression evidence |
| SAFE-01, SAFE-04, SAFE-07 | PWR-002 | ASIC power initialization | active power sequencing requires `hardware-regression`; boot safe-state can be exact `hardware-smoke` | Detector-gated wrapper flash-monitor capture only; no ASIC power sequencing or mining start in this plan | board `205`, selected port, source commit, reference commit, exact command, firmware identity, logs, observed power/init gate, conclusion, redaction review | Firmware reports mining disabled, ASIC work submission disabled, hardware control disabled, and no active power initialization | ASIC initialization proceeds without complete power/thermal/safety evidence, missing safe-state log, or unsafe active sequence | `safe-baseline/flash-command-evidence.json`, `safe-baseline/flash-monitor.log`, redaction review | `hardware-smoke` for exact safe startup; `hardware-regression` for active power-init parity | PWR-002 stays below `verified` until active power-init subclaims have targeted regression evidence |
| SAFE-01, SAFE-07 | PWR-003 | Core voltage control | active DS4432U voltage writes require `hardware-regression`; suppressed or pending voltage effect can be observe-only | No voltage write probe in this plan; detector-gated wrapper log may record suppressed or `hardware_evidence_pending` status only | board `205`, selected port, source commit, reference commit, exact command, voltage request/status if logged, logs, observed behavior, conclusion, redaction review | Evidence shows voltage writes are suppressed or unavailable and clearly labeled `hardware evidence pending` | Any voltage write attempted without bounded values, abort conditions, recovery steps, and fail-closed output | `voltage-control/flash-command-evidence.json`, `voltage-control/flash-monitor.log`, redaction review | `hardware-smoke` only for exact suppressed/safe-unavailable subclaim; `hardware-regression` required for active voltage-control parity | PWR-003 must not become `verified` from black-box flash/monitor smoke |
| SAFE-01, SAFE-07 | PWR-005 | DS4432U support | active DS4432U register writes require `hardware-regression`; adapter constants and suppressed writes remain implementation evidence | No raw I2C or DS4432U write command; wrapper logs can cite safe-unavailable voltage-control state if present | board `205`, selected port, source commit, reference commit, exact command, DS4432U status if logged, logs, conclusion, redaction review | Support remains documented as present but not actively exercised; active write parity is `hardware evidence pending` | Raw I2C write, active register write, or voltage-control claim without bounded recovery | `voltage-control/flash-command-evidence.json`, `voltage-control/flash-monitor.log`, redaction review | `hardware-smoke` for safe-unavailable status only; `hardware-regression` for DS4432U active writes | PWR-005 stays below `verified` for active DS4432U behavior until a bounded regression probe exists |
| SAFE-01, SAFE-07 | PWR-006 | INA260 support | read-only telemetry freshness can use `hardware-smoke` for the exact observed board-named claim | Detector-gated wrapper capture plus an existing safe firmware/API/log telemetry route if a later task owns it; no raw I2C reads in Plan 01 | board `205`, selected port, source commit, reference commit, exact command or safe telemetry route, timestamp/freshness status, logs, current/voltage/power observed or safe-unavailable, conclusion, redaction review | Current, bus voltage, and power are either fresh and attributed to board `205` or explicitly safe-unavailable without stale-value overclaim | Missing freshness, cached value treated as fresh, read failure hidden, or telemetry used to verify voltage writes | `power-telemetry/flash-command-evidence.json`, `power-telemetry/flash-monitor.log`, redaction review | `hardware-smoke` for exact read-only telemetry or safe-unavailable board claim | PWR-006 may cite a narrow read-only subclaim; it does not verify DS4432U writes or power sequencing |
| SAFE-02, SAFE-04, SAFE-07 | THR-001 | Thermal model | thermal observation can use `hardware-smoke`; overheat/fault handling requires `hardware-regression` | Detector-gated wrapper log plus existing safe thermal/API/log route if present; no overheat stimulus in this plan | board `205`, selected port, source commit, reference commit, exact command, thermal reading/status, freshness, logs, conclusion, redaction review | Temperature status is fresh or explicitly unavailable, and overheat state is not inferred from synthetic stress | Overheat stimulus, fault injection, hidden invalid sentinel, or broad thermal parity claimed from boot smoke | `thermal-fan/flash-command-evidence.json`, `thermal-fan/flash-monitor.log`, redaction review | `hardware-smoke` for exact read-only thermal observation; `hardware-regression` for overheat/fault handling | THR-001 stays below `verified` for active overheat and failure paths |
| SAFE-02, SAFE-04, SAFE-07 | THR-002 | Fan controller task | RPM observation can use `hardware-smoke`; fan duty effects require `hardware-regression` | Detector-gated wrapper log or safe read-only RPM route; no fan duty actuation in this plan | board `205`, selected port, source commit, reference commit, exact command, RPM/status, duty command if any, logs, conclusion, redaction review | RPM/status is observed or explicitly unavailable without changing fan duty; any visible fan note is bounded and exact | Fan duty command sent without bounded probe, RPM inferred without source, or fan behavior broadly verified from boot log | `thermal-fan/flash-command-evidence.json`, `thermal-fan/flash-monitor.log`, redaction review | `hardware-smoke` for exact RPM/safe-unavailable observation; `hardware-regression` for fan duty effects | THR-002 remains below `verified` for active fan duty behavior |
| SAFE-03 | THR-003 | PID behavior | pure PID logic remains `unit` and must not verify fan hardware | `just test` for pure logic; no hardware probe required for PID math | source commit, test command, fixture identity, result summary | Pure PID constants, clamps, and decisions pass unit tests | Unit tests fail, fixture drift is unexplained, or PID result is used to claim fan hardware behavior | Safety test output or plan summary, not hardware pack output | `unit` | THR-003 can remain unit-evidenced only; it cannot promote live fan hardware |
| SAFE-01, SAFE-02, SAFE-07 | IO-001 | I2C initialization | startup SSD1306/I2C observation can use `hardware-smoke`; shared safety peripherals need exact read-only or regression evidence | Detector-gated wrapper capture; no raw I2C writes or ad hoc bus scans in this plan | board `205`, selected port, source commit, reference commit, exact command, I2C/peripheral log markers, logs, conclusion, redaction review | Evidence names the exact observed bus/peripheral status and does not infer DS4432U/INA260/EMC2101 active behavior | Raw bus writes, ambiguous peripheral presence, or startup display evidence used for all safety peripherals | `safe-baseline/flash-command-evidence.json`, `display-input/flash-monitor.log`, redaction review | `hardware-smoke` for exact startup/display or read-only status; `hardware-regression` for active safety peripheral behavior | IO-001 stays narrow; shared I2C safety claims require their own evidence |
| SAFE-06 | UI-001 | Display behavior | startup SSD1306 observation can use `hardware-smoke`; runtime display behavior requires `hardware-regression` | Detector-gated wrapper capture and physical observation only for startup display; no runtime display/input route in Plan 01 | board `205`, selected port, source commit, reference commit, exact command/probe, display observation, logs, conclusion, redaction review | Startup display observation is recorded as startup-only and runtime display/input remains `hardware evidence pending` | Startup OLED evidence is used to verify runtime display pages, LVGL parity, or input behavior | `display-input/flash-command-evidence.json`, `display-input/flash-monitor.log`, redaction review | `hardware-smoke` for startup-only observation; `hardware-regression` for runtime display parity | UI-001 broad runtime display remains below `verified` with owner/follow-up |
| SAFE-06 | UI-002 | Screen rendering flow | startup screen observation can support only startup subclaim; runtime screen flow requires `hardware-regression` | Detector-gated wrapper capture and startup-only display notes; no runtime screen carousel probe in this plan | board `205`, selected port, source commit, reference commit, exact command/probe, screen observation, logs, conclusion, redaction review | Evidence states startup-only four-line/debug screen scope and names runtime screen flow as pending | Runtime screen/page parity claimed without a real runtime route and physical observation | `display-input/flash-command-evidence.json`, `display-input/flash-monitor.log`, redaction review | `hardware-smoke` for startup-only subclaim; `hardware-regression` for runtime flow | UI-002 remains below `verified` for runtime screen flow |
| SAFE-06 | UI-003 | Input behavior | runtime input hardware behavior requires `hardware-regression` | No input actuation in Plan 01; wrapper logs may record `display_input_status=runtime_gap reason=hardware_evidence_pending startup_only=true` | board `205`, selected port, source commit, reference commit, exact command, input/display gap log, conclusion, redaction review | Runtime input remains explicitly pending with owner/follow-up and no stale Phase 6 overclaim | Input parity claimed from startup display or without physical/runtime input route | `display-input/flash-command-evidence.json`, `display-input/flash-monitor.log`, redaction review | No promotion token from Plan 01; `hardware-regression` required for runtime input parity | UI-003 stays below `verified` until a bounded input route is exercised |
| SAFE-05, SAFE-09 | SELF-001 | Self-test lifecycle | pure lifecycle remains unit/workflow; self-test hardware submodes require `hardware-regression` | No self-test hardware submode in Plan 01; wrapper logs may record safe-unavailable or blocked self-test status only | board `205`, selected port, source commit, reference commit, exact command, self-test state/status, logs, conclusion, redaction review | Self-test hardware submodes that touch voltage/fan/ASIC work are recorded `hardware evidence pending` unless later bounded recovery exists | Any self-test starts hardware work, fan duty, voltage, ASIC work, reboot, or failure path without recovery | `self-test-watchdog-load/flash-command-evidence.json`, `self-test-watchdog-load/flash-monitor.log`, redaction review | `hardware-smoke` for exact safe-unavailable status only; `hardware-regression` for hardware submodes | SELF-001 broad hardware lifecycle remains below `verified` |
| SAFE-07, SAFE-09 | STAT-002 | Statistics task | safe-unavailable statistics status can use `hardware-smoke`; live producer and sensor values need exact evidence | Detector-gated wrapper capture or existing safe log/API status route if present; no live mining or stress in Plan 01 | board `205`, selected port, source commit, reference commit, exact command/probe, statistics/safety telemetry status, logs, conclusion, redaction review | Statistics reports safe zero/unavailable or exact observed live values without pretending mining/load evidence exists | Live sensor or load responsiveness claimed without route, freshness, or bounded workload | `self-test-watchdog-load/flash-command-evidence.json`, `power-telemetry/flash-command-evidence.json`, redaction review | `hardware-smoke` for exact safe-unavailable or read-only status; `hardware-regression` for load/stress producer behavior | STAT-002 stays below `verified` for live history and load behavior |
| SAFE-01, SAFE-02, SAFE-04, SAFE-07 | API-002 | System info response | safe-unavailable API/log status can use `hardware-smoke` for exact observed board-named claim | Wrapper-owned serial log for API route shell and safety status only; no direct HTTP probe in Plan 01 | board `205`, selected port, source commit, reference commit, exact command, firmware identity, safety telemetry projection if captured, logs, conclusion, redaction review | System info safety status is explicit, safe-unavailable or exact observed telemetry, and redacted | Live HTTP/system info parity claimed without a safe route, detector gate, redaction review, or source/reference commits | `safe-baseline/flash-command-evidence.json`, `flash-monitor.log`, redaction review | `hardware-smoke` for exact safe-unavailable API/log status; no active-control promotion | API-002 can cite narrow safety status notes without verifying broad safety hardware rows |
| SAFE-01, SAFE-02, SAFE-04, SAFE-07, SAFE-09 | API-006 | Live WebSocket telemetry | safe-unavailable WebSocket/log status can use `hardware-smoke`; live telemetry cadence and hardware values need exact evidence | Wrapper-owned serial log only in Plan 01; no WebSocket client probe unless a later task adds a redacted safe procedure | board `205`, selected port, source commit, reference commit, exact command/probe, telemetry/log status, logs or frames if later captured, conclusion, redaction review | Captured status is exact, redacted, and does not claim live hardware telemetry beyond what was observed | WebSocket parity, cadence, or live sensor values claimed without captured frames, freshness, and redaction review | `safe-baseline/flash-command-evidence.json`, optional later redacted frame artifact, redaction review | `hardware-smoke` for exact safe-unavailable/log status; `hardware-regression` for bounded load/telemetry responsiveness | API-006 broad live telemetry remains below verified hardware claims until safe live evidence exists |
| EVD-05 | EVD-05 | Evidence governance | evidence-layer contract and promotion guard | `just parity`, required `rg` checks, Python body-separator checks, and scoped `git diff --check`; live evidence later starts with detector gate | source commit, reference commit when hardware is involved, command list, artifact paths, redaction review, conclusion | Documentation names exact evidence tiers and `just parity` reports no invalid checklist claims | Missing row, body separator, unsafe command authorization, or checklist overclaim | This ledger, artifact README, redaction review, and plan summary | `unit`, `workflow`, `api-compare`, `hardware-smoke`, or `hardware-regression` only when exact claim supports it | EVD-05 governs evidence, but does not itself verify any active safety behavior |

## Component Evidence Packs

All component evidence packs live under `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/`. The pack README defines the expected artifact names and conclusion rules.

- `safe-baseline`: detector gate, board-info result, boot identity, safe-state serial markers, package/firmware identity, source/reference commits, and redaction review.
- `power-telemetry`: INA260 read-only current, bus voltage, power status, freshness, read success/failure, and safe-unavailable status.
- `voltage-control`: DS4432U voltage-control evidence. Active write behavior remains `hardware evidence pending` until bounded values, stop conditions, recovery, and fail-closed output exist.
- `thermal-fan`: EMC2101 temperature and fan RPM observations first; fan duty actuation remains pending until bounded recovery exists.
- `self-test-watchdog-load`: self-test safe status, watchdog supervisor yield/liveness, and bounded responsiveness evidence. Hardware self-test, fault injection, and mining stress remain pending unless later bounded recovery exists.
- `display-input`: startup display observations and explicit runtime display/input gap tracking.
- `parity-guard`: `just parity` output and notes proving safety-critical overclaims are rejected.

## Execution Log

| Step | Status | Evidence |
| --- | --- | --- |
| Phase 11 Plan 01 hardware run | not run | This plan is documentation-only and does not run live hardware commands. |
| Detector gate | passed | `just detect-ultra205` found exactly one likely Ultra 205 port; selected port: /dev/cu.usbmodem1101. Board-info command: `espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive`. Board-info output recorded ESP32-S3 revision v0.2, 40 MHz crystal, 16MB flash, WiFi/BLE/embedded flash features, MAC address `f0:f5:bd:4a:ab:cc`, secure boot disabled, and flash encryption disabled. |
| Wrapper-owned flash-monitor capture | passed | `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence` generated `flash-command-evidence.json` and `flash-monitor.log`. JSON records board `205`, selected port `/dev/cu.usbmodem1101`, firmware commit `048e765e854d24ac87b711c1dfe92bcaa2d4c085`, reference commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`, `capture_status=timed_out_after_trusted_output`, `trusted_output=true`, observed firmware commit `048e765e854d`, observed reference commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`, and conclusion `passed - wrapper-owned serial boot evidence captured; HTTP/static/recovery/OTA/rollback parity not claimed`. |
| Observed safe boot markers | passed | `flash-monitor.log` records `bitaxe-rust boot: board=Ultra 205 asic=BM1366`, `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`, `display_input_status=runtime_gap reason=hardware_evidence_pending startup_only=true`, `ota_boot_validation=not_pending state=factory`, `safety_supervisor=started thread=bitaxe-safety-supervisor cadence_ms=100`, and `safety_supervisor_step=yield reason=yield_interval_reached`. |
| Active voltage, fan, overheat, fault, self-test hardware, mining stress, raw write, erase, rollback, interrupted-update probes | prohibited | `hardware evidence pending` until a later plan documents bounded recovery. |

## Checklist Promotion Rules

Broad mixed rows stay below `verified` unless every active subclaim has matching evidence. A row that combines pure logic, read-only telemetry, safe-unavailable status, active control, and failure handling cannot be verified by one narrow boot or smoke observation.

Narrow observed subclaims can be cited in checklist notes without verifying the whole row. For example, a board-named safe-unavailable voltage status can support a PWR-003 note, but it does not verify active DS4432U voltage writes. Startup SSD1306 evidence can support startup display observations, but it does not verify runtime display, screen flow, LVGL, or input parity.

`hardware-smoke` can verify only the exact observed board-named claim: safe startup, safe-unavailable API/log/WebSocket status, read-only INA260 telemetry, read-only thermal status, fan RPM observation, startup display observation, or another explicitly bounded non-actuating observation.

`hardware-regression` is required for active DS4432U voltage writes, fan duty effects, overheat/fault handling, self-test hardware submodes, ASIC reset/power sequencing, runtime input/display hardware behavior, watchdog/load stress, and any failure-path parity.

`tools/parity` remains the source of truth for rejecting unsafe verified rows. If documentation, checklist notes, or an evidence pack appear to permit a safety-critical verified row without matching `hardware-smoke` or `hardware-regression`, the parity guard wins and the row remains below `verified`.

## Secret Redaction Review

Every generated Phase 11 evidence artifact must be reviewed before it is cited. The review must inspect serial logs, JSON evidence, pasted terminal output, API responses, WebSocket frames, and manual observations for:

- Wi-Fi SSIDs and passwords.
- Pool URLs, usernames, and passwords.
- Private endpoints and private URLs.
- NVS secret values.
- API tokens.
- Local private IP disclosure beyond necessary bench evidence.
- Pasted raw terminal secrets.

The redaction template for this plan is `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/redaction-review.md`. When redaction uncertainty remains, stop and record `hardware evidence pending`.

## Residual Risks

- Active DS4432U voltage writes are not verified by this plan.
- Fan duty actuation and physical fan response are not verified by this plan.
- Overheat stimulus, fan faults, ASIC faults, and other failure-path handling are not verified by this plan.
- Self-test hardware submodes that touch voltage, fan, or ASIC work are not verified by this plan.
- Runtime display/input parity is not verified by startup SSD1306 evidence.
- Watchdog/load responsiveness under bounded workload is not verified by this plan.
- API/WebSocket live safety telemetry can be cited only for exact redacted observations from a safe route; Plan 01 does not run those probes.
- Phase 11 produced wrapper-owned safe boot artifacts, but read-only INA260 telemetry, EMC2101 telemetry, fan RPM, API safety telemetry, WebSocket safety telemetry, self-test hardware submodes, active voltage control, fan duty, fault paths, and watchdog/load stress remain `hardware evidence pending` unless specifically observed in later bounded evidence.

## Conclusion

Phase 11 now has a board-205-only evidence ledger, runbook, recovery protocol, surface matrix, component evidence-pack contract, redaction gate, checklist promotion rules, and detector-gated wrapper-owned safe boot evidence.

Conclusion: passed for detector-gated Ultra 205 safe boot evidence, trusted wrapper capture, safe-state boot marker, runtime display/input gap marker, factory OTA boot-validation marker, and watchdog supervisor startup/yield marker. Active safety-control, stress, destructive, fault-injection, self-test hardware, raw-write, erase, rollback, interrupted-update, runtime input/display parity, and mining stress claims remain `hardware evidence pending`.

## Final Verification

- `cargo test -p bitaxe-safety --all-features` - passed, 32 unit tests plus doc tests.
- `cargo test -p bitaxe-parity --all-features` - passed, 49 tests.
- `just parity` - passed with `validation_errors: none`.
- `just test` - passed, 13 Bazel test targets.
- `git diff -- reference/esp-miner --exit-code` - passed; reference tree remained read-only.
- `git diff --check` for Phase 11 touched scopes - passed.
