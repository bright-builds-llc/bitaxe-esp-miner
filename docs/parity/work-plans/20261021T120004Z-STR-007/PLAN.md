# Parity work plan

- Run ID: `20261021T120004Z-STR-007`
- Parity row: `STR-007`
- Initial status: `verified`
- Source commit: `d95d7002980a72f178ef94821d30508c08af3093`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-local-mining-hardware-revision`

## Selection

October local mining hardware runs added evidence for this row: `docs/parity/evidence/20261021-ultra205-local-mining-soak-006.md`, `docs/parity/evidence/20261021-ultra205-local-mining-soak-005-rejudged.md`.

## Scope and non-scope

- A passage-only metadata revision: the status stays `verified`, the
  evidence column is unchanged, and the notes gain one passage.
- Authorized by the owner on 2026-10-21 through `docs/parity/verified-revisions/2026-10-21-local-mining-hardware.json`, which pins
  the evidence and the digest of the new notes. The parity tool accepts
  only that additive revision, once.
- The passage keeps every existing non-claim and states its own.

## Verification

- Each cited run sealed `passed`, and an independent review agreed.
- `just parity` and `just verify-redaction` report no errors.
