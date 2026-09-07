# Fixed Serial/JTAG qualification — attempt 011

Ultra 205, macOS, desktop Chrome and direct Web Serial 0.2.
Firmware: `cdbf47e0a8b9e7d6e3f7ecd0be9e04bb3ff2748a`.
ELF SHA-256: `3097145a04a4073e3b13ebfa871dc6d2ffc7a17ce933578c3356ef291394b0ad`.
Gate: `b877570b31dba3c0a743e88aed3f3b4d02e0aa4f`.

## Qualified transport and recovery

The initial installation and all four required state-preserving update cycles
returned to exact, healthy, stable application execution. Each cycle verified
fresh browser possession, actual 65536-byte request and response payloads,
matching Device Identity/settings/authorization marks, mining disabled,
acknowledged restoration and serial cleanup. No factory reset or credential
provisioning occurred. Thirteen original runtime artifacts were preserved and
verified against the immutable context.

The deliberately invalid-signature Start returned a correlated
`authentication_failed`, released the port, and permitted fresh possession.
Ledger review before and after reported the original campaign, masks 1,
180000 ms charged and no pending reservation. No valid window was issued for
that rejection test. This physically verifies rejection/reconnection behavior,
not mining or ASIC shutdown timing.

## Original window 1

A successor bound attempt-010's failed normal window and admitted only the two
original unused reservations. Window 1 was signed and its memory-only delivery
consumed. Start failed immediately as `start_failed / operation_failed`; no
running state was observed and no foreground-loss fault was triggered.

Fresh recovery succeeded without a reflash. Device observations identified
`first_failure=readiness`, mask 55, fresh zero fan RPM, 31 C temperature,
approximately 5.48 V and 0.36–0.37 W. The retained generation reported zero
active milliseconds, zero dispatched work, zero submissions and no preparation
or shutdown stage started. This rejection did not energize ASIC work. It does
not prove a three-second physical shutdown, because preparation never began.

The initial `authentication_failed` diagnostic could predate the live Start
from the intentional negative test; it was not adopted as the live cause.
Recovery instead reported `session_failed` and the readiness discriminator.
A subsequent software reproduction also found that the browser replaced the
typed rejection with unrecognized `status_failed`, producing the generic
`operation_failed` projection. The original observation remains unchanged.

Fresh possession-bound ledger review confirmed masks 3, 210000 ms charged and
no pending reservation. The entire window remains charged despite zero work;
no reservation was refunded. Original window 2 remains unused, with only
30000 ms available. Neither an accepted share nor foreground-loss timing was
verified.

## Final state and follow-up

Explicit restoration and close confirmed inactive leases, mining disabled,
matching Device Identity and settings, and released serial ownership. The
authorization high-water comparison changed after the authorized signed Start,
as expected for consumed authorization; no reset is inferred. The owned
supervisor exited and was reaped, its listener and serial nodes had no holder,
and the qualification page was closed. The tested image remains installed.

Protected evidence retains all cycle receipts, the immutable first failure,
window samples, final state, recovery report and fresh ledger review. No signed
runtime grants, pool inputs or private identity/binding digests were promoted.
ADR-0025 requires corrected fan sequencing and a no-mining cooling proof before
any attempt to use the last original reservation. Windows 0 and 1 remain
consumed/unverified, and unrelated mining, Stratum and parity blockers remain.
