---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 3-2026-06-26T22-40-40
generated_at: 2026-06-26T22:51:59.597Z
---

# Phase 3: BM1366 ASIC Protocol And Safe Initialization - Context

**Gathered:** 2026-06-26
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 3 delivers typed BM1366 protocol logic, a narrow UART adapter boundary, and staged Ultra 205 initialization decisions that fail closed. The phase includes CRC, packet framing, register commands, work encoding, result parsing, nonce/domain/error fixtures, semantic command and observation types, fake UART transcript tests, chip-detect-first hardware smoke planning, parity checklist updates, and explicit evidence boundaries for live ASIC behavior.

This phase does not deliver the Stratum v1 mining loop, pool connection, accepted-share evidence, production work dispatch, broad safety controllers, API telemetry, OTA, or non-205 ASIC verification. Runtime work submission remains disabled until Phase 4. ASIC initialization, frequency changes, voltage effects, reset behavior, UART timing, and chip detection cannot be marked `verified` without Ultra 205 hardware evidence.

</domain>

<decisions>
## Implementation Decisions

### Pure BM1366 Protocol Surface

- **D-01:** Implement the full pure BM1366 protocol surface in `crates/bitaxe-asic`, not only CRC/register stubs. The pure crate should cover CRC5, CRC16-FALSE, command packet framing, job packet framing, register command payloads, BM1366 work encoding, result parsing, nonce/domain decoding, invalid-job rejection, and protocol faults.
- **D-02:** Keep pure BM1366 work and result modeling separate from live mining. Fixed-size work payload encoding and result parsing are in scope, but runtime `SendWork` behavior must remain disabled or explicitly diagnostic until Phase 4 owns the first mining loop.
- **D-03:** Use upstream facts as testable constants: command packets use CRC5, job packets use CRC16-FALSE, result frames are 11 bytes, BM1366 job IDs advance by 8 and mask with `0xf8`, nonce-derived ASIC/core decoding follows `BM1366_process_work`, and BM1366 has 112 normal cores plus 8 small-core IDs per job-id low bits.
- **D-04:** Treat frequency, voltage, nonce-space, difficulty mask, PLL, and register-write values as typed data decisions and init-plan commands. Unit/golden tests may prove pure output bytes, but hardware effects stay below `verified` until board-named smoke or regression evidence exists.

### UART Adapter Boundary

- **D-05:** Model semantic command and observation types in `bitaxe-asic`, for example `Bm1366Command::{SetVersionMask, ReadChipId, SetChipAddress, WriteRegister, SetFrequency, SetNonceSpace, SendWork}` and `Bm1366Observation::{ChipId, RegisterRead, JobNonce, ProtocolFault}`.
- **D-06:** Keep frame bytes, preamble `0x55 0xAA`, length fields, CRCs, and raw register details inside the ASIC crate. Firmware and user-facing control logic should not construct raw BM1366 byte packets directly.
- **D-07:** Firmware owns ESP-IDF effects: UART1 pins/driver setup, initial 115200 baud, exact read/write timeouts, waiting for TX before baud changes, RX buffer clearing, reset GPIO, delays, logging, and visible status updates.
- **D-08:** Add a fake UART transcript seam that can inject exact reads, timeouts, partial reads, malformed preambles, bad CRCs, unknown registers, and chip-count mismatches. Tests should assert emitted semantic actions, emitted frame bytes, decoded observations, and fail-closed status.

### Staged Safe Initialization

- **D-09:** Use a staged `Bm1366InitPlan` or equivalent pure state machine with stages for preflight, reset, UART default baud, chip detect, register init, frequency/nonce setup, max baud, and initialized-no-mining state.
- **D-10:** Start live Ultra 205 evidence with a chip-detect-first staged gate. Full staged init may follow only after required board, config, power, thermal, and safety preflight tokens exist and are evidenced.
- **D-11:** Reuse Phase 2 config/catalog facts as gate inputs: board version `205`, `VerificationScope::ActiveUltra205`, family `Ultra`, ASIC model `BM1366`, expected ASIC count `1`, supported frequency/voltage options, DS4432U and INA260 capabilities, and deferred non-205 scopes.
- **D-12:** Fail closed on every missing gate or stage error: no mining, no production work submission, no false initialized status, reset held low or ASIC enable disabled where that is the safe adapter action, and a visible `asic_status` or boot log reason.
- **D-13:** Treat pre-init thermal gating conservatively. Upstream chip temperature is not meaningful before ASIC init, so live init should either require lower-level thermal/fan/power evidence or block with a typed preflight-missing status.

### Evidence And Provenance Strategy

- **D-14:** Use a boundary-split fixture/evidence matrix. Pure protocol rows can advance with unit/golden fixtures; mixed live UART/reset/init rows remain `implemented` or `in-progress` until Ultra 205 hardware evidence exists.
- **D-15:** BM1366 fixtures should carry metadata fields such as source file, source function or behavior, pinned reference commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`, license posture, generated/manual derivation note, and checklist IDs.
- **D-16:** Prefer hand-authored metadata fixtures for early cases, and introduce a generated reference fixture harness only if manual byte derivation becomes numerous or error-prone. Generated output remains GPL-risk source data and must be labeled accordingly.
- **D-17:** Hardware evidence files for init or chip detection must record board, port, command, firmware commit, reference commit, relevant logs, observed result, fail-closed conclusion, and any skipped hardware gate.
- **D-18:** Do not copy upstream C expression into MIT-only files. Use module or behavior breadcrumbs to reference the pinned C implementation, and isolate any intentionally ported GPL-derived expression or fixture data with conservative license notes.

### the agent's Discretion

The agent may choose exact module names, Rust type names, fixture file formats, state-machine representation, fake UART transcript schema, and plan count. Those choices must preserve functional core plus imperative shell, keep runtime work submission disabled, satisfy ASIC-01 through ASIC-08, and avoid marking safety-critical hardware behavior `verified` without hardware evidence.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Requirements

- `.planning/ROADMAP.md` - Phase 3 goal, success criteria, verification expectations, and research flags.
- `.planning/REQUIREMENTS.md` - ASIC-01 through ASIC-08 plus safety and evidence requirements.
- `.planning/PROJECT.md` - Ultra 205 BM1366 first target, architecture constraints, provenance constraints, and seed layout.
- `.planning/STATE.md` - Completed Phase 1 and 2 decisions, current Phase 3 focus, and hardware evidence blockers.
- `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md` - Safe boot/log boundary and disabled mining/hardware-control decisions.
- `.planning/phases/02-ultra-205-config-and-nvs-model/02-CONTEXT.md` - Ultra 205 catalog, NVS/config decisions, and hardware effect deferrals.

### Upstream BM1366 Reference Files

- `reference/esp-miner/config-205.cvs` - Ultra 205 BM1366 default frequency, voltage, model, and board identity.
- `reference/esp-miner/main/device_config.h` - Ultra 205 board catalog, BM1366 profile values, capabilities, and expected ASIC count.
- `reference/esp-miner/components/asic/asic.c` - ASIC dispatch by model and shared calls for init, work, frequency, nonce space, and register reads.
- `reference/esp-miner/components/asic/bm1366.c` - BM1366 packet framing, initialization sequence, work encoding, result parsing, register reads, baud changes, and nonce decoding.
- `reference/esp-miner/components/asic/include/bm1366.h` - BM1366 job struct layout and public function boundary.
- `reference/esp-miner/components/asic/crc.c` and `reference/esp-miner/components/asic/include/crc.h` - CRC5 and CRC16-FALSE behavior.
- `reference/esp-miner/components/asic/asic_common.c` and `reference/esp-miner/components/asic/include/asic_common.h` - chip-counting, receive-work validation, difficulty mask, timeout, and shared result types.
- `reference/esp-miner/components/asic/include/serial.h` - Upstream serial boundary and UART defaults.
- `reference/esp-miner/main/power/asic_init.c` - Reset/UART/chip-detect/max-baud initialization shell.
- `reference/esp-miner/main/power/asic_reset.c` - ASIC reset GPIO timing behavior.
- `reference/esp-miner/main/work_queue.c` and `reference/esp-miner/main/work_queue.h` - Upstream queue behavior relevant to later work dispatch, but not production mining in this phase.

### Existing Rust Integration Points

- `crates/bitaxe-asic/src/lib.rs` - Current deferred ASIC placeholder to replace with Phase 3 pure protocol modules.
- `crates/bitaxe-asic/BUILD.bazel` - Bazel target that must expose new source files and tests.
- `crates/bitaxe-config/src/catalog.rs` - Ultra 205/BM1366 catalog, capabilities, and verification scope.
- `crates/bitaxe-config/src/defaults.rs` - Ultra 205 defaults including BM1366 model, 485 MHz, and 1200 mV.
- `crates/bitaxe-config/src/validation.rs` - Typed frequency/voltage validation to reuse or mirror at gate boundaries.
- `crates/bitaxe-core/src/lib.rs` - Safe-state and board/ASIC identity types.
- `firmware/bitaxe/src/main.rs` - Current safe boot/log firmware shell and disabled mining/work/control status.
- `docs/parity/checklist.md` - ASIC rows ASIC-001 through ASIC-008 and evidence status rules.

### Architecture, Evidence, And Policy

- `docs/adr/0001-device-user-parity.md` - Observable parity definition.
- `docs/adr/0003-esp-idf-rust-production-stack.md` - ESP-IDF Rust stack and adapter boundary.
- `docs/adr/0005-read-only-reference-implementation.md` - Reference implementation policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist evidence policy.
- `docs/adr/0008-reference-breadcrumb-comments.md` - Breadcrumb placement policy.
- `docs/adr/0009-monorepo-package-layout.md` - Crate and firmware path ownership.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence requirements and safety-critical hardware gate.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - GPL guardrails for upstream-derived behavior and fixtures.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205/BM1366 first parity target and deferred Gamma 601 scope.
- `PROVENANCE.md` - Provenance, SPDX, reference usage, and release review policy.
- `docs/parity/evidence/ultra-205-pivot-safe-state-smoke-2026-06-26.md` - Prior Ultra 205 safe-state hardware evidence; does not verify ASIC init.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `crates/bitaxe-config` already models Ultra 205/BM1366 catalog data, supported frequency/voltage options, active verification scope, and typed validation for hardware-sensitive config values.
- `crates/bitaxe-core` already exposes Ultra 205/BM1366 identity and the safe-state log contract that must remain true until Phase 3 intentionally adds guarded init status.
- Phase 2 fixture patterns in `crates/bitaxe-config/fixtures/*` provide a model for metadata-rich, reference-derived fixture data.
- `docs/parity/checklist.md` already contains ASIC rows for dispatch, BM1366 init, work send, result parsing, serial transport, CRC, frequency transition, and deferred BM1370.

### Established Patterns

- Pure domain logic belongs in `crates/*`; ESP-IDF, UART, GPIO, delays, and hardware side effects belong in `firmware/bitaxe`.
- Tests use Arrange, Act, Assert comments and one concern per unit test.
- Rust multi-file modules should use `foo.rs` plus `foo/` rather than `foo/mod.rs`.
- Option-bearing internal names should use `maybe_` naming unless a public or trait contract forces otherwise.
- Reference breadcrumbs should sit at module or behavior boundaries, not on every line.

### Integration Points

- `crates/bitaxe-asic` should become the owner of BM1366 CRCs, packet codecs, command/observation types, init planning, work encoding, result parsing, register mapping, fixtures, and fake UART transcript logic.
- `firmware/bitaxe` should gain only narrow adapter logic for UART/reset/status as the plan demands, and must keep mining/work submission disabled unless a plan explicitly adds diagnostic-only, fail-closed hardware smoke.
- `docs/parity/checklist.md` and `docs/parity/evidence/` should record pure fixture evidence separately from live hardware evidence so statuses do not overclaim safety-critical behavior.

</code_context>

<specifics>
## Specific Ideas

- Preferred path: full pure codec plus gated adapter, not protocol-only and not direct upstream C sequence behind a flag.
- Preferred init path: chip-detect-first staged gate before full staged init; full init depends on explicit power/thermal/preflight evidence.
- Preferred UART seam: semantic command/observation state machine emits adapter actions; firmware interprets them.
- Preferred evidence strategy: boundary-split fixtures and hardware evidence matrix; generated fixture extraction only if hand-derived byte fixtures become too risky.
- Runtime state should include an initialized-no-mining or equivalent status so Phase 3 can prove ASIC communication without enabling production mining.

</specifics>

<deferred>
## Deferred Ideas

- Stratum v1 socket behavior, pool authorization, mining loop integration, accepted-share evidence, and production work dispatch belong to Phase 4.
- Full safety controller behavior for voltage, power, fan, thermal, PID, faults, and self-test belongs to Phase 6.
- API exposure of ASIC status, telemetry, and logs belongs to Phase 5.
- Non-205 boards, BM1370, BM1368, BM1397, and all-board ASIC verification remain unverified or deferred until each has its own evidence set.
- A generated reference-fixture extraction tool may be added later if fixture volume justifies it.

</deferred>

---

*Phase: 03-bm1366-asic-protocol-and-safe-initialization*
*Context gathered: 2026-06-26*
