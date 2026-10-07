# Upstream-default soak

`just ultra205-soak preflight|serve|finish` runs one 600-active-second Ultra 205
soak at upstream defaults (485 MHz, 1200 mV, 100% fan), mining Stratum V1 against
the owner's pool through the browser Gate. The decision behind it is
[ADR-0033](../../docs/adr/0033-signed-hardware-profile-and-soak-allowance.md).

The command adds no authority. It refuses to run unless
`task-ultra205-default-profile-soak-reverification` is active and contains the
exact line `Ultra 205 upstream-default soak hardware: enabled.` That task's
contract still governs every effect, the evidence, recovery and the stop
conditions.

## Sequence

`P` is a new mode-0700 ignored parent and `R=P/attempt-NNN` must not exist yet.
Run every command in a shell with `umask 077`, so redirected outputs are
mode 0600; preflight and finish refuse other modes.

1. `just package`, then install the exact package as the task contract
   specifies.
1. `just detect-ultra205 > P/detector.stdout.log`, then run preflight within
   60 s of that detector output:

   ```sh
   just ultra205-soak preflight --private-root R --firmware-root <repo> \
     --gate-root <gate repo> --firmware-commit <HEAD> --gate-commit <pinned Gate> \
     --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json \
     --authority-directory <protected authority> --detector P/detector.stdout.log
   ```

   Preflight checks that both repos are clean and pushed, the exact package,
   admitted trust, the pinned Gate bundle and the canonical `soak_observer` and
   `soak_judge` binaries. It records the detected physical identity and writes
   `R/context.json`.
1. Start `just ultra205-soak serve --private-root R --authority-directory <protected authority> --pool-credentials <ignored pool file>`
   detached, with separate mode-0600 stdout and stderr files. It listens on the
   Gate origin `127.0.0.1:48765` and writes `R/server-owner.json`.
1. In the dedicated Gate tab, open `http://127.0.0.1:48765/` and press Connect.
   Then call these page operations in order:
   1. `soakSupervisor.startObserver()`. This reads one station endpoint while
      idle and starts the observer, which polls HTTP and reads the WebSocket on
      separate threads.
   1. Wait at least 60 s; `GET /observer/idle-proof` reports progress.
   1. `workerAcceptance.submitCoolingReview()`.
   1. `workerAcceptance.submitBudgetReview()`, which reviews the device soak
      ledger in `soak` mode.
   1. `workerAcceptance.prepareStartAuthorization()`. The server signs only after
      the idle proof has passed: at least 60 s, at least 20 HTTP samples, one
      unbroken WebSocket, and no reconnect or failure.
   1. `workerAcceptance.loadSignedWindow()`, then `workerAcceptance.startWindow()`.
      The device ends work at 600 active seconds and completes safe-stop. The Gate
      stops only after that and never renews after the gate closes.
   1. After restoration, `workerAcceptance.submitSoakCompletion()`. The server
      waits for terminal samples, stops the observer, runs `soak_judge` and writes
      `R/result.json`.
   1. Navigate the tab to `about:blank` (AGENTS.md "Persistent Gate Browser Tab").
1. Stop the server, then seal with either of:

   ```sh
   just hardware-operator owner-finish --owner ultra205-soak --private-root R
   just detect-ultra205 > P/final-detector.stdout.log && just ultra205-soak finish --private-root R
   ```

   Finish requires the server's process group gone, the port free, a fresh
   same-device detector and no serial holder. A soak that never completed seals
   as `unverified` with `completion_missing`.

## Judgement

`R/result.json` is `passed` only when every one of these holds:

- The device ends the soak by its budget, with work admitted for 600–619.05 active seconds.
- Safe-stop completes at `fan_paused`.
- `mineonboot` stays false and the soak ledger is charged exactly once.
- Pool settings are retained.
- The final state is released.
- Every running sample stays inside the live limits: 4.5–5.5 V, at most 15 W,
  below 75 C, fan above 0 RPM, watchdog alive.
- At least one accepted share.
- `soak_judge` credits all twenty 30-second HTTP and WebSocket windows, with no
  WebSocket gap over 5 s.
- Each transport sees mining stopped by the end of the ordered safe-stop
  (160 s after the gate closes), and shows the paused baseline after
  restoration. The legacy 10 s rule measured from a serial marker that no
  longer reaches USB, and safe-stop now includes an up-to-120-s cooling proof.

The journal and records hold no pool, endpoint or credential values. The result
never promotes parity.
