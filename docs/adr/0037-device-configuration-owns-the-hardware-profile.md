# ADR-0037: Device configuration owns the hardware profile; the Work Lease authorizes work

## Status

Accepted on 2026-10-16 by the owner. It replaces decisions 1 and 2 of
[ADR-0033](0033-signed-hardware-profile-and-soak-allowance.md): the signed
`hardwareProfile`, and the rule that `upstream-default` needs a soak. ADR-0033's
soak allowance, ledger, pre-signed renewals and Controller 0.4 decisions stay.

Until `task-lease-without-hardware-profile` lands, firmware and the Gate
still run ADR-0033's signed presets. The implementation tasks below carry the
change.

## Context

- **Where the profile comes from today.** A Work Lease carries the mining
  hardware profile: `conservative` (400 MHz, 1100 mV, fan 100%) or
  `upstream-default` (485 MHz, 1200 mV, fan 100%). The firmware fixes the
  profile for the lease's life. The actuation adapter
  (`mining_actuation_adapter.rs`) accepts only those two pairs. Frequency and
  core voltage saved through HTTP settings are display-only, and so would BAP's
  be.
- **Why it ended up there.** The history shows this grew out of
  qualification, not out of a decision about authority:
  - Real actuation first shipped on 2026-07-28 with the two closed presets,
    because only those pairs had drivers, ramp plans and hardware evidence.
    The firmware never applied saved frequency or voltage.
  - ADR-0033 put the preset into the lease because the Gate had become the only
    working mining path. It says the field "adds no authority".
- **What the Gate's own design says.** A Work Lease authorizes one challenge's
  work. Afterwards the Worker restores its Mining Baseline, "the Worker's
  ordinary mining state". Voltage, frequency, fan and thermal policy stay
  local; remote control is deferred to a later version (Gate ADR-0017,
  ADR-0051; `docs/design/v1-worker-management.md`).
- **The cost of the current design.** It diverges from upstream, where the
  power-management task re-reads frequency and voltage from NVS and applies
  changes live. That divergence blocks parity for HTTP settings and BAP
  `frequency` and `asic_voltage` (`task-parity-bap-settings-effects`).

## Decision

1. **The lease authorizes how much work, never how it is done.** A grant
   carries purpose, budget, active time, protocol, renewals and ledger
   ordinals. It carries no frequency, voltage or fan setting. The
   `hardwareProfile` field is retired. Once the Gate and host signer stop
   emitting it, the firmware refuses any grant that still carries it. A
   mismatched pair therefore fails closed, as ADR-0033 already arranged for
   unknown fields.
2. **Device configuration is the hardware profile.** Mining runs the saved
   settings `frequency`, `coreVoltage`, `autofanspeed`, `manualFanSpeed`,
   `minFanSpeed` and `temptarget`. They are set through HTTP, BAP or defaults,
   as on upstream.
3. **The firmware owns the safety range.** At work admission, and for every
   live change, frequency and voltage must lie in the board catalog's range.
   For the Ultra 205 that is 400–575 MHz and 1100–1300 mV
   (`AsicFrequencyMhz::ultra_205_bm1366`, `CoreVoltageMv::ultra_205_bm1366`).
   - **At admission:** an out-of-range setting refuses work with a typed
     reason. It is never clamped.
   - **While mining:** an out-of-range change keeps the applied values and is
     reported.
   - **Saving:** settings writes keep their own wider schema ranges, as on
     upstream.
4. **Changes apply live while mining, as on upstream.** A firmware owner
   re-applies saved frequency and voltage changes during an active lease:
   - It raises voltage before frequency and lowers frequency before voltage,
     stepping frequency with the same ramp plans as preparation and shutdown.
   - Every step stays inside the runtime safety predicate.
5. **The budget is mining time; shutdown is bounded on its own.** Amended
   2026-10-16 by the owner's decision, replacing "the budget reserves the
   catalog-maximum tail".
   - A grant's `maximumActiveMilliseconds` is the work window. The work gate
     closes at the budget, and the ledger charges the budget.
   - The ordered shutdown then runs outside the budget. It is bounded by the
     firmware's closed plan for the frequency actually running: at most
     22,550 ms at the catalog maximum.
   - The Gate's deadline for restoring the baseline allows the maximum tail
     plus cooling.
   - Safety does not change: revocation and expiry still start the shutdown
     within three seconds.
   - This replaces ADR-0033 decision 4 and ADR-0024's 14,450 ms window. A soak
     is 600,000 ms, charged as such, in a new soak ledger (fresh ordinals). The
     qualification charges (30,000 and 180,000 ms) keep their values but now
     mean mining time.
6. **The fan follows device configuration while mining.** That means the
   automatic fan controller or the manual duty, as on upstream. The fan is no
   longer pinned to 100%. Cooling proof and the ordered safe-stop keep their
   own fan commands, unchanged (ADR-0025).
7. **Evidence still gates claims, not configuration.** Pairs inside the
   catalog range may run, but a pair or fan policy without hardware evidence
   stays a non-claim:
   - Only 400 MHz/1100 mV and 485 MHz/1200 mV at 100% fan have evidence today.
   - New soaks under their own `TASKS.md` hardware contracts extend that
     evidence.
   - Qualification and acceptance campaigns set the device configuration they
     need before they start. The configuration is recorded in their evidence.

## Safety

These protections do not depend on the lease, and they stay as they are:
- the runtime revocation limits: input 4.5–5.5 V, power at most 15 W, ASIC
  below 75 C, a fresh nonzero fan reading;
- the ordered safe-stop with cooling;
- the catalog ranges;
- the 2.8 s heartbeat deadline and the time budgets of each lease.

The safety range moves from a two-pair allow-list to the board catalog
(decision 3). That range is the one upstream already offers for the Ultra 205.
No new mining authority is created: work still starts only from a signed
lease, and every hardware run still needs an active `TASKS.md` contract.

## Consequences

- HTTP settings and BAP `frequency`, `asic_voltage`, `fan_speed` and
  `autofanspeed` take effect as on upstream. That closes the divergence in
  `task-parity-bap-settings-effects` and in the HTTP settings path.
- This needs coordinated changes:
  - the Gate repo (`bitaxe-turnstile-system`) parser and the host signer, plus
    a new pinned Gate commit;
  - the firmware lease parser, actuation adapter, ramp plans and fan
    controller;
  - the soak tooling, which must set the device configuration before a soak
    instead of signing a profile.
- Soak and qualification evidence must record the device configuration that
  ran.

## Not decided here

- **Mining outside a lease.** In the Gate's design the Worker returns to
  ordinary local mining after a lease; this firmware returns to a paused
  baseline. That is a separate decision (`task-mining-baseline-after-lease`).
- **Other boards' catalog ranges and evidence.** They follow when those boards
  are in scope.

## Implementation

Planned tasks, in dependency order:
1. `task-asic-actuation-catalog-profiles`
2. `task-fan-policy-from-device-config`
3. `task-lease-without-hardware-profile`, which includes the Gate change and
   ADR acceptance
4. `task-live-hardware-settings-owner`
5. `task-device-config-profile-evidence-soaks`
