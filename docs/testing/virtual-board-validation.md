# Ultra 205 local validation

The new functional model and shared runtime are an **unqualified development
foundation**. Full-board qualification and mandatory pre-flash rollout remain
blocked until all required integration profiles pass. No virtual result grants
hardware authority, changes parity, repairs a predecessor seal or enables mining.
The active record is `task-ultra205-virtual-board-validation` in `TASKS.md`.

## Commands

The repository command surface routes through Bazel:

```sh
just virtual-board run --scenario stale-safety-step5 --backend host --seed 205 --evidence-dir scratch/virtual-board/new-host
just virtual-board run --scenario stale-safety-step5 --backend qemu --seed 205 --manifest scratch/virtual-board/guest/virtual-package.json --evidence-dir scratch/virtual-board/new-target
just qualify-virtual-board --evidence-dir scratch/virtual-board/new-qualification
just preflash-validate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --evidence-dir scratch/virtual-board/new-preflash
```

Parents must already exist; execution roots must be new. Host exploration may
omit a production manifest. Qualification builds a canonical package unless an
explicit package is supplied; preflash always builds first. Guest packages bind
their distinct ELF and configuration to the paired physical package. Emulator
installation and doctor are documented in [virtual-emulator.md](virtual-emulator.md).

Host runs use `bitaxe_virtual_board_run_v3` and include controller, fixture and
Cargo lock identities. Earlier run-v1/v2 evidence remains unchanged. The compiler embeds its declared
input closure; a stale executable cannot attest to a changed checkout.
`result.json` and `validation-report.json` distinguish failed, passed and
unsupported checks. Non-passing executions return a nonzero exit status after
saving available facts. Logs, complete target flash copies, raw cores and debugger
output remain private in ignored roots. Package freezing is withheld on any
required failure, unsupported coverage or changed input. Commands never flash.

## Shared code and interfaces

`bitaxe-runtime` contains the production atomic authority gate, typed mining
preparation, fan freshness/safety decisions and ordered shutdown bounds. Physical
firmware reexports those implementations and retains its original global gate,
cutoff, stack sizes, limits and size-oriented release optimization.

`bitaxe-virtual-board` supplies register observations, modeled dynamics,
persistence and byte exchanges. Its independent endpoint checks production ASIC
packets and reconstructed nonce headers. It cannot sign grants or decide whether
unsafe mining should start. Unknown operations return an explicit unsupported
error. Its [model notes](../../crates/bitaxe-virtual-board/README.md) identify
uncalibrated curves and register coverage limits.

`bitaxe-simulation` composes the actual controller, Ed25519 verification,
possession, ledger, safety checks and runtime with synthetic trust and board I/O.
Host and guest call the same scenario function. A fixed seed corpus plus an
expanded reproducible sweep checks deterministic complete results. The three
step-5 scenarios distinguish stale observations, zero fan proof and other unsafe
readings; none identifies the cause of the retained physical status001 failure.

Model capability budgets and actual target allocator measurements are separate.
The retained 3,451-byte free / 2,176-byte largest-block observation is a pressure
profile, not a reconstruction of the device free-list. Default routing constants
match the retained 2,048-byte threshold and 98,304-byte reserve; each candidate
must independently bind its resolved SDK configuration.

## Qualification and rollout

The central catalog in `scripts/virtual-board/profiles.mjs` selects all scenarios
for unknown changes. Both backends, every fixed seed and all supplemental
integration checks are mandatory for full qualification. Missing or duplicate
results, unsupported checks and unfrozen source block qualification.

The lifecycle runner preserves the first failure while collecting independent
facts and performing bounded cleanup. Resource release and natural completion
have separate conclusions. Historical retained-resource proof defaults false.
A late completion cannot publish a result through an expired stage token.

Virtual manifests and marked ELFs are rejected by physical flash admission.
Explicit virtual manifests fail before capture preparation, physical discovery
and sensitive-input loading. The matching-report requirement for updates and
live qualification is a pending rollout milestone; it must not be represented
as active enforcement before complete qualification. Emergency Stop, Close and
failure-only recovery continue through their existing independent contracts.

Host scenarios reach the Controller only through the production inbound serial
link core (`bitaxe_worker_control::serial::link`, shared with firmware
`bwg_worker_usb/link.rs`): real `bwg-worker-serial/0.2` Hello, sequenced Control
records, replayed one-second heartbeats and a clean Close. Device output crosses
the production writer core (`bitaxe_worker_control::serial::writer`, shared with
firmware `bwg_worker_usb/writer.rs`): the hello_ack, device heartbeats, receive and
terminal credit, and control replies or rejections. A Rust host peer applies the
pinned Gate rules to that output (`serial_output`). The required
`framed-serial-link` coverage row passes only when every host link and output check
passes. The Rust peer is a fast in-process stand-in.

The pinned Gate serial controller itself (`bitaxe-turnstile-system`, run by Bun
through its injectable serial runtime) drives a real-time virtual device process,
`tools/virtual-board serve-serial`, over the child's stdio: Hello, signed
capability `discover`, `prove_possession`, heartbeat, baseline `status` and a
clean `close()`. The device serves the firmware's signed capability and the Gate
trusts the firmware deployment-trust update keys. The required
`gate-controller-serial` coverage row passes only when
`//scripts:virtual_board_gate_serial_test` passes.

The pinned acceptance page itself, built from the pinned Gate sources and served
by the unmodified no-mining supervisor on an ephemeral loopback port, runs from
the served bytes under Bun with browser globals shimmed. A trusted Connect click
admits the device through the page's own browser serial runtime; a refresh and
Close follow, and every published state passes the supervisor's `/record` judge.
`gate-page-http-seams` passes only when `//scripts:virtual_board_gate_page_test`
passes.

Leases cross the same seams. Each test run generates a throwaway lease authority
with the pinned Gate's `worker-development-authority.ts init` in a private
temporary directory; no existing key is read. The virtual deployment trust pairs
the firmware's update authority, so the real signed capability still verifies,
with that throwaway work-lease authority, and the device verifies leases with it
(`serve-serial --trust`), as firmware does with the trust it compiles in. The
production `signAttempt` composition signs one normal qualification attempt per
control session. The Gate controller starts, renews and restores it, and closes
with a lease active; the served page, behind a test-only loopback lease
supervisor that reuses the production client and `/record` judge, prepares,
loads and starts the window, renews on its own cadence, then stops and closes,
or closes with the lease active. The device begins a revocation generation per
Hello, renews by extending the gate's lease deadline, reports Gate-shaped
`worker-qualification-v1` evidence, and services its link and writer at board
time while the owner prepares and shuts down, as the firmware link and writer
threads do. `gate-lease-flows` passes only when both Gate tests pass.

Non-claims: the lease supervisor is test-only and has no campaign lineage,
preflight, pool input, cooling or budget review, so the production campaign and
iterative supervisors' preflight is not exercised; owner stack headroom in the
device's evidence is modeled, not measured, its watchdog is reported absent and
no work is dispatched; `physical_host_owner_release` (close-rejection) stays
unsupported because no virtual flow yet rejects a restore under a lease; the
browser is shimmed, not Chromium; Bun comes from the host PATH; port 48765 and the
owner's Gate tab are never used.

Current unsupported coverage includes encrypted fixture/share/ACK integration,
the close-rejection owner release, capability-specific interception of real allocation
paths, retained V2 status serialization, and several composed timing/recovery/
process-ownership scenarios. The active task records exact target execution
failures and remaining blockers. Models and simulator passes cannot establish
physical electrical, RF, USB timing or hardware parity. Virtual runs promote no
parity row; `docs/parity/progress.jsonl` is authoritative.
