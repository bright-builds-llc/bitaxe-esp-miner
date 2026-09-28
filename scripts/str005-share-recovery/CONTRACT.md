# Share001 failure-only current recovery

Owner: `task-str005-v2-accepted-share-probe`. Disabled until reviewed source and
this exact contract are committed and pushed. Command surface:

```text
just str005-share-recovery preflight --private-root <fresh-absolute-root> --share-root <sealed-share001> --gate-root <pinned-gate>
just str005-share-recovery serve --private-root <same-root>
just str005-share-recovery finish --private-root <same-root>
```

Publication requires the compiled enable flag and exactly one active task with
both exact lines:

- `Share failure recovery hardware: enabled.`
- `Share failure recovery seal: 1125f5d0dea1fa310ad3001cfa3c2aaa775924aacd1240a68555021ce559261e.`

## Identity and privacy

The sole predecessor is the failed Share001 sealed inventory named above. Keep it
immutable. Its exact running firmware is
`f000872f2e436aa7cdaa8cbfa41eee965a27731e`, ELF
`a3e257418d625aef5e9423fe28f598377f92b092b634264d3cc1e86b4e83e3c2`, Gate
`9643e87664397a321c715a3a1b1bb6c1183b83ea`. Verify its sealed context, original
attempt ID and before ledger. That ledger is historical reference only; it never
supplies guessed post-failure accounting. Current boot must be measured after
failed boot15; do not require or infer a particular next boot.

Require clean pushed firmware and Gate source, exact MODULE Gate pin, original
Gate asset hashes and unchanged task/contract during the run. Only fresh ignored
0700 roots and 0600 private artifacts/logs are admitted. No raw endpoint,
credential, possession binding, signed authority or device log is public output.
Diagnostics are projected through the pinned Gate parser. Raw errors never enter
records; earliest closed-category error remains separately recorded.

## Allowed flow

Before serving, save `just detect-ultra205` output privately in the parent as
`detector.stdout.log`; it must be \<=60 seconds old and identify the same sole
physical device in native serial runtime. Require no existing serial holder.
The command opens one local browser owner, never another serial controller.

Connect the exact installed image for the Gate before baseline. The page stops
and closes independently, records that same-page preservation baseline, then
configures candidate identity and requires a fresh native reconnect. A one-use
server challenge and fresh authenticated possession binding admit collection;
only a digest of that binding is retained. The candidate baseline ID must equal
the before baseline ID.

Collect ledger, original budget and diagnostics independently of status success.
Stop, refresh current restored state, then query idle-first. Only the Gate's exact
`v2_idle_correlation` category permits one fallback query using Share001's known
attempt ID. Other rejection categories, wrong attempts, malformed records and
transport failures remain failures. Close runs independently even after a failed
begin, status, Stop or timeout. No automatic retry is permitted.

Each observation has a one-use server stage token and \<=30-second deadline.
Expired or completed stage writes reject, including late diagnostics. The finite
collection admission window is ten minutes, separate from the proof's freshness
window. Stop and Close each retain their existing \<=150-second safety/cleanup
bound. Their observed result and the error summary can be preserved even after
ordinary observation deadlines expire.

No signer, fixture process, grant/reservation, Start, renewal, mining, heartbeat
suppression, reset, flashing, core read or core clear is available. There is no
force option. This collector never selects an ordinal.

## Finalization and proof

Close the page and stop the repository server. Save another fresh detector output
as `final-detector.stdout.log`. Finalization verifies actual server process and
listener disappearance and absence of holders on every observed serial node.
A failed or missing release check remains an explicit blocker; seal partial
artifacts and their typed first failure.

`current_safe_recovery` means the measured current ledger is settled, the
original budget is exhausted, exact identity and current restoration/preservation
match, device authority is inactive, and actual host/serial resources released.
An idle status cannot prove historical device-resource release; report
`historical_resource_unavailable=true`. A retained record must independently prove
terminal/quiescent/socket-closed/fence-released state for the known attempt.
Qualification and parity promotion are always false; parity remains 90/95.

Only finalization of fresh verified safe evidence may produce
`current-recovery.json`: existing V1 for actual idle or V2 for actual retained
release. Bind exact source/ELF/Gate/physical identity and measured accounting.
Its timestamp is the immutable actual collection begin, never finish time; reject
proof creation after 120 seconds. A complete current recovery may therefore have
no fresh effect proof. Core acquisition, if later authorized, has its own command,
contract and one-use proof claim; this command cannot perform it.

## Verification and stop conditions

Run `just test`'s scoped `//scripts:str005_share_recovery_test` before enabling.
Cover disabled/future/duplicate task gates; wrong predecessor and identities;
actual idle and retained branches; no ordinal assumptions; fresh challenge/session;
missing restoration/cleanup; typed-only fallback; independent failed reads;
late writes; stale proof; immutable failed finalization; and real Gate
parse/transition behavior. No test may open a device.

Any changed source, identity, seal, stale detector, pending accounting, unavailable
restoration or ambiguous cleanup stops dependent effects. Preserve the outcome.
Another collection uses a new root after demonstrated progress and publication;
never rewrite Share001 or turn current-state proof into historical qualification.
