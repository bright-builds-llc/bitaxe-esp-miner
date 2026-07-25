---
generated_by: gsd-plan-phase
lifecycle_mode: interactive
phase_lifecycle_id: 34-2026-07-15T03-26-15
generated_at: "2026-07-15T03:26:15.749Z"
---

# Phase 34: Provenance, Runtime Health, and Coherent Operator Snapshot - Context

**Gathered:** 2026-07-15
**Status:** Ready for planning
**Source:** PRD Express Path (`34-PRD.md`)

<domain>
## Phase Boundary

Phase 34 creates one coherent read-only operator snapshot and truthful identity/health projections. Build identity and LCD provenance are Plan 01 in Wave 1. Coherent snapshot revision, remaining runtime identity, and passive health belong to later plans. This execution pass may implement Plan 01 only and may not use hardware or enter Phase 35.

</domain>

<decisions>
## Implementation Decisions

### Canonical identity
- One typed identity owns full 40-character source commit, derived 12-character short commit and label, release/dev channel, firmware-input dirty boolean, and optional exact release tag.
- One versioned provenance stamp wraps that identity with the separately declared semantic version and full pinned reference commit. The shared Rust model is the sole derivation/validation authority; shell gathers primitive inputs and Starlark only transports them.
- Allowed exact tags match `vMAJOR.MINOR` or `vMAJOR.MINOR.PATCH`; zero means dev, one means release, and multiple matching tags fail closed.
- Labels are exactly `<hash>`, `<hash>-dirty`, `<hash>-dev`, or `<hash>-dirty-dev`. Dirty precedes dev and remains independent from channel.
- Clean dev and clean release builds may qualify exact-source engineering evidence. Dirty builds are rejected before hardware admission.

### Dirty scope and build graph
- Dirty includes staged, unstaged, deleted, renamed, and untracked nonignored firmware/package inputs from one checked-in pathspec contract. Planning, docs, evidence, reference, scratch, ignored files, and unrelated host tools do not affect the label.
- Bazel workspace status emits only the five stable primitive Bitaxe keys for source commit, dirty state, release tag, semantic version, and reference commit. A Starlark rule consumes `ctx.info_file`, ignores ordinary Bazel status keys, rejects unknown Bitaxe keys, and invokes the shared Rust materializer rather than deriving identity itself.
- Firmware actions explicitly depend on all transitive Rust/root/build inputs so dirty-to-dirty edits invalidate the action independently of the boolean status transition.
- ELF and manifest consume the same generated identity stamp. Packaging never re-queries live Git for the firmware identity.

### Runtime and public surfaces
- Canonical firmware Cargo builds require the identity stamp and fail with a `just build` instruction when it is absent or invalid.
- Output-local supplemental sdkconfig defaults set the ESP-IDF application version to the build label; a generated output-local sdkconfig prevents stale local override.
- LCD renders `fw <build_label>` without truncation. The retained machine marker is the unsuffixed full hash, followed by one redacted structured build-identity record.
- System-info keeps `version` as the human label and adds `semanticVersion`, `sourceCommit`, `referenceCommit`, `appElfSha256`, `buildChannel`, `sourceDirty`, and nullable `releaseTag`; live WebSocket receives the same additive fields.
- The running ESP-IDF application descriptor's lowercase ELF SHA-256 is the non-circular flashed-package identifier. It is compared with host ELF inspection; the final package digest remains a separate host-side evidence field.
- Heartbeat, PATCH, restart, and unrelated public response shapes remain unchanged. Machine evidence never parses the presentation label.

### Package and admission
- Active package manifests become schema v3, retain top-level full `source_commit`, and add semantic version, pinned reference commit, inspected application-descriptor ELF SHA-256, and structured `build_identity` fields derived from the same stamp.
- Historical v2 evidence is not rewritten and remains readable only where historical evidence intentionally requires it.
- Active admission validates schema v3 and identity consistency in the concrete `tools/flash` flash/flash-monitor path before USB port resolution, rejects explicit-image hardware runs without an admitted manifest and every dirty package, and accepts clean dev or release packages when full commit, HEAD, embedded identity, application-descriptor ELF SHA-256, and package digest agree.

### Later Phase 34 plans
- Later waves must cover OBS-06, SYS-03, SYS-04, SYS-05, HLT-01, HLT-02, HLT-03, and HLT-04 without expanding Plan 01.
- All later work remains passive and read-only: no active watchdog intervention, hardware self-test effects, load/fault experiments, actuation, mining, archived lineage work, credentials, direct UART/pins, OTA, other boards, or broad promotion.

### the agent's Discretion
- Exact module/file partitioning and private helper names, provided the typed identity has one derivation authority and public field names/behaviors above remain exact.
- How later Phase 34 plans divide coherent snapshot, remaining platform identity, and passive health work, provided dependencies and requirement coverage are explicit.

</decisions>

<canonical-refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Approved Phase 34 contract
- `.planning/phases/34-provenance-runtime-health-and-coherent-operator-snapshot/34-PRD.md` — Exact build identity, dirty scope, public surfaces, package/admission behavior, tests, and delivery boundary.
- `.planning/ROADMAP.md` — Phase 34 goal, requirements, prohibitions, and Phase 35 dependency.
- `.planning/REQUIREMENTS.md` — OBS-06, SYS-01 through SYS-05, and HLT-01 through HLT-04 acceptance requirements.

### Existing implementation and evidence boundary
- `.planning/phases/33-confirmed-settings-durability/33-VERIFICATION.md` — Phase 33 software completion, historical evidence non-promotion, and CFG-12 Phase 35 ownership.
- `AGENTS.md` — Build, hardware, evidence, GSD, direct-UART/pin, and archived-lineage rules.
- `standards/core/architecture.md` — Functional-core and imperative-shell boundary.
- `standards/core/verification.md` — Required verification and pre-commit ordering.
- `standards/core/testing.md` — Behavior-oriented test expectations.
- `standards/languages/rust.md` — Rust implementation rules.

</canonical-refs>

<specifics>
## Specific Ideas

- Use a dependency-free typed identity model that derives redundant values rather than accepting contradictory inputs.
- Use stable workspace-status keys for identity transitions, plus explicit Bazel file inputs for dirty-to-dirty cache invalidation.
- The maximum label is 22 characters; `fw ` plus that label exactly fills the existing 25-character LCD line.
- Emit the ESP-IDF application version from a generated defaults file instead of relying on ESP-IDF's context-sensitive Git fallback.

</specifics>

<deferred>
## Deferred Ideas

- Phase 35 owns current-package detector-gated hardware qualification, CFG-12/EVD-13 closure, and parity admission.
- Additional mutable settings, active control, watchdog intervention, hardware self-test execution, mining, OTA, other boards, and archived Phase 28.1.1 work remain outside Phase 34.

</deferred>

*Phase: 34-provenance-runtime-health-and-coherent-operator-snapshot*
*Context gathered: 2026-07-15 via PRD Express Path*
