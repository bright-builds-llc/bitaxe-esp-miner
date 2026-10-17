# ADR-0038: Mining after a lease

## Status

Accepted on 2026-10-16 by the owner (`task-mining-baseline-after-lease`).

## Decision

The owner chose **option B** and the three recommended sub-decisions:
1. **Local mining as upstream.** With no lease, the device mines its saved
   pool whenever it is not paused. A Gate Work Lease interrupts local mining,
   and the device restores it afterwards: the Mining Baseline. Device-local
   protection bounds local mining instead of a heartbeat.
2. **Overheat recovery as upstream, inside the catalog range.** After at least
   30 s and a chip reading of 45 C or less, mining restarts 100 mV and
   100 MHz lower. If that pair would leave the catalog range, the device stays
   stopped until resumed.
3. **Resume after resets, with a crash-loop guard.** Mining resumes on boot,
   but the device starts paused when the reset-retained boot record shows
   repeated abnormal resets in a short time.
4. **Mine by default.** A freshly configured device mines once Wi-Fi and a
   pool are set, with `startMiningOnBoot` defaulting to true. Test and
   evidence provisioning keeps writing false.

ADR-0016/0017's rule that a lease is a readiness requirement, and ADR-0033
and ADR-0037's "work starts only from a signed lease", are replaced for local
mining. They still govern every Work Lease. Until the implementation tasks
below land, the firmware stays lease-only.

## Context

**What the firmware does today**
- Nothing mines without a lease. Every mining path needs a bounded lease:
  either a signed Gate Work Lease, or the stale one-time campaign that the
  flash tool writes. The session checks for a lease before preparing the ASIC
  (`recovery_policy.rs`, `production_mining_session/readiness.rs`).
- `mineonboot` (`startMiningOnBoot`) only sets whether a boot starts paused or
  running. With no lease nothing mines either way, though `miningPaused` then
  reads false.
- `POST /api/system/pause` and `/resume` exist (API-009 is verified). They
  change only this boot's intent and cannot start mining.
- After a lease the ordered safe-stop runs: frequency down, reset held low,
  voltage off, ASIC off, fan at 100% until the chip reads 45 C or less, then
  fan at 30%. The device then stays idle.
- With no fan owner, no fan duty is written at all.

**What upstream does.** ESP-Miner mines whenever it is powered, connected and
not paused. Pause is held in RAM and starts false at boot.
- Its power task powers the ASIC down while paused, on a hardware fault, or
  when no pool is reachable. It starts it again when the condition clears.
- Its fan task runs 100% on overheat, 30% while paused or with no pool, and
  otherwise the PID or manual duty.
- On overheat (ASIC above 75 C or VR above 105 C) it stops mining and saves
  fan 100% and `overheat_mode`. After at least 30 s and a chip reading of
  45 C or less, it lowers voltage and frequency by 100 each and restarts on
  its own.
- It has no input-voltage or power cutoff for the Ultra 205.

**What the Gate expects.** The Worker has a Mining Baseline: "the Worker's
ordinary mining state captured before a Work Lease begins and restored when
that lease ends" (Gate ADR-0010, ADR-0017; `CONTEXT.md`).
- Between leases the Worker mines its own pool, which is configured locally.
- A lease interrupts that with the grant's pool, then restores it. Grant
  credentials are never saved.
- Remotely, the Gate may pause or resume ordinary mining. The pool,
  frequency, voltage and fan stay local (Gate ADR-0051).

**Why mining sits behind leases.** ADR-0016 and ADR-0017 made a lease a
readiness requirement. ADR-0021 added the 2.8 s heartbeat cutoff, and
ADR-0024/0033 added bounded budgets. These came from qualification:
hardware control needed evidence before anything ran unattended. ADR-0037
explicitly left mining outside a lease undecided.

**Protections that do not need a lease**
- the session's 1 s safety re-check, which stops on power above 15 W, input
  outside 4.5–5.5 V, or ASIC at 75 C or more;
- the ordered safe-stop with cooling proof;
- the catalog range checks (400–575 MHz, 1100–1300 mV);
- the task and supervisor watchdogs.

**Protections that exist only during a lease**
- the 2.8 s heartbeat cutoff;
- the lease and budget deadlines;
- the per-sample revocation on an unsafe reading or a zero fan reading
  (`revocation/unsafe_detail.rs`: it runs only while a lease is active);
- the fan controller, which needs a lease or campaign owner.

ADR-0037's safety list counts the heartbeat as lease-independent. In code it
is tied to the lease, so this ADR corrects that reading.

## Options

### A. Stay lease-only (today)

The device mines only under a signed lease and is idle otherwise.
- **For:** the strongest envelope, with a heartbeat and a bounded budget, and
  all of it already evidenced. Nothing new to build or soak.
- **Against:**
  - It stays a divergence from upstream and from the Gate's own model.
  - A Bitaxe without a running Gate does nothing.
  - HTTP and BAP settings, which now apply live, wait for the next lease.
  - `startMiningOnBoot` and pause/resume stay mostly cosmetic.

### B. Local mining as upstream, with leases taking over (recommended)

With no lease, the device mines its saved pool whenever it is not paused, as
upstream does. A Gate lease interrupts local mining, and the device restores
it afterwards: the Mining Baseline.
- **For:** device-user parity, which is the project's core value. It matches
  the Gate's model, makes saved settings and pause/resume meaningful, and
  needs no host.
- **Against:** it is unbounded mining with no heartbeat. Only device-local
  protections bound it. It needs new code and long soaks before any claim.

**What B needs**
1. **Device-local protection for every mining owner, not just leases.**
   - The unsafe-reading and zero-fan revocation must apply to local mining.
   - A local fan owner runs the automatic or manual fan while mining, and
     30% while paused or without a pool, as upstream does.
   - Our stricter limits stay: 15 W, input 4.5–5.5 V, and the catalog
     range.
2. **Upstream's power-down conditions.** Power the ASIC down while paused,
   after a pool failure (upstream retries, then probes every 30 s), and on a
   fan or sensor fault. Power it up again when the condition clears.
3. **A local owner.** The production session mines the saved primary or
   fallback pool with the saved profile. Reuse the campaign path's pool
   reading and fan owner, but drop its one-time budget.
4. **Lease handover.** Today lease Start is refused unless the session is
   idle. B needs:
   - an ordered stop of local mining before the lease prepares;
   - the lease;
   - restoration of local mining after the lease, or of the paused state if
     it was paused.

   Grant credentials are never saved. The preservation fingerprint keeps
   proving that.
5. **Boot behavior.** `startMiningOnBoot` decides whether each boot starts
   mining. Test and evidence tooling keeps provisioning it false, as it does
   now, so existing contracts are unchanged.
6. **Evidence.**
   - Soaks of local mining under their own contracts, growing from 30
     minutes to several hours, before any parity claim.
   - A hardware check that a lease interrupts local mining and restores it.
   - "Unbounded mining" stays a non-claim until those soaks exist.

### C. Opt-in local mining with a bound

Like B, but local mining runs only for an owner-set duration, for example
some hours, and then needs a local re-arm. Alternatively it needs a presence
signal, such as an open AxeOS page.
- **For:** a bound replaces the heartbeat.
- **Against:**
  - It diverges from upstream in a way users notice: a miner that stops on
    its own.
  - A presence signal defeats unattended mining.
  - The bound adds little safety beyond B's device-local limits, which are
    what protect upstream devices.

## Sub-decisions (chosen as recommended)

1. **Overheat recovery.**
   - Upstream restarts on its own at 100 mV and 100 MHz lower once the chip
     reads 45 C or less.
   - Recommended: match upstream, but only within the catalog range. If the
     lowered pair would leave the range, stay stopped until resumed.
   - The alternative is to stop and stay stopped until an operator resumes.
2. **After an abnormal reset** (panic, watchdog or brownout).
   - Upstream resumes mining on every boot.
   - Recommended: resume, but start paused when the reset-retained boot
     record shows repeated abnormal resets in a short time. This stops a
     crash loop from power-cycling the ASIC repeatedly.
3. **Default for a fresh device.**
   - Upstream mines once Wi-Fi and a pool are configured.
   - Recommended: the same, with `startMiningOnBoot` defaulting to true, as
     the schema already does. Provisioning for tests and evidence keeps
     writing false.

## Consequences

These tasks follow, in dependency order. Each hardware effect needs its own
`TASKS.md` contract.
1. `task-local-protection-every-owner`: device-local revocation (unsafe
   reading, zero fan) for every mining owner, and a local fan owner with
   upstream's 30% while paused or without a pool.
2. `task-local-power-down-conditions`: power the ASIC down while paused,
   after a pool failure, and on a fan or sensor fault, as upstream does. Add
   the overheat recovery above.
3. `task-local-mining-owner`: the production session mines the saved pool
   and profile with no lease.
4. `task-lease-handover-and-baseline-restore`: an ordered stop of local
   mining before a lease prepares, then restoration of local mining or the
   paused state afterwards.
5. `task-boot-resume-and-crash-loop-guard`: resume on boot and the
   crash-loop guard.
6. `task-local-mining-evidence-soaks`:
   - soaks of local mining from 30 minutes to several hours;
   - a hardware check that a lease interrupts local mining and restores it;
   - an overheat recovery check only if a safe, contracted stimulus exists.

   "Unbounded mining" stays a non-claim until these soaks exist.

The Gate needs no protocol change: it already expects a Mining Baseline. Its
tooling's `mine_on_boot=false` checks keep working for test contracts.
