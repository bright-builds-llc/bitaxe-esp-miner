# Typed Automation Checklist Migration

- Parity rows: `WF-001`, `STR-007`, `REL-002`, `REL-09`, `CFG-07`, `V12-HOSTNAME-205`, `V12-PACKAGE-IDENTITY-205`
- Initial status: mixed
- Migration ledger: `docs/parity/automation-migration.json`

## Plan

1. Bind every checklist pointer or status change to the typed automation migration ledger.
2. Preserve verified status only where the ledger records proven equivalence.
3. Downgrade legacy-schema hardware claims whose equivalence is unresolved.
4. Validate the hash-chained checklist, progress view, redaction, and pinned reference.

## Safety boundary

This plan changes checklist metadata and claim status only. It does not perform hardware work or promote any parity claim.
