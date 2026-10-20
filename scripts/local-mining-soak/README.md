# Local mining soak

`just local-mining-soak preflight|run|pause` runs one local mining soak on the admitted
Ultra 205 (ADR-0038 task 6). Each soak needs its own exact enable line,
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
   5 W, both journaled. Then it judges.

**Crash tripwire:** a run refuses (`soak_previous_panic`) before any write
when the device's last reset was a panic. That boot's dump waits in the
core-dump partition, so read it first under `task-handover-panic-diagnosis`.

`just local-mining-soak pause --private-root P/attempt-NNN` is the recovery
action for a run that died. It pauses the device and confirms the ASIC
powered down. Before the resume, the soak requires that same proof on the
paused baseline, and each limit judges only a fresh reading.

Live stops: a reset, a device pause (`pauseReason`), a hardware or power fault,
an overheat. While mining it also stops on input outside 4.5–5.5 V, more than
15 W, an ASIC at 75 C or above, a stopped fan, or readings stale on two
samples in a row. It also stops if the
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

`P/attempt-NNN` holds `context.json`, `journal.ndjson` and `result.json`.
These hold metadata only: the endpoint stays in `P/endpoint`, and no pool
value is recorded. Commit only numeric categories from `result.json`, after
`just verify-redaction`.
