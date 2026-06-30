# Phase 12 ASIC And Mining Hardware Evidence

## Scope

Phase 12 records Ultra 205 board `205` evidence for BM1366 ASIC initialization boundaries and first mining-loop behavior. The phase can support only the exact claims proved by detector-gated commands, generated artifacts, redaction review, and checklist promotion checks.

In scope:

- Ultra 205 BM1366 detector, package identity, and safe boot evidence.
- Chip-detect and staged initialization evidence when the diagnostic gate is explicitly enabled.
- Diagnostic work-send/result-receive evidence only through typed, bounded repo-owned probes.
- Controlled mining smoke or bounded soak only after detector, safety, chip-detect, recovery, and redaction prerequisites exist.
- Checklist promotion guarded by `just parity`.

Out of scope:

- Non-205 boards or other ASIC families.
- Stratum v2, BAP, long-running tuning, or production pool optimization.
- Final release HTTP, OTA, recovery, rollback, erase, and interrupted-update evidence.
- Voltage, fan, thermal, raw ASIC writes, or mining stress outside a phase-approved recovery procedure.

## Hardware Gate

Every live Phase 12 hardware attempt starts with `just detect-ultra205`.

Detection is successful only when exactly one likely ESP USB serial port is found and the detector's board-info command succeeds:

```bash
espflash board-info --chip esp32s3 --port <port> --non-interactive
```

If detection succeeds, record board `205`, selected port, source commit, reference commit, exact command, board-info output, and detector output before any flash, monitor, chip-detect, probe, smoke, or soak command.

Stop and record pending evidence when any of these cases occur:

- Zero likely ESP USB serial ports.
- Multiple likely ESP USB serial ports.
- Board-info failure.
- Target other than board `205`.
- Missing package identity or firmware/source commit identity.
- Missing recovery path for the requested hardware behavior.
- Missing safety evidence required by the requested tier.
- Missing controlled pool or fake-pool instructions for mining smoke.
- Redaction uncertainty for generated logs or command output.

## Recovery And Stop Conditions

The default recovery action for Phase 12 is to stop at the current tier, keep mining/work submission disabled, preserve generated logs for redaction review, and record the conclusion as pending or blocked.

Stop immediately when:

- Any command targets a board other than `205`.
- Any command would use raw serial writes, direct `espflash` outside the repo wrapper, direct `esptool.py`, erase, rollback, voltage/fan stress, direct pool scripts, or unbounded mining stress.
- Firmware logs show missing safety, power, thermal, ASIC, or hardware-evidence gates.
- Logs contain unredacted pool credentials, worker secrets, Wi-Fi credentials, private endpoints, NVS secret values, API tokens, or pasted raw terminal secrets.
- A smoke or soak would run without duration bounds, stop conditions, watchdog/telemetry observations, and safe-stop instructions.

Allowed conclusion strings:

- `passed for detector/package/safe boot`
- `passed for chip-detect smoke`
- `passed for diagnostic work/result smoke`
- `passed for bounded mining smoke`
- `controlled no-share condition`
- `blocked by detector gate`
- `blocked by recovery prerequisite`
- `blocked by redaction review`
- `hardware evidence pending`

## Allowed Command Set

Allowed repo-owned commands before a later plan adds a bounded probe:

```bash
just detect-ultra205
just package
just flash-monitor board=205 port=<path> evidence-dir=docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence
just monitor port=<path>
just parity
just test
```

A plan-approved Phase 12 probe command may be used only after the active plan defines its command line, prerequisites, stop conditions, generated artifacts, and redaction procedure.

## Prohibited Commands Unless A Later Task Adds Bounded Recovery

The following are prohibited in Phase 12 unless a later task adds bounded recovery and explicit evidence instructions:

- Raw serial writes.
- Direct `espflash` outside the repo wrapper.
- Direct `esptool.py`.
- Direct pool scripts.
- Erase, rollback, interrupted-update, or raw flash writes.
- Voltage or fan stress.
- Unbounded mining stress.
- Any command that commits pool credentials, worker secrets, Wi-Fi credentials, private endpoints, NVS secrets, API tokens, or raw terminal secrets.

## Evidence Ladder

| Tier | Prerequisites | Pass Criteria | Failure Criteria | Allowed Conclusion Strings | Supported Checklist Rows |
| --- | --- | --- | --- | --- | --- |
| 0 detector/package/safe boot | `just detect-ultra205`, package identity, board `205`, source commit, reference commit, redaction scope | Detector finds one ESP32-S3 port, board-info succeeds, wrapper capture is trusted, safe-state logs show mining/work/control disabled | No port, multiple ports, board-info failure, missing identity, untrusted output, or redaction uncertainty | `passed for detector/package/safe boot`, `blocked by detector gate`, `hardware evidence pending` | WF-004, WF-005, SYS-003, API-002, API-006, STAT-002 as scoped safe-state evidence only |
| 1 BM1366 chip-detect and staged initialization | Tier 0, documented chip-detect diagnostic command, no-mining scope, safety gate still fail-closed | Logs show chip-detect status and no production mining or work submission; conclusion separates chip-detect from full init | UART timeout, chip-count mismatch, setup fault, missing safety/power/thermal evidence, or no chip-detect marker | `passed for chip-detect smoke`, `blocked by recovery prerequisite`, `hardware evidence pending` | ASIC-002 and ASIC-005 only for the exact observed chip-detect/no-mining boundary |
| 2 diagnostic work-send/result-receive | Tier 1, typed bounded diagnostic probe, expected BM1366 work/result observation, timeout behavior, no pool secrets | Diagnostic work dispatch and result/timeout behavior are observed through repo-owned typed probe artifacts | Probe absent, result timeout without expected fail-closed status, malformed result, invalid job, or unsupported raw write path | `passed for diagnostic work/result smoke`, `blocked by recovery prerequisite`, `hardware evidence pending` | ASIC-003 and ASIC-004 for diagnostic work/result only |
| 3 controlled mining smoke | Tiers 0-2, safety evidence, controlled fake-pool or redacted pool procedure, watchdog/status observations, safe stop | Pool lifecycle, job construction, work dispatch decision, share/no-share outcome, telemetry/status, watchdog responsiveness, and safe-stop are recorded | Missing pool control, missing ASIC gate, missing safety gate, missing telemetry/watchdog, leaked secret, or unsafe stop path | `passed for bounded mining smoke`, `controlled no-share condition`, `blocked by recovery prerequisite`, `hardware evidence pending` | STR-006, STR-007, STR-008, API-002, API-006, STAT-002 for exact observed scope |
| 4 bounded mining soak | Tier 3, duration, stop conditions, thermal/power/watchdog observations, reconnect/fallback scope when exercised | Bounded run completes or stops safely with accepted/rejected share or controlled no-share evidence and post-run safe state | Unbounded run, unsafe temperature/power/watchdog status, serial silence, reconnect loop without stop, leaked secret, or missing safe-stop | `passed for bounded mining smoke`, `controlled no-share condition`, `blocked by recovery prerequisite`, `hardware evidence pending` | STR-006, STR-007, STR-008, API-002, API-006, STAT-002 when exact soak metadata exists |
| 5 checklist and parity promotion | Relevant tier evidence, redaction review complete, checklist notes updated, `just parity` passes | Rows are promoted only when evidence token and notes match exact claim; unsupported claims remain below `verified` | `just parity` fails, row overclaims evidence, missing redaction review, or claim exceeds observed tier | `hardware evidence pending`, `passed for chip-detect smoke`, `passed for bounded mining smoke`, `controlled no-share condition` | ASIC-002, ASIC-003, ASIC-004, ASIC-005, ASIC-007, STR-006, STR-007, STR-008, API-002, API-006, STAT-002, EVD-05 |

## Claim Matrix

| Requirement | Checklist Row | Claim | Prerequisites | Allowed Probe | Required Metadata | Pass Criteria | Failure Criteria | Allowed Evidence Token | Promotion Boundary |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ASIC-07 | ASIC-002 | BM1366 initialization evidence | Tier 0 plus safety/power/thermal gate status | Chip-detect diagnostic or future staged-init probe | board, port, commits, command, board-info, firmware identity, logs, redaction, conclusion | Chip-detect or staged-init marker is observed with no mining/work submission | Missing gate, UART/setup fault, chip-count mismatch, or no marker | hardware-smoke or hardware-regression | Do not verify full init from safe boot or pure tests |
| ASIC-07 | ASIC-003 | BM1366 work send evidence | Tier 1 plus typed diagnostic work probe | Plan-approved diagnostic work probe | board, port, commits, command, work id, expected observation, actual observation, redaction, conclusion | Typed work dispatch observed or controlled no-dispatch reason is recorded | Probe absent, raw write path, timeout without fail-closed status | hardware-smoke or hardware-regression | Diagnostic evidence does not prove production mining work |
| ASIC-07 | ASIC-004 | BM1366 result receive evidence | Tier 2 plus valid-job/result observation | Plan-approved diagnostic result probe | board, port, commits, command, job id, result/share/no-share outcome, redaction, conclusion | Result, timeout, or controlled no-result behavior matches typed expectation | Invalid job, malformed result, unbounded wait, or missing observation | hardware-smoke or hardware-regression | Do not verify live result parsing from golden fixtures alone |
| ASIC-07 | ASIC-005 | ASIC serial transport evidence | Tier 0 plus chip-detect UART evidence | Chip-detect diagnostic | board, port, commits, command, UART status markers, redaction, conclusion | Serial path participates in chip-detect/no-mining diagnostic | UART unavailable, timeout, unsupported raw serial path | hardware-smoke or hardware-regression | Serial transport verification stays scoped to observed diagnostic |
| ASIC-07 | ASIC-007 | Frequency transition behavior | Tier 1 plus explicit frequency transition recovery plan | None until a later plan adds bounded recovery | board, port, commits, command, frequency command, safety state, redaction, conclusion | Bounded frequency transition observed with safe stop | Missing recovery path or unsafe control risk | hardware-regression | No promotion in Phase 12 unless later task adds bounded recovery |
| STR-06 | STR-006 | First Ultra 205 mining loop evidence | Tiers 0-3, safety and ASIC gates, controlled pool/fake-pool condition | Controlled mining smoke or fail-closed preflight hook | board, port, commits, command, pool category, work state, telemetry/status, watchdog, redaction, conclusion | Loop connects config, Stratum, ASIC dispatch/result state, global state, and safety gates or records safe-blocked proof | Missing ASIC gate, missing safety gate, missing pool control, or leaked secret | hardware-smoke or soak | Fake-pool/unit tests do not verify live mining |
| STR-07 | STR-007 | Mining smoke and soak criteria | Evidence ladder and redaction contract | Runbook, preflight hook, or bounded mining smoke/soak | command, board, port, commits, logs, result, duration for soak, redaction, conclusion | Criteria are documented and applied to evidence | Criteria missing required metadata or stop conditions | workflow for criteria; hardware-smoke or soak for live proof | Criteria documentation alone stays below live mining proof |
| STR-07 | STR-008 | Live mining smoke and soak evidence | Tier 3 or 4 | Controlled mining smoke or bounded soak | board, port, commits, command, accepted/rejected share or controlled no-share, redaction, conclusion | Accepted/rejected share or controlled no-share condition plus telemetry/watchdog/safe-stop | No share/no-share rationale, no telemetry, no watchdog, leaked secret, unbounded run | hardware-smoke or soak | Verified live mining requires exact smoke/soak details |
| EVD-05 | API-002 | System info reflects observed ASIC/mining status | Tier 0 or higher plus API/status observation | Wrapper logs or later HTTP probe | board, port, commits, command, status source, redaction, conclusion | Status observed through live firmware path and scoped to observed tier | API not queried or only pure DTO evidence exists | hardware-smoke when live, api-compare/unit otherwise | Do not infer live HTTP parity from serial boot only |
| EVD-05 | API-006 | WebSocket telemetry reflects observed ASIC/mining status | Tier 3 or later plus WebSocket observation | Later Phase 13 HTTP/WebSocket probe unless added safely | board, port, commits, command, telemetry source, redaction, conclusion | Live telemetry stream observed with scoped mining/ASIC fields | WebSocket not queried or private endpoint leaked | hardware-smoke when live, api-compare/unit otherwise | Remains pending without live WebSocket evidence |
| EVD-05 | STAT-002 | Statistics task reflects observed mining status | Tier 3 or later plus statistics producer observation | Controlled mining smoke/soak or future status probe | board, port, commits, command, samples, redaction, conclusion | Live statistics sample reflects the observed mining state | No live producer, no sample, or unsafe/missing stop path | hardware-smoke or soak | Safe zero/unavailable fixtures do not prove live statistics |
| EVD-05 | EVD-05 | Verification layers are complete for promoted claims | Relevant tier evidence plus host checks and parity | `just parity`, `just test`, Rust checks, hardware gate | commands, outputs, evidence paths, redaction, conclusion | Unit/golden/API/workflow/hardware layers match promoted rows | Missing layer, failed command, or overclaiming row | workflow plus exact hardware token where needed | Phase validation can pass with pending hardware only when rows remain unpromoted |

## Execution Log

No Phase 12 live command has run yet.

| Time | Command | Result | Evidence Pack | Conclusion |
| --- | --- | --- | --- | --- |
| 2026-06-30 | plan setup only | pending | all Phase 12 packs | hardware evidence pending |

## Checklist Promotion Rules

- Broad ASIC and mining rows stay below `verified` unless evidence covers the exact claim.
- Live work-send, result-receive, mining smoke, and soak claims require `hardware-smoke`, `hardware-regression`, or soak evidence as applicable.
- Fake-pool tests, unit tests, golden tests, package evidence, and safe boot logs do not prove live hardware mining.
- `STR-007` criteria documentation can use `workflow`; `STR-008` live smoke/soak needs exact hardware evidence and share/no-share metadata.
- Checklist notes must cite this evidence file when Phase 12 changes row wording.
- `just parity` is the canonical overclaim guard before any checklist or validation sign-off.

## Secret Redaction Review

Initial status: pending - no Phase 12 live artifacts reviewed yet.

Before committing generated artifacts, inspect all logs and JSON for:

- Pool URLs, usernames, passwords, and worker names.
- Wi-Fi SSIDs and passwords.
- Private endpoints and private API URLs.
- NVS secret values.
- API tokens.
- Local private IP disclosure beyond necessary bench evidence.
- Pasted raw terminal secrets.

## Residual Risks

- Phase 12 may end with useful pending evidence if detector, chip-detect, controlled pool, redaction, or recovery prerequisites are missing.
- Chip-detect/no-mining evidence does not prove full BM1366 initialization or production work submission.
- Controlled no-share evidence requires explicit rationale; otherwise it cannot support live mining proof.
- Final release HTTP/OTA/recovery evidence remains Phase 13-owned.

## Conclusion

Current conclusion: hardware evidence pending.

Phase 12 will promote only rows whose exact claims are supported by detector-gated artifacts, redaction review, and passing parity checks.
