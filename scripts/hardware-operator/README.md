# Hardware operator helpers

`just hardware-operator <action> ...` replaces the scratch shell glue that agents
repeated during the STR-005 work. Each action runs one fixed sequence of
existing repo commands, refuses bad inputs before its first effect, and prints
one JSON line.

These helpers grant no authority. Before using any of them on hardware, an
active `TASKS.md` block must define the exact command, evidence and privacy
policy, recovery, retry bounds and stop conditions (the Effectful Hardware Task
Gate). Every detector, finish, serve, monitor and core-dump read they run is the
same repo command an agent would otherwise type by hand.

Exit codes: `0` success; `1` when the action finished but a command it ran
failed (nonzero `finish_exit` or `read_exit`, a failed heap window or
judgement, or a parent `operator_error` reply); `2` for a typed refusal, printed
as `{"event":"hardware_operator_error","code":"..."}`. All output files are mode
0600 under mode-0700 directories. Paths may be relative to the directory where
you ran `just`.

The owner, sequence and heap actions append step rows (codes, exit statuses and
counts, never device values) to `hardware-operator.jsonl` in the private
parent, so a run that stops partway shows its last completed step. They are
deliberately not rerunnable over their own outputs: an existing output file is
refused (`*_output_exists`) rather than overwritten, so evidence is never
replaced. Continue with a fresh root and parent under the task's attempt policy.

## Noise-serial operator

This is the repo-owned version of the scratch `parent.mjs`, `launch-operator.sh`
and `operator-cmd.sh`. It follows the same-page workflow in
[`../str005-noise-serial/README.md`](../str005-noise-serial/README.md).

```sh
just hardware-operator noise-launch --private-root "$ATTEMPT_ROOT"
just hardware-operator noise-send --private-root "$ATTEMPT_ROOT" --command status
just hardware-operator noise-send --private-root "$ATTEMPT_ROOT" --command install --index 0
just hardware-operator noise-send --private-root "$ATTEMPT_ROOT" --command browser-closed
just hardware-operator noise-send --private-root "$ATTEMPT_ROOT" --command stop
just hardware-operator noise-send --private-root "$ATTEMPT_ROOT" --command cleanup
just hardware-operator noise-send --private-root "$ATTEMPT_ROOT" --command exit
just hardware-operator noise-stop-holder --private-root "$ATTEMPT_ROOT"
just hardware-operator noise-await --private-root "$ATTEMPT_ROOT"   # only after a send timed out
```

- **`noise-launch`** creates `<root>.operator/` exclusively, so a second launch
  for the same attempt is refused with `noise_operator_already_launched`. It
  then makes `commands.fifo` and starts two processes in their own sessions:
  `scripts/hardware-operator/noise-parent.mjs`, reading the FIFO, and a holder
  that keeps the FIFO's write end open. Both survive the command and the
  agent's shell, so the parent can wait for the owner without a tool deadline.
  The launch waits up to 60 s for the parent's first line. If the parent
  reports an error or exits before it is ready, the launch also stops the
  holder; the parent itself stops its supervisor and exits on any failure
  before readiness.
- **`noise-send`** writes one command and returns the parent's single reply.
  The commands are `install` with `--index` 0–4, plus `status`,
  `browser-closed`, `stop`, `cleanup` and `exit`. A lock refuses concurrent
  sends. The reply wait has an automated 900 s bound (`--timeout-seconds`, at
  least 1); install runs a detector and a flash. The wait stops early if the
  parent exits. The parent answers in order, so the n-th command's reply is
  its n-th line after readiness. Each command is recorded in
  `commands.jsonl` before it is written, and its reply after it arrives.
- **`noise-await`** returns the reply to the last command after its own sender
  timed out or was killed (for example by a tool time limit shorter than the
  install). Until then `noise-send` refuses with `noise_operator_reply_pending`,
  so a late reply is never credited to the next command.
- **`noise-stop-holder`** refuses while the parent still reads commands
  (`noise_operator_parent_live`). Send `exit` first; the parent accepts it only
  after its supervisor has stopped.

Close the Gate page by navigating the dedicated tab to `about:blank` (AGENTS.md
"Persistent Gate Browser Tab") before sending `browser-closed`.

## Owner stop and finish

```sh
just hardware-operator owner-finish --owner str005-accepted-share --private-root "$ROOT"
just hardware-operator owner-finish --owner str005-step5-restart --private-root "$ROOT" --stage restart
just hardware-operator owner-finish --owner str005-control-diagnostic-recovery --private-root "$ROOT" --wait-collection
```

The owners are a closed table in `owners.mjs`:

- heartbeat: `str005-heartbeat-shutdown`, `str005-heartbeat-probe`
- share: `str005-accepted-share`, `str005-share-probe`
- recovery: `str005-share-recovery`, `str005-control-diagnostic-recovery`, `str005-panic-recovery`
- restart, which needs `--stage recovery|restart`: `str005-step5-restart`,
  `str005-startup-preparation`, `str005-heartbeat-preparation`
- soak: `ultra205-soak` ([upstream-default soak](../fixed-usb-soak/README.md))
- restoration: `bwg-restoration` ([BWG-007 serial restoration](../bwg-restoration/README.md))

The action reads `server-owner.json`, or `<stage>/server-owner.json` for a
staged owner. It sends SIGTERM only to the recorded process, and only while that
process still has the recorded pid, start time and process group. Then it waits
until the whole process group is gone and proves that no process listens on the
recorded port. It writes `final-detector.stdout.log`, or
`<stage>-final-detector.stdout.log` for a staged owner, in the root's parent.
Last, it runs `just <owner> finish --private-root <root> [--stage <stage>]`
into `finish.stdout.log` / `finish.stderr.log` in the same parent. Existing
output files, a missing owner record and a held port are all refused before the
next step.

`--wait-collection` first waits for the page's `finished.json`, but never past
the 120 s window opened by `collection-begin.json`.

## Restart sequencing

```sh
just hardware-operator restart-sequence --owner str005-step5-restart --private-root "$ROOT"
```

The sequence runs these steps in order:

1. Wait for the recovery stage's collection.
1. Stop and finish the recovery stage.
1. Require `recovery/result.json` to say `current_recovery_complete: true`.
1. Write `restart-detector.stdout.log`.
1. Start `just <owner> serve --private-root <root> --stage restart` detached,
   writing `restart-serve.stdout.log` / `.stderr.log`.
1. Wait for `restart/server-owner.json`. A serve that exits or is not ready
   before the window closes is stopped (its whole process group), so no
   untracked serve keeps the Gate port.

Every step must happen within 120 s of the recovery's collection-begin (`FRESH_MS`). An expired window is
refused before the owner is stopped or the serve starts. Afterwards, finish the
restart stage with `owner-finish --stage restart`.

## Recovery plus core-dump read

```sh
just hardware-operator recovery-core-dump --owner str005-control-diagnostic-recovery \
  --private-root "$ROOT" --core-root "$CORE_ROOT" [--wait-collection]
```

This action finishes the recovery owner and requires a fresh
`current-recovery.json`. It writes `core-detector.stdout.log` and checks that the
detected physical identity matches the proof. Then it starts one read-only
`just core-dump-read` while the proof is still inside its 120 s window. The
installed source, ELF digest and physical identity all come from the proof.
`core-dump-read` still enforces its own active-task line, its one-shot proof
claim and an absent core root.

## Passive heap capture

```sh
just hardware-operator heap-capture --parent "$PRIVATE_PARENT" --name idle --windows 3 --seconds 1200 \
  --min-free-bytes 16384 --min-largest-block-bytes 8192 --min-samples 20
```

For each window `<name>-NNN`, the action runs a fresh detector and then one
receive-only `just monitor`, writing into `<name>-NNN.capture.log`. Every window
must see the same physical identity. Windows are at least 360 s long (AGENTS.md
"Flash And Monitor Timeouts"). With all three thresholds, it judges every
window together with the `just internal-heap-series` rules and prints only
numeric categories. It never opens a Worker or a Gate connection.

## Tests

`bazel test //scripts:hardware_operator_noise_operator_test` and the other
`hardware_operator_*_test` targets run without hardware. They use real
processes, a real FIFO, real `lsof` port checks and a stand-in `just` on `PATH`.
