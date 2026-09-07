# ADR-0026: Retained preparation diagnostics and iterative qualification

Accepted 2026-09-07 under the owner's explicit implementation authorization.
Attempt-012 exhausted the original 240000-ms campaign and rebooted during
preparation without identifying the failing step. Preserve that campaign and
all original evidence. Add allocation-free reset-retained preparation receipts
and separately signed, durably accounted qualification attempts to identify
and correct failures before completing the outstanding live acceptance.

Every diagnostic attempt has a 30000-ms maximum active duration. Final
acceptance uses one exact firmware/browser pair for normal 180000 ms,
foreground loss 30000 ms and heartbeat loss 30000 ms. Full reservations are
charged before preparation and never refunded. The owner authorizes iteration
without an additional overall ceiling, but every retry requires published,
verified progress against the preceding failure or missing discriminator.

The optional signed qualificationAttempt allowance is mutually exclusive with
legacy acceptanceCampaign. Its worker-qualification-attempt-v1 schema binds
identity, strictly advancing ordinal, purpose and exact duration. Separate NVS
accounting retains cumulative charge and interrupted reservations; the legacy
ledger, Device Identity, authorization sequence marks and restoration journal
remain unchanged. Diagnostic receipts never grant execution authority.

Fixed USB Serial/JTAG, direct browser Web Serial, exact signed application
identity, three-second revocation/shutdown initiation and all conservative
safety limits remain mandatory. Collect retained crash evidence before another
reset or reflash. No raw memory dumps, electrical fault injection, direct pins,
implicit factory reset, erase-flash, unrelated discovery or automatic parity
promotion is authorized. End every attempt with proven recovery/cooling and
released ownership, or stop hardware work with the failure preserved.
