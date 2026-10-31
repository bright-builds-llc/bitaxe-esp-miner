# Parity work plan

- Run ID: `20261031T001100Z-API-004`
- Parity row: `API-004`
- Initial status: `verified`
- Source commit: `3e6ff4e9f921d5b48c311711507ed6208099ac8a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-http-access-gate-peer-address`

## Selection

API-004 owns `reference/esp-miner/main/http_server/http_server.c` but makes no
claim about its access gate. The gate read its peer wrongly and is now fixed, and
both boards have hardware evidence: `docs/parity/evidence/20261030-access-gate-peer-address.md`.

## Scope and non-scope

- A passage-only metadata revision: the status stays `verified`, the evidence
  column and the owned target are unchanged, and the notes gain one passage.
- Authorized by the owner on 2026-10-30, who read the exact passage, through
  `docs/parity/verified-revisions/2026-10-30-access-gate-peer-address.json`, which pins the evidence and the digest of the new notes. The
  parity tool accepts only that additive revision, once.
- The passage names the fix and the check, and states what is not shown on
  hardware and the known differences from upstream. Every earlier limit stays.
- Those differences are `task-http-access-gate-upstream-differences`, not this
  revision.

## Verification

- Both cited runs printed `passed: true`, each on the boot of its own capture.
- An independent review found no defect in the peer read. Its findings were fixed
  before the evidence was committed, or are stated in the passage.
- `just parity` and `just verify-redaction` report no errors.
