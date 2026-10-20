# Parity work plan

- Run ID: `20261020T120000Z-NET-001`
- Parity row: `NET-001`
- Initial status: `verified`
- Source commit: `a24684f1b634103ad61dfb6d6ae9e53592837d00`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-local-mining-handover-panic-and-outages`

## Selection

Local mining soaks traced recurring outages to a Wi-Fi `beacon_timeout`
with modem power save on, which upstream turns off. The firmware now turns
it off too, and soaks 004 and 005 show markedly fewer disconnects:
`docs/parity/evidence/20261019-ultra205-handover-loop-and-wifi-power-save.md`, `docs/parity/evidence/20261020-ultra205-local-mining-soak-005.md`.

## Scope and non-scope

- A metadata revision only: the status stays `verified`, the evidence column
  gains `soak`, and the notes gain one passage with the `connect.c`
  power-save breadcrumb and the soak evidence.
- Authorized by the owner on 2026-10-20 through
  `docs/parity/verified-revisions/2026-10-20-wifi-power-save.json`, which pins
  the added evidence and the digest of the new notes. The parity tool
  accepts only that additive revision, once.
- The remaining disconnect's cause and router/RF failures stay non-claims.

## Verification

- Soak 004 attempt 002 and soak 005 sealed `passed`, and independent
  reviews agreed `complete`.
- `just parity` and `just verify-redaction` report no errors.
