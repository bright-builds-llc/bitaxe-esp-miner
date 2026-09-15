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

### Exact statistics-install failure successor 002

Stage-B 001 remains failed and immutable. Its failed inventory SHA256 is
`883b0a4f40eb97c59ec03f517d283d556e4ea3412de9e807e44371fdcf7ab3f1`;
the source-bound failure auditor SHA256 is
`72064ed17475fcf45c96d61d7ed95b7330e4830f8c6982ec6f644c7c7abd7101`.
One ordinary write completed, but its retained startup had `first_failure=statistics`.
No authenticated post-install preservation or accounting, controlled restart,
health qualification or cadence authority was established. The captured ordinal
507/reset category `other` does not explain earlier boots.

The sole additional preparation is:

```text
just fixed-usb-qualification reset-origin-restart-preflight ... \
  --stage-a-result <accepted-observation-005/result.json> \
  --supersede-install-failure <sealed-restart-001-root>
```

Use a fresh sibling root, the corrected clean published firmware/package, the
verified Gate build and a protected `software_correction` progress receipt citing
that exact failure seal and the targeted regression evidence. The classifier
accepts only that sealed failure class and independently rederives its unchanged
artifacts, original accounting, before-install closed journal, startup failure
and actual host cleanup. It admits no generic retry or recursive failed-successor
reuse. It exclusively reserves observation 005's `restart-assignment-2.json`
before creating the child. The original assignment and failed root are untouched;
a partial new preparation keeps its assignment consumed.

The successor's `before_source` is the image actually installed by failed 001:
firmware `a74701d68505070dd00435005641fc3882ae8af8`, ELF
`a3f11024e17aa091157c4b44e410cbd07e0e2eb516f350e9e087a4b029680299`,
bound to that root's retained package. It is not Stage A's earlier image. The
original accepted Stage A remains the ancestry anchor. A fresh page captures a
new private preservation baseline on the currently installed image; it cannot
import or attest the old page's unobserved post-install baseline.

`prepareInstallation()` first reads fresh authenticated idle state and unchanged
accounting (next 17, last 16, charged 1,380,000 ms, no pending allowance; original
240,000 ms and masks 7/7). It then refreshes and records
`before-install-failure-review.json`: exact current firmware/ELF, the same captured
boot ordinal and reset category, advancing boot/startup uptime beyond the failed
capture, and the known statistics startup failure. Other failures, a new boot or
missing proof stop preparation. This receipt explicitly records an unhealthy
startup inspection, not a recovery or health success. It precedes the final
closed/released journal row and is required again by the installation consumer
and independent installation judge.

Successor 002 permits one corrected state-preserving install and one authenticated
restart, each exclusively consumed in that fresh root. It retains all existing
source, process-observer, cleanup, installation chronology and terminal-failure
guards. After installation, the same page keeps its newly observed private
baseline across reconfiguration. The corrected image must independently satisfy
the unchanged healthy 130-second capture, fresh accounting and controlled restart
proof before any passing receipt exists.

For this new context only, `statistics_startup_required:true` additionally binds
closed statistics startup metadata. A successful `active` receipt has stack
8192 bytes, capabilities 2052 (`INTERNAL|8BIT`), unavailable errno, and numeric
before/after heap measurements, with each largest block no larger than its same
snapshot's free bytes. No allocation-size delta is inferred between those two
snapshots. Preserve `prepared` before `active`; reject failed, cancelled or
prepared-only final evidence. Require active evidence in the installation
capture, the new-pair observation snapshots, and after the expected restart boot
(and any permitted same-port reopen). These markers receive no boot/startup
advance, timing or authority credit. Earlier contexts without these markers
retain their historical validation rules.

Only a strictly accepted `controlled_restart_verified` successor 002 may use the
existing conditional `--supersede-restart` cadence branch. It still binds accepted
005, recovery 1, original 017 and charged 016; it carries no mining grant. Fresh
cadence preparation 17-2 repeats all four update/browser cycles and all three
phases with the ordinary 180,000 ms allowance. After its consumed allowance is
finalized, accounting must be next 18, last 17, charged 1,560,000 ms and no pending
allowance. No prior failure is promoted and no charge is reset or refunded.

Before reserving successor 002, the preflight checks that the exact hash-bound
Gate bundle contains its `statistics_startup schema=v1 state=` decoder grammar.
Live context validation repeats that capability check together with the existing
clean published source and bundle identity checks. An older Gate bundle such as
e3 without this decoder cannot reserve the new assignment or reach installation
consumption. This guard does not change the historical failed-001 reader and is
not a substitute for the required active statistics evidence after installation.

### Exact reconnect allocation failure successor 003

Successor002 remains an unverified failed installation on firmware
`582fc2bf94779e6ea867e5de87b97695df5a20cd`, Gate
`72235d884c3605ff77d484f210fcaa0d7218b47e`, and ELF
`8095cfcf3991129188a39bc28f56796ee1859fd22d70ada6a1dc8c7359051558`.
Its retained statistics task became active, but the closed network diagnostic
reports `phase=reconnect_spawn error=no_memory`. This is not a healthy startup
or an accepted controlled restart. The exact sealed failure and its independent
audit are the only eligible predecessor for preparation003.

The [failure report](../parity/evidence/20260915-reconnect-startup-failure.md)
binds failed-inventory SHA256
`29c8f6bb14b1962676bdd8af4a5c9b406ea81ab0f68c61882c51c2680eb0d7ad`
and producer SHA256
`4946a65aaa619c0dd291ca09932cadaa87ed2c7f1eccb7507cf8ef2a34e5efab`.

Use the existing `reset-origin-restart-preflight --supersede-install-failure`
command with that failed002 root, a verified software-correction receipt, and a
fresh protected sibling directory. The finite lineage is 003 → failed002 →
failed001 → accepted Stage-A005 and its original ancestry. Each historical
reader retains its original predicates. Unknown failures, altered inventory,
recursive lineage, conflicting assignments and archived-task authority are
rejected. Reserve `restart-assignment-3.json` exclusively before creating the
child; interrupted preparation does not release the assignment.

The correction reserves the unchanged 8192-byte Wi-Fi reconnect and USB receive
threads before late startup allocations fragment internal memory. Both remain
inactive until their existing activation boundaries. Statistics retains its
existing reservation, activation boundary and cadence. Tests must cover failed
spawn, cancelled preparation, queued events before activation, subscription
rollback and absence of USB reads before activation. Native verification must
check unchanged task configuration, storage and stack costs and the existing
4-MiB image limit. Early reservation alone is not evidence that the complete
service set fits on hardware.

Capture the initialized Wi-Fi credential state once during preparation and
reserve reconnect only for valid station credentials. Missing/invalid states
must not attempt that allocation; later startup consumes the captured state
without a second read. Retain the existing association-failure fallback and its
single prepared worker. No credential value enters diagnostics or evidence.

The size-recovery experiment may change only the existing
`profile.release.package.bitaxe-firmware` optimization level from `s` to `z`.
Dependencies, the global release profile and ESP-IDF/C options remain fixed.
Cargo's [profile documentation](https://doc.rust-lang.org/cargo/reference/profiles.html#opt-level)
describes both size settings and cautions that results must be measured.
Freeze the selected setting in the published source and include it in the
native audit. Recheck main, owner, statistics, receive and reconnect paths;
compiler optimization can change frames and instruction timing even when the
configured stacks, priorities and deadlines are unchanged. The existing cadence
and independent shutdown criteria still determine hardware acceptance.

Bind the before-install source to the image actually installed by failed002,
not to Stage A or failed001. Before consuming the sole installation, obtain
fresh authenticated idle baseline and accounting: next 17, last 16, charged
1380000 ms, pending false; original campaign remains charged 240000 ms with masks
7/7. Require the same captured boot ordinal 1/reset category `other`, advancing
boot/startup uptime beyond the failed capture, exact runtime identity, active
statistics metadata, and only the known network reconnect-spawn/no-memory
failure. This review records an unhealthy installed image and cannot establish
post-install preservation retrospectively. Missing or changed proof stops the
new context without an installation claim.

The new firmware must be changed, clean and published with its exact clean
package. The existing verified Gate72235d8 may remain unchanged when its retained
page and bundle hashes match. Bind the observer, validator, client, trust and
all thirteen runtime artifacts before effects. The new context allows one
state-preserving installation and one authenticated restart under the unchanged
observation, source, timing and cleanup requirements above. No new mining
authority, reset fallback, NVS alteration or repeated effect is introduced.

After a strictly accepted controlled restart, the existing cadence17-2 path
still requires fresh accounting, four update/reconnect cycles and all three
captures with the original thresholds and independent heartbeat shutdown proof.
It consumes one normal 180000-ms reservation and expects next 18, last 17,
charged 1560000 ms, pending false after finalization. A failure leaves cadence,
USB qualification and migration active and all earlier failures unchanged.

### Exact HTTP task allocation failure successor 004

The separately sealed003 failure is bound by
[its report](../parity/evidence/20260915-http-task-startup-failure.md): inventory
`d560b740bcca4936dd6c39722a81165112decc5579ce2c4f78d8d588903c855c`, producer
`7e90435786f3ab61594991757863ed8d7f3eae4e841e0d2252c6416e2c33acac`.
It completed one write on253658ce/ELF6991aab2/Gate72235d8, then failed HTTP task
creation. It provides no post-install authentication, restart or mining proof.

Use the same `reset-origin-restart-preflight --supersede-install-failure`
command, now restricted to that exact additional failure class. Its finite
ancestry is004→failed003→failed002→failed001→accepted005 and the existing older
chain. Preserve every earlier assignment and seal. Exclusively reserve
`restart-assignment-4.json` before creating the fresh protected child; changed,
recursive, conflicting, interrupted or archived-task inputs confer no effects.

Verify the targeted allocation-order correction: create the16KiB HTTP server
before its8KiB deferred worker inside the existing HTTP startup stage, publish
routes/readiness only after both succeed, and release the server if deferred
startup fails. No buffer, reserve, stack, affinity, priority or deadline change
is permitted by this correction. Run production-boundary regression tests,
ordered software/native verification and exact clean publication before effects.

The before-install configuration must bind the image actually installed by003.
Fresh authenticated idle accounting must remain next17, last16, charged1380000 ms,
pending false; the original campaign remains charged240000 ms and masks7/7.
Require the same boot1/other with advancing boot/startup uptimes beyond29801/29802,
active statistics matching the retained startup record, the exact
`storage_http_failure/http_server/http_task` observation, SPIFFS available and
HTTP not ready. Accept only the recorded `network/entered/storage_http` and
`runtime_ready/failed/storage_http` startup states for this unhealthy pre-install
inspection. The browser's pre-install diagnostic wait is specifically scoped to
that expected failed state; post-install observation still requires complete,
healthy startup. Other failures or missing proof stop before installation.

One installation and one authenticated restart remain the maximum in the new
context. No signing/mining credentials enter this stage. All existing exact
identity, preservation, process observation, healthy130-second observation,
restart30-second limit and actual cleanup rules remain. Only an independently
accepted controlled restart admits cadence17-2 and its unchanged four cycles,
three captures, normal180000-ms reservation, shutdown/cooling and accounting
requirements. Missing proof leaves the three broader tasks active.
