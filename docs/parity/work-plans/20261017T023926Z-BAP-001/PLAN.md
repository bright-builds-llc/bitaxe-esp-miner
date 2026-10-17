# Parity work plan

- Run ID: `20261017T023926Z-BAP-001`
- Parity row: `BAP-001`
- Initial status: `verified`
- Source commit: `b27e753fd8ee671a3a2fb75237d659cba2515720`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-bap-settings-effects` (archived after soak 010)

## Selection

BAP-001's non-claims say the firmware answers accepted settings with
`set_failed`. `task-parity-bap-settings-effects` routed them to the existing
owners, and soak 010 showed them acknowledged as the reference does and taking
effect live during a lease.

## Scope and non-scope

- A metadata revision only: the status stays `verified`, the evidence column
  gains `soak`, and the notes gain a short passage citing soak 010.
- Authorized by the owner on 2026-10-16 through `docs/parity/verified-revisions/2026-10-16-bap-settings.json`, which pins the
  added evidence and the digest of the new notes. The parity tool accepts only
  that additive revision, once.
- Wi-Fi `ssid` and `password` settings stay non-claims.

## Verification

- Soak 010 sealed `passed`, and an independent review agreed `complete`.
- `just parity` and `just verify-redaction` report no errors.
