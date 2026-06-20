# GSD New Project Brief: Bitaxe Rust Firmware

Pass this document to GSD New Project as the idea document:

```bash
$gsd-new-project --auto @docs/project/gsd-new-project-brief.md
```

## What To Build

Create a Rust firmware monorepo for Bitaxe ESP-Miner. The project should implement a full Rust version of the Bitaxe ESP32 miner firmware with device-user parity against upstream `bitaxeorg/ESP-Miner`.

The upstream C implementation must be included as a pinned, read-only git submodule at `reference/esp-miner`. It is the reference implementation for behavior, tests, fixtures, parity breadcrumbs, and audits. It must not be modified by normal project work.

## Core Value

A Bitaxe owner can build, flash, run, configure, monitor, and update Rust firmware on real Bitaxe hardware with the same observable behavior they expect from upstream ESP-Miner.

## Canonical Parity Definition

Parity means device-user parity, not byte-for-byte or source-structure parity. The Rust firmware must match the observable behavior that a Bitaxe user, administrator, mining pool, API client, or flashing tool relies on from upstream ESP-Miner.

Device-user parity includes:

- Mining behavior and work submission.
- Supported Bitaxe board configs.
- ASIC support, with Gamma 601 BM1370 prioritized first.
- Stratum v1 and Stratum v2 behavior.
- AxeOS HTTP API, WebSocket API, OTA routes, recovery behavior, and static asset packaging compatibility.
- NVS/settings behavior.
- Self-test behavior.
- Display and input behavior.
- Power, voltage, thermal, and fan control behavior.
- Logging, statistics, scoreboard, and live telemetry behavior.
- Firmware image packaging and USB flashing ergonomics.

Device-user parity does not require preserving C module boundaries, FreeRTOS task layout, or internal implementation quirks unless those details affect observable behavior.

## First Hardware Target

Prioritize Bitaxe Gamma 601 with BM1370 ASIC for early hardware bring-up, smoke tests, and flash ergonomics. The user also has access to a Bitaxe 205, but 601 is preferred.

Reference breadcrumbs:

- `reference/esp-miner/config-601.cvs`
- `reference/esp-miner/main/device_config.h`
- `reference/esp-miner/components/asic/bm1370.c`
- `reference/esp-miner/components/asic/include/bm1370.h`

At inspection time, upstream `config-601.cvs` defines:

- `devicemodel`: `gamma`
- `boardversion`: `601`
- `asicmodel`: `BM1370`
- `asicfrequency`: `525`
- `asicvoltage`: `1150`

## Accepted Architecture And Workflow Decisions

- The repo is the new Rust firmware monorepo, not a Rust branch inside an upstream ESP-Miner fork.
- Use ESP-IDF Rust bindings for the first production stack.
- Bazel is the canonical automation graph for build, test, package, flash, and release-shaped workflows.
- `just` is the ergonomic command surface for humans.
- Keep upstream ESP-Miner read-only at `reference/esp-miner`.
- The parity checklist is audit evidence, not a loose task list.
- Rust source should use reference breadcrumbs at module and behavior boundaries.
- The initial monorepo layout separates ESP-IDF firmware from pure Rust crates.
- AxeOS scope is API and asset compatibility first, not an initial Angular UI rewrite.
- USB flashing is a first-class `just` workflow.
- Parity verification requires explicit evidence.
- Original project code is MIT-first where legally possible, with GPL guardrails for upstream-derived work.

Decision records:

- `docs/adr/0001-device-user-parity.md`
- `docs/adr/0002-rust-firmware-monorepo.md`
- `docs/adr/0003-esp-idf-rust-production-stack.md`
- `docs/adr/0004-bazel-automation-with-just-wrapper.md`
- `docs/adr/0005-read-only-reference-implementation.md`
- `docs/adr/0006-parity-checklist-as-audit-evidence.md`
- `docs/adr/0007-prioritize-gamma-601-bm1370-bring-up.md`
- `docs/adr/0008-reference-breadcrumb-comments.md`
- `docs/adr/0009-monorepo-package-layout.md`
- `docs/adr/0010-axeos-api-and-asset-compatibility.md`
- `docs/adr/0011-usb-flashing-ergonomics.md`
- `docs/adr/0012-parity-verification-evidence.md`
- `docs/adr/0013-mit-first-with-gpl-guardrails.md`

## Seed Repository Layout

Use this as the starting monorepo shape:

```text
/
  MODULE.bazel
  Justfile
  Cargo.toml
  rust-toolchain.toml
  PROVENANCE.md
  reference/
    esp-miner/
  firmware/
    bitaxe/
  crates/
    bitaxe-core/
    bitaxe-asic/
    bitaxe-stratum/
    bitaxe-config/
    bitaxe-api/
    bitaxe-test-support/
  tools/
    flash/
    parity/
    xtask/
  scripts/
    verify-reference-clean.sh
  docs/
    project/
    parity/
    adr/
```

The Rust implementation should prefer a functional core with thin imperative shells. Put pure protocol, config, job, hashing, API model, and parity logic in crates that can be unit tested without hardware. Keep ESP-IDF, FreeRTOS, Wi-Fi, NVS, SPIFFS, OTA, serial, GPIO, I2C, and hardware side effects in the firmware app or adapters.

## Command Surface

`just` commands should wrap Bazel targets where practical:

```bash
just build
just test
just package
just flash board=601
just flash board=601 port=/dev/cu.usbmodem...
just monitor port=/dev/cu.usbmodem...
just flash-monitor board=601 port=/dev/cu.usbmodem...
just verify-reference
just parity
```

Flashing behavior:

- `board=601` maps to Gamma 601 BM1370 behavior.
- If `port` is omitted, discover likely ESP serial ports.
- If discovery is ambiguous, fail with a clear chooser-style message.
- Build/package first by default unless an explicit image path is supplied.
- Print the underlying flashing command for debugging.
- Do not claim non-601 boards are hardware-verified until evidence exists.

## First Milestone

The first milestone should be project foundation plus Gamma 601 bring-up path, not full firmware parity.

It should produce:

- Bazel, Cargo, and ESP-IDF Rust skeleton.
- Pinned read-only `reference/esp-miner` submodule.
- Reference-clean verification.
- Initial Rust crate layout.
- Justfile commands for build, test, package, flash, monitor, flash-monitor, reference verification, and parity reporting.
- Seed parity checklist and helper tooling.
- Provenance and licensing docs.
- A minimal firmware image that can boot and log on Gamma 601.

Mining does not need to work in the first milestone. Full device-user parity remains the overall project goal and should be split into later roadmap phases.

## Parity Checklist Requirements

Create and maintain `docs/parity/checklist.md` as the audit source of truth. Each checklist item should include:

- Observable behavior surface.
- Reference breadcrumb path or function.
- Rust-owned implementation pointer when known.
- Status: `not-started`, `in-progress`, `implemented`, `verified`, or `deferred`.
- Verification evidence.
- Notes about behavior that must match upstream.

Evidence types:

- `unit`: pure Rust unit tests compare behavior to reference-derived fixtures.
- `golden`: generated output matches checked-in golden data derived from upstream behavior.
- `api-compare`: Rust firmware response matches upstream OpenAPI/schema or captured upstream response.
- `hardware-smoke`: behavior observed on Gamma 601 hardware, with command/log captured.
- `hardware-regression`: repeatable hardware test or scripted probe passes.
- `deferred`: accepted gap with reason and owner.

Safety-critical and hardware-control surfaces require hardware evidence before `verified`: voltage, fan, thermal, power, and ASIC initialization.

## Breadcrumb Policy

Rust modules that port reference behavior should include module-level breadcrumbs:

```rust
// Reference: reference/esp-miner/components/asic/bm1370.c
// Parity: docs/parity/checklist.md#asic-bm1370-initialization
```

Use narrower breadcrumbs for tricky functions, constants, register sequences, protocol edge cases, or API response details:

```rust
// Reference: reference/esp-miner/components/asic/bm1370.c:BM1370_init
```

Do not add line-by-line translation comments.

## Licensing And Provenance

Default posture:

- Keep original project scaffolding, docs, scripts, and independently authored Rust code MIT-licensed where possible.
- Treat `reference/esp-miner` as GPL-3.0 upstream evidence.
- Track behavior provenance through breadcrumbs and parity docs.
- Mark intentionally ported GPL-covered source expression as GPL-3.0-compatible rather than MIT-only.
- Review distributed firmware image licensing before release.
- Keep third-party dependency licenses visible in release prep.

See `PROVENANCE.md`.

## Out Of Scope For Initial Project Setup

- Rewriting the Angular AxeOS UI.
- Mining parity in the first milestone.
- Bare-metal `no_std` firmware as the first production stack.
- Modifying upstream ESP-Miner files inside `reference/esp-miner`.
- Claiming all board configs are hardware-verified before evidence exists.
- Treating a feature as verified without recorded evidence.

## GSD Configuration Recommendations

When `$gsd-new-project --auto` asks configuration questions, use these defaults unless there is a reason to override:

- Granularity: Coarse.
- Execution: Parallel.
- Git tracking: Yes.
- Research before planning: Yes.
- Plan check: Yes.
- Verifier: Yes.
- AI models: Balanced.

## Reference Baseline

During prerequisite discovery, upstream ESP-Miner was inspected at commit `c1915b0a63bfabebdb95a515cedfee05146c1d50` on 2026-06-20. When creating the submodule, pin the selected upstream commit explicitly and document any later reference refresh.
