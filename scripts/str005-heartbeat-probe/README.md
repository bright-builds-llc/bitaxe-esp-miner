# Heartbeat shutdown successor

Software only. `inspect.mjs inspect --private-root <absolute-path>` is read-only;
`HARDWARE_ENABLED=false` and argument admission reject every effect command.
The active task must publish its exact prospective firmware/Gate admission and
producer command before this module is connected to hardware.

The browser coordinator accepts the existing Gate, startup admission, retained
status reader, failure-independent recovery collector, fixture release and a
passive observer. Prepare the passive observer before the fixture's ten-second
start window. Its `requireArmed` must independently check the actual child owner
and connected stream. Its `requireAlive` must reject observer loss. `stop` must
join and reap the observer using the existing bounded owner.

A normal 180,000-ms reservation and 60,000-ms initial grant are required. Select
the ordinal from fresh accounting; no renewals or accepted-share prerequisite.
Start must complete within 30 seconds; obtain same-generation work-ready and ASIC
dispatch, then use Gate's actual `suppressHeartbeats` no later than five seconds
after the reply and 35 seconds after invocation. The returned private status must
prove both lease and work-gate headroom of at least 5,000 ms. No null retained-status
query is permitted after Start. Keep the original baseline and fault checkpoint
on the same page through fresh-session recovery.

The coordinator waits 8,000 ms passively after suppression, covering the native
3,000-ms initiation ceiling plus 5,000 ms of observation. It then attempts Stop,
independent recovery, Close, observer termination and fixture release. Failures
stop promptly; an ambiguous late Start reply triggers another Stop/Close and
never another Start. Browser request timing never proves physical shutdown.

The independent judge uses retained native atoms: gate closure 2,800–3,000 ms after
the last valid heartbeat, shutdown initiation no earlier than closure and no
later than 3,000 ms after that heartbeat, and the 15,550-ms work/authority reserve.
It rejects post-cutoff dispatch/submission initiation, a different cause,
unrelated failures, generation changes, missing tail, missing recovery checkpoint,
missing charge, unproved cooling and resources. A pre-share cancellation may keep
its truthful `rejected/authority` outcome. This is not accepted-share evidence.

Persist only the existing projected retained-status shape; never persist socket
tuples, fixture credentials or authority material. Browser performance times and
supervisor timestamps are different clocks. The producer must retain a server
`heartbeat-start-claim.json` before Start and a `suppression-confirmed.json`
containing server `confirmedAtHostMs` plus browser `clientConfirmedAtMs`. Do not
compare their absolute values. Existing observer claim/journal/result/stop files
are independently rejudged by `judgeObserver`.

Startup 003 seal is a prerequisite, not permission to run on an old firmware or
infer the next ordinal. A new installed image requires exact fresh identity,
applicable native audits, capture/cutoff proof and the separately published
preparation contract. No flashing, clearing, replay or parity promotion is exposed
by this module.

## Runnable ownership and required publication

`main.mjs preflight|serve|finish` composes the existing startup server, signing,
fixture, preparation, native status and recovery seams. It is disabled until the
active task contains `Heartbeat probe hardware: enabled.` and one exact
`Heartbeat preparation sha256: <64 lowercase hexadecimal characters>.` line.
Preflight arguments are `--private-root`, `--preparation-root`, `--gate-root`
and `--fixture-binary`. Serve accepts only `--private-root` and
`--authority-directory`; finish accepts only `--private-root`.

Before preflight, use `preparation.mjs preflight|serve|finish` with its separate
`Heartbeat preparation hardware: enabled.` task gate. Its initial inputs are
`--private-root`, `--share-root` and `--gate-root`; serve and finish require
`--private-root` and `--stage recovery|restart`. It independently rejudges the
sealed accepted-share result, collects fresh known-attempt recovery, then uses
the existing restart-only page/observer. The same page proves preservation and
unchanged ledgers across exactly one software boot before closing. Neither this
stage nor heartbeat preflight issues a grant, clears a dump or flashes.

Both gates are currently disabled. Retain startup, share, capture, clear and
preparation evidence independently. Preparation and live consumption can use
separate published host revisions only when the preparation source is an ancestor
and its original contract hash still matches; Gate, installed firmware and
physical identity must match exactly. The new image's original capture/clear
admission is revalidated with the share producer adapter; its historical idle
state cannot substitute for the new restart and fresh live baseline.
