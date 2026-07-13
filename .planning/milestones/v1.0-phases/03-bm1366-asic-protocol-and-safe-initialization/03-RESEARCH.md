# Phase 03: bm1366-asic-protocol-and-safe-initialization - Research

**Researched:** 2026-06-26
**Domain:** BM1366 ASIC protocol, ESP-IDF UART adapter, safe Ultra 205 initialization
**Confidence:** HIGH for pure protocol scope, MEDIUM-HIGH for live init planning

<user_constraints>
## User Constraints (from CONTEXT.md)

Source for this entire section: [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]

### Locked Decisions

#### Pure BM1366 Protocol Surface

- **D-01:** Implement the full pure BM1366 protocol surface in `crates/bitaxe-asic`, not only CRC/register stubs. The pure crate should cover CRC5, CRC16-FALSE, command packet framing, job packet framing, register command payloads, BM1366 work encoding, result parsing, nonce/domain decoding, invalid-job rejection, and protocol faults.
- **D-02:** Keep pure BM1366 work and result modeling separate from live mining. Fixed-size work payload encoding and result parsing are in scope, but runtime `SendWork` behavior must remain disabled or explicitly diagnostic until Phase 4 owns the first mining loop.
- **D-03:** Use upstream facts as testable constants: command packets use CRC5, job packets use CRC16-FALSE, result frames are 11 bytes, BM1366 job IDs advance by 8 and mask with `0xf8`, nonce-derived ASIC/core decoding follows `BM1366_process_work`, and BM1366 has 112 normal cores plus 8 small-core IDs per job-id low bits.
- **D-04:** Treat frequency, voltage, nonce-space, difficulty mask, PLL, and register-write values as typed data decisions and init-plan commands. Unit/golden tests may prove pure output bytes, but hardware effects stay below `verified` until board-named smoke or regression evidence exists.

#### UART Adapter Boundary

- **D-05:** Model semantic command and observation types in `bitaxe-asic`, for example `Bm1366Command::{SetVersionMask, ReadChipId, SetChipAddress, WriteRegister, SetFrequency, SetNonceSpace, SendWork}` and `Bm1366Observation::{ChipId, RegisterRead, JobNonce, ProtocolFault}`.
- **D-06:** Keep frame bytes, preamble `0x55 0xAA`, length fields, CRCs, and raw register details inside the ASIC crate. Firmware and user-facing control logic should not construct raw BM1366 byte packets directly.
- **D-07:** Firmware owns ESP-IDF effects: UART1 pins/driver setup, initial 115200 baud, exact read/write timeouts, waiting for TX before baud changes, RX buffer clearing, reset GPIO, delays, logging, and visible status updates.
- **D-08:** Add a fake UART transcript seam that can inject exact reads, timeouts, partial reads, malformed preambles, bad CRCs, unknown registers, and chip-count mismatches. Tests should assert emitted semantic actions, emitted frame bytes, decoded observations, and fail-closed status.

#### Staged Safe Initialization

- **D-09:** Use a staged `Bm1366InitPlan` or equivalent pure state machine with stages for preflight, reset, UART default baud, chip detect, register init, frequency/nonce setup, max baud, and initialized-no-mining state.
- **D-10:** Start live Ultra 205 evidence with a chip-detect-first staged gate. Full staged init may follow only after required board, config, power, thermal, and safety preflight tokens exist and are evidenced.
- **D-11:** Reuse Phase 2 config/catalog facts as gate inputs: board version `205`, `VerificationScope::ActiveUltra205`, family `Ultra`, ASIC model `BM1366`, expected ASIC count `1`, supported frequency/voltage options, DS4432U and INA260 capabilities, and deferred non-205 scopes.
- **D-12:** Fail closed on every missing gate or stage error: no mining, no production work submission, no false initialized status, reset held low or ASIC enable disabled where that is the safe adapter action, and a visible `asic_status` or boot log reason.
- **D-13:** Treat pre-init thermal gating conservatively. Upstream chip temperature is not meaningful before ASIC init, so live init should either require lower-level thermal/fan/power evidence or block with a typed preflight-missing status.

#### Evidence And Provenance Strategy

- **D-14:** Use a boundary-split fixture/evidence matrix. Pure protocol rows can advance with unit/golden fixtures; mixed live UART/reset/init rows remain `implemented` or `in-progress` until Ultra 205 hardware evidence exists.
- **D-15:** BM1366 fixtures should carry metadata fields such as source file, source function or behavior, pinned reference commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`, license posture, generated/manual derivation note, and checklist IDs.
- **D-16:** Prefer hand-authored metadata fixtures for early cases, and introduce a generated reference fixture harness only if manual byte derivation becomes numerous or error-prone. Generated output remains GPL-risk source data and must be labeled accordingly.
- **D-17:** Hardware evidence files for init or chip detection must record board, port, command, firmware commit, reference commit, relevant logs, observed result, fail-closed conclusion, and any skipped hardware gate.
- **D-18:** Do not copy upstream C expression into MIT-only files. Use module or behavior breadcrumbs to reference the pinned C implementation, and isolate any intentionally ported GPL-derived expression or fixture data with conservative license notes.

### the agent's Discretion

The agent may choose exact module names, Rust type names, fixture file formats, state-machine representation, fake UART transcript schema, and plan count. Those choices must preserve functional core plus imperative shell, keep runtime work submission disabled, satisfy ASIC-01 through ASIC-08, and avoid marking safety-critical hardware behavior `verified` without hardware evidence.

### Deferred Ideas (OUT OF SCOPE)

- Stratum v1 socket behavior, pool authorization, mining loop integration, accepted-share evidence, and production work dispatch belong to Phase 4.
- Full safety controller behavior for voltage, power, fan, thermal, PID, faults, and self-test belongs to Phase 6.
- API exposure of ASIC status, telemetry, and logs belongs to Phase 5.
- Non-205 boards, BM1370, BM1368, BM1397, and all-board ASIC verification remain unverified or deferred until each has its own evidence set.
- A generated reference-fixture extraction tool may be added later if fixture volume justifies it.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ASIC-01 | BM1366 packet, register, and CRC codecs are implemented as pure Rust logic with reference-derived fixtures. [VERIFIED: .planning/REQUIREMENTS.md] | Use `bitaxe-asic::bm1366::{crc, packet, registers}` with CRC5 and CRC16-FALSE fixtures from the pinned reference tree. [VERIFIED: reference/esp-miner/components/asic/crc.c; reference/esp-miner/components/asic/bm1366.c] |
| ASIC-02 | BM1366 work encoding and result parsing match upstream behavior for job payloads, nonces, domains, and error cases. [VERIFIED: .planning/REQUIREMENTS.md] | Model fixed-size work payloads, 11-byte result frames, job-id masking, nonce-derived ASIC/core decoding, register responses, and invalid-job rejection in pure tests. [VERIFIED: reference/esp-miner/components/asic/bm1366.c; reference/esp-miner/components/asic/asic_common.c] |
| ASIC-03 | ASIC model dispatch supports BM1366 as the V1 active path and represents other upstream ASIC families as deferred or not-yet-verified paths. [VERIFIED: .planning/REQUIREMENTS.md] | Keep `BM1366` active for board 205 and expose explicit deferred states for non-205 families. [VERIFIED: crates/bitaxe-config/src/catalog.rs; docs/parity/checklist.md] |
| ASIC-04 | The firmware contains a narrow UART adapter boundary that translates typed ASIC commands and observations between pure Rust logic and ESP-IDF serial I/O. [VERIFIED: .planning/REQUIREMENTS.md] | Firmware should interpret typed adapter actions using ESP-IDF UART/GPIO primitives; raw BM1366 bytes stay in the ASIC crate. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md; VERIFIED: local cargo registry esp-idf-hal-0.46.2] |
| ASIC-05 | Ultra 205 BM1366 reset, preflight, and staged initialization fail closed unless required board, power, thermal, and config gates pass. [VERIFIED: .planning/REQUIREMENTS.md] | Implement a pure `Bm1366InitPlan` whose inputs include Phase 2 catalog facts and explicit evidence tokens; any missing token returns a fail-closed status. [VERIFIED: crates/bitaxe-config/src/catalog.rs; VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md] |
| ASIC-06 | Frequency and voltage transition decisions are range-checked in pure Rust and require explicit hardware evidence before being marked verified. [VERIFIED: .planning/REQUIREMENTS.md] | Reuse or mirror `bitaxe-config` frequency/voltage newtypes for Ultra 205 options and keep hardware effect status below `verified` without evidence. [VERIFIED: crates/bitaxe-config/src/validation.rs; docs/adr/0012-parity-verification-evidence.md] |
| ASIC-07 | BM1366 initialization, work-send, and result-receive behavior have hardware-smoke evidence before release parity is claimed. [VERIFIED: .planning/REQUIREMENTS.md] | Split pure fixture evidence from hardware-smoke evidence files and do not promote live rows to verified without Ultra 205 logs. [VERIFIED: docs/parity/checklist.md; docs/adr/0012-parity-verification-evidence.md] |
| ASIC-08 | ASIC modules and tricky behavior boundaries include reference breadcrumbs pointing to the pinned upstream implementation and parity checklist rows. [VERIFIED: .planning/REQUIREMENTS.md] | Add behavior-level breadcrumbs for CRCs, framing, init sequence, UART receive semantics, and nonce decoding, not line-by-line copied C. [VERIFIED: docs/adr/0008-reference-breadcrumb-comments.md; docs/adr/0013-mit-first-with-gpl-guardrails.md] |
</phase_requirements>

## Summary

Phase 3 should make `crates/bitaxe-asic` the pure owner of BM1366 protocol behavior and make `firmware/bitaxe` a narrow interpreter of typed UART/reset/status actions. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md] The upstream reference confirms that the BM1366 path includes two CRC families, different command/job frame length rules, 11-byte result frames, chip-count validation, job-id masking, nonce-derived ASIC/core decoding, baud changes, and a long register-init sequence. [VERIFIED: reference/esp-miner/components/asic/bm1366.c; reference/esp-miner/components/asic/crc.c; reference/esp-miner/components/asic/asic_common.c]

The live Ultra 205 path should be planned as chip-detect-first, then full staged initialization only after board/config/power/thermal/safety evidence tokens exist. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md] Missing preflight tokens, UART faults, malformed frames, CRC failures, chip-count mismatch, unknown registers, and invalid job IDs should produce typed faults and visible fail-closed statuses rather than mining, production work dispatch, or a false initialized state. [VERIFIED: .planning/REQUIREMENTS.md; VERIFIED: reference/esp-miner/components/asic/asic_common.c]

**Primary recommendation:** Plan a pure BM1366 protocol/state-machine wave first, a fake transcript adapter wave second, and live firmware chip-detect/full-init evidence as separate gated work that cannot mark safety-critical behavior `verified` without Ultra 205 hardware logs. [VERIFIED: .planning/ROADMAP.md; VERIFIED: docs/adr/0012-parity-verification-evidence.md]

## Project Constraints (from AGENTS.md)

- `AGENTS.md` requires reading `AGENTS.bright-builds.md`, `standards-overrides.md` when present, and relevant `standards/` pages before plan, review, implementation, or audit work. [VERIFIED: AGENTS.md]
- The project must keep upstream `reference/esp-miner` pinned and read-only; the reference tree is behavioral evidence, not a workspace for edits. [VERIFIED: AGENTS.md]
- Ultra 205 with BM1366 is the first hardware priority; other upstream boards remain deferred until they have their own evidence. [VERIFIED: AGENTS.md; VERIFIED: .planning/REQUIREMENTS.md]
- Functional core plus imperative shell is mandatory: pure protocol/config logic belongs in crates, while ESP-IDF, FreeRTOS, UART, GPIO, delays, and hardware orchestration stay in firmware adapters. [VERIFIED: AGENTS.md; VERIFIED: standards/core/architecture.md]
- Bazel is the canonical automation graph and `just` is the human command surface. [VERIFIED: AGENTS.md]
- Parity claims need a checklist row plus evidence; implemented code alone is not enough to claim parity. [VERIFIED: AGENTS.md; VERIFIED: docs/adr/0006-parity-checklist-as-audit-evidence.md]
- Hardware-control surfaces such as voltage, fan, thermal, power, and ASIC initialization require hardware evidence before verified parity. [VERIFIED: AGENTS.md; VERIFIED: docs/adr/0012-parity-verification-evidence.md]
- MIT-first source is preferred, while intentionally ported GPL-covered expression or fixture data must be isolated and conservatively labeled. [VERIFIED: AGENTS.md; VERIFIED: docs/adr/0013-mit-first-with-gpl-guardrails.md]
- Rust commits require `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`. [VERIFIED: AGENTS.md; VERIFIED: standards/languages/rust.md]
- Rust modules should prefer `foo.rs` plus `foo/` over `foo/mod.rs`, avoid `unwrap()`, use `thiserror` for library errors, use `anyhow` for application errors, and use `maybe_` naming for `Option` values. [VERIFIED: AGENTS.md; VERIFIED: standards/languages/rust.md]
- Unit tests should test one concern and use Arrange, Act, Assert comments when helpful. [VERIFIED: AGENTS.md; VERIFIED: standards/core/testing.md]
- No project skills were found under `.claude/skills/` or `.agents/skills/`. [VERIFIED: `find .claude/skills .agents/skills -maxdepth 2 -name SKILL.md -print`]

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `bitaxe-asic` | local crate `0.1.0` | Pure BM1366 protocol, typed commands/observations, init state machine, fake transcripts, and ASIC dispatch. | This crate already exists as the Phase 3 placeholder and is the intended pure ownership boundary. [VERIFIED: crates/bitaxe-asic/src/lib.rs; VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md] |
| `bitaxe-config` | local crate `0.1.0` | Ultra 205 catalog, BM1366 profile, active verification scope, frequency and voltage validation. | Phase 2 already established these catalog facts and Phase 3 decisions require reusing them as init gates. [VERIFIED: crates/bitaxe-config/src/catalog.rs; VERIFIED: crates/bitaxe-config/src/validation.rs] |
| `thiserror` | `2.0.18`, published 2026-01-18 | Structured library errors for protocol, init, and transcript failures. | Project Rust rules prefer `thiserror` for library errors and the version is current on the registry. [VERIFIED: AGENTS.md; VERIFIED: crates.io API] |
| `esp-idf-svc` | `0.52.1`, published 2026-03-10 | Firmware integration crate that re-exports ESP-IDF HAL and services. | The firmware already depends on it, and the local source re-exports `hal`. [VERIFIED: firmware/bitaxe/Cargo.toml; VERIFIED: local cargo registry esp-idf-svc-0.52.1] |
| `esp-idf-hal` | `0.46.2`, published 2026-03-10 | UART and GPIO wrappers used through `esp_idf_svc::hal` unless a direct dependency becomes necessary. | The local HAL source exposes `UartDriver::new`, `read`, `write`, `clear_rx`, `wait_tx_done`, and `change_baudrate`, matching the adapter needs. [VERIFIED: local cargo registry esp-idf-hal-0.46.2; CITED: https://docs.rs/esp-idf-hal/0.46.2/esp_idf_hal/uart/] |
| `log` | `0.4.33`, published 2026-06-20 | Firmware-visible logging through `EspLogger`. | The firmware already initializes `EspLogger`, and project stack uses the `log` facade for ESP-IDF firmware logging. [VERIFIED: firmware/bitaxe/src/main.rs; VERIFIED: crates.io API] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde` | `1.0.228`, published 2025-09-27 | Fixture and evidence metadata serialization. | Use when fixtures become machine-readable JSON or metadata structs. [VERIFIED: Cargo.toml; VERIFIED: crates.io API] |
| `serde_json` | `1.0.150`, published 2026-05-21 | JSON fixture loading and golden metadata checks. | Use for reference-derived fixture files only; do not put raw protocol decisions in firmware JSON parsing. [VERIFIED: Cargo.toml; VERIFIED: crates.io API] |
| `heapless` | `0.9.3`, published 2026-04-30 | Optional fixed-capacity collections for no-allocation protocol buffers. | Add only if fixed arrays/newtypes are not enough for bounded transcript/action buffers. [VERIFIED: crates.io API] |
| `embedded-io` | `0.7.1`, published 2025-10-01 | Optional testable read/write trait boundary. | Use only if fake UART tests benefit from trait-based I/O; `esp-idf-hal` already implements embedded I/O traits locally. [VERIFIED: crates.io API; VERIFIED: local cargo registry esp-idf-hal-0.46.2] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Explicit BM1366 CRC functions | A generic CRC crate | Explicit small functions are a parity surface and simpler to fixture against the pinned reference. [VERIFIED: reference/esp-miner/components/asic/crc.c] |
| Pure state machine plus firmware adapter | Directly porting `BM1366_init` into firmware | Direct porting would mix raw protocol bytes, GPIO/UART effects, and GPL provenance risks into the firmware shell. [VERIFIED: reference/esp-miner/components/asic/bm1366.c; VERIFIED: docs/adr/0013-mit-first-with-gpl-guardrails.md] |
| `esp-idf-hal::uart::UartDriver` | Raw `esp-idf-sys` UART calls | HAL already provides the required UART operations; raw sys calls should be reserved for missing HAL coverage. [VERIFIED: local cargo registry esp-idf-hal-0.46.2] |
| Hand-authored fixtures | Generated extraction harness | Hand-authored metadata fixtures are the locked early-phase choice; generated GPL-risk data is deferred until fixture volume justifies it. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md] |
| FreeRTOS task or async executor first | Embassy or async-first firmware | Phase 3 needs a narrow staged adapter, not a new scheduler model. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md; VERIFIED: research/STACK.md embedded in AGENTS.md] |

**Installation:**

No new registry dependency is required for the first pure implementation if `bitaxe-asic` uses workspace `thiserror`, existing fixed arrays, and dev-only fixture code. [VERIFIED: Cargo.toml; VERIFIED: crates/bitaxe-asic/Cargo.toml]

If the planner chooses machine-readable JSON fixtures or bounded collections, add dependencies through the workspace rather than pinning versions in the crate directly. [VERIFIED: Cargo.toml]

```bash
cargo add thiserror --package bitaxe-asic --workspace
cargo add serde --package bitaxe-asic --workspace --features derive --optional
cargo add serde_json --package bitaxe-asic --workspace --dev
cargo add heapless --package bitaxe-asic --workspace --optional
```

**Version verification:** Recommended external crate versions above were checked through the crates.io API or local Cargo registry on 2026-06-26. [VERIFIED: crates.io API; VERIFIED: local cargo registry]

## Architecture Patterns

### Recommended Project Structure

```text
crates/bitaxe-asic/src/
├── lib.rs                 # Public crate exports and deferred/active dispatch surface.
├── error.rs               # thiserror-backed protocol/init/transcript errors.
├── dispatch.rs            # Active BM1366 path plus deferred non-205 ASIC families.
├── bm1366.rs              # Public BM1366 module facade and breadcrumbs.
└── bm1366/
    ├── crc.rs             # CRC5 and CRC16-FALSE parity functions.
    ├── packet.rs          # Command/job frames, lengths, preamble, CRC selection.
    ├── registers.rs       # Typed register IDs, register payloads, response mapping.
    ├── work.rs            # Fixed-size work payload encoding and job-id tracking.
    ├── result.rs          # 11-byte result parsing and nonce/domain decoding.
    ├── init_plan.rs       # Pure staged fail-closed initialization decisions.
    └── transcript.rs      # Fake UART transcript/actions for tests.

firmware/bitaxe/src/
├── asic_adapter.rs        # Narrow interpreter for typed adapter actions.
└── asic_adapter/
    ├── uart.rs            # UART1 read/write/clear/baud operations.
    ├── reset.rs           # Reset GPIO pulse and hold-low behavior.
    └── status.rs          # Visible boot/status mapping for fail-closed outcomes.
```

This structure follows the repo Rust module rule of `foo.rs` plus `foo/` and keeps pure logic in crates while ESP-IDF effects stay in firmware. [VERIFIED: standards/languages/rust.md; VERIFIED: standards/core/architecture.md]

### Pattern 1: Typed Protocol Frames

**What:** Define `Bm1366Command`, `Bm1366Observation`, `CommandFrame`, `JobFrame`, and fixed-size `FrameBytes` types so raw bytes and CRC details are constructed only inside `bitaxe-asic`. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]

**When to use:** Use for every command packet, job packet, register read/write, work payload, and result frame. [VERIFIED: reference/esp-miner/components/asic/bm1366.c]

**Example:**

```rust
// Source: reference/esp-miner/components/asic/bm1366.c:_send_BM1366
pub enum Bm1366Command {
    SetVersionMask(VersionMask),
    ReadChipId,
    WriteRegister(RegisterWrite),
    SetFrequency(FrequencyPlan),
    SetNonceSpace(NonceSpacePlan),
    SendDiagnosticWork(Bm1366Work),
}
```

### Pattern 2: Pure Init Plan Emits Adapter Actions

**What:** Model initialization as a pure state machine that consumes typed preflight facts and emits a list or iterator of adapter actions. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]

**When to use:** Use for preflight, reset, UART default baud, chip detect, register init, frequency/nonce setup, max baud, initialized-no-mining, and fail-closed statuses. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]

**Example:**

```rust
// Source: Phase 3 decision D-09 and firmware adapter boundary D-07.
pub enum Bm1366AdapterAction {
    ResetPulse { low_ms: u32, high_ms: u32 },
    HoldResetLow,
    UseDefaultBaud { baud: u32 },
    UseMaxBaud { baud: u32 },
    ClearRx,
    WriteFrame(FrameBytes),
    ReadExact { len: usize, timeout_ms: u32 },
    DelayMs(u32),
    PublishStatus(AsicInitStatus),
}
```

### Pattern 3: Exact-Read Transcript Harness

**What:** Build a fake transcript that models exact reads, timeouts, partial reads, malformed preambles, bad CRCs, unknown registers, and chip-count mismatch. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]

**When to use:** Use for adapter boundary tests and fail-closed initialization tests before hardware smoke. [VERIFIED: .planning/ROADMAP.md]

**Example:**

```rust
// Source: reference/esp-miner/components/asic/asic_common.c:count_asic_chips and receive_work.
pub enum FakeUartEvent {
    ExpectWrite(FrameBytes),
    ReadBytes(&'static [u8]),
    Timeout,
    PartialRead(&'static [u8]),
}
```

### Pattern 4: Boundary-Split Evidence

**What:** Treat pure protocol fixture evidence separately from live UART/reset/init evidence. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]

**When to use:** Use pure fixtures to support implemented codec rows and hardware evidence files to support live behavior rows. [VERIFIED: docs/adr/0012-parity-verification-evidence.md]

**Example fixture metadata:**

```json
{
  "checklist_ids": ["ASIC-006"],
  "source_file": "reference/esp-miner/components/asic/crc.c",
  "source_behavior": "BM1366 command CRC5 over header/length/payload",
  "reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50",
  "license_posture": "reference-derived fixture data; not MIT source expression",
  "derivation": "manual"
}
```

### Anti-Patterns to Avoid

- **Raw BM1366 bytes in firmware control logic:** Raw preambles, registers, lengths, CRCs, and frame payloads belong inside `bitaxe-asic`. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]
- **False verified hardware status from unit tests:** Unit tests can prove bytes and state decisions, not live reset, chip detection, voltage, thermal, or frequency effects. [VERIFIED: docs/adr/0012-parity-verification-evidence.md]
- **Direct GPL expression copy into MIT files:** Use breadcrumbs and fixture metadata; isolate any intentionally ported GPL-derived expression or data. [VERIFIED: docs/adr/0013-mit-first-with-gpl-guardrails.md]
- **Exact upstream init sequence without preflight tokens:** Phase 3 requires fail-closed staged initialization, not unconditional hardware bring-up. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]
- **Leaking Phase 4 mining behavior into Phase 3:** Runtime work submission stays disabled or diagnostic-only until Phase 4. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| ESP-IDF UART/GPIO driver behavior | Custom serial driver or direct UART register programming | `esp_idf_svc::hal::uart::UartDriver` and ESP-IDF GPIO wrappers | HAL provides read, write, clear RX, wait TX done, and baud-change operations needed by the adapter. [VERIFIED: local cargo registry esp-idf-hal-0.46.2; CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/peripherals/uart.html] |
| Board identity and supported ranges | New ad hoc board constants in `bitaxe-asic` | `bitaxe-config` catalog and validation newtypes | Phase 2 already owns Ultra 205/BM1366 identity, count, frequency options, voltage options, and capabilities. [VERIFIED: crates/bitaxe-config/src/catalog.rs; VERIFIED: crates/bitaxe-config/src/validation.rs] |
| Firmware packet parsing | Byte parsing in `firmware/bitaxe` | `bitaxe-asic` typed codec and observations | Firmware should not know BM1366 preambles, CRCs, or raw register details. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md] |
| Evidence policy | Inline comments claiming parity | `docs/parity/checklist.md` rows plus `docs/parity/evidence/` files | Project policy requires checklist and evidence before parity claims. [VERIFIED: docs/adr/0006-parity-checklist-as-audit-evidence.md; VERIFIED: docs/adr/0012-parity-verification-evidence.md] |
| Full mining loop | Production `SendWork` runtime path | Diagnostic-only modeling with disabled production work | Phase 4 owns Stratum and mining integration. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md] |
| Generated reference extraction | Early generator for every fixture | Hand-authored fixture metadata first | The context locks hand-authored metadata fixtures first and defers generated harnesses unless volume warrants them. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md] |

**Key insight:** BM1366 behavior has several protocol-level edge cases that are easy to encode incorrectly but can be proven in pure tests, while live init has safety-critical effects that must remain gated by evidence. [VERIFIED: reference/esp-miner/components/asic/bm1366.c; VERIFIED: docs/adr/0012-parity-verification-evidence.md]

## Common Pitfalls

### Pitfall 1: Confusing Transmit And Receive Preambles

**What goes wrong:** Command/job frames are transmitted with bytes `0x55 0xAA`, while upstream receive validation checks an 11-byte response whose first two bytes represent `0xAA55`. [VERIFIED: reference/esp-miner/components/asic/bm1366.c; VERIFIED: reference/esp-miner/components/asic/asic_common.c]

**Why it happens:** The upstream send path writes literal bytes, while the receive path validates a typed response buffer as a 16-bit preamble. [VERIFIED: reference/esp-miner/components/asic/bm1366.c; VERIFIED: reference/esp-miner/components/asic/asic_common.c]

**How to avoid:** Model transmit frames and receive frames as different types with separate fixture cases. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]

**Warning signs:** One preamble constant is reused for both send and receive tests without fixture coverage. [VERIFIED: reference/esp-miner/components/asic/bm1366.c; reference/esp-miner/components/asic/asic_common.c]

### Pitfall 2: Applying The Wrong CRC Or Length Rule

**What goes wrong:** BM1366 command packets use CRC5 and job packets use CRC16-FALSE, and their total length and length-field calculations differ. [VERIFIED: reference/esp-miner/components/asic/bm1366.c; VERIFIED: reference/esp-miner/components/asic/crc.c]

**Why it happens:** The send helper branches on packet type and appends a different CRC width. [VERIFIED: reference/esp-miner/components/asic/bm1366.c]

**How to avoid:** Make `CommandFrame` and `JobFrame` separate constructors instead of a generic byte builder with flags. [VERIFIED: standards/core/architecture.md]

**Warning signs:** Tests only cover one packet type or compare payload bytes without validating appended CRC bytes. [VERIFIED: .planning/ROADMAP.md]

### Pitfall 3: Treating UART Reads As All-Or-Nothing

**What goes wrong:** Upstream code treats timeout, negative read, partial length, malformed preamble, and bad CRC as distinct error/fault cases. [VERIFIED: reference/esp-miner/components/asic/asic_common.c]

**Why it happens:** HAL read semantics return data or timeout at a lower level, while ASIC protocol semantics need exact 11-byte response validation. [VERIFIED: local cargo registry esp-idf-hal-0.46.2; VERIFIED: reference/esp-miner/components/asic/asic_common.c]

**How to avoid:** Wrap UART reads in an adapter-level `read_exact_response` that preserves timeout and partial-read outcomes for the pure transcript/state machine. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]

**Warning signs:** A partial response is silently ignored, retried without status, or decoded as a zero-padded frame. [VERIFIED: reference/esp-miner/components/asic/asic_common.c]

### Pitfall 4: Advancing Job IDs Incorrectly

**What goes wrong:** BM1366 job IDs advance by 8 and valid result lookup masks with `0xf8`; the low 3 bits carry small-core identity. [VERIFIED: reference/esp-miner/components/asic/bm1366.c]

**Why it happens:** A generic incrementing job ID loses the packed small-core convention. [VERIFIED: reference/esp-miner/components/asic/bm1366.c]

**How to avoid:** Use a `Bm1366JobId` type that exposes `advance()`, `lookup_key()`, and `small_core_id()` semantics. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]

**Warning signs:** Tests assert only one job ID and do not include invalid-job rejection. [VERIFIED: .planning/REQUIREMENTS.md]

### Pitfall 5: Claiming Thermal Or Frequency Safety Too Early

**What goes wrong:** Unit/golden tests prove register bytes but cannot prove live voltage, frequency, fan, power, or thermal behavior. [VERIFIED: docs/adr/0012-parity-verification-evidence.md]

**Why it happens:** The pure protocol boundary can encode decisions but cannot observe hardware effects. [VERIFIED: standards/core/architecture.md; VERIFIED: docs/adr/0012-parity-verification-evidence.md]

**How to avoid:** Keep hardware-effect rows `implemented` or `in-progress` until Ultra 205 evidence files record command, logs, observed result, and fail-closed conclusion. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]

**Warning signs:** A parity checklist row says `verified` based only on `cargo test` or fixture output. [VERIFIED: docs/parity/checklist.md]

### Pitfall 6: GPL Provenance Drift

**What goes wrong:** Copying upstream C expression or large derived tables into MIT source creates release-review risk. [VERIFIED: docs/adr/0013-mit-first-with-gpl-guardrails.md; VERIFIED: PROVENANCE.md]

**Why it happens:** The pinned reference is close at hand and the protocol is byte-oriented. [VERIFIED: docs/adr/0005-read-only-reference-implementation.md]

**How to avoid:** Use behavior-level breadcrumbs, independently written Rust, metadata-rich fixtures, and explicit license posture fields for reference-derived data. [VERIFIED: docs/adr/0008-reference-breadcrumb-comments.md; VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]

**Warning signs:** A Rust source file contains long translated blocks from `bm1366.c` without license isolation. [VERIFIED: docs/adr/0013-mit-first-with-gpl-guardrails.md]

## Code Examples

Verified patterns from project and reference sources:

### Semantic Command Boundary

```rust
// Source: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md D-05/D-06.
pub enum Bm1366Observation {
    ChipId { chip_id: ChipId, asic_index: AsicIndex },
    RegisterRead { register: Bm1366Register, value: RegisterValue },
    JobNonce(Bm1366NonceResult),
    ProtocolFault(Bm1366ProtocolFault),
}
```

### Fail-Closed Init Decision

```rust
// Source: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md D-10/D-12/D-13.
pub enum Bm1366InitDecision {
    Actions(Vec<Bm1366AdapterAction>),
    FailClosed {
        status: AsicInitStatus,
        safe_action: FailClosedAction,
        reason: PreflightFault,
    },
}
```

### Exact Read Test Shape

```rust
#[test]
fn chip_detect_timeout_fails_closed() {
    // Arrange
    let transcript = FakeUartTranscript::new([FakeUartEvent::Timeout]);

    // Act
    let result = Bm1366InitPlan::chip_detect_only().run(transcript);

    // Assert
    assert!(matches!(result.status(), AsicInitStatus::FailClosed { .. }));
}
```

This test shape follows the project Arrange/Act/Assert testing rule and the Phase 3 fake UART transcript requirement. [VERIFIED: standards/core/testing.md; VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| C firmware directly interleaves BM1366 bytes, UART, reset, delays, global state, and mining data. | Rust plan should split pure protocol/state logic from firmware UART/GPIO effects. | Locked by project architecture and Phase 3 context on 2026-06-26. [VERIFIED: AGENTS.md; VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md] | Planner should create separate pure, adapter, firmware, evidence, and docs tasks. |
| Single reference implementation as executable truth. | Pinned read-only reference plus independent Rust behavior with breadcrumbs and parity evidence. | Established by ADR-0005, ADR-0006, ADR-0008, ADR-0012, and ADR-0013. [VERIFIED: docs/adr/0005-read-only-reference-implementation.md; docs/adr/0013-mit-first-with-gpl-guardrails.md] | Planner must include provenance and evidence updates, not just code. |
| Treating hardware smoke as optional after code compiles. | Safety-critical ASIC/power/thermal behavior cannot be verified without Ultra 205 evidence. | Established by project constraints and Phase 3 success criteria. [VERIFIED: AGENTS.md; VERIFIED: .planning/ROADMAP.md] | Planner must separate "implemented" from "verified" and include skipped-gate records where hardware evidence is absent. |

**Deprecated/outdated:**

- Production mining dispatch in this phase is out of scope; runtime work submission remains disabled or diagnostic-only until Phase 4. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]
- Unconditional BM1366 full initialization is not acceptable for Phase 3; live init must be staged and fail-closed. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]
- Direct upstream C edits are not allowed because the reference tree is read-only. [VERIFIED: AGENTS.md; VERIFIED: docs/adr/0005-read-only-reference-implementation.md]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| - | No `[ASSUMED]` claims are intentionally used in this research. | All sections | Planner should still re-check live hardware state before running any hardware smoke. [VERIFIED: docs/adr/0012-parity-verification-evidence.md] |

## Open Questions (RESOLVED)

1. **Which exact power and thermal tokens are sufficient for full live init in Phase 3?** [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]
   - What we know: Full init requires board, config, power, thermal, and safety preflight tokens. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]
   - Resolution: Phase 3 token schema is explicit: `BoardPreflightEvidence`, `ConfigPreflightEvidence`, `PowerPreflightEvidence`, `ThermalPreflightEvidence`, and `SafetyPreflightEvidence`. `chip_detect_only()` may run with board and config evidence only; `full_init()` must require all five tokens and fail closed before reset/register/frequency/max-baud/initialized stages when any token is absent. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]
   - Resolution: Missing safety evidence uses a distinct typed reason, `safety_preflight_evidence_missing`, and safe action `HoldResetLow`; unit fixtures must include a missing-safety case. [VERIFIED: .planning/REQUIREMENTS.md]
   - Recommendation retained: Plan chip-detect-only as the first hardware smoke, then allow full-init to remain blocked with typed `PreflightMissing` statuses until all preflight tokens are evidenced. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]

2. **Should the firmware depend directly on `esp-idf-hal` or only use `esp_idf_svc::hal` re-exports?** [VERIFIED: firmware/bitaxe/Cargo.toml; VERIFIED: local cargo registry esp-idf-svc-0.52.1]
   - What we know: `esp-idf-svc` re-exports `hal`, and the firmware already depends on `esp-idf-svc`. [VERIFIED: local cargo registry esp-idf-svc-0.52.1; VERIFIED: firmware/bitaxe/Cargo.toml]
   - Resolution: Use `esp_idf_svc::hal` re-exports for Phase 3 firmware UART/GPIO work. Do not add a direct `esp-idf-hal` dependency unless implementation proves the re-export path cannot satisfy a concrete compiler or Bazel crate-mapping requirement. [VERIFIED: local cargo registry esp-idf-svc-0.52.1]
   - Recommendation retained: Keep the adapter thin and HAL-facing; raw `esp-idf-sys` UART calls remain a fallback only for missing HAL coverage. [VERIFIED: local cargo registry esp-idf-hal-0.46.2]

3. **Can the visible serial port be treated as an Ultra 205 for evidence?** [VERIFIED: `espflash list-ports`]
   - What we know: `/dev/cu.usbmodem1101` is visible as an Espressif USB JTAG/serial debug unit. [VERIFIED: `espflash list-ports`]
   - Resolution: A visible serial port is not sufficient evidence of an Ultra 205 board or safe bench setup. Hardware evidence may cite a port only after the evidence file records board identity, port, command, firmware commit, reference commit, relevant logs, observed result, skipped gates, and fail-closed conclusion. [VERIFIED: docs/adr/0012-parity-verification-evidence.md]
   - Resolution: If user approval, board identity, or safe setup evidence is missing, the evidence conclusion remains `not run - hardware verification pending` and safety-critical ASIC rows remain below `verified`. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]
   - Recommendation retained: Require hardware evidence files to record board identity, port, command, logs, observed result, and fail-closed conclusion before updating verified rows. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Rust `cargo` | Pure crate tests and host verification | yes | `cargo 1.88.0-nightly` | Use project toolchain via `rustup` if host default changes. [VERIFIED: `cargo --version`] |
| Rust `rustc` | Pure crate and firmware compilation | yes | `rustc 1.88.0-nightly` | Use pinned toolchain if a future plan adds one. [VERIFIED: `rustc --version`] |
| ESP Rust toolchain | ESP32-S3 firmware builds | yes | `esp` toolchain active | Re-run `espup install --targets esp32s3 --std` if build fails. [VERIFIED: `rustup toolchain list`] |
| Bazelisk/Bazel | Canonical build/test graph | yes | Bazelisk `1.28.1`, Bazel `9.1.1` | Use `bazelisk` launcher when invoking Bazel. [VERIFIED: `bazelisk version`; VERIFIED: `bazel version`] |
| `just` | Human command surface | yes | `just 1.48.0` | Use direct Bazel/Cargo commands only for diagnosis. [VERIFIED: `just --version`] |
| `espflash` | Serial port detection, flashing, monitoring, evidence capture | yes | `espflash 4.0.1` | `cargo-espflash 4.0.1` is also available. [VERIFIED: `espflash --version`; VERIFIED: `cargo-espflash --version`] |
| Serial device | Hardware smoke candidate | partially | `/dev/cu.usbmodem1101`, Espressif USB JTAG/serial debug unit | Pure tests remain valid; hardware evidence must identify the board before verification. [VERIFIED: `espflash list-ports`] |
| `espup` | ESP Rust toolchain install/repair | yes, older than project stack note | `espup 0.15.1` | Existing `esp` toolchain is active; upgrade only if firmware build/repair requires it. [VERIFIED: `espup --version`; VERIFIED: AGENTS.md embedded stack notes] |
| `ldproxy` | ESP-IDF Rust linker proxy | present, version not reported | command panics without linker args | Validate through firmware build rather than `ldproxy --version`. [VERIFIED: `ldproxy --version`] |

**Missing dependencies with no fallback:**

- None for pure protocol planning and tests. [VERIFIED: environment probes]

**Missing dependencies with fallback:**

- Hardware identity is not proven by the visible serial port; planner should keep live verification rows blocked or skipped until board-named evidence is captured. [VERIFIED: `espflash list-ports`; VERIFIED: docs/adr/0012-parity-verification-evidence.md]

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust unit tests through Cargo and Bazel `rust_test`. [VERIFIED: crates/bitaxe-asic/BUILD.bazel; VERIFIED: Cargo.toml] |
| Config file | `crates/bitaxe-asic/BUILD.bazel`, workspace `Cargo.toml`, `MODULE.bazel`. [VERIFIED: crates/bitaxe-asic/BUILD.bazel; VERIFIED: MODULE.bazel] |
| Quick run command | `cargo test -p bitaxe-asic --all-features` and `bazel test //crates/bitaxe-asic:tests`. [VERIFIED: crates/bitaxe-asic/BUILD.bazel] |
| Full suite command | `cargo test --all-features` plus the project-required pre-commit Rust checks. [VERIFIED: AGENTS.md] |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| ASIC-01 | CRC5, CRC16-FALSE, command framing, job framing, register payloads. [VERIFIED: .planning/REQUIREMENTS.md] | unit/golden | `cargo test -p bitaxe-asic bm1366_crc --all-features` | no, Wave 0 module gap. [VERIFIED: crates/bitaxe-asic/src/lib.rs] |
| ASIC-02 | Work encoding, 11-byte result parsing, nonce/core/domain decoding, invalid-job rejection. [VERIFIED: .planning/REQUIREMENTS.md] | unit/golden | `cargo test -p bitaxe-asic bm1366_work --all-features` | no, Wave 0 module gap. [VERIFIED: crates/bitaxe-asic/src/lib.rs] |
| ASIC-03 | BM1366 active dispatch and deferred non-205 families. [VERIFIED: .planning/REQUIREMENTS.md] | unit | `cargo test -p bitaxe-asic dispatch --all-features` | no, Wave 0 module gap. [VERIFIED: crates/bitaxe-asic/src/lib.rs] |
| ASIC-04 | Typed adapter actions and fake UART transcript translation. [VERIFIED: .planning/REQUIREMENTS.md] | unit/adapter | `cargo test -p bitaxe-asic transcript --all-features` | no, Wave 0 module gap. [VERIFIED: crates/bitaxe-asic/src/lib.rs] |
| ASIC-05 | Ultra 205 staged init preflight and fail-closed outcomes. [VERIFIED: .planning/REQUIREMENTS.md] | unit plus hardware smoke | `cargo test -p bitaxe-asic init_plan --all-features`; hardware command decided by plan | no, Wave 0 module gap. [VERIFIED: crates/bitaxe-asic/src/lib.rs] |
| ASIC-06 | Frequency/voltage range checks and unverified hardware-effect status. [VERIFIED: .planning/REQUIREMENTS.md] | unit | `cargo test -p bitaxe-asic frequency_voltage --all-features` | partial in `bitaxe-config`, not in `bitaxe-asic`. [VERIFIED: crates/bitaxe-config/src/validation.rs] |
| ASIC-07 | Init, diagnostic work-send, and result-receive evidence before release parity. [VERIFIED: .planning/REQUIREMENTS.md] | evidence review plus smoke | `rg -n "ASIC-00[2-7]|BM1366" docs/parity docs/parity/evidence` | checklist exists, new Phase 3 evidence gap. [VERIFIED: docs/parity/checklist.md] |
| ASIC-08 | Reference breadcrumbs on tricky ASIC boundaries. [VERIFIED: .planning/REQUIREMENTS.md] | static review | `rg -n "reference/esp-miner|ASIC-00|BM1366" crates/bitaxe-asic firmware/bitaxe docs/parity` | no Phase 3 breadcrumbs yet. [VERIFIED: crates/bitaxe-asic/src/lib.rs] |

### Sampling Rate

- **Per task commit:** `cargo test -p bitaxe-asic --all-features` plus any touched crate test. [VERIFIED: AGENTS.md]
- **Per wave merge:** `bazel test //crates/bitaxe-asic:tests` and targeted firmware build when firmware adapter files are touched. [VERIFIED: crates/bitaxe-asic/BUILD.bazel; VERIFIED: AGENTS.md]
- **Phase gate:** Full project Rust pre-commit sequence, parity checklist review, and hardware evidence review for any row proposed as verified. [VERIFIED: AGENTS.md; VERIFIED: docs/adr/0012-parity-verification-evidence.md]

### Wave 0 Gaps

- [ ] `crates/bitaxe-asic/src/error.rs` covers typed protocol/init/transcript failures. [VERIFIED: crates/bitaxe-asic/src/lib.rs]
- [ ] `crates/bitaxe-asic/src/bm1366.rs` and `crates/bitaxe-asic/src/bm1366/` modules cover CRC, packet, registers, work, result, init plan, and transcript. [VERIFIED: crates/bitaxe-asic/src/lib.rs]
- [ ] `crates/bitaxe-asic/fixtures/bm1366/` or equivalent fixture location stores metadata-rich reference-derived cases. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md]
- [ ] `crates/bitaxe-asic/BUILD.bazel` exposes new source files and tests. [VERIFIED: crates/bitaxe-asic/BUILD.bazel]
- [ ] `docs/parity/checklist.md` ASIC rows are updated with pure fixture evidence and live evidence status boundaries. [VERIFIED: docs/parity/checklist.md]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | Phase 3 does not add user authentication surfaces. [VERIFIED: .planning/ROADMAP.md] |
| V3 Session Management | no | Phase 3 does not add sessions or API auth state. [VERIFIED: .planning/ROADMAP.md] |
| V4 Access Control | yes | Gate active behavior to Ultra 205 BM1366 and represent non-205 paths as deferred/not verified. [VERIFIED: .planning/REQUIREMENTS.md; VERIFIED: crates/bitaxe-config/src/catalog.rs] |
| V5 Input Validation | yes | Parse raw frames into typed domain values, validate config/frequency/voltage ranges, and reject malformed UART responses. [VERIFIED: standards/core/architecture.md; VERIFIED: crates/bitaxe-config/src/validation.rs; VERIFIED: reference/esp-miner/components/asic/asic_common.c] |
| V6 Cryptography | limited | CRC5 and CRC16-FALSE are protocol integrity checks, not security controls. [VERIFIED: reference/esp-miner/components/asic/crc.c] |
| V7 Error Handling And Logging | yes | Fail closed with visible `asic_status` or boot log reasons for every missing gate or stage error. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md] |

### Known Threat Patterns for BM1366 Init Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malformed or partial UART frame reaches control logic | Tampering | Exact response length, preamble, CRC, register, and job-validity validation before producing typed observations. [VERIFIED: reference/esp-miner/components/asic/asic_common.c; VERIFIED: reference/esp-miner/components/asic/bm1366.c] |
| Raw command injection through user-facing or firmware code | Elevation of Privilege | Keep raw packet construction private to `bitaxe-asic`; firmware accepts typed adapter actions only. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md] |
| Unsafe hardware enablement from missing power/thermal/config evidence | Tampering / Denial of Service | Pure preflight tokens and fail-closed statuses before live init, frequency, or voltage effects. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md; VERIFIED: docs/adr/0012-parity-verification-evidence.md] |
| Parity overclaiming without hardware proof | Repudiation | Checklist rows and evidence files must distinguish pure fixture evidence from live Ultra 205 verification. [VERIFIED: docs/adr/0006-parity-checklist-as-audit-evidence.md; VERIFIED: docs/adr/0012-parity-verification-evidence.md] |
| GPL expression leakage into MIT source | Supply Chain / Legal Integrity | Use read-only breadcrumbs, independent Rust implementation, fixture metadata, and conservative license labels. [VERIFIED: docs/adr/0013-mit-first-with-gpl-guardrails.md; VERIFIED: PROVENANCE.md] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md` - locked decisions, discretion, deferred scope, canonical references. [VERIFIED: local file]
- `.planning/REQUIREMENTS.md` - ASIC-01 through ASIC-08 and safety evidence requirements. [VERIFIED: local file]
- `.planning/ROADMAP.md` - Phase 3 goal, success criteria, verification expectations, and research flags. [VERIFIED: local file]
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/*.md`, `standards/languages/rust.md` - repo and Bright Builds constraints. [VERIFIED: local files]
- `reference/esp-miner/components/asic/bm1366.c` - BM1366 frames, init sequence, work encoding, result parsing, baud, nonce decoding. [VERIFIED: pinned local reference tree]
- `reference/esp-miner/components/asic/crc.c` - CRC5 and CRC16-FALSE behavior. [VERIFIED: pinned local reference tree]
- `reference/esp-miner/components/asic/asic_common.c` - chip counting, receive validation, difficulty mask, timeout helpers. [VERIFIED: pinned local reference tree]
- `reference/esp-miner/main/power/asic_init.c` and `asic_reset.c` - upstream reset/UART/init shell. [VERIFIED: pinned local reference tree]
- `crates/bitaxe-config/src/catalog.rs`, `defaults.rs`, `validation.rs` - Ultra 205/BM1366 catalog and validation facts. [VERIFIED: local files]
- `crates/bitaxe-core/src/lib.rs` and `firmware/bitaxe/src/main.rs` - current safe-state and firmware boot/log integration. [VERIFIED: local files]
- `docs/parity/checklist.md`, ADR-0005/0006/0008/0012/0013/0014, `PROVENANCE.md` - evidence and provenance rules. [VERIFIED: local files]
- Local Cargo registry source for `esp-idf-svc-0.52.1` and `esp-idf-hal-0.46.2` - HAL re-export and UART API coverage. [VERIFIED: local cargo registry]
- Crates.io API responses for `thiserror`, `heapless`, `embedded-io`, `esp-idf-svc`, `esp-idf-hal`, `log`, `serde`, and `serde_json`. [VERIFIED: crates.io API]

### Secondary (MEDIUM confidence)

- ESP-IDF v5.5.4 UART documentation - UART driver concepts and API behavior. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/peripherals/uart.html]
- ESP-IDF v5.5.4 GPIO documentation - GPIO direction/level behavior for reset adapter planning. [CITED: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/peripherals/gpio.html]
- Docs.rs `esp-idf-hal` UART page for public Rust API surface. [CITED: https://docs.rs/esp-idf-hal/0.46.2/esp_idf_hal/uart/]
- Docs.rs `esp-idf-svc` page for crate-level API documentation. [CITED: https://docs.rs/esp-idf-svc/0.52.1/esp_idf_svc/]

### Tertiary (LOW confidence)

- None. [VERIFIED: source review]

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - versions were verified through local manifests, local Cargo registry source, and crates.io API. [VERIFIED: Cargo.toml; VERIFIED: crates.io API]
- Architecture: HIGH for pure/protocol/adapter split because it is locked by context and Bright Builds architecture; MEDIUM-HIGH for exact live preflight token design because the context defines categories but not the final token schema. [VERIFIED: .planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md; VERIFIED: standards/core/architecture.md]
- Pitfalls: HIGH for BM1366 protocol edge cases because they come from the pinned upstream reference; MEDIUM-HIGH for hardware smoke planning because serial hardware is visible but board identity and safe setup still require evidence. [VERIFIED: reference/esp-miner/components/asic/bm1366.c; VERIFIED: `espflash list-ports`]

**Research date:** 2026-06-26
**Valid until:** 2026-07-26 for pure protocol and project architecture; 2026-07-03 for tool versions and live hardware availability. [VERIFIED: current_date; VERIFIED: environment probes]
