# Bounded mining startup probe

Current status: startup001 consumed this trial. Start and clearing are disabled.
See [the partial result](../parity/evidence/20260928-str005-startup001-partial.md);
a recovery-only continuation cannot qualify the missing historical checkpoint.

Owner: `task-str005-mining-startup-probe`. The command and active task admit one
trial only after this reviewed source is committed and pushed. Runtime capture,
clear and fresh-session checks remain mandatory. The private root is
`scratch/str005-startup/startup001/attempt`. This command does not reuse an
archived Share002 effect command or grant.

## Objective and prerequisites

Prove one completed authenticated Start, increasing work dispatch, normal Stop,
restoration, authorization checkpoint, accounting completion and actual release.
An accepted share is not required. No heartbeat suppression, lease renewal,
flashing, factory reset, NVS reset/refund, external pool or capture self-test is
part of this command.

The current prerequisite is the exact installation007 capture seal published in
the active task. Capture, archive and recovery inputs must come from that same
sealed root. The adapter verifies the actual self-test ACK/boot/identity sequence,
SDK store success for that boot, official full-partition read result and geometry,
raw partition digest, full ELF identity and checksum, selected captured cutoff,
source-bound native audits and existing healthy recovery proof. It derives these
facts from producer files, not the older aggregate `core_capture_verified` field.
The real pinned offline decoder runs again during preflight. This does not claim
that the historical Share002 panic cause was captured.

Host source identity is separate from installed firmware identity. Host-only
supervisor changes do not require reflashing the already qualified image.
Flash and fixture executable arguments must resolve to canonical Bazel outputs;
the fixture also requires its source-bound build receipt. The official clear CLI
independently checks its own provenance and existing panic-task authorization.

## Private inputs and commands

Use an ignored owner-only parent and a new absent attempt child. Keep stdout and
stderr in separate mode-0600 sibling files. Never create the attempt child by
redirecting into it before preflight. A private bindings JSON contains only paths:

```json
{
  "schema": "str005-startup-proof-inputs-v1",
  "captureRoot": "/absolute/sealed/installation007/attempt",
  "archiveRoot": "/absolute/sealed/installation007/attempt",
  "archiveRelative": "self-test-core/core-dump.private.bin",
  "decoderRoot": "/absolute/sealed/installation007/attempt/cutoff-review",
  "recoveryRoot": "/absolute/sealed/installation007/attempt",
  "recoveryRelative": "candidate-recovery-002/current-recovery.json"
}
```

The intended sequence after activation is:

1. Run `just detect-ultra205`, privately retaining its output as
   `detector.stdout.log` beside the new attempt; detection expires after 60 seconds.
1. Run `just str005-startup-probe preflight --private-root <new-child> --bindings <private-json> --gate-root <qualified-Gate> --fixture-binary <canonical-fixture> --flash-binary <canonical-flash>`.
1. Run `just str005-startup-probe serve --private-root <child> --authority-directory <existing-private-authority>`. The initial page is
   read-only; it does not open the authority directory or launch a fixture.
1. Native-connect the installed image. Record the before baseline, both ledgers,
   diagnostics and preservation, then Close. This emits the existing
   `str005-current-recovery-proof-v1` schema at the current host source. All
   observations, including Close, must finish within 120 seconds of the server's
   collection challenge; the proof retains that original timestamp.
1. Collect a fresh same-physical detector as `clear-detector.stdout.log` beside
   the child. Run `just str005-startup-probe clear --private-root <child>`.
   The wrapper launches only the official archive-bound `core-dump-clear`, binds
   exact argv/source/physical proof/archive hash to its exit, checks the original
   full partition against the archive, verifies all erased readback bytes and
   seals the clear child. No result fields are invented for the existing CLI.
1. Collect `startup-detector.stdout.log` fresh after clearing, then native-connect
   the candidate using the same retained page. Cooling, fixture, signing and Start
   remain denied until verified clear evidence is present. Public-trust signer
   preparation occurs after cooling and before the fixture's ten-second window.
1. Run the one startup probe. After it closes, native-connect again and collect
   fresh recovery. Round zero is partial same-session evidence only; a distinct
   fresh logical session is mandatory for completion.
1. Stop the server, collect fresh `final-detector.stdout.log`, and run
   `just str005-startup-probe finish --private-root <child>`. The independent
   evaluator verifies cleanup and seals either success or truthful partial
   evidence. Unknown live process ownership blocks sealing until resolved.

Clearing requires a fresh 120-second recovery proof and its separate published
panic-task enablement. A failed or timed-out clear is never replayed. Preserve
its first failure and official cleanup result, verify holder absence, and obtain
new recovery evidence through an admitted path before further effects.

## Bounds and stop conditions

The device reserves a full normal **180,000 ms** allowance at a freshly measured
ordinal. The initial signed lease is **60,000 ms**, with **zero renewals**. Never
assume ordinal18, refund a short attempt, or substitute host issuance for device
reservation evidence.

The current qualified Gate accepts an empty renewal array. Its 20-second renewal
timer starts after the controller Start reply, as verified by executing the
actual Gate load/start functions with mocked hardware dependencies. Require a
completed reply within 30 seconds; observe increasing dispatch for at most five
seconds after it. Request Stop immediately on the first proven increase or any
failure, and no later than 35 seconds from invocation. Measured overruns reject
qualification. These are host request bounds, not a claim that ASIC power is off
at that time. Actual authority remains bounded by the device lease and existing
independent shutdown logic. An ambiguous late reply triggers Stop/Close again;
it never triggers another Start.

Stop and Close each have a 150-second observation ceiling. Each recovery read is
independently bounded at 30 seconds and late read completion cannot submit new
evidence after that read's deadline. Each read is attempted despite earlier read
failures; cleanup does not depend on successful evidence persistence. Fixture
reaping uses its existing 1.5-second TERM interval and five-second total bound,
then verifies process/listener absence. The clear process has a deliberate
1,200-second outer ceiling enclosing two full reads, erase, ROM admission and
application return; this is not a claim of typical duration.

No failed Start is retried in this lifetime. A successor requires demonstrated
progress, a new private child, fresh logical authority and newly measured
accounting. Record missing retained resources, failed authorization checkpoints,
unknown reservation consumption or missing cleanup as blockers. Keep sealed
historical evidence immutable and parity at 90/95.

## Software verification

The Node suite tests disabled admission, real Gate zero-renewal/timer functions,
late Start cleanup, one-shot signing, stale baseline rejection, independent
recovery, late-write suppression, fresh recovery rounds, partial finalization,
canonical tool identity and a real synthetic child-process clear boundary.
Synthetic CLI/core fixtures are software tests, never hardware capture proof.

Run `STARTUP_GATE_ROOT=<qualified-Gate> node --test scripts/str005-startup-probe/probe.test.mjs` for lightweight local verification.
Canonical wiring is `bazel test //scripts:str005_startup_probe_test`; builds and
hardware remain separate from the software-only tests.
