# Current recovery after startup001

Owner: `task-str005-mining-startup-probe`. This is a failure-only continuation,
not another startup attempt. Code and task admit one continuation only after
this reviewed source is committed and pushed. The exact immutable predecessor seal is
`950a8e55b5efb468905a4edd2e739cfd02e47013797e026bf0218af955234cb7`.

## Diagnosis and scope

Startup001 completed Start, increased dispatch and requested normal Stop. Its
collector then queried V2 status with a null attempt ID. Firmware treats null as
an idle-only query and rejects it when a retained record exists. The resulting
`command_rejected/invalid_transition` occurs before Gate's local
`v2_idle_correlation` condition and revokes the serial epoch. The same query on
fresh reconnect repeated the failure before collection began.

Confirmed Starts are now queried directly with their exact saved attempt ID.
A host-created ID does not prove Start occurred: unknown Start state deliberately
avoids either speculative status query. No fallback probes a revoked session.
The regression executes the real Gate command decoder, V2 controller and page
operations against a simulated firmware wire boundary with these exact rules.

This continuation admits only fresh authenticated reads, idempotent normal Stop,
and Close. It has no signer, grant, renewal, fixture, Start, core-dump clearing,
flashing, reset, mining or self-test route. It does not consume an ordinal.

## Execution contract

Use the absent child `scratch/str005-startup/recovery001/attempt` under an
ignored mode-0700 parent, with private mode-0600 sibling outputs. The
preflight reads the sealed startup context, confirmed Start record, exact
`before.attempt.id`, measured accounting and retained qualified Gate assets.
The current clean/pushed host source stays separate from installed firmware.

1. `just str005-startup-probe recover-preflight --startup-root <sealed-startup001> --private-root <new-child>`.
1. Collect `just detect-ultra205` privately as sibling `detector.stdout.log`.
   It must identify the predecessor's same single physical device and admitted
   Serial/JTAG runtime profile, with no holder and an age below 60 seconds.
1. `just str005-startup-probe recover-serve --private-root <child>`, retaining
   stdout/stderr privately. Native-connect the new read-only page to the exact
   installed identity, then collect current recovery once.
1. Collection independently attempts the ledger, original campaign budget,
   diagnostics and known-attempt retained status. Finally, it attempts Stop,
   current state and Close even after failures. Reads are bounded at 30 seconds;
   Stop and Close each at 150 seconds. Late reads cannot persist after their
   stage closed. Failures retain only closed stage/category labels.
1. Stop the owned server, collect fresh sibling `final-detector.stdout.log`, and
   run `just str005-startup-probe recover-finish --private-root <child>`.
   Verify host ownership, listener absence and holders on both original and
   current nodes before sealing the independent partial or complete observations.

Any effectful route is rejected. No Start or read retry is authorized by a missing
record. Preserve the result and stop if identity, accounting, restoration or
resource release is unproven. A missing retained record remains a missing proof.

## Claims and non-claims

The new page establishes its own current preservation baseline. It cannot import
or reconstruct the old page's private baseline or authorization checkpoint.
Results therefore always report historical preservation and authorization
checkpoint verification as false, startup qualification as incomplete, and no
parity promotion. They may independently establish current accounting, current
restoration, retained resource release and actual serial release. Startup001's
sealed evidence remains unchanged and cannot become qualified by relabeling
these current-only observations. Parity remains 90/95.
