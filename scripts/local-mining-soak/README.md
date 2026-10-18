# Local mining soak

`just local-mining-soak preflight|run|pause` runs one local mining soak on the admitted
Ultra 205 (ADR-0038 task 6). Each soak needs its own exact enable line,
`Ultra 205 local mining soak <id> hardware: enabled.`, in the active
`task-local-mining-evidence-soaks` block. `--soak` at preflight chooses it:
- `001` mines 30 minutes;
- `002` mines 2 hours and must show a nonzero hashrate.

There is no Gate lease. The device mines the pool saved in its NVS once the
operator resumes it, and stops when paused. The soak:

1. reads the station endpoint from a completed `just otawww-endpoint` handoff
   in the same session (at most 30 minutes old);
2. confirms the installed firmware, the handed-off boot and the paused
   baseline (`miningPaused`, `startMiningOnBoot` false, no `pauseReason`);
3. saves the owner's pool from the ignored `pool-credentials.json` as the
   primary pool (owner decision 2026-10-17: keep it);
4. resumes, samples `/api/system/info` every 2 s, and stops at the soak's
   continuous mining time, at the first live stop, or at the run deadline
   (the soak's time plus 10 minutes);
5. pauses on every path from the resume on, then waits up to 180 s for the
   ordered safe-stop: two samples in a row with fresh input power at most
   5 W, both journaled. Then it judges.

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
   `--private-root P/endpoint`, and one Connect in the dedicated Gate tab. The
   device must be paused, so the Hello is admitted and the link is quiet
   afterwards.
2. `just local-mining-soak preflight --soak <id> --private-root P/attempt-NNN --endpoint-root P/endpoint --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --pool-credentials pool-credentials.json`
3. `nohup caffeinate -is just local-mining-soak run --private-root P/attempt-NNN --pool-credentials pool-credentials.json > P/run.stdout.log 2> P/run.stderr.log &`.
   `caffeinate` keeps the host awake, because a sleeping host stops watching a
   mining device. `-s` needs AC power, so keep the Mac on AC power and the lid
   open.

`P/attempt-NNN` holds `context.json`, `journal.ndjson` and `result.json`.
These hold metadata only: the endpoint stays in `P/endpoint`, and no pool
value is recorded. Commit only numeric categories from `result.json`, after
`just verify-redaction`.
