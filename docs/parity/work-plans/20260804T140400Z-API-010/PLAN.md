# Parity work plan

- Run ID: `20260804T140400Z-API-010`
- Parity row: `API-010`
- Initial status: `in-progress`
- Source commit: `5adaa99a3017d722cd1bc65f0edcdd5c8154a163`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api010-theme-route`

## Selection

The deterministic selector reported no open plan after `CFG-006`. Earlier
implemented candidates retain audited live hardware, network, mining, safety,
release, or other-board evidence gaps. `API-010` is the first bounded pure
candidate: the NVS key names already exist, while the pinned reference supplies
a small GET/POST contract and the current firmware has established access,
response, serialized NVS, and confirmed-snapshot owners that can implement it
without hardware effects.

## Scope and non-scope

Model the exact upstream dark color-scheme and accent-color defaults. Add a
typed GET projection from the confirmed snapshot and a POST planner that
enforces the upstream 1023-byte body ceiling, rejects malformed JSON, ignores
wrong-typed or unknown fields like the reference, serializes present accent
colors, and yields the exact success object. Persist accepted writes under the
existing settings transaction lock, commit once, independently reload and
reconcile the requested values, then publish the complete confirmed snapshot.

Register private-network-gated `/api/theme` GET and POST handlers before the
wildcard API handlers. Extend both route manifests, the static upstream AxeOS
theme-service usage fixture, and captured-response golden evidence.

Do not import upstream Angular source or generated assets, claim browser-level
UI behavior, change hardware controls, touch credentials, add network effects,
or run hardware. Full installed AxeOS navigation and visual integration remain
owned by `UI-004`; live NVS durability remains separate evidence before
`API-010` can be verified.

## Implementation

- [ ] Add typed theme defaults, wire response, POST decision, reconciliation,
      and generic public error contracts with focused golden tests.
- [ ] Extend the firmware settings adapter with a serialized, commit/reload/
      reconcile/publish theme transaction and register GET/POST handlers.
- [ ] Add both routes to Phase 05/07 ownership, AxeOS static usage, captured
      fixtures, and API-compare regressions.
- [ ] Record `RESULT.md` only after focused and repository-wide gates pass.

## Verification and promotion

Run focused `bitaxe-config`, `bitaxe-api`, firmware-build, and API-compare
targets, then the mandatory ordered Rust checks, Bright Builds, `just test`,
parity/progress, redaction, reference cleanliness, and diff checks. Transition
only `API-010` to `implemented` with `unit,golden,api-compare` evidence. Do not
mark it `verified` without live route/durability evidence and installed-browser
coverage. No hardware or recovery contract exists in this plan.
