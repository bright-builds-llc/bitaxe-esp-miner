# Parity work plan

- Run ID: `20261015T043356Z-BAP-001`
- Parity row: `BAP-001`
- Initial status: `implemented`
- Source commit: `47c0fa1fdebec626e1026d19f5c3384d75ccd0ba`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-bap-value-sources`
- Continues plan: `docs/parity/work-plans/20261015T023348Z-BAP-001/PLAN.md`
- Hardware ordinal: `009`

## Selection

BAP-001 stayed `implemented` because the firmware's answer and subscription
values diverged from the reference. `task-parity-bap-value-sources` fixed them.
The bound ordinal 008 review found five more divergences, which were fixed
before the owner-authorized ordinal 009.

## Scope and non-scope

- Promote BAP-001 on the identity-bound ordinal 009 and its independent review.
- No firmware change after the bound run: the only later source change is a
  code comment.
- BAP settings effects and the publish order of several due subscriptions are
  out of scope; they are tracked in `task-parity-bap-settings-effects`.

## Verification

- A sealed probe record (`complete`) and a sealed binding record (`bound`).
- An independent review against the pinned reference.
- A redacted summary in
  `docs/parity/evidence/20261015-009-bap-uart-live-probe.md`.
- `just verify-redaction` and `just parity` report no errors.
