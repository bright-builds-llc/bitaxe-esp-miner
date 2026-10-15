# Parity work plan

- Run ID: `20261015T024309Z-BAP-002`
- Parity row: `BAP-002`
- Initial status: `implemented`
- Source commit: `ecc7f122201a7e2df6df31cda9e85741ef8508db`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-bap-live-accessory-verification`
- Continues plan: `docs/parity/work-plans/20261015T023347Z-BAP-002/RESULT.md`
- Hardware ordinal: `006` (no new hardware)

## Selection

The previous run recorded the reviewed, identity-bound evidence. It held the
promotion only because the parity validator treated every `BAP-*` row as
deferred non-205 scope. On 2026-10-14 the owner decided to relax that guard and
promote BAP-002.

## Scope and non-scope

- Narrow the validator so that a `BAP-*` row may cite Ultra 205 evidence only
  through the identity-bound BAP live probe. Every other deferred-scope rule
  is unchanged.
- Promote BAP-002 on the existing evidence.
- No new hardware run. BAP-001 stays `implemented`
  (`task-parity-bap-value-sources`).

## Verification

- Validator regressions: a row citing the bound probe is accepted; a row citing
  the probe without its binding, or reused Ultra 205 evidence, is rejected.
- `just parity` reports no validation errors.
