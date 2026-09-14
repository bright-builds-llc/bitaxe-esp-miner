# Stage B controlled restart qualification

Status: adopted by `task-cpu0-telemetry-cadence-qualification`. Stage B becomes eligible
only after Stage A has a `readResetOrigin()`-validated `observed_stable` result,
the exact new firmware/Gate pair and host supervisor are verified and published,
and source-bound preflight admits the concrete commands below. Observation failures
001–004 and startup-recovery-1 retain their original outcomes and inventories.

## Objective and limits

On the same admitted Ultra 205, preserve Device Identity, settings, authorization
high-water state and both ledgers across one ordinary firmware installation,
then observe exactly one authenticated software restart. The new firmware is
necessary because installed `daa69bb03274968e78d131e9b0bf13ff12f64ff0` has no such
command. The supervisor receives no Work Lease signing authority, mining grant,
pool input or budget reservation. Ordinary offline UpdateAuthority signing for
clean package publication remains part of the existing role-separated workflow
(ADR-0093). No NVS
seeding/erase, factory reset, HTTP control or DTR/RTS/ROM/power-cycle restart
substitution is permitted. The ROM transition inherent in the one admitted
ordinary installation is confined to that installation, never a reset fallback.

One fresh context, one exclusive predecessor assignment, one installation claim,
one server instance and one restart claim are allowed. Consume each claim before
its effect, including an interrupted attempt that never produces an effect child.
The installation guard checks terminal-failure absence before and after writing
its exclusive claim. A failure during that write leaves the claim consumed.
The operator wrapper must reject any subsequently observed failure before
launching flash; this is not a claim of absolute cross-process atomicity.
No automatic retry or server restart; failures preserve evidence and close.
Terminal failure is sticky across asynchronous verification, claim writes and
HTTP response dispatch; a concurrent rejected request cannot resurrect admission.

## Commands and source binding

Repo-owned commands (through `just fixed-usb-qualification`):

- `reset-origin-restart-preflight`: require accepted Stage-A result, original
  campaign-record locator, new clean/pushed firmware and Gate commits, canonical
  package manifest, verified progress input and a fresh protected private root.
- `reset-origin-restart-serve`: serve only the bound assets, phase-specific
  context/activation, immutable state/accounting/diagnostic receipts and the
  single restart lifecycle. No signer, lease, fault or arbitrary command route.
- `reset-origin-restart-consume-install`: consume the sole installation while
  its process observer is already armed and before executing the flash child.
- `reset-origin-restart-install-review`: independently inspect the completed
  write and factual startup evidence below; retain the legacy outcome verbatim.
- `reset-origin-restart-judge` and `reset-origin-restart-review`: offline complete
  evidence validation and immutable result publication/revalidation.

Preflight binds the Stage-A result digest and full immutable lineage, both
runtime identities, the new Gate page/bundle, all 13 new package artifacts,
public trust, complete host-validator digest, corrected no-mining client hash,
restart client hash, and exact process/serial observer implementations. Reserve
an exclusive sibling assignment before creating/copying the child. Stage-A
runtime artifacts remain read-only and source-bound; they are not rebuilt merely
because the host HEAD changes. The new package is required for the new command.

The context has exactly two allowed firmware configurations: installed Stage-A
firmware before installation and the new published firmware after installation.
Both use the same new Gate bundle. `/context` returns exactly Gate's five-key
restart configuration; phase state is separate at `/supervisor-state`. Wait for
the Gate page's initial autoconfiguration before operator actions. The browser
cannot supply an arbitrary source,
ELF, phase or replacement port. Only validated installation evidence advances
the context to the new firmware configuration.

## Preservation across the image change

Start a fresh page from the **new Gate**, with `restartQualification:true` and
cadence/recovery modes disabled. Initially configure the exact retained daa
source/ELF, obtain fresh Hello/possession/status, capture the private preservation
baseline, and record both ledgers. This seeds the existing module-level
`WorkerPreservationBaseline`; do not export/import private digests or treat the
Stage-A result's opaque `baseline_id` as transferable baseline material.

Flush the journal, close the Worker connection and prove released USB ownership.
Keep that same page and its private preservation object alive during installation.
After installation review, configure the same page with the new source/ELF and
reconnect. `configure()` preserves the baseline, so require the same baseline ID
and successful identity/settings/high-water comparisons, mine-on-boot false,
confirmed safe baseline and inactive lease. A reload, changed Gate page, lost
baseline or failed comparison is terminal for this context.

Use separate immutable pre-install and post-install journal/accounting scopes
because their firmware identity bindings differ. The original read-only
accounting validators remain usable inside each scope. Independently compare
the pre-install ledger/budget and baseline ID to post-install before-restart
accounting; do not weaken a single-context validator to accept mixed identities.

## Single state-preserving installation and factual startup

Run fresh `just detect-ultra205`; require exactly one admitted physical device.
The ordinary flash command must retain that physical lease and successfully run
`espflash board-info --chip esp32s3 --port <admitted-port> --non-interactive`
before writes. Prearm the source-bound process observer, consume the installation
claim, then run the existing ordinary command:

```text
just flash-monitor --board 205 --port <admitted-port> \
  --manifest <new-canonical-manifest> --evidence-dir <root>/install-001 \
  --redact-evidence --capture-timeout-seconds 30
```

Retain command/observer starts, claim time, original flash result and exit status,
complete protected capture, source-bound closed startup reduction, and actual
process/USB cleanup. Observer start \<= claim \<= effect must be proven; the usual
one-second timestamp quantization applies only to the flash receipt timestamp.
The five admitted write segments must remain disjoint from NVS; no credentials,
factory reset, alternate image or implicit second write is allowed.

The legacy classifier is **not** the Stage-B factual-startup judge.
`tools/flash/src/monitor/fixed_serial.rs` labels an initial panic reset category
`reboot_observed` even without a captured transition; that also sets
`stable_boot:false` and adds `insufficient_advancing_samples`. Preserve those
values and the resulting nonzero command exit/commit-ready status unchanged.
Require `capture_mode: noninteractive`. The ordinary accepted branch permits
`completed` or `timed_out_after_trusted_output`; the initial-panic branch permits
only `timed_out_without_trusted_output`. Failed, pending, dry-run and interactive
capture never qualify for the exception.
Do not call the old `requireStartupInstallation()` and then waive its failure.

A new narrowly scoped factual validator may admit either an ordinary fully
qualified installation or this exact diagnostic-only grading case:

- Completed write and exact package/source/ELF/profile identity, native fixed USB,
  no NVS seed, complete startup and confirmed safe idle baseline.
- Complete-record reduction independently proves one captured boot ordinal with
  unchanged reset category, boot and runtime-ready uptime each advancing across
  at least 1000 ms,
  and no identity conflict, actual panic/allocation receipt, startup failure,
  malformed/error record or observed ordinal/uptime regression.
- The only legacy grading issues are `reboot_observed` plus its derived
  `insufficient_advancing_samples`, attributable by source and captured records
  solely to the initial **panic category**. Do not extend this exception to
  watchdog/brownout, a real transition, insufficient factual samples, missing
  identity or an ambiguous/failed write.

This admission permits fresh read-only validation on the new pair, not a claim
that the legacy installation passed or the preceding panic was explained.
Require a wholly new 120-second observation before any restart command. Any
other installation failure stops this consumed context without fallback reset.

## New-pair pre-restart observation

Bind the first post-install prime boot ordinal and reset category to the
installation startup reduction; any changed boot during the handoff stops the
context. Reject restart state/summary before the durable restart claim.

After exact new-pair admission and preservation comparison, record fresh
before-restart accounting. Qualification ledger must be idle at next ordinal 17,
last completed 16, charged 1380000 ms; original campaign remains masks 7/7,
charged 240000 ms, not pending. Compare both to Stage A and pre-install accounting.

Reuse the Stage-A closed diagnostic parsing, priming/deduplication and independent
observation criteria: host and fresh boot/runtime-ready spans >=120000 ms, hard
135000 ms envelope, per-category gaps \<=6000 ms, at least 30 boot and 120 healthy
startup advances, \<=4096 reduced records and \<=1024 batches. Initial panic
category remains unattributed history; actual failure receipts or another boot
transition are disqualifying. Require complete current state journaling and
safe inactive preservation throughout. The old Stage-A context/judge is not
reused as authority for the new pair.

Finish source/inventory validation before entering the timed restart operation.
Do not make recursive artifact hashing block diagnostic writes or consume the
30-second transition budget. Metadata recording remains ordered, bounded and
fully acknowledged; final flush must consume response bodies.

## One authenticated restart and independent proof

Only after that observation passes, flush the current ready state and persist
the exclusive restart claim bound to the context, that state sequence, current
boot ordinal and one private canonical 16-byte base64url request nonce. Prearm
Gate capture before dispatching:

```text
qualificationRestart({requestNonce, expectedBootOrdinal})
```

Firmware command: `qualification_restart`. Expected ordinal is an integer from
1 through 9007199254740990. Exact ACK is
`{schema:'worker-qualification-restart-v1',requestNonce,bootOrdinal,nextBootOrdinal}`,
with next=boot+1. Admission consumes the per-boot latch even if transmission later
fails. Firmware requires fresh possession, successful startup, safe native idle,
no active/cleanup/restoration obligation or pending replacement possession proof,
and mine-on-boot false. Actual completed
writer delivery plus fresh epoch/token/native-idle rechecks precede `esp_restart`.
No ACK alone proves host receipt or a restart.

Gate retains coalesced matching ACK/boot bytes by switching framing mode in the
validated ACK callback. Shared limits are 30000 ms, 512 records and 262144 bytes from
prearm through fresh admission. It keeps the selected SerialPort and ownership
lock. After a matched ACK, at most one explicit close/reopen of that same granted
object is allowed, without discovery or replacement selection. Record
`streamInterrupted:true`, `portReopens:1`, `continuity:'same_port_reopened'` when
used; never call this uninterrupted byte capture. Unsettled close/open is a
terminal cleanup failure.

Persist Gate's bounded `{summary,ack,observations,lifecycle}` evidence before
judgment. ACK evidence uses SHA256 of the canonical nonce text, never its public
value. Recompute all acceptance predicates from ordered closed observations and
lifecycle events: prearmed before acknowledged; exact correlated ACK/ordinal+1;
exactly the expected next boot with `software_cpu`; no extra transition or
failure; at least two distinct advancing complete runtime-ready observations;
exact runtime source/ELF; and fresh Hello/possession/status restoring authenticated
identity and unchanged baseline. Raw diagnostic identity is not authenticated
admission. Validate both record/byte and wall-clock bounds independently of the
summary booleans. No unrelated protocol errors are excused as expected reset.

## Completion, cleanup and continuation

Review both ledgers after restart in the newly admitted session and require
unchanged values plus the same private preservation baseline. Flush evidence,
close Worker ownership, flush the final closed journal row, close the page and
supervisor, and prove observer termination, listener/owned-child absence and
both serial nodes unowned. Keep earliest failure; a body/journal flush failure
cannot be relabelled successful cleanup merely because the Worker closed.

The final judge requires all installation, pre-observation, matched restart,
fresh accounting and final cleanup evidence, then writes an exclusive immutable
result/inventory. A passing controlled restart does not resolve the earlier
unattributed panic or promote any older failure. Only that accepted Stage-B
receipt may admit a fresh cadence preparation 17-2 on its exact pair, preserving
the existing assignment and charged predecessor. Repeat all four update/browser
cycles and all three cadence phases; no transferred cadence result, refund,
reset ledger or mining authority is conveyed by Stage B itself.

### Conditional cadence preparation after Stage B

`just fixed-usb-qualification cadence-preflight ... --supersede-restart <StageB/result.json>`
is available only after the strict Stage-B reader accepts the controlled restart
and complete cleanup. It is mutually exclusive with every other supersession
flag. The reader binds the actual accepted observation 005 and the immutable
001–004, recovery 1, original 017 and charged 016 ancestry. It never promotes those
historical failed outcomes.

Use the original charged 016 result as `--previous-receipt`, the exact accepted
Stage-B firmware/Gate/package, current clean published sources, the canonical
cadence observer and freshly admitted Work Lease authority inputs. The guard
must therefore ship with the Stage-B firmware source revision. A newer host-only
HEAD cannot silently reuse the older image as if source identity were unchanged.

Admission creates a fresh attempt ID and `restart_predecessor` receipt binding
for preparation 2 of ordinal 17. It reserves `ordinal-17-preparation-2.json`
exclusively before creating or copying the child; partial preparation never
releases that assignment. Preserve `ordinal-17.json` and all older evidence.
Compare runtime artifacts, trust and write segments, not the different
no-mining and cadence browser-client hashes.

Fresh pre-mining accounting must show next ordinal 17, last completed ordinal 16, charged 1,380,000 ms
and `pending:false`, with the original campaign still charged 240,000 ms and masks 7/7.
Repeat all four continuity/update cycles and all three cadence phases under the
unchanged thresholds. The ordinary allowance is 180,000 ms; after it is consumed
and finalized, require next ordinal 18, last completed ordinal 17, charged 1,560,000 ms and
`pending:false`. Stage B supplies no grant, budget reset, refund or completed
cadence evidence.
