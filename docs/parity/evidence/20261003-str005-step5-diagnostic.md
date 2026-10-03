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

## Rerun: start004 seals complete

The judge fixes, a no-mining qualification restart and a fresh Start produced
a fully complete diagnostic result. Start003 was refused at baseline because
the previous Start's retained V2 record only clears on reboot; restart001
cleared it. Its reviewed evidence (boot 11 -> 12, ledger and budget unchanged)
anchors the rerun.

| Boundary          | Direct evidence                                                                                     |
| ----------------- | --------------------------------------------------------------------------------------------------- |
| Start             | One `normal` attempt on boot 12, generation 3, zero renewals; reply after about 10.3 s; Stop 0.76 s later |
| Running proof     | Known-attempt status observed running; one job dispatched                                           |
| Preparation       | Current-boot receipt: all 9 steps completed; internal heap free 4,211 bytes at the last step        |
| Revocation        | `restoration_requested` (normal Stop); safe stop complete; no `unsafe_observation`                  |
| Safety readings   | Bus voltage 5.479 V before and 5.473 V after; chip temperature 31–32 °C                              |
| Accounting        | Ledger next 24 / last 23 / charged 2,640,000 ms, not pending; original budget complete              |
| Fixture           | Exact peer only, natural exit, peer/socket/listener closed, no shares received (status-only design) |
| Result            | `complete=true`, no blockers, current safe recovery, host and serial released; step-5 summary `step5_passed` |

Two independent Starts on the instrumented firmware passed preparation step 5
at near-ceiling bus voltage. Status001's revocation therefore remains
unreproduced. Any recurrence would now be named by the retained revocation
detail.
