# Parity work plan

- Run ID: `20261017T005335Z-ASIC-007`
- Parity row: `ASIC-007`
- Initial status: `verified`
- Source commit: `5a127e413193150af73374b27831203203d9aba9`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-device-config-profile-evidence-soaks`

## Selection

ADR-0037 moved the hardware profile from the Work Lease to the device's own
settings and added live settings changes while mining. Soaks 007, 008 and 009
gathered hardware evidence for that. `ASIC-007` names non-claims that this
evidence now narrows.

## Scope and non-scope

- A metadata revision only: the status stays `verified`, the evidence column
  gains `soak`, and the notes gain one sentence citing the soak evidence.
- Authorized by the owner on 2026-10-16 through
  `docs/parity/verified-revisions/2026-10-16-device-config-soaks.json`, which
  pins this row's added evidence and the digest of its new notes. The parity
  tool accepts only that additive revision, once.
- The original projection and its claim are unchanged.
- Nothing beyond what the redacted soak summaries show is claimed.

## Verification

- Soaks 007 attempt 002, 008 and 009 sealed `passed`, each with an
  independent review agreeing `complete`.
- `just parity` and `just verify-redaction` report no errors.
