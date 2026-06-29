# Phase 10: Route Manifest And API Compare Unification - Research

**Researched:** 2026-06-29  
**Domain:** Rust route manifest ownership, firmware route reporting, API/static/OTA parity tooling, evidence guardrails  
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

Source: copied from `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`]

### Locked Decisions

### Manifest Ownership

- **D-01:** Treat `crates/bitaxe-api/src/route_shell.rs` as the pure route manifest owner for Phase 10.
- **D-02:** Use the existing Phase 7 superset route list as the canonical manifest direction. `phase07_routes()` should be the source consumed by firmware route reporting and compare tooling.
- **D-03:** Preserve `phase05_routes()` as a compatibility projection or phase-specific view for Phase 5 API schema and captured-response evidence. Do not let the Phase 5 view remain the authority for Phase 7 static, recovery, firmware OTA, or OTAWWW gap route ownership.
- **D-04:** Do not add a checked-in JSON manifest plus Rust code generation in this phase. That would add build graph and generated-output drift without enough value over the existing typed route table.
- **D-05:** Do not rewrite firmware handler registration to be fully manifest-driven in this phase. Keep handlers as explicit ESP-IDF adapters and prove reporting/compare alignment through typed helpers and tests.

### Firmware Route Reporting

- **D-06:** Firmware route count and route-report logging should consume the Phase 7 route manifest, not `phase05_routes().len()`.
- **D-07:** Reporting should include or make testable the ownership of Phase 7 routes: firmware OTA, OTAWWW gap, recovery, and static wildcard.
- **D-08:** Firmware adapters may keep manual registration order for ESP-IDF correctness: explicit recovery, API routes, update routes, WebSockets, unknown API fallbacks, and static wildcard last.
- **D-09:** Add host-testable route reporting helpers in the pure API crate where practical so firmware only formats or logs manifest-derived data.
- **D-10:** If actual handler registration cannot be mechanically derived from the manifest in this phase, add regression tests that compare the declared Phase 7 manifest against the adapter/reporting contract.

### API Compare Regression Behavior

- **D-11:** `tools/parity api-compare` should consume the same Phase 7 manifest data for route presence and classification checks.
- **D-12:** Keep Phase 5 schema and captured-response checks. Phase 10 should not discard the existing OpenAPI and fixture evidence for `/api/system/*`, logs, WebSockets, commands, and settings.
- **D-13:** Move route presence and ownership assertions away from duplicate Phase 5 JSON route lists where practical. JSON fixtures may still describe schema, captured response, static usage, and evidence policy, but should not be the independent route source of truth.
- **D-14:** Compare tooling must fail when a required Phase 7 route is missing from the Rust manifest.
- **D-15:** Compare tooling must fail when a required Phase 7 route is downgraded, for example firmware OTA losing `FirmwareUpdate`, OTAWWW losing `AxeOsStaticUpdateGap`, `/recovery` losing `Recovery`, or `/*` losing `StaticFiles`.
- **D-16:** Compare tooling must fail when fixture or policy data incorrectly classifies static, recovery, firmware OTA, OTAWWW, or release-sensitive routes as verified from unit, workflow, package, or API compare evidence alone.

### Parity Evidence Classification

- **D-17:** Record Phase 10 evidence as tooling and manifest evidence only, using existing evidence labels such as `unit`, `workflow`, and `api-compare`.
- **D-18:** Prefer updating existing relevant checklist rows and adding a Phase 10 evidence record over inventing a new evidence taxonomy such as `manifest-compare`.
- **D-19:** If a dedicated checklist row is needed to make the manifest/tooling claim queryable, word it narrowly so it cannot be confused with live HTTP route verification.
- **D-20:** Keep `API-004`, `API-007`, `API-008`, `FS-001`, `OTA-001`, `OTA-002`, `REL-001`, `REL-002`, and `REL-003` below live/release verification where their current blockers still apply.
- **D-21:** Phase 10 evidence should include an explicit claim-boundary matrix stating which surfaces were proven by manifest/tooling checks and which live surfaces remain not run or Phase 13-owned.

### Testing And Verification Shape

- **D-22:** Unit tests should prove `phase07_routes()` contains the Phase 7-owned routes with the expected method and kind.
- **D-23:** Unit tests should prove route reporting derives from the Phase 7 manifest count and ownership metadata.
- **D-24:** API compare tests should cover missing route failure, route kind downgrade failure, and verified-overclaim failure for static/recovery/update routes.
- **D-25:** Run `just parity` after implementation and record the command result in Phase 10 evidence.

### the agent's Discretion

The agent may choose exact helper names, fixture names, report wording, checklist row wording, and whether route policy lives entirely in `tools/parity/src/api_compare.rs` or partly in `crates/bitaxe-api/src/route_shell.rs`. Those choices must preserve the functional-core/imperative-shell boundary, keep firmware handlers thin, keep the upstream reference read-only, use typed data instead of ad hoc string scanning where practical, and avoid claiming live HTTP or OTA behavior.

### Deferred Ideas (OUT OF SCOPE)

- Fully manifest-driven firmware route registration is deferred unless a later phase accepts the larger ESP-IDF adapter refactor.
- A generated JSON-to-Rust route manifest pipeline is deferred unless release governance later needs a non-Rust canonical route artifact.
- Live HTTP, static asset serving, `/recovery`, valid firmware OTA, invalid OTA rejection, OTAWWW behavior, rollback, erase, failed-update recovery, and interrupted-update recovery evidence remain Phase 13 scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| API-09 | Static AxeOS assets and recovery page behavior remain compatible enough for device administration without requiring an Angular rewrite in V1. [VERIFIED: `.planning/REQUIREMENTS.md`] | Use `phase07_routes()` and `axeos-route-usage.json` to keep `/recovery` and `/*` visible as Phase 7-owned route surfaces while preserving the existing no-Angular-rewrite boundary. [VERIFIED: `crates/bitaxe-api/src/route_shell.rs`; VERIFIED: `tools/parity/fixtures/api/axeos-route-usage.json`; VERIFIED: `docs/adr/0010-axeos-api-and-asset-compatibility.md`] |
| API-10 | API compare fixtures prove Rust responses match the upstream schema or captured upstream responses for representative success and error cases. [VERIFIED: `.planning/REQUIREMENTS.md`] | Preserve `schema_routes` and `captured_response_checks` in `phase05-required-routes.json`; move route presence/kind checks to the Phase 7 typed manifest. [VERIFIED: `tools/parity/fixtures/api/phase05-required-routes.json`; VERIFIED: `tools/parity/src/api_compare.rs`] |
| REL-01 | Partition layout, filesystem layout, SPIFFS/static assets, and recovery assets support upstream-style flash and administration flows. [VERIFIED: `.planning/REQUIREMENTS.md`] | Keep route tooling from marking filesystem/static/recovery as live-verified; record manifest/tooling evidence only while checklist rows remain below verified until live static and recovery smoke exists. [VERIFIED: `docs/parity/checklist.md`; VERIFIED: `docs/parity/evidence/phase-08-ultra-205-release-gate.md`] |
| REL-02 | Firmware OTA route behavior accepts, rejects, applies, logs, and recovers from updates with upstream-compatible observable behavior. [VERIFIED: `.planning/REQUIREMENTS.md`] | Require `POST /api/system/OTA` to remain `RouteKind::FirmwareUpdate` in the Phase 7 manifest, but do not claim valid upload, invalid image rejection, reboot, rollback, or recovery evidence in this phase. [VERIFIED: `crates/bitaxe-api/src/route_shell.rs`; VERIFIED: `docs/parity/checklist.md`] |
| REL-03 | OTAWWW/static-asset update behavior is implemented or explicitly reported as a V1 parity gap with evidence and owner. [VERIFIED: `.planning/REQUIREMENTS.md`] | Require `POST /api/system/OTAWWW` to remain `RouteKind::AxeOsStaticUpdateGap`, and keep OTAWWW classified as an explicit gap unless later hardware-regression/interrupted-update evidence exists. [VERIFIED: `crates/bitaxe-api/src/route_shell.rs`; VERIFIED: `.planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md`; VERIFIED: `docs/parity/checklist.md`] |
| EVD-01 | Each V1 parity surface records observable behavior, reference breadcrumb, Rust pointer, status, evidence, and notes. [VERIFIED: `.planning/REQUIREMENTS.md`] | Update checklist/evidence with a narrow Phase 10 manifest/tooling claim and explicit non-claims for live HTTP/static/recovery/OTA/release behavior. [VERIFIED: `docs/adr/0006-parity-checklist-as-audit-evidence.md`; VERIFIED: `docs/adr/0012-parity-verification-evidence.md`] |
</phase_requirements>

## Summary

Phase 10 should be planned as a narrow pure-data consumer refactor: keep `crates/bitaxe-api/src/route_shell.rs` as the canonical typed route table, make firmware reporting consume `phase07_routes()`, and make `tools/parity api-compare` use the same Phase 7 manifest for route presence and kind checks. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: `crates/bitaxe-api/src/route_shell.rs`; VERIFIED: `firmware/bitaxe/src/http_api.rs`; VERIFIED: `tools/parity/src/api_compare.rs`]

The current drift is concrete: `phase07_routes()` already contains `FirmwareUpdate`, `AxeOsStaticUpdateGap`, `Recovery`, and `StaticFiles`, but firmware startup logging still imports `phase05_routes()` and logs `phase05_routes().len()`, and API compare still imports `phase05_routes()` for its Rust route set. [VERIFIED: `crates/bitaxe-api/src/route_shell.rs`; VERIFIED: `firmware/bitaxe/src/http_api.rs`; VERIFIED: `tools/parity/src/api_compare.rs`]

**Primary recommendation:** Add small host-testable Phase 7 route-report and route-kind policy helpers around `phase07_routes()`, inject the route slice into API compare validation for regression tests, preserve Phase 5 schema/captured-response fixture checks, and document Phase 10 as manifest/tooling evidence only. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: `tools/parity/src/api_compare.rs`; VERIFIED: `docs/parity/evidence/phase-05-axeos-api-logs-and-telemetry.md`]

## Project Constraints (from AGENTS.md)

- Read `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/index.md`, and relevant standards pages before planning or implementation. [VERIFIED: `AGENTS.md`; VERIFIED: `AGENTS.bright-builds.md`; VERIFIED: `standards/index.md`]
- Keep upstream ESP-Miner read-only at `reference/esp-miner`; use it as behavioral evidence rather than a workspace for project edits. [VERIFIED: `AGENTS.md`; VERIFIED: `docs/adr/0005-read-only-reference-implementation.md`; VERIFIED: `PROVENANCE.md`]
- Preserve functional core / imperative shell: pure logic belongs in crates, while ESP-IDF, HTTP serving, SPIFFS, OTA, FreeRTOS, logging, and task orchestration stay in firmware adapters. [VERIFIED: `AGENTS.md`; VERIFIED: `standards/core/architecture.md`; VERIFIED: `standards/languages/rust.md`; VERIFIED: `docs/adr/0003-esp-idf-rust-production-stack.md`]
- Use Bazel as the canonical automation graph and `just` as the human command surface. [VERIFIED: `AGENTS.md`; VERIFIED: `.planning/PROJECT.md`; VERIFIED: `Justfile`]
- For Rust commits, run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` before committing when feasible; run repo-native verification as the relevant gate. [VERIFIED: `AGENTS.md`; VERIFIED: `standards/core/verification.md`; VERIFIED: `standards/languages/rust.md`]
- Unit-test pure and business logic, keep tests focused on one concern, and use Arrange/Act/Assert comments when the structure is not trivially obvious. [VERIFIED: `AGENTS.md`; VERIFIED: `standards/core/testing.md`; VERIFIED: `standards/languages/rust.md`]
- Prefer typed data and typed parsers over ad hoc string scanning for structured project behavior. [VERIFIED: `AGENTS.md`; VERIFIED: `standards/core/architecture.md`; VERIFIED: `standards/languages/rust.md`]
- Do not use standalone `---` body separators in GSD or other frontmatter-parsed Markdown files. [VERIFIED: `AGENTS.md`]
- No repo-local project skills were found under `.claude/skills` or `.agents/skills`. [VERIFIED: command `find .claude/skills .agents/skills -maxdepth 2 -name SKILL.md -print`]

## Standard Stack

### Core

| Library / Surface | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| `crates/bitaxe-api::route_shell` | local workspace crate, Rust 2021 | Owns `AxeosRoute`, `RouteMethod`, `RouteKind`, `phase05_routes()`, and `phase07_routes()`. [VERIFIED: `crates/bitaxe-api/src/route_shell.rs`; VERIFIED: `Cargo.toml`] | It is the locked pure manifest owner for this phase. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`] |
| `tools/parity` | local workspace crate, Rust 2021 | Owns API compare, checklist parsing, and invalid verified-claim guards. [VERIFIED: `tools/parity/src/api_compare.rs`; VERIFIED: `tools/parity/src/main.rs`; VERIFIED: `tools/parity/Cargo.toml`] | It is the existing parity command surface for API compare and checklist validation. [VERIFIED: `tools/parity/src/main.rs`; VERIFIED: `Justfile`] |
| Bazel `rust_test` targets | Bazel 9.1.1 available locally | Runs `//crates/bitaxe-api:tests` and `//tools/parity:tests`. [VERIFIED: command `bazel --version`; VERIFIED: `crates/bitaxe-api/BUILD.bazel`; VERIFIED: `tools/parity/BUILD.bazel`] | Existing repo verification and `just test` route through Bazel. [VERIFIED: `Justfile`] |
| `just` | 1.48.0 available locally | Human entrypoint for `test`, `parity`, and other repo workflows. [VERIFIED: command `just --version`; VERIFIED: `Justfile`] | Repo-local guidance and project constraints define `just` as the human command surface. [VERIFIED: `AGENTS.md`; VERIFIED: `.planning/PROJECT.md`] |

### Supporting

| Library / Surface | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| `serde` / `serde_json` | `serde 1.0.228`, `serde_json 1.0.150` from workspace deps | Parse existing JSON fixtures for API compare. [VERIFIED: `Cargo.toml`; VERIFIED: `tools/parity/src/api_compare.rs`] | Keep fixture parsing typed; do not replace with string scans. [VERIFIED: `standards/core/architecture.md`; VERIFIED: `tools/parity/src/api_compare.rs`] |
| `camino` | `1.2.3` from workspace deps | UTF-8 workspace paths in parity tooling. [VERIFIED: `Cargo.toml`; VERIFIED: `tools/parity/src/main.rs`] | Keep existing path style in `tools/parity`. [VERIFIED: `tools/parity/src/main.rs`] |
| `anyhow` | `1.0.102` from workspace deps | CLI/tool error handling. [VERIFIED: `Cargo.toml`; VERIFIED: `tools/parity/src/api_compare.rs`; VERIFIED: `tools/parity/src/main.rs`] | Continue existing host-tool error style. [VERIFIED: `tools/parity/src/api_compare.rs`; VERIFIED: `tools/parity/src/main.rs`] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| `phase07_routes()` typed Rust manifest | Checked-in JSON manifest plus codegen | Explicitly deferred because it adds generated-output and build-graph drift in this phase. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`] |
| Explicit firmware handler registration plus manifest-backed reporting | Fully manifest-driven handler registration | Explicitly deferred because it is a larger ESP-IDF adapter refactor and has ordering risk around WebSockets, unknown API fallbacks, and wildcard static handling. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: `firmware/bitaxe/src/http_api.rs`] |
| Existing narrow OpenAPI text checks | Add a YAML/OpenAPI parser dependency | Phase 5 deliberately kept OpenAPI checks narrow and dependency-free; Phase 10 does not need new schema parsing. [VERIFIED: `.planning/STATE.md`; VERIFIED: `tools/parity/src/api_compare.rs`] |

**Installation:** No new packages should be installed for Phase 10. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: `Cargo.toml`; VERIFIED: `tools/parity/BUILD.bazel`]

**Version verification:** This phase uses existing workspace dependencies and locally verified tools rather than adding registry packages; Bazel 9.1.1, `just` 1.48.0, Cargo 1.88.0-nightly, Rust 1.88.0-nightly, `rg` 15.1.0, and Git 2.53.0 are available. [VERIFIED: commands `bazel --version`, `just --version`, `cargo --version`, `rustc --version`, `rg --version`, `git --version`]

## Architecture Patterns

### Recommended Project Structure

```text
crates/bitaxe-api/src/
├── route_shell.rs        # Phase 5 and Phase 7 route manifest, route-kind labels, and route report helpers
└── lib.rs                # Re-export helpers consumed by firmware and tools

firmware/bitaxe/src/
├── http_api.rs           # Thin ESP-IDF HTTP/WebSocket adapter and manifest-derived logging
├── static_files.rs       # Explicit recovery and wildcard static adapter
└── filesystem.rs         # SPIFFS status adapter used by static/recovery behavior

tools/parity/src/
├── api_compare.rs        # Pure API/static/OTA compare checks with injected route manifest for tests
└── main.rs               # CLI file loading, checklist report, and invalid verified-claim guards

tools/parity/fixtures/api/
├── phase05-required-routes.json  # Schema and captured-response checks, not Phase 7 route authority
└── axeos-route-usage.json        # AxeOS static usage and evidence-policy fixture

docs/parity/
├── checklist.md          # Narrow manifest/tooling evidence updates without live overclaim
└── evidence/phase-10-route-manifest-and-api-compare-unification.md
```

The structure above matches existing crate, firmware, tool, fixture, checklist, and evidence locations. [VERIFIED: `crates/bitaxe-api/BUILD.bazel`; VERIFIED: `firmware/bitaxe/src/http_api.rs`; VERIFIED: `tools/parity/BUILD.bazel`; VERIFIED: `tools/parity/fixtures/api/phase05-required-routes.json`; VERIFIED: `docs/parity/checklist.md`]

### Pattern 1: Route Manifest As Pure Data

**What:** Keep `AxeosRoute { path, method, kind }` and `RouteKind` as the typed source for route ownership. [VERIFIED: `crates/bitaxe-api/src/route_shell.rs`]

**When to use:** Use `phase07_routes()` for Phase 7 route presence, ownership, and report summaries; use `phase05_routes()` only for Phase 5 compatibility views around schema and captured-response evidence. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: `crates/bitaxe-api/src/route_shell.rs`; VERIFIED: `tools/parity/fixtures/api/phase05-required-routes.json`]

**Example:**

```rust
// Pattern source: crates/bitaxe-api/src/route_shell.rs and Phase 10 D-06/D-09.
pub struct RouteReport {
    pub total: usize,
    pub firmware_update_routes: usize,
    pub otawww_gap_routes: usize,
    pub recovery_routes: usize,
    pub static_file_routes: usize,
}

pub fn phase07_route_report() -> RouteReport {
    let routes = phase07_routes();
    RouteReport {
        total: routes.len(),
        firmware_update_routes: routes
            .iter()
            .filter(|route| route.kind == RouteKind::FirmwareUpdate)
            .count(),
        otawww_gap_routes: routes
            .iter()
            .filter(|route| route.kind == RouteKind::AxeOsStaticUpdateGap)
            .count(),
        recovery_routes: routes
            .iter()
            .filter(|route| route.kind == RouteKind::Recovery)
            .count(),
        static_file_routes: routes
            .iter()
            .filter(|route| route.kind == RouteKind::StaticFiles)
            .count(),
    }
}
```

The example is a recommended shape, not existing code. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`]

### Pattern 2: Firmware Adapters Format Manifest-Derived Data Only

**What:** Replace the firmware log dependency on `phase05_routes().len()` with a helper derived from `phase07_routes()`, and keep actual handler registration explicit. [VERIFIED: `firmware/bitaxe/src/http_api.rs`; VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`]

**When to use:** Use this in `start_http_api()` after handler registration so startup evidence reports the Phase 7 route manifest without changing ESP-IDF registration order. [VERIFIED: `firmware/bitaxe/src/http_api.rs`; VERIFIED: `firmware/bitaxe/src/static_files.rs`]

**Example:**

```rust
// Pattern source: firmware/bitaxe/src/http_api.rs and route_shell.rs.
let report = phase07_route_report();
log::info!(
    "axeos_api_route_shell=started manifest_routes={} firmware_update_routes={} otawww_gap_routes={} recovery_routes={} static_file_routes={}",
    report.total,
    report.firmware_update_routes,
    report.otawww_gap_routes,
    report.recovery_routes,
    report.static_file_routes
);
```

The log key should avoid implying that every ESP-IDF handler registration is counted if unknown `/api/*` fallbacks remain outside the manifest. [VERIFIED: `firmware/bitaxe/src/http_api.rs`; VERIFIED: `crates/bitaxe-api/src/route_shell.rs`]

### Pattern 3: API Compare With Injected Route Slice For Regression Tests

**What:** Production `run_api_compare` should use `phase07_routes()`, while a private/test helper should accept `&[AxeosRoute]` so missing-route and downgraded-kind regressions can be tested without mutating global constants. [VERIFIED: `tools/parity/src/api_compare.rs`; VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`]

**When to use:** Use route injection for tests that simulate `POST /api/system/OTA` losing `FirmwareUpdate`, `POST /api/system/OTAWWW` losing `AxeOsStaticUpdateGap`, `/recovery` losing `Recovery`, `/*` losing `StaticFiles`, or a route disappearing from the supplied manifest. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: `crates/bitaxe-api/src/route_shell.rs`]

**Example:**

```rust
// Pattern source: tools/parity/src/api_compare.rs and Phase 10 D-14/D-15/D-24.
fn run_api_compare_with_routes(
    request: &ApiCompareRequest<'_>,
    loader: &impl JsonFixtureLoader,
    rust_routes: &[AxeosRoute],
) -> Result<ApiCompareReport> {
    // Existing schema and captured-response checks stay on the Phase 5 fixture.
    // Route presence and kind policy use rust_routes, which production passes as phase07_routes().
    todo!("planner should implement this by refactoring existing run_api_compare")
}
```

The example is a planning pattern; the existing code currently calls `phase05_routes()` in `rust_route_set()`. [VERIFIED: `tools/parity/src/api_compare.rs`]

### Anti-Patterns to Avoid

- **Using `phase05-required-routes.json` as Phase 7 route authority:** It currently lists Phase 5 route paths only and does not include `/recovery` or `/*`. [VERIFIED: `tools/parity/fixtures/api/phase05-required-routes.json`; VERIFIED: `crates/bitaxe-api/src/route_shell.rs`]
- **Deriving both expected and actual route sets from `phase07_routes()` inside the same check:** That cannot fail when a route is accidentally removed unless tests inject a modified route slice or a fixture/policy route still requires the missing route. [VERIFIED: `tools/parity/src/api_compare.rs`; VERIFIED: `tools/parity/fixtures/api/axeos-route-usage.json`]
- **Renaming the log to `registered_routes` while counting only manifest entries:** Firmware also registers unknown `/api/*` fallbacks that are not in `phase07_routes()`, so a manifest count should be clearly labeled as manifest/report data. [VERIFIED: `firmware/bitaxe/src/http_api.rs`; VERIFIED: `crates/bitaxe-api/src/route_shell.rs`]
- **Promoting live route behavior from manifest evidence:** Phase 8 and Phase 9 evidence explicitly leave live HTTP/static/recovery/OTA/rollback behavior unverified. [VERIFIED: `docs/parity/evidence/phase-08-ultra-205-release-gate.md`; VERIFIED: `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md`]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| Canonical route manifest | JSON-to-Rust generator or route parser | `phase07_routes()` plus small typed helpers in `route_shell.rs` | Codegen is explicitly deferred and the typed Rust table already exists. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: `crates/bitaxe-api/src/route_shell.rs`] |
| Firmware handler synchronization | Full manifest-driven ESP-IDF registration | Explicit handlers plus manifest-derived reporting/tests | Registration order matters for recovery, API, update, WebSockets, unknown API fallbacks, and static wildcard. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: `firmware/bitaxe/src/http_api.rs`; VERIFIED: `firmware/bitaxe/src/static_files.rs`] |
| Route drift detection | `rg` scans over source strings | Typed `AxeosRoute` set and `RouteKind` policy checks | Typed data catches method/kind downgrades that plain path scanning can miss. [VERIFIED: `crates/bitaxe-api/src/route_shell.rs`; VERIFIED: `standards/core/architecture.md`] |
| OpenAPI parsing expansion | New YAML/OpenAPI dependency | Existing narrow `openapi_*` helpers for schema/property checks | Phase 10 only changes route authority; schema/captured-response checks should stay as they are. [VERIFIED: `tools/parity/src/api_compare.rs`; VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`] |
| Verified-claim policing | Manual prose review only | Existing `tools/parity/src/main.rs` verified-row guards plus targeted Phase 10 compare policy tests | The parity tool already rejects release/OTA verified overclaims from weak evidence classes. [VERIFIED: `tools/parity/src/main.rs`; VERIFIED: command `just parity`] |

**Key insight:** This phase is about aligning consumers of an existing typed manifest, not inventing a new manifest format. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: `crates/bitaxe-api/src/route_shell.rs`]

## Runtime State Inventory

| Category | Items Found | Action Required |
| --- | --- | --- |
| Stored data | No project-owned runtime databases or SQLite files were found outside generated `.embuild`, `target`, `bazel-*`, and `reference` paths. [VERIFIED: command `find . -path './reference' -prune -o -path './bazel-*' -prune -o -path './target' -prune -o -path './.embuild' -prune -o -type f ...`] Route-related stored source artifacts exist in `tools/parity/fixtures/api/*.json`, `docs/parity/checklist.md`, and evidence logs. [VERIFIED: `tools/parity/fixtures/api/phase05-required-routes.json`; VERIFIED: `tools/parity/fixtures/api/axeos-route-usage.json`; VERIFIED: `docs/parity/checklist.md`] | Code/doc edit only; no data migration. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`] |
| Live service config | None found for route manifest state; `just parity` and `api-compare` read checked-in files and the pinned reference tree. [VERIFIED: `tools/parity/src/main.rs`; VERIFIED: command `just parity`] | None. [VERIFIED: `tools/parity/src/main.rs`] |
| OS-registered state | None found for this route manifest refactor; no `.service`, app-owned `.plist`, or process-manager config files were found outside generated tool state. [VERIFIED: command `find ... -name '*.service' -o -name '*.plist' -o -name 'ecosystem.config.*'`] | None. [VERIFIED: command `find ...`] |
| Secrets/env vars | No repo-owned `.env` or `*.env` files were found outside excluded generated/reference/build paths. [VERIFIED: command `find ... -name '.env' -o -name '*.env'`] | None; Phase 10 should not introduce secrets. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`] |
| Build artifacts | `target`, `.embuild`, and Bazel output directories exist or are referenced; current firmware/evidence logs may still show `registered_routes=15` until rebuilt and refreshed. [VERIFIED: command `find . -maxdepth 3 -type d ...`; VERIFIED: `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening/flash-monitor.log`; VERIFIED: `firmware/bitaxe/src/http_api.rs`] | Rebuild/test affected targets after edits; do not hand-edit generated `.embuild`, `target`, or Bazel outputs. [VERIFIED: `AGENTS.md`; VERIFIED: `Justfile`] |

## Common Pitfalls

### Pitfall 1: Self-Referential Route Checks

**What goes wrong:** A compare check that builds both expected and actual route sets from `phase07_routes()` cannot detect a route removed from `phase07_routes()`. [VERIFIED: `tools/parity/src/api_compare.rs`; VERIFIED: `crates/bitaxe-api/src/route_shell.rs`]

**Why it happens:** The current compare code already has a pattern of `phase05_routes()` as the Rust route source; if Phase 7 expected policy is not supplied separately or test-injected, missing-route tests lose their independent signal. [VERIFIED: `tools/parity/src/api_compare.rs`]

**How to avoid:** Use `phase07_routes()` as production actual data, keep schema/captured-response/static-usage fixtures as policy inputs where appropriate, and add test-only route-slice injection to simulate missing and downgraded routes. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: `tools/parity/fixtures/api/axeos-route-usage.json`]

**Warning signs:** Tests only mutate JSON fixtures and never mutate or inject a Rust route slice. [VERIFIED: `tools/parity/src/api_compare.rs`]

### Pitfall 2: Regressing Phase 5 Evidence While Fixing Phase 7 Route Authority

**What goes wrong:** Replacing the Phase 5 fixture wholesale can drop existing schema and captured-response checks for `/api/system/*`, logs, WebSockets, commands, and settings. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: `tools/parity/fixtures/api/phase05-required-routes.json`]

**Why it happens:** `phase05-required-routes.json` mixes route paths, schema routes, captured response checks, and firmware-smoke metadata. [VERIFIED: `tools/parity/fixtures/api/phase05-required-routes.json`; VERIFIED: `tools/parity/src/api_compare.rs`]

**How to avoid:** Rename or narrow internal meanings if useful, but preserve `schema_routes`, `captured_response_checks`, and fixture loader behavior. [VERIFIED: `tools/parity/src/api_compare.rs`; VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`]

**Warning signs:** API compare checked counts for schema/captured responses unexpectedly fall from the current baseline of 95 schema checks and 47 captured-response checks. [VERIFIED: command `bazel run //tools/parity:report -- api-compare`]

### Pitfall 3: Counting Adapter Fallbacks As Manifest Routes

**What goes wrong:** Firmware logs can imply the manifest count equals every ESP-IDF URI handler registration. [VERIFIED: `firmware/bitaxe/src/http_api.rs`; VERIFIED: `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening/flash-monitor.log`]

**Why it happens:** Firmware registers unknown `/api/*` fallbacks and a static wildcard after API/update/WebSocket routes, while `phase07_routes()` models the AxeOS route surfaces rather than every fallback handler. [VERIFIED: `firmware/bitaxe/src/http_api.rs`; VERIFIED: `crates/bitaxe-api/src/route_shell.rs`]

**How to avoid:** Label report output as manifest-derived route reporting and include kind breakdowns for Phase 7-owned routes. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`]

**Warning signs:** Evidence claims "registered 17 routes" while ESP-IDF logs still show separate `/api/*` fallback registrations. [VERIFIED: `firmware/bitaxe/src/http_api.rs`; VERIFIED: `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening/flash-monitor.log`]

### Pitfall 4: Verified-Overclaim Through Docs

**What goes wrong:** A checklist row can be promoted to `verified` from unit/workflow/package/API compare evidence even though live static, recovery, OTA, rollback, or interrupted-update behavior remains unrun. [VERIFIED: `tools/parity/src/main.rs`; VERIFIED: `docs/parity/checklist.md`]

**Why it happens:** Manifest/tooling evidence proves route alignment, not live device behavior. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md`]

**How to avoid:** Keep Phase 10 checklist/evidence wording narrow and rely on `just parity` verified-row guards after doc updates. [VERIFIED: `tools/parity/src/main.rs`; VERIFIED: command `just parity`]

**Warning signs:** `API-007`, `FS-001`, `OTA-001`, `OTA-002`, `REL-001`, `REL-002`, or `REL-003` move to `verified` without hardware/release evidence. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: `tools/parity/src/main.rs`]

## Code Examples

Verified patterns from existing code and recommended Phase 10 extensions:

### Existing Phase 7 Manifest Shape

```rust
// Source: crates/bitaxe-api/src/route_shell.rs
axeos_route!("/api/system/OTA", Post, RouteKind::FirmwareUpdate),
axeos_route!("/api/system/OTAWWW", Post, RouteKind::AxeOsStaticUpdateGap),
axeos_route!("/recovery", Get, RouteKind::Recovery),
axeos_route!("/*", Get, RouteKind::StaticFiles),
```

Those entries already exist in `PHASE07_ROUTES`. [VERIFIED: `crates/bitaxe-api/src/route_shell.rs`]

### Existing Drift To Replace

```rust
// Source: firmware/bitaxe/src/http_api.rs
log::info!(
    "axeos_api_route_shell=started registered_routes={}",
    phase05_routes().len()
);
```

The code above is the firmware reporting drift Phase 10 should replace with Phase 7 manifest-derived reporting. [VERIFIED: `firmware/bitaxe/src/http_api.rs`; VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`]

### Existing Compare Source To Refactor

```rust
// Source: tools/parity/src/api_compare.rs
fn rust_route_set() -> BTreeSet<String> {
    phase05_routes()
        .iter()
        .map(|route| route_key(route_method_label(route.method), route.path))
        .collect()
}
```

The production route set should move from `phase05_routes()` to a Phase 7 route slice while preserving Phase 5 schema and captured-response checks. [VERIFIED: `tools/parity/src/api_compare.rs`; VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
| --- | --- | --- | --- |
| Phase 5 route manifest as API compare route authority | Phase 7 superset manifest should be authority for route presence and ownership | Locked for Phase 10 on 2026-06-29 | Plan should refactor consumers, not debate manifest ownership. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`] |
| Firmware startup logs `registered_routes=15` from `phase05_routes().len()` | Firmware should report Phase 7 manifest count and ownership metadata | Phase 10 gap closure | Serial evidence after rebuild should no longer preserve the stale Phase 5 route count. [VERIFIED: `firmware/bitaxe/src/http_api.rs`; VERIFIED: `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening/flash-monitor.log`] |
| API compare fixture duplicates route presence in `phase05-required-routes.json` | Schema/captured-response checks stay in the fixture; Phase 7 route presence/kind checks use typed routes | Phase 10 gap closure | Missing route, downgrade, and overclaim tests become targeted to the route manifest and evidence policy. [VERIFIED: `tools/parity/fixtures/api/phase05-required-routes.json`; VERIFIED: `tools/parity/src/api_compare.rs`] |
| Release rows manually protected by prose alone | `tools/parity/src/main.rs` rejects invalid verified release/OTA/filesystem claims | Already implemented before Phase 10 | Phase 10 docs should run `just parity` after checklist/evidence updates. [VERIFIED: `tools/parity/src/main.rs`; VERIFIED: command `just parity`] |

**Deprecated/outdated:**
- Treating `phase05_routes()` as the authority for Phase 7 static/recovery/OTA/OTAWWW route ownership is outdated for Phase 10 planning. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`]
- Treating `api-compare` as live HTTP evidence is incorrect; Phase 5 evidence says firmware smoke was not run, and Phase 8/9 evidence keeps live HTTP/static/recovery/OTA behavior unverified. [VERIFIED: `docs/parity/evidence/phase-05-axeos-api-logs-and-telemetry.md`; VERIFIED: `docs/parity/evidence/phase-08-ultra-205-release-gate.md`; VERIFIED: `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md`]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |

All claims in this research were verified against repo files or local command output; no `[ASSUMED]` claims are used. [VERIFIED: this research session]

## Open Questions (RESOLVED)

1. **Should Phase 10 add a dedicated checklist row for manifest/tooling alignment?**  
   - What we know: D-19 permits a narrowly worded row only if queryability needs it. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`]  
   - What's unclear: The planner may decide existing rows can express the evidence cleanly. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`]  
   - Recommendation: Prefer updating existing `API-004`, `API-008`, `FS-001`, `OTA-001`, `OTA-002`, and release rows plus adding a Phase 10 evidence record; add a dedicated row only if the manifest/tooling claim would otherwise be hard to query. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: `docs/parity/checklist.md`]
   - RESOLVED: Plan 03 will update existing relevant checklist rows and add `docs/parity/evidence/phase-10-route-manifest-and-api-compare-unification.md` as the primary Phase 10 evidence ledger. A dedicated checklist row is not required by default and should be added only if execution finds the manifest/tooling claim cannot be expressed without ambiguity in existing rows. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-03-PLAN.md`; VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| Bazel | Build/test/API compare/parity targets | yes | 9.1.1 | None needed. [VERIFIED: command `bazel --version`] |
| `just` | Human verification commands | yes | 1.48.0 | Use direct Bazel commands only for diagnosis. [VERIFIED: command `just --version`; VERIFIED: `Justfile`] |
| Cargo | Rust pre-commit verification | yes | 1.88.0-nightly | Bazel tests cover targeted plan verification if full Cargo firmware checks are blocked. [VERIFIED: command `cargo --version`; VERIFIED: `Justfile`] |
| Rust compiler | Rust pre-commit verification | yes | 1.88.0-nightly | None needed for targeted Bazel tests. [VERIFIED: command `rustc --version`] |
| `rg` | Code and evidence audits | yes | 15.1.0 | `grep` if unavailable. [VERIFIED: command `rg --version`] |
| Git | Reference guard, commit, workspace status | yes | 2.53.0 | None. [VERIFIED: command `git --version`] |

**Missing dependencies with no fallback:** None found for Phase 10 research and planning. [VERIFIED: commands listed above]

**Missing dependencies with fallback:** None found. [VERIFIED: commands listed above]

## Validation Architecture

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Bazel `rust_test` with Rust unit tests. [VERIFIED: `crates/bitaxe-api/BUILD.bazel`; VERIFIED: `tools/parity/BUILD.bazel`] |
| Config file | `crates/bitaxe-api/BUILD.bazel`, `tools/parity/BUILD.bazel`, `Justfile`. [VERIFIED: those files] |
| Quick run command | `bazel test //crates/bitaxe-api:tests //tools/parity:tests` [VERIFIED: command passed during research] |
| API compare command | `bazel run //tools/parity:report -- api-compare` [VERIFIED: command passed during research] |
| Checklist guard command | `just parity` [VERIFIED: command passed during research] |
| Full suite command | `just test` [VERIFIED: `Justfile`] |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| API-09 | `/recovery` and `/*` remain Phase 7-owned route surfaces and are not counted as live verified static/recovery behavior. | unit + API compare fixture policy | `bazel test //crates/bitaxe-api:tests //tools/parity:tests` and `bazel run //tools/parity:report -- api-compare` | Existing files yes; add Phase 10 tests. [VERIFIED: `crates/bitaxe-api/src/route_shell.rs`; VERIFIED: `tools/parity/src/api_compare.rs`] |
| API-10 | Schema and captured-response checks remain intact while route presence/kind checks move to Phase 7 manifest. | API compare regression | `bazel run //tools/parity:report -- api-compare` | Existing files yes; refactor needed. [VERIFIED: `tools/parity/src/api_compare.rs`; VERIFIED: `tools/parity/fixtures/api/phase05-required-routes.json`] |
| REL-01 | Static/recovery manifest evidence is recorded without promoting live filesystem/static/recovery parity. | checklist/evidence guard | `just parity` | Existing file yes; doc update needed. [VERIFIED: `docs/parity/checklist.md`; VERIFIED: `tools/parity/src/main.rs`] |
| REL-02 | Firmware OTA remains `FirmwareUpdate` in the manifest without claiming live OTA upload/recovery behavior. | unit + API compare route-kind policy | `bazel test //crates/bitaxe-api:tests //tools/parity:tests` | Existing files yes; add downgrade test. [VERIFIED: `crates/bitaxe-api/src/route_shell.rs`; VERIFIED: `tools/parity/src/api_compare.rs`] |
| REL-03 | OTAWWW remains `AxeOsStaticUpdateGap` and overclaim checks reject static-update verification without required evidence. | unit + API compare fixture policy + checklist guard | `bazel test //tools/parity:tests` and `just parity` | Existing files yes; extend API compare tests. [VERIFIED: `crates/bitaxe-api/src/route_shell.rs`; VERIFIED: `tools/parity/src/main.rs`; VERIFIED: `tools/parity/src/api_compare.rs`] |
| EVD-01 | Checklist/evidence rows record the Phase 10 manifest/tooling claim and non-claims. | docs + parity report guard | `just parity` | Existing file yes; add Phase 10 evidence file. [VERIFIED: `docs/parity/checklist.md`; VERIFIED: `tools/parity/src/main.rs`] |

### Sampling Rate

- **Per task commit:** Run `bazel test //crates/bitaxe-api:tests //tools/parity:tests` after route/helper/tooling edits. [VERIFIED: current command passed]
- **Per wave merge:** Run `bazel run //tools/parity:report -- api-compare` and `just parity` after fixture/checklist/evidence updates. [VERIFIED: current commands passed]
- **Phase gate:** Run `just test`, `bazel run //tools/parity:report -- api-compare`, and `just parity` before `/gsd-verify-work`; if committing in this Rust repo, also honor the AGENTS.md Rust pre-commit command sequence. [VERIFIED: `AGENTS.md`; VERIFIED: `Justfile`; VERIFIED: `.planning/config.json`]

### Wave 0 Gaps

- [ ] Add host-testable route-report helper tests in `crates/bitaxe-api/src/route_shell.rs` for Phase 7 manifest count and ownership metadata. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: `crates/bitaxe-api/src/route_shell.rs`]
- [ ] Refactor `tools/parity/src/api_compare.rs` so route presence/kind checks use an injected route slice in tests and `phase07_routes()` in production. [VERIFIED: `tools/parity/src/api_compare.rs`; VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`]
- [ ] Add API compare regression tests for missing Phase 7 route, route kind downgrade, and verified-overclaim for static/recovery/update routes. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: `tools/parity/src/api_compare.rs`]
- [ ] Add `docs/parity/evidence/phase-10-route-manifest-and-api-compare-unification.md` with a claim-boundary matrix and command results. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: existing evidence doc patterns in `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md`]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V2 Authentication | no | Phase 10 does not add authentication behavior; existing route access gates are not authentication. [VERIFIED: `crates/bitaxe-api/src/route_shell.rs`; VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`] |
| V3 Session Management | no | Phase 10 does not add sessions. [VERIFIED: `crates/bitaxe-api/src/route_shell.rs`; VERIFIED: `tools/parity/src/api_compare.rs`] |
| V4 Access Control | yes | Preserve private-network/origin route access planning and do not weaken update route gates. [VERIFIED: `crates/bitaxe-api/src/route_shell.rs`; VERIFIED: `crates/bitaxe-api/src/update_plan.rs`] |
| V5 Input Validation | yes | Keep typed JSON fixture parsing and route-kind enums; do not replace with ad hoc string scans. [VERIFIED: `tools/parity/src/api_compare.rs`; VERIFIED: `crates/bitaxe-api/src/route_shell.rs`; VERIFIED: `standards/core/architecture.md`] |
| V6 Cryptography | no new crypto | Phase 10 does not change checksum, signing, or cryptographic verification logic. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: `tools/parity/src/api_compare.rs`] |

### Known Threat Patterns for Rust Route/Parity Tooling

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Route downgrade hidden by duplicate fixtures | Tampering | Compare Phase 7 route `RouteKind` values from typed Rust data and add downgrade tests. [VERIFIED: `crates/bitaxe-api/src/route_shell.rs`; VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`] |
| Evidence overclaim from unit/workflow/API compare alone | Repudiation | Keep verified-row guards in `tools/parity/src/main.rs` and run `just parity`. [VERIFIED: `tools/parity/src/main.rs`; VERIFIED: command `just parity`] |
| Static/recovery wildcard swallowing API routes | Tampering / Denial of Service | Preserve explicit registration order and keep static wildcard last. [VERIFIED: `firmware/bitaxe/src/http_api.rs`; VERIFIED: `firmware/bitaxe/src/static_files.rs`; VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`] |
| Secret leakage in evidence docs | Information Disclosure | Keep Phase 10 evidence to command results and manifest/tooling claims; do not record private endpoints, Wi-Fi credentials, pool credentials, or NVS secrets. [VERIFIED: `AGENTS.md`; VERIFIED: `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md`] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md` - locked Phase 10 decisions, discretion, deferred scope, code insights, verification shape. [VERIFIED: file read]
- `.planning/REQUIREMENTS.md` - API-09, API-10, REL-01, REL-02, REL-03, EVD-01 descriptions and traceability. [VERIFIED: file read]
- `.planning/ROADMAP.md` - Phase 10 goal, success criteria, dependency, and verification expectations. [VERIFIED: file read]
- `.planning/STATE.md` - prior Phase 5/7/8/9 decisions and accumulated route/evidence context. [VERIFIED: file read]
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/verification.md`, `standards/core/testing.md`, `standards/languages/rust.md` - repo-local and Bright Builds rules. [VERIFIED: file read]
- `crates/bitaxe-api/src/route_shell.rs` and `crates/bitaxe-api/src/lib.rs` - route manifest types, Phase 5 and Phase 7 route tables, exports, and tests. [VERIFIED: file read]
- `firmware/bitaxe/src/http_api.rs`, `firmware/bitaxe/src/static_files.rs`, `firmware/bitaxe/src/filesystem.rs` - firmware route logging, handler registration, static/recovery, and SPIFFS adapter surfaces. [VERIFIED: file read]
- `tools/parity/src/api_compare.rs`, `tools/parity/src/main.rs`, `tools/parity/fixtures/api/*.json` - API compare, fixture policy, and checklist guard surfaces. [VERIFIED: file read]
- `docs/parity/checklist.md`, `docs/parity/evidence/phase-05-axeos-api-logs-and-telemetry.md`, `phase-07-ota-filesystem-release.md`, `phase-08-ultra-205-release-gate.md`, and `phase-09-flash-monitor-evidence-wrapper-hardening.md` - evidence boundaries and current claims. [VERIFIED: file read]
- `docs/adr/0001-device-user-parity.md`, `0003`, `0005`, `0006`, `0009`, `0010`, `0012`, `0013`, and `PROVENANCE.md` - architecture, evidence, reference, AxeOS, and license/provenance policies. [VERIFIED: file read]
- `reference/esp-miner/main/http_server/http_server.c`, `openapi.yaml`, `recovery_page.html`, `main/filesystem.c`, and AxeOS service files - upstream route and static/recovery/OTA behavior evidence. [VERIFIED: file read]

### Secondary (MEDIUM confidence)

- Local command outputs: `bazel test //crates/bitaxe-api:tests //tools/parity:tests`, `bazel run //tools/parity:report -- api-compare`, and `just parity` all passed during research. [VERIFIED: command output]
- Local environment probes for Bazel, `just`, Cargo, Rust, `rg`, and Git. [VERIFIED: command output]

### Tertiary (LOW confidence)

- None used. [VERIFIED: this research session]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Phase 10 adds no new dependencies and uses existing local crates/tools verified from repo files and command output. [VERIFIED: `Cargo.toml`; VERIFIED: `Justfile`; VERIFIED: command output]
- Architecture: HIGH - Locked decisions, repo standards, and existing code all point to functional-core manifest helpers plus thin firmware/tool adapters. [VERIFIED: `.planning/phases/10-route-manifest-and-api-compare-unification/10-CONTEXT.md`; VERIFIED: `standards/core/architecture.md`; VERIFIED: `crates/bitaxe-api/src/route_shell.rs`]
- Pitfalls: HIGH - Each pitfall is tied to a current code path, fixture shape, or evidence guard. [VERIFIED: `firmware/bitaxe/src/http_api.rs`; VERIFIED: `tools/parity/src/api_compare.rs`; VERIFIED: `tools/parity/src/main.rs`; VERIFIED: `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md`]
- Validation: HIGH - Existing Bazel targets and commands passed during research; Nyquist validation is enabled in `.planning/config.json`. [VERIFIED: `.planning/config.json`; VERIFIED: command output]

**Research date:** 2026-06-29  
**Valid until:** 2026-07-29 for codebase-local planning unless route/evidence surfaces change first. [VERIFIED: current repository state on 2026-06-29]
