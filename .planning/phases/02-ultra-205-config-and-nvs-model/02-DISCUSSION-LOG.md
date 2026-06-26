# Phase 2: Ultra 205 Config And NVS Model - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-06-26T15:47:58.690Z
**Phase:** 2-Ultra 205 Config And NVS Model
**Mode:** Yolo
**Areas discussed:** Ultra 205 defaults, board and ASIC identity, NVS schema, validation, persistence boundary, golden fixtures and evidence

---

## Ultra 205 Defaults

| Option | Description | Selected |
| --- | --- | --- |
| Reference-derived fixtures | Use `config-205.cvs` as the source of truth and test exact Rust defaults against it. | yes |
| Hard-coded constants only | Keep constants in Rust without a fixture/provenance layer. | no |
| Defer defaults | Leave defaults until API or firmware adapter work. | no |

**User's choice:** Auto-selected reference-derived fixtures.
**Notes:** This directly satisfies CFG-01 and avoids guessed constants. The existing Phase 1 `Phase1BoardSelection` already includes a subset of these values and should be expanded or replaced by the Phase 2 model.

---

## Board And ASIC Identity

| Option | Description | Selected |
| --- | --- | --- |
| Ultra 205 active plus full scoped catalog | Model all upstream board/ASIC entries but mark non-205 paths unverified or deferred. | yes |
| Ultra 205 only | Model only board 205 and leave other entries invisible. | no |
| All boards verified by table presence | Treat the catalog as support evidence for every board. | no |

**User's choice:** Auto-selected Ultra 205 active plus full scoped catalog.
**Notes:** ADR-0014 requires Ultra 205/BM1366 as the first evidence-backed path. Other entries remain scoped for parity and future planning, not verified hardware support.

---

## NVS Schema

| Option | Description | Selected |
| --- | --- | --- |
| Typed schema metadata | Represent NVS key name, stored type, default, REST name, min/max, array behavior, and source breadcrumb. | yes |
| Loose map of strings | Store settings as raw string keys and parse in call sites. | no |
| Firmware-only NVS | Put schema directly in the ESP-IDF firmware adapter. | no |

**User's choice:** Auto-selected typed schema metadata.
**Notes:** Upstream compatibility depends on exact NVS keys, REST names, legacy migrations, bool-as-u16 behavior, and float-as-string persistence. A typed schema gives later API and firmware code one source of truth.

---

## Validation

| Option | Description | Selected |
| --- | --- | --- |
| Parse at boundaries | Convert raw user/API/NVS values into typed domain values before business logic. | yes |
| Validate at every use site | Pass primitives around and re-check them when needed. | no |
| Trust upstream inputs | Accept values as long as upstream uses similar names. | no |

**User's choice:** Auto-selected parse at boundaries.
**Notes:** This follows the repo standards and keeps invalid frequencies, voltages, temperatures, fan duty values, ports, hostnames, and credentials from reaching later hardware or API code as unchecked primitives.

---

## Persistence Boundary

| Option | Description | Selected |
| --- | --- | --- |
| Pure snapshot plus adapter later | Build host-testable default/load/update/reload semantics now; defer ESP-IDF NVS side effects to an adapter. | yes |
| Direct ESP-IDF NVS now | Implement live NVS reads/writes before the pure model is stable. | no |
| No persistence model | Only expose defaults and leave reload behavior for a later phase. | no |

**User's choice:** Auto-selected pure snapshot plus adapter later.
**Notes:** Phase 2 can prove persistence semantics through host tests without writing flash. Real reboot reload smoke should be added when the firmware storage adapter exists.

---

## Golden Fixtures And Evidence

| Option | Description | Selected |
| --- | --- | --- |
| Fixture-backed parity evidence | Add machine-readable fixtures with provenance and update checklist rows according to evidence type. | yes |
| Unit tests without provenance | Test Rust values without source breadcrumbs. | no |
| Implementation-only status | Mark parity rows complete because code exists. | no |

**User's choice:** Auto-selected fixture-backed parity evidence.
**Notes:** ADR-0012 allows pure config parity to be verified by unit/golden evidence, but hardware-control effects remain unverified without hardware smoke or regression evidence.

---

## the agent's Discretion

- Exact module split.
- Newtype and enum names.
- Fixture file format.
- Error enum names.
- Host-testable persistence abstraction shape.
- Plan granularity.

## Deferred Ideas

- Firmware ESP-IDF NVS adapter and real reboot smoke.
- Settings HTTP PATCH handlers and API compatibility.
- BM1366 initialization, voltage/fan/thermal/power effects, and mining behavior.
- OTA/filesystem/release behavior.
- Non-205 hardware verification.
