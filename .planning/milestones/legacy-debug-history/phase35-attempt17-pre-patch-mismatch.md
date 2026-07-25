---
status: resolved
trigger: "Phase 35 attempt 17 reaches HTTP readiness, then rejects the production Boot A pre-mutation epoch as pre_patch_mismatch."
created: 2026-07-22T00:22:22Z
updated: 2026-07-22T00:30:58Z
---

## Current Focus

hypothesis: Confirmed and repaired in software - the production epoch adapter did not populate the setting digest and constructed snapshot evidence from API fields and fabricated retained content that the live interfaces do not provide.
test: Complete. The hermetic production adapter consumes a serial-classified boot identity, an actual system-info document, the live WebSocket envelope payload, and the actual retained-log download. It rejects missing identity, setting, revision, marker, or chronology boundaries.
expecting: The repaired Boot A pre-mutation snapshot hashes the validated private hostname and therefore matches the already-ready original settings read without exposing the raw value.
next_action: Commit the verified repair, run exact-current-HEAD preflight, and invoke fresh attempt 18 under the standing progress-gated authority.

## Symptoms

expected: After the original settings classifier reports `ready`, the Boot A pre-mutation epoch binds the same private setting, one serial-classified boot identity, an API snapshot, a later live WebSocket snapshot, and both exact retained markers before PATCH.
actual: Attempt 17 reached HTTP `ready`, then the epoch comparison returned `pre_patch_mismatch` before mutation.
errors: The supervisor seal contains only the typed primary category and no secondary restoration or cleanup failure. No raw origin, setting, hostname, network identity, or device identifier is recorded here.
reproduction: The previous production adapter deterministically omitted `setting_digest`; its fixtures supplied that field, so the fixture-only success path concealed the mismatch.
started: Phase 35 attempt 17 at exact source `98463e8a735233b4e283b6535d3c9f375a984523`.

## Eliminated

- HTTP request readiness: attempt 17 received a complete schema-v2 response with status, headers, and body.
- HTTP server task capacity: attempt 17 validates the 16 KiB server-task repair at runtime.
- Mutation, restoration, or cleanup interference: the mismatch occurred before PATCH; cleanup completed with no secondary failure.
- Raw setting mismatch at the device boundary: the previous epoch document had no production setting digest to compare, so it could not establish a contradictory value.
- Cross-session acceptance: the repaired adapter requires serial, API, and WebSocket session identity equality.

## Evidence

- timestamp: 2026-07-22T00:08:00Z
  checked: Attempt-17 private epoch key/type inventory without rendering values.
  found: The production epoch contained no setting digest, its boot ordinal source was absent from the API response, and the retained document contained only a locally constructed line.
  implication: The fixture and production paths implemented different evidence contracts; the production comparison necessarily failed before mutation.
- timestamp: 2026-07-22T00:12:00Z
  checked: Attempt-17 private API and WebSocket structural projections without rendering operational identifiers.
  found: The API supplies session and revision but not boot ordinal; the WebSocket capture is an event envelope whose data object carries the snapshot identity, and its revision is later than the API revision.
  implication: Boot ordinal must come from the serial classifier, the WebSocket data object must remain intact, and equality of API and WebSocket revisions is invalid for completion-ordered publication.
- timestamp: 2026-07-22T00:16:00Z
  checked: Phase 33 restart proof and the current Phase 35 passive reboot slice.
  found: Phase 33 establishes HTTP service loss before taking the post-restart byte offset. Phase 35 took its offset before the restart request, which could mix trailing Boot A bytes into Boot B classification.
  implication: The repaired reboot adapter must establish service loss, then slice the passive trace and invoke the post-restart classifier with the baseline session and ordinal.
- timestamp: 2026-07-22T00:22:22Z
  checked: Focused Rust evidence tests and the uncached Phase 35 correlated-evidence shell suite.
  found: Later WebSocket revision is accepted, same revision is rejected, production capture uses actual protected artifacts, incoherent boundaries fail without console disclosure, and post-restart classifier arguments are byte-boundary scoped.
  implication: The root cause and the adjacent reboot-boundary defect are covered hermetically; the full repository gate remains before commit and hardware.

## Resolution

root_cause: Fixture mode supplied a complete synthetic epoch, while production inferred nonexistent API boot-ordinal data, discarded the WebSocket envelope structure, fabricated one retained marker, assigned the API revision to both projections, fabricated a one-millisecond interval, and omitted the setting digest used by the pre-PATCH equality gate.
fix: Build production epochs only from the serial boot classifier and protected live API, WebSocket-data, and retained-log artifacts; hash the validated private hostname; require same-session and strictly later WebSocket revision; retain real monotonic bounds; and classify Boot B only from bytes captured after proven service loss. Capture command failures remain private and the passive monitor is reaped on all exits.
verification: The focused and full ordered Rust gates, affected uncached Bazel suites, canonical firmware image build, reference, parity, lifecycle, shell syntax/style, and diff checks pass. Staged redaction verification, the commit, and exact-head preflight remain before attempt 18.
files_changed: [`scripts/phase35-correlated-evidence.sh`, `scripts/phase35-correlated-evidence-effects.sh`, `scripts/phase35-correlated-evidence-test.sh`, `tools/parity/src/phase35_evidence.rs`, `tools/parity/src/phase35_evidence/tests.rs`, `tools/parity/fixtures/phase35/eligible.json`]
