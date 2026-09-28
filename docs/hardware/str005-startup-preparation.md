# No-mining preparation for startup002

Owner: `task-str005-mining-startup-probe`. This is a separate prospective command,
not a reactivation of consumed Start, clear, recovery or self-test gates. Code
and the task marker must be enabled in clean published source before effects.
The required marker is `Startup preparation hardware: enabled.`

## Objective and limits

The retained terminal startup001 record makes a null V2 idle query invalid.
Prepare a new boot with one ordinary qualification restart, preserving all
settings, Device Identity, authorization replay state and both ledgers. Do not
change firmware or partition contents, clear a core, issue a grant, launch a
pool fixture, start mining, or retry a restart. Existing V2/restart configuration
exclusion and sticky-mode guards remain intact.

Two separate pages and owners are deliberate:

1. A V2 read-only page observes the known retained attempt, current accounting,
   restoration, quiescent resources and serial release.
1. After that owner exits, a new restart-only page establishes its own private
   baseline, performs one authenticated normal restart, verifies the observer
   and unchanged state, then closes. It never changes that page into V2 mode.

The sole reset uses the existing Gate controller, granted browser serial port,
firmware eligibility checks, acknowledgment barrier and restart observer. No
alternate transport or host reset mechanism is introduced.

## Immutable inputs and identity

The initial command accepts exactly these published parent seals:

- startup001: `950a8e55b5efb468905a4edd2e739cfd02e47013797e026bf0218af955234cb7`
- recovery001: `cee3e1fa3d570774dc4f68c2cf576f7be5bee2bed31aafb3a277f3196366b07a`

It validates the parent inventories and re-evaluates recovery evidence. The
startup `start-admitted.json` boot and the recovery status boot must match; the
known attempt ID and installed identity are joined across both parents. The
fresh preparation recovery must observe that same boot and released record.
For the actual parents this is boot8. The only accepted next boot is boot9 with
explicit `software_cpu`; these values are derived from the parents, not guessed.

The current Gate comes from `--gate-root`, must be clean/pushed and match the
current `MODULE.bazel` pin. Its retained page, bundle and public trust are hashed.
The context keeps `gate_commit` separate from `historical_gate_commit`, and
current host `source_commit` separate from the unchanged installed
`firmware_commit` and full `app_elf_sha256`. Parent roots and hashes remain in
private context and result files. Private identities are not printed by the CLI.

## Command sequence

Use a new ignored mode-0700 root with mode-0600 private output files outside its
not-yet-created children. Preserve all parent seals.

```sh
just str005-startup-preparation preflight --private-root <new-root> --startup-root <sealed-startup001> --recovery-root <sealed-recovery001> --gate-root <current-qualified-Gate>
just str005-startup-preparation serve --private-root <root> --stage recovery
just str005-startup-preparation finish --private-root <root> --stage recovery
just str005-startup-preparation serve --private-root <root> --stage restart
just str005-startup-preparation finish --private-root <root> --stage restart
```

Before each serve, retain `just detect-ultra205` as sibling
`recovery-detector.stdout.log` or `restart-detector.stdout.log`. It must identify
one admitted runtime profile on the same physical device, be at most 60 seconds
old, and have no serial holder. Connect through the page's native gesture and
run its one collector. Stop its server and close that page before proceeding.
Before each finish, retain a fresh corresponding
`recovery-final-detector.stdout.log` or `restart-final-detector.stdout.log`.
Finish checks owned process exit, listener absence, and holders on both locators.

The fixed children are `recovery/` and `restart/`. A child's existing claim or
path cannot be reused. Stage recovery is sealed before restart admission. Its
observations and host release must remain within 120 seconds of its owner start.
The restart-only page separately begins a fresh, at-most-120-second baseline
collection. Restart admission rechecks the fresh recovery prerequisite and
current page's boot, identity, preservation and both ledgers.

The server persists one fresh nonce/boot restart claim before delivering it.
The request is sent once. A missing or ambiguous reply never authorizes another
request. The actual observer must report a matched acknowledgment, exact
successor boot, software reset, expected source/full ELF and two advancing healthy
runtime-ready observations within 30 seconds, with at most one same-port reopen.
The independently validated typed packet is retained even when it cannot pass
qualification. New core/preparation diagnostic categories remain preserved but
do not replace the existing restart proof requirements.

All ordinary reads have a 30-second ceiling; Stop and Close have independent
150-second ceilings and run even after failed reset observation or persistence.
Late reads cannot submit evidence after their stage closes. Post-reset state and
both ledgers are read again, and the same restart page's preservation baseline
must still match. Final Close must prove lease inactivity, restoration and actual
serial release. Any missing condition yields a partial failed result, never a
claim of idle admission or an automatic retry.

## Output contract

The root `context.json` schema is `str005-startup-preparation-context-v1` and
contains installed identity, current/historical Gate identities, physical
binding, known prior attempt, unchanged ledger/budget, derived prior boot and
`parentRoots`/`parentSeals`. Stage recovery retains the existing current-only
recovery schema and adds its collection start time.

The final root `result.json` schema is `str005-startup-preparation-result-v1`.
It exposes `complete`, blockers, before/after boot ordinals, `software_cpu`, both
unchanged ledgers, same-page preservation, parent joins, `recoverySealSha256`,
`restartSealSha256` and `restartEvidenceSha256`. The complete packet is
`restart/evidence.json`; each child and the final root have sealed inventories.

The result is preparation evidence. It never qualifies startup001's missing
historical authorization checkpoint and never promotes parity. A future startup
page must independently establish fresh V2 idle admission on the resulting
boot. The old clear archive remains untouched; this command does not itself
claim another clearing operation. Parity stays 90/95.

## Verification

Software tests exercise the real Gate restart adapter, page operation, observer,
private preservation model and diagnostic parser with a simulated serial wire.
They cover same-stream and one-reopen success, one-request consumption, mode
guards, wrong nonce/boot/identity, panic instead of software reset, timing limits,
partial packets, ledger/preservation drift and unconditional cleanup. They are
software proof, not device results.

## Collection-clock correction

Preparation001 stopped before reset because its 120-second freshness bound began
at server launch. Connection/UI setup used 86 seconds; the final check rejected
130-second server age although the oldest collected ledger was only 44 seconds
old. Its failed result stays sealed. Preparation002 follows a tested correction:
a one-use `/recovery-begin` receipt is written immediately before authenticated
reads. The timestamp cannot be recreated after any observation or on a second
begin request. Server lifetime is not observation age. Both restart admission
and independent startup consumption check that exact receipt and the original
120-second freshness bound. No clock limit is extended and no old proof is
retimestamped.

## Failure before begin and handoff

Preparation002's recovery was valid, but the second page reached begin after its
fresh proof expired; no reset was claimed. Its cleanup nevertheless released
USB. A regression now ensures that failure before begin cannot prevent recording
Close, and that the first closed server failure category survives cleanup.
Preparation003 batches the existing owner-release/detector/finalization/handoff
steps and native UI readiness checks; it retains the same 120-second freshness
bound and requires a new live claim. Neither sealed failure is retimestamped or
reclassified. The restart claim remains unavailable until every before-proof
passes, irrespective of whether cleanup evidence was recorded.
