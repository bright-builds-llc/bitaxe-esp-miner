# Ultra 205 soak

`just ultra205-soak preflight|serve|finish` runs one Ultra 205 soak with 600
seconds of mining, using Stratum V1 against the owner's pool through the browser
Gate. The decisions behind it are
[ADR-0033](../../docs/adr/0033-signed-hardware-profile-and-soak-allowance.md) and
[ADR-0037](../../docs/adr/0037-device-configuration-owns-the-hardware-profile.md).

- **Profile:** the lease carries no hardware profile. When the observer starts,
  `serve` writes the soak's device settings (`SOAK_DEVICE_SETTINGS` in
  `device-settings.mjs`: 485 MHz, 1200 mV, automatic fan) with an HTTP PATCH,
  while the device is idle. It reads them back from `/api/system/info` and
  records both in the private `device-settings.json`. A read-back that does
  not match refuses the soak. `preflight --profile` chooses another entry of
  `SOAK_PROFILES`: `catalog-maximum-frequency` is 575 MHz at 1200 mV with the
  automatic fan. `serve` writes only the settings of the profile the context
  names.
- **BAP changes (optional):** `preflight --bap-changes` freezes
  `SOAK_BAP_CHANGES` (`bap-changes.mjs`) with the `upstream-default-bap`
  profile, which also stores the manual fan speed. `serve --bap-port
  /dev/cu.usbserial-…` then drives the settings through the accessory port,
  using the frozen `bap-client` binary, while the soak mines with its work
  gate open:
  - at 100 s: `show_new_block` 1 then 0, and `found_block` 0, each read back
    over BAP;
  - at 180 s: `frequency` 400, then `asic_voltage` 1100;
  - at 280 s: `fan_speed` 80, which selects the manual fan;
  - at 420 s: `asic_voltage` 1200, `frequency` 485, `fan_speed` 100, then
    `auto_fan` 1.

  Every operation must return its expected acknowledgement or read-back. The
  reference sends no acknowledgement for `fan_speed`, so that one must draw
  no refusal within the firmware's 40 s save window. The segments are judged
  as for live changes, and they also check the saved fan settings. The
  manual fan's median speed must be at least 1.3 times the automatic fan's
  before it.
- **Settings restore:** at completion, while the device is idle, `serve`
  sends one PATCH and records `settings-restore.json`:
  - a soak on another profile restores the upstream default, so later leases
    never inherit it;
  - a soak whose live changes stopped part-way restores its own profile.
  A failed restore fails the soak.
- **Live changes (optional):** `preflight --live-changes` freezes
  `SOAK_LIVE_CHANGES` (`live-changes.mjs`) into the context. While the soak
  mines with its work gate open, `serve` sends one PATCH at 200 s of mining
  (400 MHz, 1100 mV) and one at 400 s (back to 485 MHz, 1200 mV). It records
  each in `live-change-N.json`. Ending on the starting pair keeps the settings
  fingerprint equal to the page's baseline. `serve` accepts only that exact
  plan from the context.

  The judge needs every change recorded with a matching read-back. Segments
  run from 20 s after each change to the next request; the first segment
  starts at settled mining, and the last ends at the last open-gate record.
  In each segment:
  - every mining HTTP sample must carry the target settings;
  - there must be at least five fresh measured core-voltage readings and
    five nonzero counter-hashrate readings.

  Later segments are judged against the first, so a fixed ADC offset cannot
  fail them:
  - the median measured voltage must move by the settings' step, within
    30 mV;
  - the median hashrate must change by the frequency ratio, within 0.12.

  A soak that sent a change but not the whole plan restores its profile once
  at completion, and stays `unverified`.
- **Budget:** the soak budget (v2) is 600,000 ms of mining, charged in the v2
  soak ledger. The ordered shutdown runs after it, bounded at 22,550 ms.

The command adds no authority. It refuses to run unless
`task-parity-bap-settings-effects` is active and contains the exact line
`Ultra 205 upstream-default soak hardware: enabled.` That task's contract still
governs every effect, the evidence, recovery and the stop conditions. Its
predecessors (`task-ultra205-default-profile-soak-reverification`, `-002`
through `-006`, and `task-device-config-profile-evidence-soaks`) are
archived. After this task is archived the command refuses
again; a later soak needs a new active task with its own contract and the
enabling line, with `SOAK_TASK` in `contract.mjs` pointed at it.

## Interrupting local mining

`preflight --interrupt-local-mining` freezes a soak whose lease interrupts
local mining (ADR-0038 task 6). It pairs with `just local-mining-soak` lease
check `lease-001`, which watches this soak's `R/result.json`. It refuses live
changes and other profiles. It needs its own enable line,
`Ultra 205 lease interruption check hardware: enabled.`, in the active
`task-local-mining-evidence-soaks` block; the standard line never enables
it, and its line never enables a standard soak.

- **The first Connect is refused.** The device mines locally, so it ignores
  the Hello and stops local mining through its ordered safe-stop. The page
  fails with `connect_failed` at stage `hello`. Once the ASIC is down, reload
  and Connect again within the 5-minute hold. The judge excuses those
  refusals only before the observer starts, and requires at least one
  (`interrupt_refusal_unobserved`).
- **Reload quickly.** After `startObserver()`, reload and Connect within
  15 s. A longer quiet link lets local mining resume. The idle proof then
  fails, and the next refused Hello comes after the observer started, so it
  counts as `browser_failure`. That attempt cannot recover: no grant is
  signed, and the local soak pauses at its deadline.
- **Shares.** The device keeps the local run's share counters until the
  lease starts mining; the first mining sample carries the restart. The
  window judge (`--local-mining-interrupt`) allows one restart to zero, from
  a sample that was not mining, within the first 30-second window, and
  counts shares from it.
- **Final state.** The run intent survives the lease, so the last sample of
  each transport must show `miningPaused` false, not mining, and `mineonboot`
  off (`final_state_not_resumable` otherwise). Local mining resumes 15 s
  after the link goes quiet, after the observer stops; the local soak judges
  that.

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
      idle, writes the soak's device settings and starts the observer, which
      polls HTTP and reads the WebSocket on separate threads.
   1. Reload the page and press Connect again. The page compares the final
      state against the first settings snapshot it saw, so the snapshot must
      come after the write. The budget review refuses a page loaded before it
      (`soak_settings_baseline_stale`). The observer keeps running.
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

- The device ends the soak by its budget, after exactly 600 active seconds.
- Safe-stop completes at `fan_paused`.
- `mineonboot` stays false and the soak ledger is charged exactly once.
- Pool settings are retained.
- The final state is released, and it comes from the page that signed the
  soak, so its settings comparison covers the whole soak.
- Every running sample stays inside the live limits: 4.5–5.5 V, at most 15 W,
  below 75 C, fan above 0 RPM, watchdog alive.
- At least one accepted share.
- `soak_judge` credits all twenty 30-second HTTP and WebSocket windows, with no
  WebSocket gap over 5 s.
- Each transport sees mining stopped by the end of the ordered safe-stop
  (160 s after the gate closes), and shows the paused baseline after
  restoration (with `--interrupt-local-mining`, the run intent instead). The
  legacy 10 s rule measured from a serial marker that no
  longer reaches USB, and safe-stop now includes an up-to-120-s cooling proof.

The journal and records hold no pool, endpoint or credential values. The result
never promotes parity.
