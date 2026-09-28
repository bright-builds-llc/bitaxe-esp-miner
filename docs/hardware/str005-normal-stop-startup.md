# Normal-stop startup qualification

Owner: `task-str005-mining-startup-probe`. This is a fresh successor to consumed
startup001, not a replay or amendment of its sealed result. The command is eligible only after this source, reviewed Gate pin and complete
implementation are verified, committed and pushed. Runtime admission gates
remain mandatory.

## Scope

One new normal allowance reserves 180,000 ms at an ordinal read from the device.
The signed lease is 60,000 ms with zero renewals. Require a Start reply within
30 seconds, observe increasing same-generation dispatch for at most five seconds
after reply, and request Stop immediately on success or failure, no later than
35 seconds after invocation. These are request bounds, not electrical-off claims.
Device heartbeat, lease and ordered shutdown/cooling limits remain unchanged.

There is no flash, core clearing, factory reset, NVS reset, external pool,
heartbeat suppression or self-test action in this command. A separate published
[preparation](str005-startup-preparation.md) owns the single no-mining restart.
No accepted share is required by this startup claim.

## Identity and prerequisites

Keep installed firmware `361425b9` and exact ELF `7f3ea3ce…7ebf2`. The full identities
and explicit historical/current Gate distinction are in the
[compatibility review](str005-normal-stop-compatibility.md) and its committed
machine-readable companion. Preflight checks the exact Gate pin and source diff;
wire/controller protocol and dependency files must remain unchanged.

Revalidate installation007's original capture and selected cutoff from their
actual producer files and bytes, keeping historical Gate d3ac374 immutable.
Run the real checksum/full-ELF/cutoff decoder again. Inspect the exact installed
ELF for native USB ownership symbols and selected signed-Start stack headroom.
No newly built firmware is substituted for the installed image.

Independently rejudge the sealed preparation's recovery and restart children,
matched nonce/ACK, explicit software-reset boot+1, current identity, same-page
preservation and unchanged ledgers. Confirm all owned servers/listeners exited.
The prior erased-core proof is reusable only through this complete lineage:
startup001 verified clear → startup001 admitted boot → recovery001 same boot →
fresh preparation recovery same boot → one observed software restart. The fresh
startup baseline must match the resulting boot exactly; any intervening reboot
or accounting change rejects admission.

## Commands and one-shot flow

Use `scratch/str005-startup/startup003/attempt` as an absent child under an ignored
mode-0700 parent. Keep stdout/stderr in distinct mode-0600 sibling files. The
prepared evidence is at `scratch/str005-startup/preparation003/attempt`.

1. After preparation seals successfully, run `just detect-ultra205` into sibling
   `detector.stdout.log`. Require the same single physical device, supported
   native profile, no holder and age at most 60 seconds.
1. Run `just str005-startup-probe preflight --private-root <absent-child> --preparation-root <sealed-preparation> --gate-root <reviewed-Gate> --fixture-binary <canonical-Bazel-fixture>`.
1. Run `just str005-startup-probe serve --private-root <child> --authority-directory <existing-protected-authority>`. Initial configuration
   neither opens the authority directory nor launches the fixture.
1. Native-connect, collect the exact fresh idle baseline and both ledgers, Close,
   and configure the candidate on that same page. Original private preservation
   state stays in the page; it is not reconstructed from public hashes.
1. Collect sibling `startup-detector.stdout.log` while the port is released,
   native-connect the candidate, and invoke the one startup button. Cooling and
   budget reviews precede signer preparation and the fixture's ten-second
   readiness window. The signer issues exactly one fresh grant, with no renewal.
1. Gate captures the actual private post-authorization observation from the
   validated Start response. Normal Stop first restores the device, then checks
   a fresh restored observation against that expected generation/hash. It cannot
   bless an unexplained authorization advance or manufacture a checkpoint.
1. Main-test failure cannot skip recovery reads, Close or fixture release. After
   closure, native-connect again on the same page. The original checkpoint and
   baseline must match fresh state; collect known-attempt retained status,
   accounting, restoration and actual release. Do not query null after a
   confirmed Start; an unknown Start makes no speculative status query.
1. Close the tab and owned server. Retain fresh sibling
   `final-detector.stdout.log`, then run `just str005-startup-probe finish --private-root <child>` to independently judge and seal the result.

Reads are individually bounded at 30 seconds; Stop and Close at 150 seconds each.
Fixture release retains the existing 1.5-second TERM interval and five-second
total bound, with independent process/listener absence. Late Start replies trigger
another Stop/Close, never another Start. Earliest failures remain distinct from
cleanup failures. Raw operational artifacts and any authorized future core dump
stay private; public evidence contains only closed categories, counts, durations,
booleans and digests.

## Completion and stop conditions

Require completed Start, increasing dispatch, independently retained
`asic_dispatch`, requested normal Stop, qualified cooling, original-baseline and
checkpoint continuity, exact 180,000-ms ledger completion, no pending reservation,
restoration and actual host/device release. The evaluator preserves V2 terminal
categories; only precisely correlated post-Stop authority cancellation or clean
pre-share terminal evidence is eligible. Earlier failures remain disqualifying.

Missing, stale or contradictory proof seals a partial result and leaves the task
open. Do not replay a consumed attempt or retry an unchanged boundary. A new
attempt requires a tested targeted fix or objective permitted remediation and
published contract coverage. No refunds, inferred ordinals, historical evidence
repair or parity promotion. Accepted-share and heartbeat-loss qualification
remain separate prerequisites for later integration.

Startup002 passed preflight but issued no grant or Start: preparation stopped
with a detector older than its unchanged 60-second bound. Its partial result and
successful no-work recovery stay sealed. Startup003 reuses the verified
preparation003 and measures fresh physical state. The driver batches native
connect, direct DOM readiness and Run into one UI step. The owner retains the
first closed failure phase/category and detector age, independently of cleanup.
A repeated authoritative failure is not permission for a blind retry.
