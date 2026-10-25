# Local mining soak

`just local-mining-soak preflight|run|pause` runs one local mining soak on the admitted
Ultra 205 (ADR-0038 task 6), or, for `gamma601-first-001`, on the admitted Gamma 601. Each soak needs its own exact enable line,
`Ultra 205 local mining soak <id> hardware: enabled.` (or the soak's own
`enableLine`), in the active block of the task that owns it. A soak names
its task in `SOAKS` (`task`); otherwise it is the archived
`task-local-mining-evidence-soaks`, so soaks 001 to 005 now refuse. The
handover loop and the lease check belong to `task-handover-panic-diagnosis`.
`--soak` at preflight chooses the soak:
- `001` mines 30 continuous minutes;
- `002` accumulates 2 hours of mining within 2 h 30 min and must show a
  nonzero hashrate. It tolerates outages the device recovers from on its
  own, up to 60 s each, and counts them.
- `003` is soak 002 at several hours: 4 hours of mining accumulated within
  5 hours, with the same outage tolerance.
- `004` repeats soak 002 (2 hours within 150 minutes) to catch outages
  with their break logs.
- `005` runs overnight: 8 hours of mining accumulated within 9 hours, with
  the same outage tolerance and break logs.
- `006` repeats soak 003 (4 hours within 5) on the PSRAM atomics fix
  (ADR-0039), owned by `task-local-mining-atomics-soak`.
- `handover-001` is the handover loop. Its enable line is `Ultra 205 local
  mining handover loop hardware: enabled.` It mines locally while the
  operator repeats a lease-free handover: a refused Gate Connect, an admitted
  Connect left idle, and a closed page, after which local mining comes back.
  It passes after 20 recovered breaks with no reset. A break that has not
  recovered within 10 minutes stops it, as does any live stop.
- `lease-001` is the lease interruption check. Its enable line is the Gate
  soak's interruption line, `Ultra 205 lease interruption check hardware:
  enabled.`, so one line enables both halves. See [Lease interruption
  check](#lease-interruption-check).
- `overheat-001` is the overheat check, owned by
  `task-local-mining-overheat-check`. Its enable line is `Ultra 205 local
  mining overheat check hardware: enabled.` It runs only on the test-only
  firmware variant. See [Overheat check](#overheat-check).
- `overheat-002` is the same check from the default 485 MHz / 1200 mV,
  owned by `task-local-mining-overheat-stays-stopped`. Its enable line is
  `Ultra 205 local mining overheat stays-stopped check hardware:
  enabled.` The lowered step (385 MHz) would leave the catalog, so the
  trip must leave the device paused by itself (`pauseReason` `overheat`),
  with no restart.
- `stopping-001` is the stopping check, owned by
  `task-mining-activity-stopping`. Its enable line is `Ultra 205 mining
  activity stopping check hardware: enabled.` It mines 3 minutes (outages
  tolerated as in soak 002), then pauses. It journals every sample from
  the pause until the stop confirms (`stop_sample`). The status must move
  only from `active` to `stopping` to `paused`, show `stopping` at least
  once, and never read `active` while fresh power shows the ASIC idle
  (5 W or less). Before the fix it read `active` there for 13-18 s.
- `stopping-002` repeats `stopping-001` on the polled cooling proof
  (ADR-0016), owned by `task-safe-stop-polled-cooling`. Its enable line is
  `Ultra 205 polled cooling stopping check hardware: enabled.` Every
  `stopping` sample after the first must also read a zero hashrate, not
  the value frozen at the stop (`stop_hashrate_frozen`).
- `watchdog-001` repeats `stopping-002` and also requires the owner's
  task watchdog fresh in every sample, through preparation, mining and the
  stop: every sample must read `feed_fresh` (`watchdog_not_fresh`). Samples record runtime health's
  `taskWatchdogReason` and owner subphase. It is owned by
  `task-preparation-rollback-cooling-watchdog`; its enable line is
  `Ultra 205 preparation watchdog check hardware: enabled.`
- `regression-001` is phase C's regression check on the Ultra 205
  (ADR-0041), owned by `task-gamma-601-bring-up`. Its enable line is
  `Ultra 205 board seam regression check hardware: enabled.` It applies
  `watchdog-001`'s checks. Before any write, the baseline must also report
  `storedBoard` `205` with `boardIdentity` `match`
  (`soak_board_identity`).
- `boot-resume-001` is the boot-resume check, owned by
  `task-local-mining-boot-resume`. Its enable line is `Ultra 205 local mining
  boot resume check hardware: enabled.` See [Boot-resume
  check](#boot-resume-check).
- `gamma601-first-001` is the Gamma 601's first local mining, owned by
  `task-gamma-601-bring-up`. Its enable line is `Gamma 601 first local
  mining hardware: enabled.` It runs only on the test-only
  `gamma601-mining` firmware. See [Gamma 601 first
  mining](#gamma-601-first-mining).
- `gamma601-overheat-001` is the overheat check on the Gamma 601, owned by
  `task-gamma-601-bring-up`. Its enable line is `Gamma 601 local mining
  overheat check hardware: enabled.` It runs only on the test-only
  `gamma601-overheat-trip-test` firmware. See [Gamma 601 overheat
  check](#gamma-601-overheat-check).

There is no Gate lease. The device mines the pool saved in its NVS once the
operator resumes it, and stops when paused. The soak:

1. reads the station endpoint from a completed `just otawww-endpoint` handoff
   in the same session (at most 30 minutes old);
2. confirms the installed firmware, the handed-off boot and the paused
   baseline (`miningPaused`, `startMiningOnBoot` false, no `pauseReason`);
3. saves the owner's pool from the ignored `pool-credentials.json` as the
   primary pool (owner decision 2026-10-17: keep it);
4. resumes, samples `/api/system/info` every 2 s, and stops at the soak's
   mining time, at the first live stop, or at the run deadline (soak 001:
   40 minutes; soak 002: 150 minutes);
5. pauses on every path from the resume on, then waits up to 180 s for the
   ordered safe-stop: two samples in a row with fresh input power at most
   5 W (the 601: 6.5 W), both journaled. Then it judges.

**Crash tripwire:** a run refuses (`soak_previous_panic`) before any write
when the device's last reset was a panic. That boot's dump waits in the
core-dump partition, so read it first under `task-handover-panic-diagnosis`.

`just local-mining-soak pause --private-root P/attempt-NNN` is the recovery
action for a run that died. It pauses the device, turns mining on boot off
if it reads on, and confirms the ASIC powered down. For the overheat
check it also waits for the recovery to settle and restores the settings
that the journaled baseline recorded. Before the resume, the soak requires that same proof on the
paused baseline, and each limit judges only a fresh reading.

**Mining** in every soak means hashing: the status reads `active`, not
paused, and no fresh input power reading shows the ASIC idle (5 W or
less; the 601: 6.5 W or less). A stale power reading counts as hashing; the stale-readings live
stop bounds staleness. Mining time, outages, spans, breaks, the start and
restore checks and the break logs all count hashing, and so do
`first_mining_*` and `local_stopped_after_ready_ms`. Runs recorded before
2026-10-20 counted the status.

The status alone read `active` at 0.8–4.8 W while the ASIC idled: for
13–18 s once in soak 005 and around every lease check's interrupt, and for
single samples in the handover loop. Ramps have read from 5.03 W, so the
5 W threshold has little margin there; at most it moves a break's edge by
one sample. Plain soaks hash above 10 W. Each verdict reports `status_only_samples`, the
samples where the status said mining but the ASIC was not hashing. The
live limits below still apply whenever the status says mining, and the
limit maxima in each verdict cover those samples.

Live stops: a reset, a device pause (`pauseReason`), a hardware or power fault,
an overheat. While mining it also stops on input outside 4.5–5.5 V, more than
15 W (the 601: 22 W), an ASIC at 75 C or above, a stopped fan, or readings stale on two
samples in a row. The 601 also stops on a regulator above 90 C. It also stops if the
device is unreachable for 30 s, or if mining has not started 5 minutes after
the resume.

## Sequence

With `P` a new mode-0700 parent under `scratch/local-mining-soak/`, in a shell
with `umask 077`:

1. `just otawww-endpoint preflight|serve|finish` with
   `--private-root P/endpoint`, and a Connect in the dedicated Gate tab, again
   after each reload the page asks for
   ([attempts](../otawww-endpoint/README.md#attempts)). The device must be
   paused, so the Hello is admitted and the link is quiet afterwards.
2. `just local-mining-soak preflight --soak <id> --private-root P/attempt-NNN --endpoint-root P/endpoint --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --pool-credentials pool-credentials.json`
3. `nohup caffeinate -is just local-mining-soak run --private-root P/attempt-NNN --pool-credentials pool-credentials.json > P/run.stdout.log 2> P/run.stderr.log &`.
   `caffeinate` keeps the host awake, because a sleeping host stops watching a
   mining device. `-s` needs AC power, so keep the Mac on AC power and the lid
   open.

## Break logs

When mining resumes after a break, or the device answers again after being
unreachable, the soak reads `/api/system/logs` at once. The retained log
holds about 40 minutes of lines, so a later read would miss the break. The
raw log goes to `break-log-NNN.private.txt` (mode 0600), because it can hold
the station address. The journal's `break_log` event keeps only the
value-free `wifi_reconnect=`, `wifi_reconnect_probe=` and `local_mining=`
lines from 60 s before the break to its end. A failed read is journaled as
`break_log_failed` and never stops the soak.

## Lease interruption check

`--soak lease-001 --lease-root R` pairs this soak with one
`just ultra205-soak --interrupt-local-mining` run whose private root is `R`.
`R` must not exist at this preflight. Over HTTP a lease's mining looks like
local mining, so the check follows the order of events, journaling each
phase:

1. **before**: 5 minutes of local mining (`lease_ready`).
2. **interrupt**: the first Gate Connect is refused, and its Hello stops local
   mining (`local_stopped`). The Gate soak then runs its lease. The phase
   ends when `R/result.json` exists (`lease_ended`, with the Gate soak's
   result and closed failure categories). If local mining never stopped,
   the check stops at once.
3. **restore**: local mining must be back within 3 minutes of
   `lease_ended` (`local_restored`): 15 s after the link goes quiet, plus the
   pool probe and the ASIC's start. The clock does not wait on the operator:
   `submitSoakCompletion()` closes the link before the Gate soak writes its
   result.
4. **after**: 5 more minutes of local mining, then the pause.

Outages up to 60 s are tolerated in the local phases only; the lease's own
idle time is never an outage. The hashrate and share proofs come from the
local phases only, because the lease's mining reads the same over HTTP; the
Gate soak judges the lease's own. `local_stopped` is the first idle sample
after `lease_ready`; the Gate soak's refused Connect shows its cause.

A pause sent while the lease runs (at the deadline, on an unreachable device
or an interruption) cannot be confirmed, because the lease's session shows
the run intent. The requested pause still holds, and the device stops when
the lease ends; run the recovery `pause` to confirm it. The run deadline is 90 minutes. The verdict
passes only when every phase happened in order, the Gate soak `passed`, and
every soak 002 criterion holds over the whole run.

## Boot-resume check

`--soak boot-resume-001` checks `startMiningOnBoot` across two software
restarts, journaling each step:

1. It admits the paused baseline and saves the pool, as every soak does.
2. It turns mining on boot on and reads it back (`mine_on_boot_enabled`).
3. It requests one restart. Within 180 s a new boot session must report
   reset category `software_cpu`, the same firmware and the next boot
   ordinal (`rebooted`).
4. With no resume command, mining must start within 5 minutes and
   accumulate 5 minutes within 15, with outages up to 60 s. Both limits
   count from the check's first sample of that boot.
5. It turns mining on boot off, reads it back, pauses and confirms the ASIC
   powered down.
6. It requests a second restart, observed as in step 3. The new boot must
   stay paused, not mining, with mining on boot off, until a sample 120 s
   after its first, and end powered down.

Each restart is requested once. An ambiguous response
(`restart_ambiguous`) is resolved by watching for the new boot, never by a
second request. While mining on boot is on, the Gate refuses Connect, so
only the handed-off endpoint can reach the device. Every stop from step 2
on therefore pauses at once, then turns mining on boot off if it does not
read off (retried for up to 180 s), then confirms the ASIC powered down.

## Overheat check

`--soak overheat-001` needs the test-only firmware variant
`overheat-trip-test` (`just package-overheat-trip-test`, ADR-0040). Its
local overheat trip is 55 C instead of 75 C, so the recovery fires at
normal mining temperatures, about 60 C. Preflight refuses a package whose
`firmware_variant` does not match the soak: this check runs only on the
test variant, `gamma601-first-001` only on `gamma601-mining`,
`gamma601-overheat-001` only on `gamma601-overheat-trip-test`, and every
other soak only on production.

1. It admits the paused baseline and saves the pool, as every soak does,
   and saves 500 MHz / 1200 mV. From the default 485 MHz the lowered pair
   would leave the catalog (400 MHz floor), so recovery could not restart.
2. It resumes. The chip reaches 55 C within about two minutes of hashing,
   and the trip sets `overheat_mode` and the fan to manual 100%
   (`overheat_tripped`).
3. After 30 s of readings at 45 C or less, mining must restart at
   400 MHz / 1100 mV (`overheat_restarted`) and hash 5 more minutes. If it
   trips again, the next step would leave the catalog, so the device must
   pause itself (`overheat_stays_stopped`). Both outcomes pass.
   While cooling it reads the retained log once (kept privately) and
   requires the trip's cause: the revocation detail before the overheat
   `run_ended` line must name no failing fact (`trigger=unsafe_sample
   fact=none`), which only the lowered trip produces.
4. It pauses, then waits until the firmware's recovery has settled, for up
   to 15 minutes: the recovery continues while paused, and could otherwise
   save its lowered pair over the restored one. Settled means:
   - the settings it writes have not changed for 150 s;
   - the chip has read 45 C or less for the last 40 s;
   - the retained log shows the latest overheat entry followed by
     `overheat_recovered` or `overheat_stopped`, with no failed save after
     it.
   Then it restores the original frequency, core voltage, fan mode and
   speed, and `overheat_mode` 0, and watches them hold for 60 s. A device that paused itself
   is resumed once to clear its stop, which starts a short run, then
   paused again.

Live stops are soak 002's, except that the overheat and the device's own
overheat pause are expected. A break may last 180 s, the stop allowance,
because a trip's ordered safe-stop proves the chip cool before
`overheat_mode` shows; an unreachable device still stops it after 60 s. A hashing chip at 60 C means the lowered trip
did not fire, and stops the check at once (`overheat_trip_missed`). The
trip must come within 10 minutes of hashing, and the restart or the device
pause within 10 minutes of the trip. The run deadline is 45 minutes.

## Gamma 601 first mining

`--soak gamma601-first-001` is the 601's first local mining (ADR-0041
phase F3). Preflight takes the 601's package manifest,
`bazel-bin/firmware/bitaxe/gamma601/bitaxe-gamma601-package.json` from
`just package-gamma601-mining`, and refuses any other `firmware_variant`;
every Ultra 205 soak refuses `gamma601-mining`.

1. Before any write, the baseline must report `storedBoard` `601` with
   `boardIdentity` `match` (`soak_board_identity`).
2. It saves the pool, then, in a second `PATCH /api/system`, the soak's
   `profile`: `frequency` 400 and `coreVoltage` 1150. `/api/system/info`
   must read both back (`profile_saved`), or it refuses before the resume
   (`soak_profile_not_saved`). The profile stays saved afterwards.
3. It mines 10 continuous minutes and needs a nonzero hashrate and an
   accepted share. The run deadline is 20 minutes.

Its limits (`GAMMA601_LIMITS` in `judge.mjs`) differ from the 205's:

- power stops above 22 W, the firmware's own 601 stop (19 W target plus
  3 W margin);
- the 601 reads power as VOUT x IOUT plus a fixed 5 W offset, so a
  powered-down ASIC reads about 5.0 W. It counts as idle at 6.5 W or
  less, for the stopped proof, hashing and the recovery `pause`;
- `vrTemp` stops above 90 C when the device reports it, below the
  firmware's 105 C throttle;
- the chip (75 C) and input (4.5–5.5 V) limits are the 205's.

## Gamma 601 overheat check

`--soak gamma601-overheat-001` is the [overheat check](#overheat-check) on
the 601 (ADR-0041 phase F3.4), on the test-only
`gamma601-overheat-trip-test` firmware (`just
package-gamma601-overheat-trip-test`), whose local trip is 55 C. It runs
under the 601's limits (above), including their idle power for every
hashing, final and pause check. It differs from `overheat-001` in these
ways:

1. Before any write, the baseline must report `storedBoard` `601` with
   `boardIdentity` `match` (`soak_board_identity`). It must also read
   400 MHz / 1150 mV (`baselineProfile`, F3.3's measured pair), or the check
   refuses before the resume (`overheat_baseline_profile`): the cleanup's
   clearing resume runs at the restored baseline pair.
2. It saves 500 MHz / 1150 mV and resumes until the trip.
3. A stopped 601 has no chip reading: its chip reads unavailable with reason
   `asic_unpowered`. So cool, for the restart and for the settle wait, means
   that reason with a fresh `vrTemp` at 95 C or less (`restartCooling` in
   `GAMMA601_LIMITS`); `asic_not_initialized` or a stale regulator is not
   cool.
4. The restart must come at 400 MHz / 1050 mV (`restartProfile`) and show a
   nonzero hashrate while hashing there (`overheat_restart_no_hashrate`).
   Then it mines 5 minutes (`restarted`), or trips again and pauses itself,
   since the next step would leave the catalog
   (`restarted_then_stays_stopped`). Both outcomes pass.

The trip window, restart window, outage allowance, missed-trip stop (a
hashing chip at 60 C) and the 45-minute deadline are `overheat-001`'s.

`P/attempt-NNN` holds `context.json`, `journal.ndjson` and `result.json`.
These hold metadata only: the endpoint stays in `P/endpoint`, and no pool
value is recorded. Commit only numeric categories from `result.json`, after
`just verify-redaction`.
