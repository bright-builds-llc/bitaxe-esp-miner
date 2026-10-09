# BWG-007 serial restoration

`just bwg-restoration preflight|serve|finish|publish` runs one BWG-007
restoration attempt on one Ultra 205 through the browser Gate's restoration
page. The attempt covers eight scenarios in a fixed order: `completion`,
`pause`, `cancel`, `expiry`, `monotonic_uncertainty`, `disconnect`, `reboot`
and `authorization_negatives`. The decision behind it is
[ADR-0035](../../docs/adr/0035-serial-bwg-restoration-campaign.md).

The command adds no authority. It refuses to run unless
`task-bwg007-real-worker-restoration` is active and contains the exact line
`BWG-007 serial restoration hardware: enabled.` That task's contract still
governs every effect, the evidence, recovery and the stop conditions.
That task passed attempts 008 and 009 and is archived, so the command now
refuses. A future run needs a new active task with its own contract and the
enabling line, with `TASK` in `contract.mjs` pointed at it.
Preflight also requires the Gate commit to equal the `MODULE.bazel` Gate pin,
so the pin must first move to a Gate commit that carries the restoration page.

## What the owner does

- One serve owner and one Gate page load cover the whole attempt.
- Every scenario has its own Gate scope (`POST /activate` answers the same
  challenge on every connect of that scenario). `reboot` and
  `authorization_negatives` share one scope, so N1 can replay the pre-reboot
  Start.
- Leases are unbudgeted Conservative Stratum V1 grants: 60,000/20,000 ms, or
  30,000/10,000 ms with no renewal for `expiry`. Contract caps: at most 10
  signed Starts and 2 signed renewals per attempt, and at most 2 re-arms per
  physical scenario.
- Signing goes through the Gate development authority over stdin and stdout,
  so no signing intermediate reaches disk. Signed artifacts stay in memory and
  are delivered once. The reboot Start is kept in memory for N1 and burned
  after its one replay.
- The supervisor client (`/supervisor-client.mjs`, no imports) records each
  page operation's closed result with the page state to `R/records.jsonl`.
  Host events go to `R/campaign-events.jsonl`, watcher lines to
  `R/watcher.jsonl`, and each judged scenario to `R/scenario-NN-<name>.json`.
  Every file is mode 0600.
- The first unverified scenario stops the attempt. A new attempt reruns all
  eight scenarios.

## Sequence

`P` is a new mode-0700 ignored parent and `R=P/attempt-NNN` must not exist yet.
Run every command in a shell with `umask 077`, so redirected outputs are
mode 0600. Preflight and finish refuse other modes.

Every attempt must start on a fresh device boot. The clock stimulus is spent
once per boot and the rejection counter is per boot, so a boot already used by
an earlier attempt fails the first scenario with
`fact_stimulusCounterConsistent`. The install provides a fresh boot. Without
an install, the permitted `espflash board-info` resets the chip. Drive one
scenario at a time, and check its result before starting the next.

1. Install the exact package as the task contract specifies. Then run
   `bazel build //tools/flash:flash`, which the recipe also runs.
1. Run `just detect-ultra205 > P/detector.stdout.log`. Then run preflight within
   300 s of that detector output:

   ```sh
   just bwg-restoration preflight --private-root R --firmware-root <repo> \
     --gate-root <gate repo> --firmware-commit <HEAD> --gate-commit <pinned Gate> \
     --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json \
     --authority-directory <protected authority> \
     --pool-credentials <ignored pool file> --detector P/detector.stdout.log
   ```

   Preflight checks all of these and has no device effect:

   - both repos are clean and pushed;
   - the task line is present;
   - the Gate pin matches;
   - the exact package and admitted trust;
   - the Gate restoration page and bundle digests;
   - the presence watcher binary (`bazel-bin/tools/flash/flash`).

   It records the detected physical identity in `R/context.json`.

1. Start serve detached, with separate mode-0600 stdout and stderr files:
   `just bwg-restoration serve --private-root R --authority-directory <protected authority> --pool-credentials <ignored pool file>`.
   Serve listens on the Gate origin `127.0.0.1:48765`, re-checks the frozen
   sources before every signing, and writes `R/server-owner.json`.
1. In the dedicated Gate tab, open `http://127.0.0.1:48765/`. Before each
   journaled click, check `document.visibilityState` read-only. Press Connect
   (the trusted `#connect` gesture) whenever a step says `connect`. Run every
   other operation as `restorationSupervisor.run("<operation>")`, so its result
   is recorded. `GET /supervisor-state` always names the current step. Its
   fields are:
   - `checkpoint`;
   - `human_checkpoint_armed`;
   - `safe_state`;
   - `local_action`;
   - `instruction`, while the owner must act;
   - `observe`;
   - `automated_bounds`;
   - `waiting_for_human_has_no_deadline: true`;
   - `admission_settled`, true when the scenario's latest record allows the
     settle step below.

   **Settle** before every lease. Serve signs (`prepareStart`) and delivers
   (`loadScenarioLease`) a lease only when the current scenario's latest
   recorded page state was taken while connected and shows the device's
   admission diagnostic at stage `idle` or `complete`. Otherwise it answers
   `settle_required`. Run `admissionDiagnostic` and repeat it until it reports
   one of those stages; this wait has no deadline. The page refreshes the stage
   only when the device's periodic diagnostic arrives while connected, so poll
   after connecting. The diagnostic is non-authoritative: serve uses it only to
   hold back a lease until the previous lease's native shutdown has finished
   (attempt-006), never to admit one. A record from an earlier scenario never
   settles the next one.

   The steps for each scenario are:
   1. `completion`: connect, settle, `prepareStart`, `loadScenarioLease`,
      `startScenarioLease`, wait at least 20 s, `renewOnce`,
      `restoreChallengeSatisfied`, then `workerRestoration.submitCompletion()`.
   1. `pause` and `cancel`: connect, settle, `prepareStart`,
      `loadScenarioLease`, `startScenarioLease`, then `pause` or `cancel`, then
      `submitCompletion()`.
   1. `expiry`: as above without a stop. Poll `statusReview` until the device
      reports `lease_expired` (at least 30 s), then `submitCompletion()`.
   1. `monotonic_uncertainty`: connect, `clockDiscontinuityStimulusReview`,
      settle, `prepareStart`, `loadScenarioLease`, `startScenarioLease`,
      `triggerClockDiscontinuity`. Poll `statusReview` at most every 2 s until
      the device reports `monotonic_reset` (within 20 s), then
      `submitCompletion()`.
   1. `disconnect` and `reboot` (physical checkpoints, below).
   1. `authorization_negatives`, run right after the reboot and before any
      Start:
      - N1: connect, `statusReview`, then `replayArtifact` (the pre-reboot
        Start). The device refuses a Start until its once-per-boot reboot
        report has been delivered and acknowledged. The passing reboot
        scenario normally receives it, so serve releases the replay on that
        carried report, or else on a status in this connection reporting the
        reboot. After the device's rejection the page disconnects. Connect, then `authorizationRejectionReview`.
      - N2: settle, then `prepareStart` (the server signs one Start and holds
        it). Keep the page connected for 61 s, then `replayArtifact`. Connect,
        then `authorizationRejectionReview`.
      - N3: within 45 s of that review, `replayArtifact` (the same Start under
        the new possession). Connect, then `authorizationRejectionReview`.
      - N4: settle, `prepareStart`, `loadScenarioLease`, `startScenarioLease`,
        `renewOnce`, then `replayArtifact` (the accepted renewal). The device
        safe-stops with `control_failed`, then its own fail-safe disconnect
        re-confirms the stored reason as `connectivity_lost`, the scenario's
        terminal reason. Connect, then `authorizationRejectionReview`,
        `statusReview` and `submitCompletion()`.

1. After the last scenario, or after any failure, navigate the tab to
   `about:blank` with the page's own `location.replace('about:blank')`
   (AGENTS.md, Persistent Gate Browser Tab).
1. Stop serve, then seal with one of these:

   ```sh
   just hardware-operator owner-finish --owner bwg-restoration --private-root R \
     --pool-credentials <ignored pool file>
   just detect-ultra205 > P/final-detector.stdout.log && \
     just bwg-restoration finish --private-root R --pool-credentials <ignored pool file>
   ```

   Finish requires the server's process group gone, the port free, a fresh
   same-device detector (at most 300 s old) and no serial holder. It refuses
   without `--pool-credentials` (`restoration_finish_pool_credentials_required`);
   the file must be the same mode-0600, Git-ignored pool file serve used.
   Before sealing, finish scans every file in `R` for the exact pool endpoint,
   host, user and password (values of at least 8 characters, raw and
   JSON-escaped) and for credential shapes: compact JWS, base64url runs of 40
   or more characters that mix upper case, lower case and digits (never a
   lower-case hex digest), and `challenge_`/`lease_` identifiers with a random
   suffix. The pool values stay in memory and are never printed. Serve's
   verdict (`R/campaign-result.json`) is sealed as `R/result.json` with
   `credential_scan: {files, hits}`. Any hit makes the attempt `unverified`
   with `credential_in_evidence`, unless the campaign had already failed, in
   which case its earlier failure is kept. An attempt that never completed
   seals as `unverified` with `completion_missing`.

1. After an independent review, run
   `just bwg-restoration publish --private-root R`. It requires an unchanged
   sealed root, `result: passed`, a credential scan with zero hits, all eight
   scenarios passed and one context and device identity. It then writes
   `docs/parity/evidence/bwg-worker-restoration/bwg007-attempt-NNN-<scenario>.json`
   for all eight scenarios, or none if any write fails.

### Physical checkpoints (`disconnect`, `reboot`)

The owner acts only at the three human checkpoints. Every human wait has no
deadline; only the finite effects have automated bounds.

1. `awaiting_operator_ready`: no lease, no instruction. When the owner confirms
   readiness, the agent posts
   `{"scenario":"<name>","checkpoint":"awaiting_operator_ready"}` to
   `POST /checkpoint/ready`. A local tool may post without an Origin; a browser
   must be same-origin. The reply must name the current checkpoint, so a stale
   reply is refused.
1. `ready_for_lease`: settle, then run `prepareStart`, `loadScenarioLease`,
   `startScenarioLease` and `statusReview` (required before `reboot`, to
   capture the pre-reboot high-water epoch). Then run `beginPhysicalWindow`.
   Begin starts the presence watcher (`flash usb-presence-watch`). The removal
   instruction becomes visible only after the watcher has proved the admitted
   device present (bound 10 s).
1. `remove_usb` (USB only; barrel power stays) or `remove_power` (USB and
   barrel): the owner removes them now. Removal counts only when the watcher
   observes the disappearance, within 45 s of the instruction and at least 5 s
   before the delivered lease could end (delivery plus 60 s), so the device
   still sees the transport loss inside its lease. A later removal is expired
   authority (`rearm_required`), not device evidence.
1. `absence_bounding`: the absence must last at least 5 s (`disconnect`) or
   10 s (`reboot`). Then run `armPhysicalWindow`. Only then does serve print
   `action_token=bwg-restoration-restore-watcher-armed-v1 response_required=false`
   and publish the restore instruction. A device that reappears before that is
   expired authority.
1. `restore_usb` or `restore_power`: the owner reconnects USB, or restores
   barrel power first and then USB. No response is expected. The watcher
   requires the same physical identity with a new enumeration identity, then a
   3 s stability gate (accessible, unheld, Serial/JTAG runtime). Until then,
   `/activate` refuses every reconnect.
1. `reconnect_ready`: connect, `statusReview`, then `submitCompletion()`.

`rearm_required` waits for a new readiness reply (checkpoint `rearm_required`),
at most twice per scenario. End any page lease first with `cancel`.
`POST /checkpoint/cancel` with `{"scenario":"<name>"}` stops the attempt,
which seals `unverified` with `operator_cancelled`.

## Judgement

Each scenario passes only when all of these hold:

- the page closed cleanly;
- the device confirmed its baseline with the scenario's terminal reason;
- exactly one lease started in the scenario's segment;
- no page operation failed, apart from expected serial loss in `disconnect`,
  `reboot` and `authorization_negatives`.

Each scenario must also prove its own facts:

- `completion`, `pause`, `cancel`: the operator's own stop ended the lease;
  `completion` also needs an accepted renewal.
- `expiry`: no renewal; the device ended the lease with `lease_expired`, no
  sooner than 27 s after the lease was loaded (the 30 s window less a 3 s
  tolerance).
- `monotonic_uncertainty`: seven facts:
  1. an idle stimulus before the Start;
  2. an exact acknowledgement during the lease;
  3. the device, not the page, ended the lease with `monotonic_reset`;
  4. no restore, pause, cancel or renewal in the segment (a relabelled stop is
     rejected);
  5. the stop was observed within 20 s;
  6. no reconnect between the Start and the stop;
  7. the stimulus review is `consumed`, with the discontinuity counter exactly
     one above its pre-Start value.
- `disconnect`, `reboot`: watcher before the removal instruction; removal seen
  by the watcher; the absence bound met; the restore token before the restore
  instruction; the same physical identity with a new enumeration; stability
  before reconnect; at most 2 re-arms; page order begin → disconnected → armed
  → connected → terminal status. `reboot` also requires the stimulus reset to
  idle with count 0, no rejection and no high-water advance since boot, and a
  pre-reboot mining status.
- `authorization_negatives`: the reboot report delivered before N1, either by
  the passing reboot scenario (its carried `rebootReported`, with no lease
  started since) or by a status in N1's own connection. Then four device-attributed rejections, with rejection ordinals
  1–4 since the reboot and the device's wire categories
  (`authentication_failed`, except `admission_required` for N2):
  - N1: Start, signature valid, context mismatch, replay guard at or below the
    durable high-water, not advanced this boot. Its fingerprint was first
    observed no later than the pre-reboot epoch.
  - N2: Start, signature not evaluated, context expired.
  - N3: Start, signature valid, context mismatch, replay guard fresh.
  - N4: Renew, signature valid, context current, replay guard at or below the
    durable high-water, advanced this boot.

  Four `replay_rejected`, no `replay_accepted`, and one accepted renewal.

The stimulus counter must also be unchanged before `monotonic_uncertainty`,
one higher through `disconnect`, and reset after `reboot`.

Every scenario must also prove, from the page's identity and pool trackers
([ADR-0036](../../docs/adr/0036-measured-bwg-restoration-closure-facts.md)):

- `deviceIdentityStable`: from the first identity observation on, every
  recorded state and the final state show epoch 1 (one device-identity key in
  this page lifetime) with non-decreasing observations.
- `poolConfigurationUnchanged`: the device's own boot-snapshot comparison
  (preservation v2) never reports a change from the Start on, and a state
  after the lease ended shows more observations than at the Start.
- `disconnect` and `reboot` also need `sameKeyReacquired`: after the
  post-reconnect terminal status, a state shows epoch 1 with more
  observations than when the physical window began.

The page does not expose live samples, so the voltage, power, temperature, fan
and watchdog limits stay firmware-enforced. The judge applies the soak's
live-limit rule to any samples it is given. Private files hold only closed
page values. A record that echoes a signed authorization, lease or challenge
id, possession binding or pool value fails the attempt and is not stored.
Projections hold allowlisted digests, closed categories and `true` facts only.
Publish writes profile `bwg-worker-restoration-result/0.3`. Its booleans are
measured, not asserted:

- `baselineConfirmed` and `cleanupConfirmed`: the judge's baseline and
  clean-close checks for that scenario;
- `campaignEventCredentialsAbsent`: the seal-time scan found no hit;
- `sameDeviceAcrossScenarios`: `deviceIdentityStable` in all eight scenarios
  and epoch 1 in the last final state;
- `poolConfigurationUnchangedPerBoot`: that scenario's
  `poolConfigurationUnchanged`.

The validator still accepts the published attempt-008 files, which use profile
0.2. The result never promotes parity by itself.
