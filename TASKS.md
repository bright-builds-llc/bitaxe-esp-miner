# Tasks

This is the repository's sole active work tracker. Use one stable, timestamped
task block per unit of work. Update only that block as work progresses, record
the verification performed, and finish with a concise completion review.

`TASKS.archive.md` is the append-only historical store, not an active tracker.
After a task is completed, explicitly cancelled, or superseded, append its full
final record there and remove it from this file in the same commit. Keep
blocked, deferred, terminal-blocker, future, and otherwise unresolved tasks
here. Stable task IDs must be unique across both files. Never restore or select
an archived task; create a new active task with a new ID and an archived-task
reference for follow-up work.

Task blocks under `## Future` remain incomplete and become ordinary automatic
selection candidates after higher-priority active work. Move the selected task
to `## Active` before implementation. Dependencies, environment, verification,
safety, and evidence gates still apply; no task or fresh progress-backed
attempt ordinal requires repeated user confirmation.

Historical plans, milestones, debug sessions, and task records under
`.planning/milestones/` are evidence and context only. They do not authorize
new work.

## Active

### task-parity-asic009-bm1368-core | 2026-08-20 | Implement the pure BM1368 protocol core

- [x] Add typed BM1368 init, work, result, register, framing, and error
      behavior under the pure ASIC crate.
- [x] Add pinned-reference provenance plus golden protocol fixtures and focused
      behavioral coverage.
- [x] Keep BM1368 firmware dispatch deferred, run every gate, and transition
      only `ASIC-009` to at most `implemented` with `unit,golden` evidence.

Plan: `docs/parity/work-plans/20260820T060848Z-ASIC-009/PLAN.md`

Authorization: local Rust/fixture/docs edits, tests, build/package, Git commit,
and push only. No credentials, protected attempt roots, detector,
USB/device/network runtime, flash, monitor, mining, restart, recovery, hardware
attempt, fault injection, external UART/BAP, pins, or electrical work.

Evidence and status boundary: `implemented` requires the complete pure protocol
surface, pinned fixture provenance, current tests, deferred dispatch, unchanged
Ultra 205 packaging, and all mandatory gates. `verified` additionally requires
a supported BM1368 board, firmware adapter, detector-gated hardware smoke/
regression, safe stop, and redaction; none are authorized or claimed here.

Verification: Implementation commit
`1dc17d9b9a8e12319b5ca01db297d6800bd38d46`; 12 focused BM1368 tests,
124 ASIC-crate tests with 1 existing ignored test, the deferred-dispatch
regression, Bazel ASIC tests, reference/package checks, ordered Rust gates, and
Bright Builds checks passed. Evidence:
`docs/parity/evidence/asic009-bm1368-core/summary.md`.

Completion review: The authorized pure software scope is complete and supports
`implemented` with `unit,golden` evidence only. Transition
`20260820T063300Z-ASIC-009` and progress sync accepted the exact plan and source
commit while leaving parity at 88/94 verified. This task remains active and
unarchived because firmware dispatch, a supported BM1368 board, detector-gated
hardware regression, safe-stop proof, and redaction are outstanding. Automatic
selection may treat ASIC-009 as hardware-blocked until those prerequisites
exist and continue to the next software-actionable row.

### task-parity-asic010-bm1397-core | 2026-08-20 | Implement the pure BM1397 protocol core

- [x] Add typed BM1397 init/frequency, work, result, framing, and error
      behavior under the pure ASIC crate.
- [x] Add pinned-reference provenance plus golden one/four-midstate protocol
      fixtures and focused behavioral coverage.
- [x] Keep BM1397 firmware dispatch deferred, run every gate, and transition
      only `ASIC-010` to at most `implemented` with `unit,golden` evidence.

Plan: `docs/parity/work-plans/20260820T064119Z-ASIC-010/PLAN.md`

Authorization: local Rust/fixture/docs edits, deterministic tests,
build/package, Git commit, and push only. No credentials, protected attempt
roots, detector, USB/device/network runtime, flash, monitor, mining, restart,
recovery, hardware attempt, fault injection, external UART/BAP, pins, or
electrical work.

Evidence and status boundary: `implemented` requires the complete pure BM1397
surface, pinned fixture provenance, current tests, deferred dispatch, unchanged
Ultra 205 packaging, and every mandatory gate. `verified` additionally requires
a supported BM1397 board, firmware adapter, detector-gated hardware regression,
safe stop, and redaction; none are authorized or claimed here.

Verification: Implementation commit
`3909a304213f81babf9d3fed38800bd2b515c0a5`; 16 focused BM1397 tests,
140 ASIC-crate tests with 1 existing ignored helper, the deferred-dispatch
regression, Bazel ASIC tests, reference/package checks, ordered Rust gates, and
Bright Builds checks passed. Evidence:
`docs/parity/evidence/asic010-bm1397-core/summary.md`.

Completion review: The authorized pure software scope is complete and supports
`implemented` with `unit,golden` evidence only. Transition
`20260820T070850Z-ASIC-010` and progress sync accepted the exact plan and source
commit while leaving parity at 88/94 verified. This task remains active and
unarchived because firmware dispatch, a supported BM1397 board, detector-gated
hardware regression, safe-stop proof, and redaction remain outstanding.
Automatic selection may treat ASIC-010 as hardware-blocked until those
prerequisites exist and continue to the next software-actionable row.

### task-ultra205-virtual-board-validation | 2026-09-30 | Full functional virtual Ultra 205 and pre-flash validation

Status: Verified software milestone; full qualification blocked by missing actual
margin/call-closure proof and unsupported mandatory integration coverage. The
corrected-target panic is resolved in emulation (campaign004, below).
Cross-reference | 2026-10-02: archived `task-device-noise-worker-stack` resolved
run017's composed Noise boundary in emulation (campaign004) and passed a real
device Noise handshake (attempt-003). Other mandatory profiles and coverage
remain open. Later virtual-path changes (`654338d0` generation-gate layout,
`24af10be`/`00f84eae` movsp-safe queues, `92f58abb` PSRAM preference) postdate
the recorded results and rotate their source bindings.
Objective: reuse production control logic in deterministic host and Xtensa/QEMU
scenarios, model every Ultra 205 functional peripheral, then qualify one canonical
pre-flash gate. References: `task-str005-v2-accepted-share-probe`, its immutable
status001/recovery006 results, and the retained installed ce8f/453d package.

- [ ] Extract shared host-buildable runtime interfaces and production logic;
  preserve physical policies, existing host regressions and historical readers.
- [ ] Implement seeded virtual BM1366, sensor/fan/voltage/ADC/display/input,
  persistence/reset/core, networking and control-transport models with explicit
  unsupported outcomes and independent model-conformance evidence.
- [ ] Add capability-aware memory/fragmentation and named-phase allocation faults,
  including observed3451-free/2176-largest and8192-internal allocation profiles.
- [ ] Exercise composed startup/Start/status/Stop/recovery with real protocol,
  admission/accounting and local fixture; preserve earliest typed failure and
  separate qualification, historical resources and current safety.
- [ ] Integrate pinned SDK5.5.4 QEMU esp_develop_9.2.2_20250817, managed checksum
  bootstrap/doctor and an unmistakable virtual dual-core S3/16MB/8MB-OPI target.
  Physical flash must reject virtual artifacts; report paired configuration diffs.
- [ ] Provide Bazel-backed virtual-board, qualify-virtual-board and preflash-validate
  commands, versioned source/package/config/model/emulator/validator-bound results.
- [ ] Qualify complete healthy lifecycle, all mandatory faults, target panic/core
  decoding, resource/ownership tests, independent goldens/mutations, repeatability
  and host/QEMU semantic agreement. Unsupported required coverage blocks qualification.
- [ ] After baseline qualification only, enforce centrally selected local profiles
  for affected flash/update/live commands before discovery or sensitive inputs.
  Emergency Stop/Close/failure-only recovery remain independently available.
- [ ] Run affected Bazel suites, ordered Cargo fmt/Clippy/build/tests, production-ELF
  audits, standards/reference/redaction/Markdown/diff checks. Commit and push verified
  milestones with truthful remaining gaps; archive this task only when all pass.

Execution contract: local software/test artifacts and synthetic virtual effects only.
No physical device detection, USB access, real signer/pool credentials, hardware
flashing, physical mining/grants, ROM operations or physical faults. Synthetic
signed frames are permitted only inside isolated virtual scenarios. All existing device-effect
flags remain disabled. Virtual builds are separately identified and cannot become
flashable physical candidates. Synthetic NVS/identities/trust and bounded private
local evidence only; no modification of sealed predecessor inventories or raw dumps.
Pinned upstream reference remains read-only. Functional models are not electrical,
RF, cycle-accurate or physical USB timing proof. Keep calibration gaps and modeled
memory distinct from actual SDK observations; no hardware parity promotion.
This task promotes no parity row (`docs/parity/progress.jsonl` is authoritative);
accepted-share stays active/unresolved. Changed SDK/model/source/
package/validator identities invalidate affected results. Unknown changes select the
full validation suite. Missing required profile, calibration or process-release proof
is a concrete blocker, never success or permission for another hardware attempt.

Local milestone review | 2026-09-30: runtime/model/command foundations implemented.
Initial ordered Cargo checks and 17 affected Bazel targets pass; the production
package builds. Host step-5 command passes; healthy host deliberately returns
unsupported for missing encrypted share integration. No device access occurred.
Full qualification and mandatory rollout remain unresolved; do not archive.

QEMU diagnostic continuation | 2026-09-30: pinned virtual guest component probes
prove boot, 8192-byte internal allocation/release, task join, NVS reset increment
and deliberate panic/core decoding. Healthy composed execution instead faults in
SDK TLSF during possession canonical serialization. Splitting the guest UART
reader reduces the dispatch frame from4800 to832 bytes with the same16384-byte
main stack, but the healthy failure recurs. This is virtual-target evidence;
physical status001's cause remains unproved. Preserve every prior private run.

Bounded next diagnostic contract: one instrumented synthetic healthy QEMU run
only, paired exact virtual ELF/source/config and pinned emulator; allocation-free
heap-integrity and task-stack observations before controller/possession boundaries,
plus an offline native call-frame audit. Allowed: local compilation, virtual flash
copies, GDB on the owned emulator and private logs/core inspection. Prohibited:
physical detection/USB/network access, signer credentials, flashing, mining,
grants, partition clearing, changed stack/safety limits and blind retries. Bound
emulator runtime to30 seconds and debugger output/time; always release/reap the
owned process group and retain partial evidence if core capture is unavailable.
Stop this diagnostic on recurrence after a targeted correction, unproved process
release, source/digest drift or missing meaningful crash state. Broader independent
host/model integration may continue without another target retry.

QEMU diagnostic015 successor | 2026-09-30: diagnostic014 was consumed without
resuming the paused guest because the pinned debugger rejected a command. Its
partial result and released process groups remain retained. Offline preflight now
passes against exact virtual ELF
b76fdd3dd942dbebc3b47143530ae772c54b83fd35f15a7cfbf1aa6a36a7ce36,
including supported debugger syntax, exported checkpoint symbols, 384-byte record
layout and verified Xtensa argument register. Permit exactly one successor
`just virtual-emulator diagnose-heap --manifest <retained-build014>/virtual-package.json --evidence-dir <repo>/scratch/virtual-emulator/diagnose015-health`.
The prior local-only effects, 30-second emulator/25-second debugger bounds,
privacy, no limit increases, cleanup and stop conditions apply unchanged. This
examines the frozen pre-extension guest; it cannot qualify the current broader
source or establish the physical failure cause. No further checkpointed target
retry is authorized by this continuation.

Corrective local qualification continuation | 2026-09-30: diagnostic015 measured
9280 stack bytes available at the pre-controller checkpoint. Its own144-byte
frame and the32-byte observer frame end before construction, giving9456 bytes
available at the constructor callsite. The exact frozen guest's resolved required
trust-key validation path alone needs10752 bytes (before additional curve leaves):
a1296-byte deficit. This is a demonstrated virtual caller-frame insufficiency,
not proof of the physical status001 cause or a complete callgraph upper bound.

Split the shared scenario initializer from the larger Start/status execution frame
and return an owned boxed controller; preserve the16KiB configured stack, SDK
extra512 bytes, all safety/authority/accounting limits and observer order. Verify
host corpus equality and build a separately marked corrected virtual package with
fresh source/ELF/config bindings. Offline audit must show the initialization frame
and constructor path fit with the existing margin before target execution. Then
permit one fresh synthetic `healthy-lifecycle` seed1 QEMU run only, with status and
allocation probes, bounded to30 seconds, in a new private evidence root. No
checkpoint diagnostic, new limit, physical discovery/effect or old grant replay.
Stop on recurrence or missing release/proof; save a precise blocker and continue
only independent host checks. Ordinary full qualification is still not authorized
by a passing component probe; all mandatory profiles and rollout criteria remain.

Evidence review correction | 2026-09-30: require natural host completion separately
from resource release; verify scenario/seed/source/package/executable bindings,
embed compiler inputs rather than attest to a later checkout, and verify copied
frozen manifest/config digests. New host run-v3 records preserve earlier v1/v2
results. Resolve compiler-stamp build setup/API errors with focused offline checks
before broad qualification. No previous result or seal may be rewritten.

Corrected target outcome | 2026-09-30: fresh guest build017 pairs the production
ELF27594c9f92e84e907d1f08acd87ea6a1c0644168a45a83c8dcfbbeeacf282b22;
virtual ELF579fdd34473fe17245f36d7e026fbd68e1041b0a7408a1cb3f958d4b030eb98e.
Further separation retires board construction and trust-validation temporaries
before controller assembly. The ordinary wrapper-inclusive selected native path
needs10752 bytes plus992 of observed leaf/iteration allowance:11744 total, leaving
4640 against the unchanged16384-byte configuration. This is a selected lower-bound
check; the complete call closure remains unproved.

The one authorized `run017-health` target attempt then failed: boot, task join,
internal allocation/release and allocator observations passed, but an unexpected
panic/reset prevented the scenario and actual stack-margin results. Host process
group release passed. The retained core's checksum and full ELF identity verify;
SDK fake frames are absent. The meaningful fault is LoadStorePIFAddrError in SDK
find_containing_heap, reached through heap_caps_free and a Rust Vec drop during
Noise mix_hash/Initiator completion inside the authenticated Start path. The
constructor progressed; the new failure's cause and actual margin remain unproved.
Raw core/debugger/registers remain private under run017-health/decoded-core. Stop further target execution under the recurrence rule;
retain exact ELF/logs/flash and any partial core. Constructor margin alone cannot
qualify target execution or establish the remaining cause. No device access occurred.

Preflash observation outcome | 2026-09-30: `just preflash-validate` saved a sealed
working-tree observation with42 passed/21 unsupported host cases across63 rows;
all63 target corpus cases were withheld because required host integration is
unsupported. Five exact production native audits passed. The development source
was dirty, full qualification was false, no package was frozen and no flash occurred.
The original report remains immutable; unavailable comparison coverage is typed
unsupported by the successor validator, not a demonstrated semantic mismatch.

Noise diagnosis continuation | 2026-10-01 | Approved retained-core and minimal probe

- [x] Independently verify retained run017 checksum/full ELF and pinned SDK task
  ABI; inspect Vec deallocation layout, stack bounds and captured heap data in a
  new private analysis root. Preserve raw dumps and offline001 findings unchanged.
- [x] Implement a host/target Noise-only probe using the actual production
  NoiseInitiator and the existing synthetic responder/transport shape; isolate
  controller/board/work/grants. Verify authentication, frame bounds, MAC/truncation
  rejection and phase ordering locally; retain a specific red-capable signal.
- [ ] Add allocation-free phase callbacks for SDK heap integrity and actual stack
  observations; keep diagnostics local to the marked virtual target. Build and
  audit exact ELF/caller frames and the existing2KiB margin before execution.
- [ ] Publish and push the complete repo-owned command/contract and verification
  receipts before a fresh bounded emulator attempt. Use fresh private evidence,
  exact package/source/config/model/emulator/validator bindings and verified release.
- [ ] Compare the minimized target result with retained full-scenario failure;
  state ranked falsifiable hypotheses before any additional diagnostic variant.
  Apply only an evidence-backed lifetime/ownership correction, add a regression,
  audit and then recheck the original composed boundary through a separate gate.
- [ ] Run affected Bazel and ordered Cargo checks, native audits, reference,
  redaction, standards, Markdown and diff checks. Commit/push truthful outcomes;
  leave this parent task and accepted-share unresolved unless all criteria pass.

Effect policy: offline inspection and synthetic virtual effects only, no physical
USB/detection/network/device effects, no signer or real credentials, no grants,
mining, work/nonce/submission or ASIC activity in the minimal probe. The synthetic
Noise handshake is solely an in-memory byte exchange. Maintain16KiB main stack,
SDK extra512 bytes, original internal/PSRAM routing/reserve and all authority,
accounting/safety limits. No original seal repair or core clearing. Raw memory,
registers/debugger output stay in ignored0700 roots/0600 files under ADR-0030;
share only redacted findings and hashes. Parity remains90/95.

noise-probe-baseline-enabled: false

Baseline execution gate enabled for exactly one clean/pushed Noise-only baseline. Admission requires a fresh exact
marked package, verified offline stack/resource bounds and a tested repo-owned
command. Once published, permit one Noise-only valid-handshake seed1 baseline,
QEMU at most30 seconds and any debugger at most25 seconds, single owned process
family, no automatic unchanged retry. Always collect partial facts/core on panic
and terminate/reap children/listeners before sealing. Missing checksum/ELF, stale
source, unexpected effects, invalid input/phase proof or unproved release stops
this stage. A differing or green minimal result diagnoses context dependence;
it is not full lifecycle or hardware qualification. Subsequent variants require
specific hypotheses and separately published scope, not another broad Start.

Offline revalidation | 2026-10-01: offline001 and independent offline002 both
verify checksum/full ELF. Main SP is10168 bytes below the SDK TCB base; observed
deduplicated native entry frames total25824 bytes. Fixture caller8464 and
constructor7136 overlap Noise completion6080 and outer frames. This proves a
virtual callsite stack contract violation. The96-byte Vec pointer is aligned,
in configured PSRAM and consistent across frames; its payload/header and heap
registry descriptor are absent. The direct registry node is unaligned/outside
configured memory. Earliest overwrite, double-free and physical cause are unproved.
See `docs/testing/20261001-noise-panic-diagnosis.md`; raw findings remain private.

Baseline publication | 2026-10-01: repo-owned command
`just virtual-emulator noise --manifest <fresh-current-clean-marked-package>/virtual-package.json --audit <fresh-exact-ELF-native-audit>.json --evidence-dir <repo>/scratch/virtual-noise-diagnostic/baseline001 --seed 1 --mode valid`.
One-use claim: ignored `scratch/virtual-noise-diagnostic/baseline001.claim.json`,
created exclusively before effect and never reset/reused. Only this command/seed/
mode is enabled. Require clean pushed code, package source equal to current HEAD,
source_dirty false, full ELF/image/config hashes, current compiled-source closure,
fixed16KiB stack/SDK routing, live current auditor+objdump replay and every required
selected native path within14336 bytes before admission. No changing declared
artifact identities to reuse a stale report. Retain all failed/unused prior roots.

Offline build001 observed Noise-only completion path12704, frame path7696 against
14336 available. Twelve native audit and23 runner regressions pass; seven actual
Noise host regressions pass. Broader crypto/indirect-call closure remains unproved,
so baseline must additionally prove all14 phase checkpoints, heap integrity,
configured stack observations and actual SDK low-water at least2048. The dirty
build001 is diagnostic development evidence, not the enabled execution package;
a fresh clean marked build and exact audit are required after this publication.
Result must bind validator/source/package/auditor/emulator and verify process/FD
release before finalization; failures/partial core survive independently. A
collection cutoff never becomes fixture natural completion. No full lifecycle,
physical diagnosis or parity qualification is implied by Noise-only success.

Baseline001 result and re-plan | 2026-10-01: the one-use claim was consumed at
pushed df639c99, clean virtual ELF80875ff9aaef090bc3f9016693c60de2fc99161771e66f9c87c2959b7750a94d,
compiled sourcebfeff883011f1d720af6c4deb018c61f7ce465f086fe8a71711e5b1131d97d62.
The minimal target panicked before a result; process/FD release passed, the virtual
core partition was empty. The first LoadStorePIFAddrError is in TLSF integrity
inspection; the second LoadProhibited is in panic/core-storage preparation, not
another baseline invocation. Preserve both partial roots and the consumed claim.

Correction from exact native caller instructions: optimized DWARF line attribution
suggested BeforeCompletion, but the actual call loads phase111AfterCompletion,
then advances to112. Valid certificate verification had returned before the
failing heap check. Do not infer that completion was never entered. The valid
certificate/strauss/odd-table path requires17264 bytes before additional branches,
exceeding even nominal16896 allocation. The old selected mix_hash fit receipt
omitted this required valid branch and cannot admit another full Noise run. No
current task bounds/checkpoint RAM were retained, so first overwrite is still
unseen; retain the distinction from the earlier run017 proved stack overrun.

Staged emulator bisection | 2026-10-01 | Explicit owner strategy change

- [x] Implement typed runtime-prefix diagnostics with checkpoint, boundary-reached
  and released conclusions. Prefixes101..110 stop before the known oversized
  completion branch and must never credit full authentication/frame/mining success.
- [x] Preserve one compiled campaign image/seed/config for runtime-prefix comparisons;
  record exact native caller frames and their delta from baseline6368. The new
  prefix helper is a separately compiled artifact, not an unchanged baseline
  frame. Compile-time code removal is a separate
  corroboration artifact with changed frame measurements, never automatic fix proof.
- [x] Strengthen native audit to bind the exact selected cutoff and constructor/act-two
  crypto-family paths. Report unresolved external and indirect edges explicitly;
  require independent live bounds and heap facts instead of claiming an upper bound. Full-completion admission must also include
  certificate verification and nested crypto, blocking the known17264-byte path.
- [x] Add a private debugger owner that pauses at reached-prefix, completed release
  or first panic/assert before secondary panic capture. Save bounded phase RAM,
  task stack bounds/SP/high-water and heap-integrity facts without invoking target
  allocation functions from GDB. Verify installed debugger grammar offline first.
- [ ] Publish/push command, tests and one-use per-cutoff inventory before effects;
  binary-search the earliest bad prefix with fresh roots and release every owner.
- [ ] Apply only a measured lifetime/ownership correction and real-boundary
  regression, rebuild/audit, then independently admit the corrected full boundary.
  Never raise stack/heap/safety limits or treat a shortened prefix as qualification.

noise-prefix-bisect-enabled: false

Effect contract: emulator-only prefix code bisection, no physical discovery/USB,
credentials, grants, work/mining/ASIC, original replay, erase or parity promotion.
Use same actual Noise initiator/responder/time100/lifetime3600/seed1 and pinned
SDK5.5.4/QEMU backend. Keep16KiB main/configured2KiB margin, SDK extra512 and
allocation routing/reserve unchanged. Allowed runtime cutoffs101,102,103,105,106,
107,109,110;110 is the last pre-completion cutoff. Known bad111 is historical
feedback only and cannot execute until a separately audited correction exists.
Each cutoff consumes one exclusive claim bound to frozen source/ELF/config and
native cutoff audit; max8 distinct prefix trials, no unchanged repeat. Select
halves adaptively from recorded outcomes; distinguish boundary reached while
owners live from cleanup/release failure. Missing facts remain unknown and halt
binary-search inference rather than becoming a healthy prefix. Per trial bound
QEMU35 seconds, debugger25 seconds, output2MiB, private new0700 root/0600 files.
Always terminate/reap QEMU/GDB descendants and close listeners before finalizing.
A cutoff image/diagnostic result is explicitly partial and physical-ineligible.
Stop on source/identity drift, unexpected effects, unproved ownership release or
any requirement to enter the known over-budget branch. Further trials after a
verified boundary-changing correction use a new explicit successor inventory;
no elapsed human-response timeout or per-trial confirmation is required.

Ranked predictions after the red-capable minimal baseline: (1) overlapping virtual
caller/crypto stack frames are strongest; retiring constructor/caller temporaries
must lower measured native peak and restore phase111 integrity at unchanged
limits. (2) earlier heap damage would already fail a pre-completion prefix or
its initial checkpoint. (3) an ownership/emulator allocator defect would persist
after stack bounds/margin and healthy pre-completion heap are independently
proved; no current evidence proves double-free or a Noise algorithm fault.

Commands: build the clean campaign package with `just virtual-emulator build`
and the pairing manifest, then generate each cutoff receipt through
`bazel run //scripts:virtual_noise_prefix_audit`. Execute only
`just virtual-emulator noise-prefix --manifest scratch/virtual-noise-diagnostic/build005-clean-prefix/virtual-package.json --audit AUDIT --stop CUTOFF --seed 1 --evidence-dir ROOT`.
The exact build/audit invocations and placeholder bindings are published in
`docs/testing/20261001-noise-panic-diagnosis.md`. `AUDIT` and `ROOT` are fresh
private paths for that admitted cutoff; all artifact identities are checked
before the one-use claim and effects. Start at 106 only if its native audit
passes, otherwise select an earlier admitted prefix.

Pre-effect verification | 2026-10-02: ordered Cargo format, Clippy, all-target
build and all-feature tests pass; nine affected Bazel suites pass. Exact native
prefix audits for 101/106/110 pass their selected-path budgets. The campaign
helper frame is 2,704 bytes, 3,664 below the separate baseline helper. This is a
changed diagnostic artifact, not a corrected full-handshake claim. The retained
110 crypto-family ECDH path is 13,600 bytes; unresolved external/indirect paths
remain explicit and live margin/heap proof is mandatory. Offline GDB SDK ABI and
command grammar pass against full ELF
`1ca18eea8596e0f4eb33c5e74e5db8c065138cd71231f2bd06d97cf1468aeaa9`.
Compiler-folded marker aliases now require mutually exclusive flag/argument
conditions. Missing Rust DWARF object types are verified from exact ELF symbol
sizes; no live function calls supply those facts. Standards, reference,
redaction, scoped Markdown and diff checks pass. One debugger subagent stopped
with a provider cybersecurity-content flag; its completed evidence was preserved,
and local artifact validation resolved the independent preflight type failure.
No prefix execution occurred before publication. Rebuild from this clean pushed
source and recheck exact package/audit/debugger identities before claiming 106.

Campaign001 result | 2026-10-02: published at 8817a8a3 and consumed cutoff106.
Clean virtual ELF `0d4d9400391793fa4285cfc2db4edde3d8cf3baaed814216efb4a23a52b3c5d0`
completed all six guest prefix checks with no observed failure, but debugger
collection timed out and supplied no independent snapshots. Prefix and cleanup
snapshot conclusions remain unknown. QEMU/GDB process groups, log descriptors
and the loopback listener were released. The one-use claim and all evidence are
immutable. Native breakpoint placement was at the hook entry before register
window setup; this is a diagnostic collection gap, not proof of a target failure.
Do not repeat or promote campaign001.

Checkpoint-only continuation | 2026-10-02 | Explicit owner constraint

- [x] Replace active prefix execution with ordinary application checkpoint
  collection. No debugger attachment, memory capture, core acquisition,
  extraction, inspection, decoding or examination of panic output.
- [x] Project only complete `VIRTUAL_U205` application records into retained
  stdout; discard other stdout and stderr without interpreting it. Bound all
  raw process bytes and partial-line buffering; fail visibly on overflow.
- [x] Keep source/ELF/config/native-audit bindings, existing stack/heap limits,
  one-use cutoff claims, unconditional host cleanup and subset-only results.
  Separate guest facts from unsupported independent task-bound observations.
- [x] Verify real process projection and release plus affected scenario/CLI
  suites; run ordered Cargo, standards/reference/redaction/Markdown/diff checks.
- [x] Publish and push this successor contract before its first trial; collect
  106 then select the remaining half using measured application records. Missing
  completion/checkpoint output stops diagnosis without reading crash material.

noise-prefix-telemetry-enabled: false

Successor effect contract: campaign002 replaces debugger-driven collection only.
Use `just virtual-emulator noise-prefix --manifest scratch/virtual-noise-diagnostic/build006-clean-telemetry/virtual-package.json --audit AUDIT --stop CUTOFF --seed 1 --evidence-dir ROOT`.
Build/package from the clean pushed source and generate each cutoff audit with
`bazel run //scripts:virtual_noise_prefix_audit`; exact manifest identities and
all selected-path limits must pass before claiming an effect. `AUDIT` and `ROOT`
are fresh ignored private paths for each admitted cutoff. Keep SDK5.5.4, pinned
QEMU, the same synthetic credentials/time/seed, main16KiB, margin2KiB and existing
memory routing/reserves. Only101,102,103,105,106,107,109,110 are allowed; no full
certificate completion, Start, grants, fixture/mining/ASIC, physical discovery,
USB, flashing, restart, real credentials or original replay. Campaign002 permits
one newly instrumented106 and at most eight distinct cutoffs total, each bound
to one fixed image/source/config/validator and an exclusive durable claim.
Campaign001 claims are never cleared or reused. QEMU35seconds, total output2MiB,
partial line64KiB; terminate/reap every descendant and release descriptors before
finalization. Retain only ordinary structured application facts and host release
records. Missing, invalid or unsafe checkpoint facts stop further inference;
unknown is never a healthy prefix. No core partition or debugger material is
read after execution. Independent live task-bound proof remains unsupported and
prevents full qualification; guest checks alone may support this explicitly
partial diagnostic comparison. Seals stay immutable, accepted-share stays
unresolved, physical-effect gates stay disabled and parity remains90/95.

Pre-effect review: real synthetic process projection/release regressions pass
35/35, ordinary telemetry runner tests pass8/8, and six affected Bazel suites
pass. Ordered Cargo format/Clippy/build/tests pass. Standards, pinned-reference,
redaction, scoped Markdown and diff checks pass. The new runner is smaller,
removes the debugger dependency from active collection, and never reads a
post-run partition. Explicit v2 result semantics preserve partial guest evidence
without claiming independent task bounds or full qualification. New record
parsing rejects malformed/non-object output before inference. Current commands
and coverage limitations are in
`docs/testing/20261002-checkpoint-only-bisection.md`. Publish this reviewed source
before the first campaign002 effect and keep earlier outcomes immutable.

Campaign002 measured outcome | 2026-10-02: source17461e37, clean virtual ELF
`6d07737269d24d3749adad65150d8285a2d7ce948469a318965f23236f9e758b`,
SDK configuration`cbe2ab13dc61885a77a0052ebee912ce0494393c41d0c3390cae293dbaabd6e1`.
Both106 and110 pass all six application prefix checks with healthy recorded heap
integrity and post-drop release. Minimum observed main free stack:106=9,672 bytes;
110=2,296 bytes, above the unchanged2,048-byte requirement. Verified host process,
descendant, descriptor and writer release passed for both. Target stderr files
are empty; only projected ordinary records were retained (2,342/3,174 bytes).
No debugger, memory/core acquisition, decoding, post-run partition inspection or
examination of panic data was used. Claims and records remain immutable. Close
campaign002 rather than spending further unchanged cutoffs. These are diagnostic
subset passes with independent task bounds unsupported and full qualification
false. They narrow the next source investigation to completion/live caller-frame
overlap; changed prefix frames prevent claiming an unchanged-baseline bisection
or a demonstrated exact failing statement.

Completion lifetime correction | 2026-10-02 | Source and build artifacts only

- [x] Extract preparation into a non-inlined helper whose native frame ends
  before certificate completion. Keep actual codecs, trust, deterministic inputs,
  clock and existing failure categories/phase ordering unchanged.
- [x] Use fallibly reserved owner storage and the existing production completion
  seam to reduce result-construction scratch; retain unconditional post-owner
  release observation on every error. Add a meaningful reservation failure test.
- [x] Verify real host handshake/frame regressions, independent trust rejection,
  exact target native paths through construction, ECDH/signing, valid certificate
  verification and nested crypto, serialization and cleanup. Report unresolved
  paths explicitly; never increase stack, heap, authority or safety limits.
- [ ] Keep every execution gate disabled while building/auditing the correction.
  Publish a separate checkpoint-only full-boundary diagnostic contract only if
  its native resource admission passes. No core/panic/debugger data is permitted
  in this continuation. Missing normal records stop inference.

Current blocker: complete-boundary resource proof and a newly admitted corrected
run do not exist yet. The known original full helper remains inadmissible. This
source-only correction is a measured hypothesis, not a physical firmware repair
or virtual-board qualification. Accepted-share remains unresolved; do not archive.

Correction source verification | 2026-10-02: ordered Cargo format/Clippy/build
and all-feature tests pass (2,581 passed, 0 failed), including all 11 Noise probe
tests with real owner-reservation failure and unreserved-slot regressions. Eleven
affected Bazel suites, standards, reference, redaction and diff checks pass. No
local Markdown formatter is installed; none was added. Host behavior only: the
corrected target's native resource paths remain unaudited, so items 3 and 4 stay
open and the development build under `build007-lifetime` is not admitted.

Responder frame correction | 2026-10-02 | Dev builds only, never executed

- [x] Measure the `85999542` correction with the longest resolved crypto-family
  descent, calibrated against campaign002 cutoff110 (static ECDH 14,272 bytes vs
  runtime ~14,088 used). Completion with certificate verification fell from
  16,160 to 12,160 bytes, but responder ECDH/sign rose to 15,104 bytes, over the
  unchanged 14,336-byte budget: the first correction moved preparation over margin.
- [x] Split preparation into sibling non-inlined frames: `prepare_initiator`,
  `respond`, `construct_responder` and `step_responder`, consuming the act-two
  result in place. The responder is already boxed upstream; no new allocation.
  Phase order, seeds, trust and fault behavior are unchanged.
- [x] Replace the full-probe native auditor with v2: required paths follow the
  corrected helpers and extend each crypto boundary by its longest resolved
  descent, including a named certificate-verification path, encrypted frame and
  outcome emission. Indirect, outside-family, cycle and drop-glue edges remain
  reported, not bounded; `complete_callgraph_bound` stays false.
- [x] Build a clean package from the pushed source and bind a v2 receipt to it.

Dev measurement (`build011-step-in-place`, dirty source, not admitted): all eight
v2 paths fit. Tightest static headroom: responder ECDH/sign 13,552 bytes (784),
completion and certificate verification 12,288 bytes (2,048). Static descents are
lower bounds; only a separately published checkpoint-only run can measure runtime
margin. Gates stay disabled.

Clean v2 receipt | 2026-10-02: source `7b2fe9e6` (clean), compiled source
`88ae2a06275ec63a50663526ca1092f5423a29763a67d6c11ebebf58a2340cbf`, virtual ELF
`73b3f2ccded8958dc47ecf7325c70389a3bce5546a13231f523b470ef49f3dec`, SDK
configuration `10f5335da267cef5bdcd383671ca20a7d41b13072b96cca00aed04a24ded7bda`,
auditor `ce7fd5da2b31e05cdbad163fedd617e46277bca04bb34bc14ae803a740002385`,
receipt `b01492292e8355218a6b7236fd82a76b1d82945f11b22eeae51c04cf45515383`.
All eight paths fit, with figures identical to the dev measurement. Read-only
`admitNoisePackage` recomputes and accepts the native proof. This satisfies the
static native resource admission only. Drop glue, indirect and outside-family
edges remain unbounded, and runtime margin, heap integrity and release are
unmeasured. No emulator, debugger or hardware effect was run. The receipt lives
under ignored `scratch/virtual-noise-diagnostic/build012-clean-split`.

Next: item 4 may now publish a separate checkpoint-only full-boundary contract
for this exact package. It needs full-probe checkpoint telemetry, a one-use claim,
a fresh protected root and an explicit runtime-margin and heap verdict before any
effect gate is enabled. Until then, full qualification stays false and
accepted-share stays unresolved. Do not archive.

Full-boundary checkpoint contract (campaign003) | 2026-10-02 | Explicit owner approval

- [x] Add `noise-checkpoint`, a full-probe runner that keeps only projected
  `VIRTUAL_U205` application records. It does not use a debugger, panic text,
  core decoding or post-run partition reads. Package admission moves to
  `noise-admission.mjs`, so the runner never loads crash tooling. The validator
  identity binds every visible emulator and core-dump script.
- [x] Publish and push this contract with both gates enabled, then build the
  package from that exact commit and bind a fresh v2 receipt to it.
- [x] Run exactly one claimed execution, record the outcome, disable both gates,
  then commit and push.

noise-full-checkpoint-enabled: false

Objective: one emulated valid seed-1 handshake, certificate completion and
encrypted frame round trip on the corrected image. Judge only application
records: boot identity, authenticated round trip, owner release, 14-phase heap
integrity, and at least 2,048 bytes of free stack at every phase.

Commands, run from a clean tree equal to `origin/main`:

```sh
just virtual-emulator build --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --evidence-dir scratch/virtual-noise-diagnostic/build013-full-checkpoint
node scripts/virtual-emulator/noise-stack-audit.mjs --elf ELF --sdkconfig CONFIG --compiled-source-sha256 SHA --output scratch/virtual-noise-diagnostic/build013-full-checkpoint/full-audit-v2.json
just virtual-emulator noise-checkpoint --manifest scratch/virtual-noise-diagnostic/build013-full-checkpoint/virtual-package.json --audit scratch/virtual-noise-diagnostic/build013-full-checkpoint/full-audit-v2.json --seed 1 --evidence-dir scratch/virtual-noise-diagnostic/full-checkpoint001
```

ELF, CONFIG and SHA must come from the build013 package. The receipt must pass
before any effect. The evidence parent stays mode 0700, and the
`full-checkpoint001` child must be absent immediately before launch. Wrapper
stdout and stderr go to separate mode-0600 sibling files.

Allowed effects: one bounded emulator process with a 60-second collection
window, owned and reaped by `runPrivate`. Prohibited: hardware, USB, network,
debugger, memory or core capture, reading target stderr or panic text, partition
inspection, and raising any stack, heap or safety limit.

Retry bound: one execution. The claim
`scratch/virtual-noise-diagnostic/full-checkpoint001.claim.json` is written
exclusively immediately before the effect. A failure before that claim consumes
nothing and may be repaired and rerun with a fresh root. Once claimed, an
unchanged retry is prohibited, and any successor needs a new contract and a
verified fix.

Stop conditions: a failed check, missing or invalid records, an unproven release,
or a source or validator change. Missing normal records stop inference without
reading crash material. Evidence and privacy: raw artifacts stay under ignored
mode-0700 roots; commits record only digests, check outcomes and numeric
observations. Passing would prove only this emulated diagnostic. Independent
task bounds stay unsupported, and full-board qualification, hardware, parity
and accepted-share remain unchanged.

Campaign003 outcome | 2026-10-02: the contract was published at `a024948a`. The
package built from that commit has compiled source
`88ae2a06275ec63a50663526ca1092f5423a29763a67d6c11ebebf58a2340cbf`, virtual ELF
`1bc1cfd7c3d5828a2670b271134192fef5c97d7f1f2a6b8d4b2cd8d3326886df`, v2 receipt
`6bd3a035bc4f0335084af59f388bdb2cfe41213ad96922118f8b33bc032e8094` and
validator `b5820382ceaa02e6a7ea37066c92f4f7f08f5eeb1aa6da06cc33783d010ec52e`.
The single claimed execution passed all five checks: boot identity, an
authenticated handshake with certificate completion and a 32-byte frame round
trip, owner release, heap integrity at all 14 phases (101–114), and the
unchanged stack margin. Minimum free main stack was 3,016 bytes, first reached
at phase 109 (responder ECDH and sign), or 13,368 bytes used against the
13,552-byte static path. Completion and the frame stayed shallower. Internal
free heap ranged from 340,723 to 346,403 bytes, with a constant 188,416-byte
largest block. The emulator was released at the collection cutoff, no writer
leases remained, and target stderr was empty. The projected application records
were 3,701 bytes. Result digest:
`8447cc4e865de46429d8a0f608d887500764c2a7df1317647dda30deeaa86a61`. The claim is
consumed, so this run cannot be replayed. Both gates are disabled again.

This supports the frame-lifetime hypothesis in emulation only. The earlier
baseline's heap-integrity fault after completion is absent from the corrected
image. That does not show which statement was originally faulty, and it is not
a device repair. Independent task bounds remain unsupported, and full-board
qualification, hardware, parity (90/95) and accepted-share are unchanged.
Remaining parent-task blockers are listed under Verification below; do not
archive.

Verification: ordered Cargo format/Clippy/build/tests,21 affected Bazel targets,
55 model tests,19 encrypted-profile tests,9 scenario tests, compiler-closure
stale-source/build regressions, process/listener and evidence regressions, five
production native audits, reference/redaction/standards/Markdown/diff checks.
Final verification receipts and public summary accompany the committed milestone.
Completion review: not complete; do not archive. Explicit remaining blockers are
missing actual margin/call-closure proof, strict live fixture/header/nonce vector
(the Gate/page/HTTP/framed-I/O integration, lease flows, the close-rejection owner
release and real allocation interception are complete in increments 1-7 below),
consolidated production lifecycle/ownership profiles, full
qualified host/target corpus and mandatory matching-report rollout. The
corrected-target panic is resolved in emulation (campaign004). Missing
calibration remains an explicit fidelity non-claim. Accepted-share stays active,
all physical-effect gates stay disabled and this task promotes no parity row.

Framed-link integration, increment 1 | 2026-10-10 | Owner-approved plan

Objective: run the production inbound `bwg-worker-serial/0.2` link decisions on
the host. Extract them from firmware `bwg_worker_usb/link.rs` into a pure
`bitaxe-worker-control::serial::link` core behind a statically dispatched effects
trait, keep the firmware file as a thin adapter, and drive the simulation through
real envelopes (Hello, sequence, heartbeat liveness, receive credit, Close) instead
of bare controller JSON. Later increments: outbound writer core; a stdio virtual
device driven by the pinned Gate controller; the Gate page plus loopback HTTP
supervisor, which alone may flip `gate-page-http-seams`.

- [x] Refresh stale status, blocker and parity text; derive the validation report's
  parity figure from the ledger instead of a literal.
- [x] Add a host characterization harness that records today's link effect
  transcripts, and keep a pre-refactor ELF as the stack baseline.
  `//firmware/bitaxe:usb_link_characterization_tests` pins 21 ordered effect
  transcripts covering every reachable failure stage. New `just audit-link-stack`
  on the pre-refactor ELF: `link::run` frame 1,264 bytes, deepest normal path
  2,928 of a 6,144-byte budget (3,216 headroom, 96 unresolved targets).
- [x] Extract the pure core with unit, allocation-failure and Gate conformance
  vector tests; reduce firmware `link.rs` to an adapter with identical transcripts.
  `bitaxe_worker_control::serial::link` (`SerialLinkReceiver`, statically
  dispatched `SerialLinkEffects`, pinned `SerialLinkFailure` codes) has 52 unit
  tests; `:serial_link_tests` drives the pinned Gate serial and integrity vectors
  and armed allocation failures. All 21 characterization transcripts reproduce
  byte for byte through the adapter.
- [x] Update source-ownership tests and the native USB ownership verifier; add a
  link-thread stack audit and compare it against the baseline. The compiler
  spills loop call targets to stack slots, so the audit now also counts every
  address-taken callee (conservative). On that basis the baseline ELF measures
  5,600 bytes and the refactored ELF 5,536 of a 6,144-byte budget, with the same
  envelope-parse deepest path; the control-owner audit is unchanged at 10,256 and
  no runtime realignment callers exist.
- [x] Drive simulation scenarios through the core; add a `framed-serial-link`
  coverage row while `gate-page-http-seams` stays unsupported. Every
  controller-backed scenario now opens a Hello, sends sequenced Control records,
  replays the host's one-second heartbeats at their original instants (keeping
  envelope parsing out of the deep preparation stack) and ends with a clean Close
  checked as `serial_link`. `heartbeat-loss` now closes the link at its liveness
  deadline (`link_heartbeat_timeout`) and, like the firmware, delivers no further
  status or restore frames; owner cleanup is still checked.
- [x] Run the affected Bazel and ordered Cargo checks, firmware build/package,
  native audits, redaction/reference/parity/standards checks; commit and push.

Authorization: repository source, tests, fixtures, documentation, build, package,
static ELF audits, host-backend virtual runs, commit and push only. No detector,
USB/device session, flash, monitor, emulator effect gate, credentials, network
discovery, mining or other hardware effect. No parity promotion.

Verification: ordered Cargo fmt/Clippy (`-D warnings`)/build/tests; all 325
Bazel tests (`just test`); the 21 pinned link transcripts; `just build`,
`just package`, the virtual guest Xtensa build (`just virtual-emulator build`,
compile and frame audit only); link, control and realignment native audits;
host-backend `just virtual-board run` rows for healthy-lifecycle, heartbeat-loss
and cancel-preparation; native USB ownership, redaction, reference, parity and
Bright Builds checks.

Completion review: increment 1 is complete. Firmware and simulation share one
inbound link implementation, pinned byte-for-byte to the previous firmware
behavior, and the virtual corpus now exercises it. Residual risks and non-claims:
the refactored firmware link has not met hardware; it does so only in a later,
separately contracted hardware task. QEMU host/target agreement and the target
main-stack margin were not rerun for the changed scenarios. The outbound writer,
the pinned Gate controller, the Gate page and the HTTP supervisor remain
unexercised, so `gate-page-http-seams` stays unsupported and qualification
stays false. Next: increment 2 (outbound writer core).

Framed-link integration, increment 2 | 2026-10-11 | Owner-approved plan

Objective: share the outbound `bwg-worker-serial/0.2` writer decisions. Extract
hello_ack, device heartbeats, receive and terminal credit, control replies,
output admission, sequencing and framed-versus-bootstrap diagnostics from
firmware `bwg_worker_usb/writer.rs` into a pure
`bitaxe-worker-control::serial::writer` core, prove byte-identical output, and
make the simulated device emit real device-to-host records that a Rust host peer
checks against the pinned Gate rules. Firmware-only diagnostics scheduling stays
in the firmware adapter.

- [x] Add a deterministic writer characterization harness with pinned
  transcripts and a writer-thread stack audit; record the pre-refactor baseline.
  `//firmware/bitaxe:usb_writer_characterization_tests` runs the unchanged
  writer loop against a scripted clock, queue and production-shaped transport
  (per-chunk, pre-drain and post-flush admission checks, real `WriteFailure`s,
  partial-output tracking) and pins 22 transcripts. New `just audit-writer-stack`
  on the pre-refactor ELF: deepest normal path 5,536 of 6,144 bytes (608
  headroom) through credit, `next_record`, `emit` and envelope encode.
- [x] Extract the core with unit and pinned Gate device-record vector tests;
  reduce firmware `writer.rs` to an adapter with identical transcripts.
  `bitaxe_worker_control::serial::writer` (`SerialRecordWriter`, a `&self`
  `SerialWriterView` plus a `&mut` `SerialOutputSink`) has 30 unit tests, and
  `:serial_writer_tests` reproduces the pinned Gate device `heartbeat` and
  `receive_credit` wire strings byte for byte. All 22 writer transcripts and the
  21 link transcripts are unchanged; the real-thread startup harness passes.
- [x] Update source-ownership tests, the native USB ownership verifier, the
  bootstrap writer audit and source inventories; compare the stack audit. The
  bootstrap writer audit now selects the core and the adapter's trait impls
  (frame sum 5,424 of 7,680, decode complete after replacing a purpose `match`
  jump table with table loads), and its synthetic corpus fixture stages every
  source. The writer thread measures 5,616 bytes against the 5,536 baseline
  (6,144 budget; an inline hint made it worse and was reverted), the link thread
  5,568, the control owner is unchanged at 10,256, and no realignment callers
  exist.
- [x] Emit simulated device records through the core, check them with a Gate-rule
  host peer (`serial_output`), and require it in `framed-serial-link` coverage.
  The simulated device acknowledges Hello, heartbeats, credits and replies (or
  rejects) through `SerialRecordWriter`; device heartbeats are replayed with host
  heartbeats at their original instants, and queued heartbeats now precede a
  long request's reply. Every controller scenario, including heartbeat-loss,
  reports a passing `serial_output`; a missing or failed one keeps
  `framed-serial-link` unsupported or failed.
- [x] Run the affected Bazel and ordered Cargo checks, firmware build/package,
  guest build, native audits, host-backend runs and repository gates; commit and
  push.

Authorization: repository source, tests, fixtures, documentation, build,
package, static ELF audits, host-backend virtual runs, commit and push only. No
detector, USB/device session, flash, monitor, QEMU run, emulator effect gate,
credentials, network discovery, mining or other hardware effect. No parity
promotion. The spilled-call-target audit fix belongs to a separate session; this
increment does not edit `scripts/virtual-emulator/noise-stack-audit.mjs` or
`scripts/link-stack-audit.mjs`.

Verification: ordered Cargo fmt/Clippy (`-D warnings`)/build/tests; all 328
Bazel tests (`just test`); 22 writer and 21 link characterization transcripts
unchanged; `:serial_writer_tests` Gate device vectors; 53 simulation unit tests
including the Gate peer, and the 35-seed scenario corpus with no failed checks;
`just build`, `just virtual-emulator build` (compile and guest frame audit only:
992/4,208 bytes, unchanged); writer, link, control and realignment native audits
plus the bootstrap writer audit on the new ELF; `just verify-native-usb-ownership`;
host-backend `just virtual-board run` for healthy-lifecycle, heartbeat-loss,
max-status, replay-challenge and cancel-preparation with passing `serial_link`
and `serial_output`; redaction, reference, parity and Bright Builds checks.

Completion review: increment 2 is complete. Firmware and simulation now share
both directions of the serial link, pinned byte for byte to the previous
firmware, and the virtual corpus judges device output against the Gate's rules.
Residual risks and non-claims: the refactored firmware writer has not met
hardware; it does so only in a later, separately contracted hardware task. The
writer thread is 80 bytes deeper (5,616 of 6,144). QEMU host/target agreement
and the target main-stack margin were not rerun. The simulation does not revoke
the link after a rejection reply as the firmware owner does, models no partial
output or diagnostics schedule, and its Rust peer is an interim stand-in for the
pinned Gate. `gate-page-http-seams` stays unsupported and qualification stays
false. Next: increment 3 (stdio virtual device driven by the pinned Gate
controller).

Framed-link integration, increment 3 | 2026-10-11 | Owner-approved plan

Objective: drive a virtual device with the real pinned Gate controller. A
`tools/virtual-board serve-serial` child speaks `bwg-worker-serial/0.2` on
stdin/stdout through the shared link and writer cores and the simulated
`WorkerControl`; the pinned Gate TypeScript controller (Bun, through its
injectable serial runtime) admits it end to end: Hello, signed capability
`discover`, `prove_possession`, heartbeat, baseline `status`, a further
`status()` and a clean `close()`. Lease flows, the Gate page and the loopback
HTTP supervisor stay out of scope; `gate-page-http-seams` stays unsupported.

- [x] Add a real-time `StdioDevice` to `bitaxe-simulation` that serves the
  firmware's signed capability, the real serial manifest digest and no
  simulation status evidence, and mirrors the firmware owner's reply,
  rejection and revocation handling. Eight in-crate tests drive it with real
  host records and the Gate-rule peer: clean admission and Close, accepted
  possession, baseline status without simulation evidence, device heartbeat
  liveness, rejection then revocation, EOF without Close, and a refused second
  Hello.
- [x] Add `serve-serial` to `tools/virtual-board` with a real-process test.
  `serve-serial --seed=N --capability=PATH [--outcome=PATH]` steps the device
  every 10 ms on the wall clock, writes only protocol bytes to stdout, and on
  stdin EOF writes a new mode-0600 outcome (categories, counts, capability
  digest) and exits 0 only for a clean session.
- [x] Add the Bun Gate driver with process-group ownership and tests for
  admission, an untrusted capability and device loss.
  `//scripts:virtual_board_gate_serial_test` runs the pinned Gate's
  `createWebSerialWorkerController` (real-time injected runtime, in-memory
  continuity, firmware deployment-trust update keys) over a `WorkerSerialPort`
  on a detached `serve-serial` child. The Gate admits the device (ready,
  baseline status, clean close; device outcome clean, exit 0, group released),
  refuses the firmware capability under the Gate fixture trust (only `discover`
  answered), and reports a killed device with a transport-loss category.
- [x] Add a required `gate-controller-serial` coverage row, update the docs, run
  the affected Bazel, ordered Cargo and repository checks; commit and push.

Authorization: repository source, tests, documentation, build, host processes
(the virtual device child and Bun running the pinned Gate code), commit and
push only. No hardware, USB, Web Serial, browser, network, QEMU run, credentials
or parity promotion.

Verification: ordered Cargo fmt/Clippy (`-D warnings`)/build/tests including the
`StdioDevice` unit tests and `serve-serial` real-process tests; all 329 Bazel
tests (`just test`) including `//scripts:virtual_board_gate_serial_test` (three
cases against the pinned Gate, each under a second); `just build`,
`just package`, `just virtual-emulator build` (compile and guest frame audit
only: 992/4,208 bytes, unchanged); host-backend `just virtual-board run` for
healthy-lifecycle (unsupported only for its existing strict-share non-claim)
and heartbeat-loss (passed); redaction, reference, parity and Bright Builds
checks.

Completion review: increment 3 is complete. The pinned Gate's own serial
controller admits, queries and cleanly closes a real-time virtual device that
runs the shared firmware link and writer cores, refuses it under a trust that
did not sign its capability, and reports its loss. Residual risks and
non-claims: Bun comes from the host PATH, as in the existing Gate fixtures, so
the test is not hermetic; no lease flow (Start/Renew needs a signing authority)
crosses the Gate; one logical session per device process; the tests run on the
wall clock with generous bounds. The device's possession is self-asserted on
initial admission, as the Gate allows. `gate-page-http-seams` stays unsupported
and qualification stays false. Next: increment 4 (the Gate page and loopback
HTTP supervisor over this device).

Framed-link integration, increment 4 | 2026-10-11 | Owner-approved plan

Objective: exercise the Gate page and loopback HTTP seams. The pinned acceptance
page, built from the pinned Gate sources and served by the unmodified production
no-mining supervisor (`createNoMiningSupervisor`) on an ephemeral loopback port,
runs from the served bytes under Bun with browser globals shimmed and drives the
real-time virtual device: configure, a trusted Connect click (Hello, signed
`discover`, possession, heartbeat, baseline status), a refresh and Close, with
every published state recorded through the supervisor's own client and judge.
Owner decision: this no-lease run flips `gate-page-http-seams` to passed, with
the non-claims recorded below.

- [x] Add the page driver and test: browser shims, the pinned bundle and page
  served by the real supervisor, and cases for admission, an untrusted
  capability, device loss, a foreign origin and privacy.
- [x] Set `gate-page-http-seams` from the test, update docs and the close-rejection
  non-claim, run the affected and full checks; commit and push. Under full-suite
  load the driver's refresh raced the page's own connect-time status (the Gate
  correctly refused it with `operation_active`); the driver now waits on the
  page's published state, and both Gate tests pass 8 of 8 under 8-way parallel
  runs.

Authorization: repository source, tests, documentation, build, host processes
(the device child, the Bun page driver, a loopback supervisor on an ephemeral
port), commit and push only. No browser, Web Serial, hardware, USB, network
beyond loopback, port 48765 or the owner's Gate tab, QEMU run, credentials or
parity promotion.

Verification: ordered Cargo fmt/Clippy (`-D warnings`)/build/tests; all 330
Bazel tests (`just test`) including `//scripts:virtual_board_gate_page_test`
(five cases, built from the pinned archive's sources) and the unchanged
no-mining supervisor tests; both Gate tests 8 of 8 under 8-way parallel runs;
`just build`, `just package`, `just virtual-emulator build` (compile and guest
frame audit only: 992/4,208 bytes, unchanged); redaction, reference, parity and
Bright Builds checks.

Completion review: increment 4 is complete and `gate-page-http-seams` is set
from a real run: the served pinned page and the unmodified no-mining supervisor
admit, refresh and close the virtual device, record every published state
through the supervisor's judge, record an untrusted capability and device loss,
and refuse a foreign origin. Non-claims: no lease flow (Start, Renew, Restore)
crosses the page, supervisor or Gate, so `physical_host_owner_release` stays
unsupported (now detailed as `lease_restore_owner_release_not_integrated`) and
close-rejection rows still block qualification; the browser is shimmed in Bun,
not Chromium; Bun comes from the host PATH; the supervisor context's package
fields are synthetic and its frozen-source check is not applicable to the
virtual device; port 48765 and the owner's Gate tab were never used.
Qualification stays false.

Framed-link integration, increment 5 | 2026-10-11 | Plan recorded at start

Objective: carry lease flows through the Gate. A throwaway lease authority from
the pinned Gate's development-authority script (generated in a private temporary
directory, never the owner's) signs a qualification-attempt Start and its
renewals; the served acceptance page, a loopback lease supervisor built from the
production signing and judging modules, and the Gate controller drive the
virtual device through Start, Renew, Stop and Close, and through Close with an
active lease.

- [x] Device: a fresh revocation generation per Hello on the real-time device;
  a runtime deployment trust (`serve-serial --trust`); renewal; Gate-valid
  `worker-qualification-v1` evidence; link and writer service while the owner
  actuates; lease outcome fields; tests over a scripted port.
- [x] Gate harness: the throwaway authority and virtual deployment trust, the
  lease supervisor, page and controller lease cases.
- [x] Coverage row `gate-lease-flows`, docs and non-claims; affected and full
  checks; commit and push.

Authorization: repository source, tests, documentation, build, host processes
(the device child, Bun page and signer processes, a loopback supervisor on an
ephemeral port), commit and push only. No browser, Web Serial, hardware, USB,
network beyond loopback, port 48765 or the owner's Gate tab, QEMU run, owner
authority keys, credentials or parity promotion.

Verification: ordered Cargo fmt/Clippy (`-D warnings`)/build/tests, including 18
real-time device tests over a scripted port (nine lease cases: Start with the link
live through preparation, a host silent during preparation, running evidence,
renewal, restore, Close with a lease, Close during preparation, an untrusted
authority); all 330 Bazel tests (`just test`) with
`//scripts:virtual_board_gate_serial_test` (six cases, three with leases) and
`//scripts:virtual_board_gate_page_test` (eight cases, three with leases); both
Gate tests four runs each at eight parallel jobs, all passing; the simulation
scenario corpus passing; `just build`, `just package`, `just virtual-emulator
build` (compile and guest frame audit only: 992/4,208 bytes, unchanged);
redaction, reference, parity and Bright Builds checks.

Completion review: lease flows cross the Gate. Under a throwaway authority the
pinned Gate controller starts, renews and restores a signed qualification attempt
and closes with one active; the served pinned page, behind a test-only loopback
lease supervisor that reuses the production `signAttempt`, client and `/record`
judge, prepares, loads, starts, renews on its own cadence, stops and closes, or
closes with the lease active; every published state passes the judge, and the
device restores cleanly each time. `gate-lease-flows` is a required coverage row
set from both Gate tests. Non-claims: the lease supervisor has no campaign
lineage, preflight, pool input or cooling and budget reviews, so the production
supervisors' preflight is not exercised; the device's owner stack headroom is
modeled, its watchdog is reported absent and it dispatches no work;
`physical_host_owner_release` stays unsupported (`rejected_lease_restore_not_integrated`)
and close-rejection still blocks qualification; the browser is shimmed in Bun; Bun
comes from the host PATH; port 48765 and the owner's Gate tab were never used.
Qualification stays false.

Framed-link integration, increment 6 | 2026-10-11 | Owner-approved scope

Objective: the close-rejection owner release, through the Gate. Owner decisions:
cover both close rejections, and move the physical claim from the in-process
close-rejection scenario (which keeps its independent-restoration check) to a
required coverage row set from the Gate tests. Under a lease, (a) the device
rejects the close-time restore (its cooling proof fails once): the Gate still
closes its port and releases ownership, records the failure through the page
and supervisor, and the device restores itself once the link drops; (b) the
native port close is rejected: the Gate correctly retains its port ownership,
the device restores independently, and ownership is released when the page
unloads.

- [x] Device: `serve-serial --fault=reject-restore` fails the first restoration
  of a started lease at the cooling-proof step; outcome counts rejected
  restorations; tests over a scripted port.
- [x] Gate controller and page cases for both rejections, with lock, ownership,
  supervisor-record and device-restoration assertions. The harness port now
  models a serial line: reader cancel and writer close reach the device only
  through the native port close, which the earlier cases still pass under.
- [x] Coverage row `gate-close-rejection-release`; drop the unsupported
  `physical_host_owner_release` scenario check; docs and non-claims; affected
  and full checks; commit and push.

Authorization: as increment 5 (repository source, tests, documentation, build,
host processes and a loopback supervisor, commit and push only).

Verification: ordered Cargo fmt/Clippy (`-D warnings`)/build/tests, including 20
real-time device tests (two new: a rejected restore is answered, and the device
restores without the host afterwards); all 330 Bazel tests (`just test`) with
`//scripts:virtual_board_gate_serial_test` (eight cases) and
`//scripts:virtual_board_gate_page_test` (ten cases); both Gate tests three runs
each at eight parallel jobs, all passing; `just build`, `just package`, `just
virtual-emulator build` (compile and guest frame audit only: 992/4,208 bytes,
unchanged); redaction, reference, parity and Bright Builds checks.

Completion review: the close-rejection owner release crosses the Gate. When the
device rejects the close-time restore under a lease, the controller and the page
still close the port and release ownership and the lock, the page records
`close_failed` through the supervisor, and the device restores itself after the
link is gone (one rejected and one completed restoration, no pending lease). When
the native port close is rejected, the Gate keeps ownership and its lock, the
page records `close_failed` with ownership retained, and the device, restored by
the close-time restore, ends cleanly once the page unloads. The physical claim is
the required `gate-close-rejection-release` coverage row; the in-process scenario
keeps its independent-restoration check. Non-claims: both rejections are
injected (one cooling-proof failure, a harness port refusal), page unload is
modeled by ending the device's input, and the increment 5 non-claims stand.
Qualification stays false for the remaining parent blockers.

Framed-link integration, increment 7 | 2026-10-11 | Owner-approved scope

Objective: real allocation interception. Owner decisions: type the three real
allocation classes at the receive-envelope parse, the V2 record snapshot and the
outbound envelope encode; fix fault provenance now so a recovered allocation
failure never claims the once-per-boot fatal slot; and make the claim a required
coverage row set from host tests with a capability-aware test allocator, with the
in-process scenarios dropping their always-unsupported "actual allocator" checks.

- [x] Seam and provenance: `bitaxe_core::fallible` marks recoverable allocation
  regions by site; firmware installs hooks that open a per-task recoverable region
  in `bitaxe_fault_provenance`, whose failure callback flags such records in the
  history ring and leaves the legacy first-failure slot to fatal failures; every
  production `try_reserve` site in firmware and its shared crates uses the seam.
- [x] Typed sites: the link receiver parses envelopes without an infallible
  payload copy and reports frame-buffer exhaustion as a typed link failure;
  `V2Record` snapshots fallibly; `SerialEnvelope::encode` writes through a bounded,
  fallible buffer. Stack audits and characterization transcripts re-run.
- [x] Interception: a capability-aware host test allocator (firmware routes these
  allocations PSRAM-first) fails real allocations by site through the production
  code and the virtual device; `bounded-real-allocation-paths` is set from it; the
  board model's default routing matches firmware; docs and non-claims; affected
  and full checks; commit and push.

Authorization: repository source, tests, documentation, build and host processes,
commit and push only. No hardware, USB, flashing, QEMU run, credentials or parity
promotion; firmware changes are build- and host-verified only.

Verification: ordered Cargo fmt/Clippy (`-D warnings`)/build/tests; all 331 Bazel
tests (`just test`) including the new `//crates/bitaxe-simulation:real_allocation_tests`
(12 cases: allocator routing and fallback, the frame buffer, V2 snapshot and encode
sites, and the virtual device ending typed on control-copy, frame-copy and encode
exhaustion and surviving PSRAM-only exhaustion), the C provenance model (three new
cases), the core-dump provenance validator (two new cases) and the native binary
audit fixture; `just build`, `just package`; native audits on the built ELF: link
5,248 and writer 5,392 bytes (down from 5,568 and 5,616), control 10,256 unchanged,
no runtime realignment callers, fault provenance passing with the recoverable entry
points in its IRAM closure (192 added bytes, one callback CAS); `just
virtual-emulator build` (compile and guest frame audit only: 992/4,208 bytes,
unchanged); redaction, reference, parity and Bright Builds checks. One Cargo test
run, made while a full Bazel suite ran concurrently, counted failure lines that
three later runs did not reproduce; the recorded runs pass.

Completion review: real allocation interception is in place and the fatal
provenance receipt is protected. Every production reservation runs in a named
recoverable region; on firmware the recorder flags those failures in its history
and leaves the first-failure receipt to unhandled ones. The receive envelope is
parsed without a payload copy, the V2 snapshot and the outbound encode allocate
only recoverably, and receive-buffer exhaustion is the new pinned link failure
`frame_allocation`. A capability-aware test allocator, routing as firmware does,
fails these real sites typed, including inside the running virtual device. The
in-process scenarios dropped their always-unsupported actual-allocator checks and
`bounded-real-allocation-paths` is set from the real tests. Non-claims: host
allocator faults, not ESP-IDF TLSF behaviour; V2 status `to_value`, controller
reply serialization, Hello strings and other unmarked allocations still abort on
exhaustion; firmware recoverable marking is not hardware-verified. The parent's
named-phase allocation profile item stays open. Qualification stays false.

### task-parity-bap-settings-effects | 2026-10-15 | Apply BAP settings and close the remaining BAP gaps

Origin: the BAP ordinal 009 review. BAP-001 is `verified` with these as
explicit non-claims.

- [x] Settings: upstream `BAP_handle_settings` applies frequency, ASIC voltage,
      Wi-Fi SSID and password, fan speed, auto fan, found block and show new
      block, and acknowledges each (fan speed silently). Done in software
      (2026-10-16): each intent now reaches the existing owners, and setting
      values parse like the reference's `atoi`/`atof` and `uint16_t` casts.
- [x] Several subscriptions due on one tick are published in subscription
      order, while the reference uses `BAP_PARAM` order
      (`firmware/bitaxe/src/bap_runtime.rs`). Subscriptions are now kept in
      `BapParameter::ALL` order, with a regression.
- [x] Check `wifi_rssi` while disconnected: the reference sends `-128`.
      Done in `task-parity-wifi-rssi-live` (`34465167`). Observed live in
      soak 006.
- [x] Report the boot-ordinal transition kind (initialized vs reinitialized
      after an invalid record), or pin the RTC record's address, so the
      ordinal can be compared across builds. System info now reports
      `bootOrdinalTransition` (`initialized`, `incremented`,
      `reinitialized_corrupt`, `reinitialized_overflow`) beside
      `bootOrdinal`.
- [ ] A settings hardware contract (moved from
      `task-parity-bap-live-accessory-verification`): electrical safety,
      allowed settings, restoration of changed settings, evidence and stop
      conditions. Each setting needs the owner's go-ahead.

Started 2026-10-16. Software only; no hardware effect without a settings
hardware contract.

Settings findings (2026-10-16):
- The HTTP settings PATCH path validates, persists and reloads settings
  through `FirmwareSettingsAdapter`.
- Frequency and core voltage persisted that way are display-only. Mining uses
  the signed Work Lease's fixed hardware profile, and nothing applies NVS
  frequency or voltage to the ASIC. Upstream applies BAP frequency live, and
  its power task re-applies NVS changes.
- Fan settings are applied live by `fan_controller_runtime`.
- Wi-Fi settings persist without a restart. The restart owner is in
  `http_api/deferred_effects.rs`.
- `block_found` and `show_new_block` are command-visible RAM state.
- The BAP owner has 832 bytes of stack headroom, so it cannot run
  persistence itself.

Plan (2026-10-16, after the owner's decisions):
- Route each intent through the existing owners, as an HTTP settings PATCH
  would:
  - frequency, core voltage, fan speed (auto fan off plus manual speed) and
    auto fan are persisted;
  - Wi-Fi SSID and password are persisted, and the device restarts as
    upstream does (owner decision 2026-10-16: allow, like upstream);
  - `found_block` and `show_new_block` are command-visible RAM state.
- Frequency and voltage stay display-only until ADR-0037's
  `task-live-hardware-settings-owner` applies them live (owner direction
  2026-10-16). Record that divergence until then.
- Parse values the way the reference's `atoi`/`atof` and `uint16_t` casts do,
  in the pure core, with tests.
- Run persistence off the BAP owner's thread, which has 720 bytes of stack
  headroom.
  - The work runs on the HTTP server task through `httpd_queue_work`. That
    task already persists PATCHes on its audited 16 KiB stack, so the change
    adds no RAM.
  - The BAP thread waits on a `bitaxe_runtime::reply`. Firmware never uses
    `std` channels, because of the Xtensa realignment bug.
  - The HTTP stack audit now also charges the queued job.

Implementation (2026-10-16):
- Pure core:
  - `bitaxe_core::bap::{reference_atoi, reference_atof, complete_setting}`;
  - `bitaxe_api::bap_settings`, which maps settings onto PATCH writes and
    block state.
- Firmware: `http_api/accessory_settings.rs` holds the queued job, with a
  claim state. A job abandoned after the 10 s wait never runs, so a
  `set_failed` answer is final.
- The adapter sends the acknowledgement. For credentials it then waits
  100 ms, sends `STA status restarting`, and calls the HTTP restart owner,
  which waits 1,000 ms before restarting.

Divergences, recorded:
- **Frequency and voltage** are persisted only; mining uses the lease's
  profile until ADR-0037's `task-live-hardware-settings-owner`.
- **Negative `found_block`** is answered `set_failed`. The count is unsigned;
  the reference stores the negative value.
- **Frequency values:** NaN and infinity are refused (the reference would
  store NaN). Hexadecimal floats read as 0.
- **Wi-Fi limits:** the settings schema refuses an SSID outside 1–32 bytes and
  a password over 63 bytes, answering `set_failed`. The reference's BAP path
  writes them.
- **Owner unavailable:** `set_failed` is sent before the HTTP server starts,
  after a 10 s queue wait, or if ESP-IDF drops the queued control message,
  which leaks that one job.
- **Unset credentials** read as empty. That matches the factory
  `config-205.cvs`; the reference's Kconfig defaults apply only to a missing
  key.
- **Command status:** BAP block flags are recorded as a block-found dismiss.
- **Checklist text:** the BAP-001 row's non-claims still say settings get
  `set_failed`. The row is `verified`, and the parity tool forbids
  `verified` → `verified` transitions, so the correction lives here.
Verification: pending. Completion review: pending.

## Future

### task-device-config-profile-evidence-soaks | 2026-10-16 | Soak the device-configured profiles on hardware

ADR-0037 step 5. Depends on `task-live-hardware-settings-owner`. Hardware;
each soak needs its own `TASKS.md` contract.

- [ ] Soak at least:
  - upstream default (485 MHz, 1200 mV) with the automatic fan;
  - one live frequency change during a lease;
  - the catalog-maximum shutdown tail.
  Record the device configuration in evidence.
- [ ] Update the PWR and ASIC parity rows whose non-claims name "arbitrary
      profiles", within what the evidence shows.

Verification: pending. Completion review: pending.

### task-mining-baseline-after-lease | 2026-10-16 | Decide whether the Worker mines on its own after a lease

Owner decision needed; not decided by ADR-0037. In the Gate's design the
Worker returns to ordinary local mining (its Mining Baseline) after a lease.
This firmware returns to a paused baseline and mines only under a lease.

- [ ] Present the options and safety implications, including a heartbeat
      replacement for local mining, and record the owner's decision in an ADR.

Verification: pending. Completion review: pending.

### task-telemetry-descriptive-extensions | 2026-10-15 | Add descriptive, unambiguous telemetry names as post-parity extensions

Owner request (2026-10-15). Start only after the parity work is complete.

Several upstream names are ambiguous or overloaded:
- `bestDiff` (all-time, persisted) versus `bestSessionDiff` (this boot);
- `best_difficulty` on BAP (all-time only);
- `found_block` (subscription) versus `block_found` (request answer);
- `fan_speed` (RPM on BAP subscriptions, but percent in settings);
- unit-less `voltage`, `current` and `power`.

Design them as extensions alongside, never instead of, the upstream fields, so
upstream clients keep working.

- [ ] Inventory the ambiguous or overloaded fields across `/api/system/info`,
      the WebSocket telemetry, BAP and the screen, with their units and scopes
      (lifetime, boot, session, connection).
- [ ] Propose additional, explicitly named fields, for example:
      `bestNonceDifficultyAllTime`, `bestNonceDifficultyBoot`,
      `fanSpeedRpm`, `fanDutyPercent`, `inputVoltageMillivolts`. Split values
      that upstream conflates (per pool connection versus per boot). Record the
      design in an ADR.
- [ ] Keep every upstream field byte-compatible, and document the extensions
      as accepted project behaviour, not parity.

Verification: pending. Completion review: pending.

### task-cross-platform-device-session-adapters | 2026-07-22 | Qualify Linux and Windows ESP device sessions

- [ ] Implement Linux physical/enumeration identity, exclusive ownership,
  receive-only observation, and bounded reacquisition behind the canonical
  device-session contract.
- [ ] Implement the corresponding Windows adapter without weakening
  exclusive ownership, request-once, or private-artifact guarantees.
- [ ] Add platform-native real-process tests.
- [ ] Keep unsupported platforms fail-closed until each exact adapter and its
      task-gated hardware evidence qualify.

Verification: Pending.

Completion review: Pending. macOS remains the only production adapter. Standing
task authorization permits ordinary implementation and task-gated evidence;
credentials, network discovery, direct UART or pin work, and evidence promotion
remain governed by their specific contracts.

### task-str005-noise-cooperative-cancellation | 2026-09-16 | Explore bounded and cancellable Noise cryptography

Status: Deferred by the owner; future backlog outside current parity work.
Do not automatically select this dependency-engineering work while that scope
deferral remains in force. A future scope decision is required to activate it.

Origin: `task-str005-noise-runtime-readiness` and the
[source/API readiness audit](docs/hardware/str005-noise-runtime-readiness.md).
The pinned crypto calls lack supported in-call cancellation and a maximum
ElligatorSwift search length. No five-second device overrun was measured.
The five-second cleanup guarantee is a prospective qualification requirement,
not established upstream parity behavior.

Scope: Assess a supported upstream solution first. If none suffices, evaluate
a narrow, reviewed dependency patch or maintained fork and its maintenance,
cryptographic-review and provenance costs before committing to that approach.
No fork or upstream contribution is commissioned by this backlog entry.

- [ ] Reassess available upstream APIs and the required cancellation guarantee;
      document whether a supported solution avoids a maintained fork.
- [ ] If activated, bound scalar-generation retries and make the native
      ElligatorSwift search cancellable/bounded without biased fallback keys,
      partial-success results or changed wire/authentication semantics.
- [ ] Propagate cancellation through act-one creation and act-two authentication;
      preserve first failure and prove actual worker/socket cleanup and disposal
      of owned secret buffers.
- [ ] Verify real-operation cancellation, deadline boundaries, interoperability,
      invalid inputs and failure cleanup; qualify indivisible native operation
      timing, stacks and memory alongside ordinary runtime owners.
- [ ] Review dependency source/provenance and publish exact tested pins before
      handing software readiness back to `task-str005-noise-runtime-readiness`.

Boundaries: Software investigation only after activation; no implicit dependency
upgrade, fork publication, device access, flashing, mining or parity promotion.
Do not edit downloaded registry sources. The frozen successor contract and all
historical evidence remain unchanged. If a different qualification strategy is
chosen, it requires a separate reviewed contract-amendment task. The linked v2
amendment supplies that decision for current Noise qualification; this backlog
entry itself grants no effect authority.

Verification: Pending future activation. Existing characterization tests and
readiness reports establish the gap, not a completed cancellation implementation.

Scope update | 2026-09-16: The [v2 parity-scope amendment](docs/hardware/str005-noise-parity-scope-amendment.md) removes this work
from the runtime/fixture/live readiness dependencies. Keep it deferred as a
stronger future cancellation/latency guarantee. It does not block v2 Noise or
later parity work; any future adoption needs its own verified contract.

Completion review: Pending future activation. No implementation or fork has
been commissioned.

## Effectful Hardware Task Gate

Standing permission for safe USB interaction remains subject to `AGENTS.md` and
`docs/hardware/hardware-attempt-policy.md`. Before any effectful hardware run,
move or add one task block under `Active` that explicitly records:

- the exact permitted repo-owned command and objective;
- the evidence destination, privacy class, and redaction policy;
- recovery, restoration, and cleanup procedures;
- retry bounds, including the unchanged-boundary stop rule; and
- accepted terminal categories and stop conditions.

If any field is missing, hardware work is not authorized. A task entry never
expands the direct-UART, pin-manipulation, privacy, safety, or archived-lineage
boundaries in `AGENTS.md`.

## Accepted Debt and Constraints

- Milestone v1.2 is administratively closed with gaps and is not a release.
- Phase 36 stopped after 8 of 10 plans. Plans 36-07 and 36-04 did not complete.
- SYS-02, EVD-11, EVD-12, and EVD-14 remain blocked. EVD-15 is satisfied by
  exact preservation, typed demotion, and explicit non-claims.
- The sole final Phase 36 hardware attempt sealed `sealed_non_promotion`,
  produced no candidate, and left device restoration unresolved.
- Do not repeat the unchanged hardware attempt. A future attempt requires new
  diagnostic information, a targeted regression-backed fix or objectively
  verified non-invasive remediation, and a complete task-scoped hardware
  contract under the gate above.
- Administrative closure, software verification, or task completion alone is
  never hardware or parity evidence.
