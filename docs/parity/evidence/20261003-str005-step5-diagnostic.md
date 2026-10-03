# Step-5 diagnostic Start: the boundary did not recur

One bounded diagnostic Start ran on the Ultra 205 with the revocation-detail
firmware. It passed preparation step 5 (core-voltage stabilization) and reached
a running generation, unlike status001. This is diagnostic evidence only: no
accepted share, renewal or parity promotion is claimed.

| Boundary            | Direct evidence                                                                                                           |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| Installed image     | Source `654338d0101521490d90330c5a4a10e5ec32e5c2`, ELF `2641c24fc3f4fc3a80a4bcb8bfd389b2d70d3e048d9e18a5771b14309b588193`, Gate `86fc62d7` |
| Install provenance  | Phase-1b reinstall passed; all phase-2 audits passed on the frozen ELF ([projection](str005-step5-reinstall/attempt-001.json)) |
| Start               | One `normal` attempt, generation 4, zero renewals; reply after about 10 s; Stop requested 0.7 s after the reply             |
| Running proof       | Known-attempt status observed with the generation running; one job dispatched                                               |
| Revocation          | `restoration_requested` (the normal Stop); safe stop completed; no `unsafe_observation`, so no revocation detail was emitted |
| Safety readings     | Bus voltage 5.475 V before and 5.476–5.479 V after (status001 had 5.464/5.475 V); chip temperature 30–35 °C; fan nonzero    |
| Accounting          | Ledger next 23 / last 22 / charged 2,460,000 ms, not pending; original budget complete                                      |
| Retained record     | Terminal; socket closed, worker quiescent, no fence retained                                                                |
| Current state       | Restoration and baseline confirmed, lease inactive, `mine_on_boot=false`, serial released                                    |

The sealed result is `complete=false` because of three blockers. None of them
is a device-safety failure:

- **Current recovery incomplete.** The status-repro judge requires
  `authorization_high_water_match`, which is false after a successful signed
  Start that advances the authorization high-water mark.
- **Fixture natural completion failed.** The local fixture did not finish its
  scripted session within the deliberately short run.
- **Owner cleanup failure.** The page's 10-second release request was rejected
  (`startup_operation_rejected`) while the fixture was still active.

Interpretation: at the same near-ceiling bus voltage, the step-5 safety
predicate did not revoke this run. So status001's revocation is not explained
by that reading alone, and its cause stays unreproduced. The firmware now
records the trigger, failing fact, value and age of any future
`unsafe_observation`, and the harness keeps that line together with the
Worker's rejection. Raw device, signer and fixture inputs stay in ignored
private roots; this summary carries only redacted categories and values.
