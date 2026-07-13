# Phase 10: Route Manifest And API Compare Unification - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `10-CONTEXT.md`; this log preserves the alternatives considered.

**Date:** 2026-06-29T15:53:51.833Z
**Phase:** 10-route-manifest-and-api-compare-unification
**Mode:** Yolo
**Areas discussed:** Manifest ownership, Firmware route reporting, API compare regression behavior, Parity evidence classification

## Manifest Ownership

| Option | Description | Selected |
| --- | --- | --- |
| Typed Phase 7 superset manifest in `bitaxe-api`, with Phase 5/Phase 7 projection helpers | Keeps functional-core ownership in pure Rust, lets firmware and tools consume typed route data, and preserves Phase 5 as a compatibility view. | Yes |
| Data-first JSON manifest with generated Rust constants | Creates a human-readable manifest artifact but adds code generation and Bazel/Cargo synchronization risk. | No |
| Drive firmware handler registration from the manifest | Strongest drift prevention but requires a larger ESP-IDF adapter refactor and careful wildcard/WebSocket ordering. | No |

**Auto-selected choice:** Typed Phase 7 superset manifest in `bitaxe-api`, with Phase 5/Phase 7 projection helpers.
**Notes:** Advisor research and local code scouting both showed `phase07_routes()` already exists and models Phase 7 ownership. The chosen path keeps the phase tightly scoped and avoids introducing generated artifacts.

## Firmware Route Reporting

| Option | Description | Selected |
| --- | --- | --- |
| Typed Phase 7 manifest API | Build on `phase07_routes()`, make firmware route logs/reporting consume typed ownership, and keep handlers as ESP-IDF adapters. | Yes |
| Manifest-driven firmware registration table | Couples registration and manifest more tightly but requires larger firmware handler changes. | No |
| Checked-in Phase 7 JSON manifest with generated Rust constants | Auditable but adds generator drift and build graph complexity. | No |

**Auto-selected choice:** Typed Phase 7 manifest API.
**Notes:** The known bug is `http_api.rs` logging `phase05_routes().len()`. The firmware should report the Phase 7 manifest while keeping explicit registration order and handler functions.

## API Compare Regression Behavior

| Option | Description | Selected |
| --- | --- | --- |
| Consume `phase07_routes()` directly in `api_compare` | Smallest route-source change; preserves Phase 5 schema/captured-response checks and catches Phase 7 route drift. | Yes |
| Promote a typed Phase 7 route manifest struct with evidence policy | Encodes policy more formally but adds more migration churn. | Partial |
| Make an external JSON Phase 7 manifest canonical and generate/parse Rust views | Human-editable but less idiomatic for const firmware route data and higher build risk. | No |

**Auto-selected choice:** Consume `phase07_routes()` directly for route checks, with typed policy helpers where needed.
**Notes:** Keep existing OpenAPI and captured-response evidence. Move route presence/kind checks to the Phase 7 manifest and add failures for missing, downgraded, or overclaimed routes.

## Parity Evidence Classification

| Option | Description | Selected |
| --- | --- | --- |
| Add a dedicated manifest/tooling checklist row | Queryable but may duplicate API-004 unless narrowly worded. | Optional |
| Update existing rows only with scoped notes | Minimal, preserves existing evidence labels and statuses. | Yes |
| Add a new `manifest-compare` evidence type | Very explicit but expands taxonomy and overlaps `api-compare`. | No |
| Use a Phase 10 claim-boundary evidence matrix | Makes non-claims explicit and fits the existing release-gate evidence style. | Yes |

**Auto-selected choice:** Update existing rows with scoped notes and add a Phase 10 evidence record with an explicit claim-boundary matrix.
**Notes:** Phase 10 can prove manifest/tooling alignment with unit/workflow/api-compare evidence. It must not promote live HTTP, static, recovery, OTA, rollback, release, safety, or mining behavior.

## the agent's Discretion

- Exact helper names, fixture names, report wording, and checklist row wording.
- Whether route policy helpers live entirely in `tools/parity` or partly in `crates/bitaxe-api`.
- Whether a narrow dedicated checklist row is needed after implementation.

## Deferred Ideas

- Manifest-driven firmware handler registration.
- Generated JSON-to-Rust route manifest.
- Live HTTP/static/recovery/OTA/rollback/release evidence.
