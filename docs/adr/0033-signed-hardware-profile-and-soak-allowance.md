# ADR-0033: Signed hardware profile and one-shot soak allowance

Accepted 2026-10-06 under `task-ultra205-default-profile-soak-reverification`,
by the owner's decision of that date.

## Context

The upstream-default soak (485 MHz, 1200 mV, 100% fan, 600 active seconds,
Stratum V1 real pool) last ran through `just mining-campaign`. That path ran
before the fixed Serial/JTAG baseline (ADR-0021, ADR-0023). It now fails closed
with `provisioning_requires_factory_reset`, and its serial markers no longer
reach USB.

Under the current design, mining starts only from a signed Worker Work Lease
delivered by the Gate. Every such lease runs the Conservative preset (400 MHz,
1100 mV). Its allowances also cap active time at 180,000 ms (`normal`), and the
Gate accepts at most 16 renewal artifacts.

## Decision

1. **Signed profile.** A Work Lease grant may carry an optional
   `hardwareProfile`: `conservative` or `upstream-default`. When the field is
   absent the profile is `conservative`, so existing grants, fixtures and
   signatures keep their exact bytes. The field is part of the signed request
   digest. Firmware fixes the profile for the lease's whole life: a renewal
   cannot change it, and a mismatch starts terminal safe-stop.
2. **Upstream-default only with a soak.** `upstream-default` is valid only on a
   grant that carries a `soakAllowance`. A soak grant must state its profile,
   use Stratum V1, and use the 60,000 ms lease with a 20,000 ms renewal.
   Firmware enforces this, and the Gate parser and the host signer refuse the
   same invalid grants.
3. **One-shot soak allowance with its own ledger.** `soakAllowance`
   (`worker-soak-allowance-v1`) is a third, mutually exclusive grant purpose,
   beside `acceptanceCampaign` and `qualificationAttempt`. Its ordinal is kept
   in a separate NVS `soak_ledger`, so the qualification ledger's fixed 30/180 s
   charges and its readers are unchanged. Older firmware ignores the new key.
   Each soak reserves its full charge before hardware preparation, is charged
   exactly once and is never refunded (ADR-0024). Replays, gaps and a pending
   reservation fail closed.
4. **Budget includes the shutdown tail.** The work gate closes at the budget
   minus the 15,550 ms pre-reset shutdown bound. A soak therefore signs and
   charges `maximumActiveMilliseconds = 615,550`, which keeps work admitted for
   exactly 600,000 active ms with the shutdown still inside the budget.
5. **Controller 0.4 stays.** Both fields are optional and omitted when absent.
   Older firmware (`deny_unknown_fields`) and the older Gate (`exactRecord`)
   reject them, so mismatched peers fail closed without a protocol bump.
6. **Pre-signed renewals.** The host signs the grant and its renewals in one
   batch, as for qualification attempts. Only soak leases may carry more than
   16 renewals, up to 36. The Gate lets the device close the work gate. It
   refreshes status before each renewal and stops renewing once the gate is
   closed.

## Safety

Existing runtime limits still revoke work on any violation:

- input voltage 4.5–5.5 V;
- power at most 15 W;
- ASIC temperature below 75 C;
- a fresh nonzero fan reading, and a safe sample within 1 s;
- the 2.8 s heartbeat deadline.

No new actuation range is introduced: the drivers already admit 485 MHz and
1200 mV for the upstream-default preset. A soak adds no authority. Each run
still needs an active `TASKS.md` hardware contract.

## Non-claims

This ADR verifies nothing on hardware. It does not authorize automatic fan
control, mining beyond the signed budget, Stratum V2 soaks or parity promotion.
