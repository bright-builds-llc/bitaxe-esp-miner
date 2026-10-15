# Parity work plan

- Run ID: `20261015T023348Z-BAP-001`
- Parity row: `BAP-001`
- Initial status: `implemented`
- Source commit: `ecc7f122201a7e2df6df31cda9e85741ef8508db`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-bap-live-accessory-verification`
- Continues plan: `docs/parity/work-plans/20260821T020549Z-BAP-001/WORKLOG.md`
- Hardware ordinal: `006`

## Selection

The identity-bound BAP probe (ordinal 006) gives BAP-001 its first live
evidence. An independent review found that the live `deviceModel` answer
differs from the reference, so this transition only records the evidence and
the divergences. The status stays `implemented`.

## Scope and non-scope

- Add `hardware-smoke` evidence and notes for the live UART2 owner.
- Record the `deviceModel` and subscription-value divergences, which are owned
  by `task-parity-bap-value-sources`.
- No status promotion and no firmware change.

## Verification

The same records and independent review as
`docs/parity/work-plans/20261015T023347Z-BAP-002/RESULT.md`, and the redacted
summary `docs/parity/evidence/20261014-bap-uart-live-probe.md`.
