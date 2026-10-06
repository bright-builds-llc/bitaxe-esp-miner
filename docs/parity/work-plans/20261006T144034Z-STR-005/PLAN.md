# Parity work plan

- Run ID: `20261006T144034Z-STR-005`
- Parity row: `STR-005`
- Initial status: `implemented`
- Source commit: `b77c6d753ea2291f0b649f1cb2deab17296ad15e`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-str005-evidence-promotion`

## Selection

`bazel run //tools/parity:report -- next-item --format json` returned no
open plan (after closing the superseded 2026-08-29 Noise-auth child) and the
candidates ASIC-009, ASIC-010, STR-005, BAP-001, BAP-002.

- ASIC-009 (BM1368) and ASIC-010 (BM1397) are skipped: they need other
  boards, and the only admitted hardware is the Ultra 205 (BM1366).
- STR-005 is actionable and owner-requested ("run the promotion"). Its only
  dependency, `task-str005-piecewise-integration-review`, completed on
  2026-10-06 under ADR-0029.

## Scope and non-scope

Promote only STR-005 from `implemented | unit,golden,workflow` to
`verified | unit,golden,workflow,hardware-regression`, citing the final
candidate's evidence. Repository evidence validation, documentation, the
checklist transition, progress synchronization, task archival, commit and
push only. No hardware, network, flash, NVS, mining or other device effect.
No other row changes.

## Implementation

- [ ] Validate every accepted child projection and provenance relationship
      for the final candidate (`2bff1004`, ELF `9783dc74…`, Gate `86fc62d7`):
      lineage head re-derivation, the install's own review, sealed result and
      seal digests for heartbeat010, restart007–009, share-current-001 and
      renew-current-002, public projections against their private results,
      and the published native audits' ELF binding.
- [ ] Confirm the independently reviewed cumulative coverage and
      compatibility in the completed integration review; diagnostics alone
      are not acceptance evidence.
- [ ] Record the validation in `WORKLOG.md` and the conclusion in
      `RESULT.md`.

## Verification and promotion

- Commands: `just str005-lineage show`, `just stratum-v2-noise-serial review`
  for psram-default-install attempt-002, digest recomputation of each sealed
  root, `just verify-redaction`, `just parity`, `just parity-progress`, the
  ordered Cargo checks, `bun scripts/bright-builds-check.ts all` and
  `just test`.
- Promotion criteria: every cited digest re-derives exactly; every owner
  decision (ADR-0032 Share002 non-claim; single-run, single-board,
  local-fixture evidence qualifying as `hardware-regression`) is recorded;
  the integration review shows no open blocker.
- Transition: `transition-item --row-id STR-005 --to verified --evidence
'unit,golden,workflow,hardware-regression'`, then `sync-progress`.
- Non-claims carried into the row: external or production pools, sustained
  or unbounded mining, other boards, mixed-protocol fallback, OTA and
  release readiness, Channel006 publication, Share002's pre-reset resource
  release, hardware negative paths other than heartbeat loss.
