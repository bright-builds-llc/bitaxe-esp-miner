# Parity work plan

- Run ID: `20261029T184500Z-SELF-001`
- Parity row: `SELF-001`
- Initial status: `verified`
- Source commit: `b5940eba918b87cacb139736144017dc186c3ace`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-gamma-601-bring-up` (F4.6; owner-approved plan
  `delightful-foraging-quill`, owner decisions 2026-10-26)

## Selection

The self-test's full lifecycle passed on both boards at the final Gamma 601
bring-up code:
- the Ultra 205's rerun, attempt 007 at `b5940eba`, under
  `docs/parity/work-plans/20261029T050000Z-SELF-001-ULTRA205-RERUN/PLAN.md`:
  `docs/parity/evidence/20261029-ultra205-f46-self-test.md`;
- the promoted production Gamma 601, attempt 009 at `bb9f2ae7`, under
  `docs/parity/work-plans/20261026T000000Z-SELF-001-GAMMA601/PLAN.md`:
  `docs/parity/evidence/20261028-gamma601-production-self-test.md`.

## Scope and non-scope

- A passage-only metadata revision: the status stays `verified`, the
  evidence column and the owned target are unchanged, and the notes gain
  one passage.
- Authorized through
  `docs/parity/verified-revisions/2026-10-29-self-test-both-boards.json`,
  which pins the evidence and the digest of the new notes. The owner's
  approved F4 plan names this revision (F4.6: verified-revision authorities
  where the hardware evidence meets the checklist rule). The parity tool
  accepts only that additive revision, once.
- The passage records the 205's rerun on the current shared code, states
  what attempt 005's hashrate window counted,
  narrows the other-boards non-claim for the Gamma 601 only, and keeps
  every other non-claim.
- Attempt 005's committed projection and summary stay unchanged.

## Verification

- Both cited runs completed, and an independent check of each against its
  private captures agreed.
- `just parity` and `just verify-redaction` report no errors.
