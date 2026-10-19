# Parity work plan

- Run ID: `20261019T120001Z-STR-007`
- Parity row: `STR-007`
- Initial status: `verified`
- Source commit: `f4cec1409474da3f324e425014e1e68de8c0b175`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-local-mining-evidence-soaks`

## Selection

ADR-0038 made local mining the Mining Baseline. The local mining soaks and the
lease interruption check produced hardware evidence that narrows named
non-claims on this row: `docs/parity/evidence/20261017-ultra205-local-mining-soak-001.md`, `docs/parity/evidence/20261018-ultra205-local-mining-soak-002.md`, `docs/parity/evidence/20261018-ultra205-local-mining-soak-003.md`.

## Scope and non-scope

- A metadata revision only: the status stays `verified`, the evidence column
  gains `hardware-regression`, and the notes gain one passage citing that evidence.
- Authorized by the owner on 2026-10-19 through
  `docs/parity/verified-revisions/2026-10-19-local-mining.json`, which pins the
  added evidence and the digest of the new notes. The parity tool accepts only
  that additive revision, once.
- Unbounded mining stays a non-claim.

## Verification

- Each cited run sealed `passed`, and an independent review agreed `complete`.
- `just parity` and `just verify-redaction` report no errors.
