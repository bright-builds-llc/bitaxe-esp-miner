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

Current unsupported coverage includes encrypted fixture/share/ACK integration,
real Gate/page/HTTP seams, capability-specific interception of real allocation
paths, retained V2 status serialization, and several composed timing/recovery/
process-ownership scenarios. The active task records exact target execution
failures and remaining blockers. Models and simulator passes cannot establish
physical electrical, RF, USB timing or hardware parity. Parity remains **90/95**.
