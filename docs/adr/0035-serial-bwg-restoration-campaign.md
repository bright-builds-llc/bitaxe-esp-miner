# ADR-0035: Serial BWG restoration campaign with bounded clock and replay-attribution seams

Accepted 2026-10-07 under `task-bwg007-real-worker-restoration`, by the owner's
decisions of that date.

## Context

[ADR-0019](0019-supervise-bwg-restoration-through-a-protected-browser-campaign.md)
specified BWG-007's eight-scenario restoration campaign over WebUSB. ADR-0021
replaced that transport with the fixed Serial/JTAG controller and browser Web
Serial, and the WebUSB campaign scripts were removed. Two scenarios were also
blocked before effects:

- `monotonic_uncertainty` needs the Worker's real decreasing-clock detection
  (`enforce_clock`) to fire in process. A reboot is a different path, and
  `restore` with a `monotonic_reset` reason only relabels a stop.
- `authorization_negatives` needs durable replay to be told apart from
  possession-context rejection. The verifier rejects a mismatched context
  before it reads the persisted high-water, and the Gate reports every failure
  as `authentication_failed`.

## Decision

1. **One serial campaign.** A new host owner, `just bwg-restoration`, runs all
   eight scenarios in one attempt, through one serve owner and one Gate page
   on the persistent Gate origin. The order is:
   1. completion;
   2. pause;
   3. terminal cancel;
   4. exclusive expiry;
   5. `monotonic_uncertainty`;
   6. USB-only disconnect;
   7. both-power reboot;
   8. `authorization_negatives`.

   The first typed failure stops the attempt, and a new attempt reruns all
   eight. Physical checkpoints follow the Asynchronous Human Checkpoints and
   restore-watcher rules in `AGENTS.md`.

2. **Bounded clock-reset stimulus.** A possession-gated, one-shot controller
   command, `clock_discontinuity_stimulus`, is accepted only during an active
   non-V2 lease with at least 5,000 ms of headroom. It is armed only after its
   acknowledgement is confirmed sent. Within 2,000 ms of real time, the next
   idle `tick` sees a monotonic sample 1,000 ms below the last observation. The
   unchanged `enforce_clock` then detects the decrease and safe-stops
   `monotonic_reset`.
   - It is consumed once per boot, even if it expires, and is never persisted.
   - Only `WorkerControl`'s idle-tick sample is perturbed. Heartbeat liveness,
     native revocation and native lease deadlines keep reading the real clock.
   - A counter that only the decreasing branch increments, read back through
     `clock_discontinuity_stimulus_review`, proves the real path fired.
3. **Metadata-only replay attribution.** The verifier still checks digest, key
   and signature. It reads the persisted high-water without writing it, and it
   advances the high-water only on acceptance, so the acceptance set is
   unchanged. It keeps a RAM-only record of the last rejection, describing
   signature, context and replay-guard outcomes as closed categories.
   `authorization_rejection_review` returns that record and the existing
   high-water digest. It has no signing or write path and returns no key,
   sequence, lease, challenge or authorization value.
4. **No new budget ledger.** Restoration leases are unbudgeted Conservative
   Stratum V1 grants, which firmware already admits. Each lease is 60,000 ms
   with a 20,000 ms renewal window; the expiry scenario uses 30,000/10,000 with
   no renewal. The task contract caps each attempt at 10 Starts and 2 renewals,
   plus at most 2 re-arms per physical scenario. These budgets existed to
   protect specific acceptance and soak campaigns, and a ledger here would add
   complexity without lowering risk.
5. **Controller 0.4 stays.** The three new commands are optional, and older
   firmware rejects them as `invalid_request`. The Gate exposes them only under
   a new, mutually exclusive `restorationQualification` mode.

ADR-0019's identity, privacy, safe-stop and all-or-nothing batch-publication
rules carry over unchanged. This ADR supersedes its WebUSB mechanics, its CDC
runtime attestation and its two "blocked before effects" paragraphs.

## Safety

Live limits stay firmware-enforced:

- input 4.5–5.5 V;
- at most 15 W;
- ASIC below 75 C;
- a fresh nonzero fan reading;
- watchdog alive;
- the 2.8 s heartbeat deadline;
- the signed lease.

The stimulus can only stop work. It adds no actuation, budget or authority.
General fault injection, ad hoc control writes, direct UART or pins, erase,
factory reset, OTA, other devices or pools, the upstream-default profile and
Stratum V2 remain prohibited. Every hardware run still needs the active task
contract and its enabling line.

## Non-claims

This ADR verifies nothing on hardware and promotes no parity row.
