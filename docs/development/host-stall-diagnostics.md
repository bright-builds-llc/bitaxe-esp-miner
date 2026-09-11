# Host stall diagnostics

Use `just diagnose-host-stalls` to distinguish command startup, first output,
program entry, execution, and cleanup delays. This command starts Node directly
so that Bazel startup and Cargo lock waits are visible. Its tests run through
Bazel. It performs no device operations.
The recipe preserves quoted arguments and suppresses command echo; it requires
Just 1.29 or newer for per-recipe positional arguments. The Node entry point can
also be invoked directly with the same options.

## Record an ordinary host command

From the repository root, create a private parent and use a fresh child for
each run. Existing children and symlinks are rejected rather than overwritten.

```sh
mkdir -m 700 scratch/my-host-diagnostics
just diagnose-host-stalls run --root "$PWD/scratch/my-host-diagnostics/build-01" --label build-01 --quiet-ms 15000 --timeout-ms 120000 -- cargo build --timings
```

Omit `--timeout-ms` to observe without an overall deadline. Quiet output triggers
diagnostics, not a failed verdict or an automatic retry. An explicit timeout or
interrupt initiates owned-process cleanup; cleanup and diagnostic completion can
extend the recorder's total duration beyond the child deadline.

The private run directory contains:

- `invocation.json`: exact argv, working directory, selected environment fields,
  PATH digest, and recorder ancestry. It never dumps the complete environment.
- `stdout.log` and `stderr.log`: at most 1 MiB each by default. Excess output is
  drained and counted, with truncation recorded.
- `capture-*.json`: quiet-triggered process metadata, bounded `lsof` output and,
  on macOS, short `sample` traces. At most two captures are made by default,
  covering at most three tracked live processes each.
- `summary.json`: UTC and monotonic event timestamps, exit status, byte counts,
  capture failures, cleanup results and final outcome.

Events distinguish `spawn_requested`, `spawned`, first stdout/stderr, `exit` and
stream `close`. The native probe additionally records `stderr_marker_observed`.
That event measures when the recorder observes the entry marker, including pipe
and scheduling delay; it is not an exact timestamp inside the child process.
Do not use total recorder duration as executable startup latency: it includes
evidence collection and cleanup.

Directories are owner-only (0700), evidence files are 0600, and the executable
probe is 0700. Keep raw evidence private: argv, process filenames and child
output may contain operational information. Share only reviewed summaries.

## Compare the same executable across execution environments

Prepare once; every launch verifies and reuses the exact executable path,
digest and working directory. Preparation invokes the installed stable Rust
compiler and records compilation separately from execution.

```sh
just diagnose-host-stalls prepare --root "$PWD/scratch/my-host-diagnostics/probe"
just diagnose-host-stalls probe --bundle scratch/my-host-diagnostics/probe --root scratch/my-host-diagnostics/agent-before --label agent-before --runs 3
```

Run the next command in the user's normal shell. For an independent application
control, use Terminal.app or iTerm2 outside ChatGPT/Codex. An embedded interactive
terminal is a useful separate condition, but still shares its hosting app.

```sh
just diagnose-host-stalls probe --bundle scratch/my-host-diagnostics/probe --root scratch/my-host-diagnostics/external-terminal --label external-terminal --runs 3
```

Then run another agent series to bracket the user series in time:

```sh
just diagnose-host-stalls probe --bundle scratch/my-host-diagnostics/probe --root scratch/my-host-diagnostics/agent-after --label agent-after --runs 3
just diagnose-host-stalls compare --left scratch/my-host-diagnostics/agent-before --right scratch/my-host-diagnostics/external-terminal --root "$PWD/scratch/my-host-diagnostics/comparison-before"
just diagnose-host-stalls compare --left scratch/my-host-diagnostics/agent-after --right scratch/my-host-diagnostics/external-terminal --root "$PWD/scratch/my-host-diagnostics/comparison-after"
```

Compare refuses different executable bytes, paths, working directories, or
deadlines, and incomplete series. Labels are caller-supplied: inspect the recorded
ancestry before interpreting them as execution environments. Successful runs
establish non-reproduction during that observation, not a fix for intermittent
stalls. Preserve failures alongside successes and use fresh evidence children.
Interrupting a probe ends the whole series, even when cleanup succeeds. An
interrupted or incomplete series cannot be reported as a complete comparison.

## Test Cargo lock amplification independently

The controlled experiment creates dependency-free Cargo fixtures outside the
repository, warms shared and separate target directories, and starts status
commands while a real Cargo build script holds the shared build lock. It never
creates synthetic lock files or clears existing caches.

```sh
DIAGNOSTIC_PARENT="$(mktemp -d)"
DIAGNOSTIC_PARENT="$(cd "$DIAGNOSTIC_PARENT" && pwd -P)"
node scripts/host-stalls/cargo-lock-experiment.mjs --root "$DIAGNOSTIC_PARENT/experiment" --cargo "$(rustup which --toolchain stable cargo)" --rustc "$(rustup which --toolchain stable rustc)"
```

The experiment alternates shared/isolated status targets twice. A trial needs
the holder marker, real Cargo lock output for the shared case, bounded startup,
no unexpected status recompilation and complete cleanup. Confounded trials do
not count as evidence for the hypothesis. This experiment uses same-host wall
timestamps for cross-process marker ordering and rejects implausible intervals;
the individual command records also retain monotonic event offsets.

Bazel's workspace-status command now uses `.bazel-workspace-status-target`,
separate from normal host Cargo output. Its Git/reference-derived identity
semantics are unchanged. This removes that particular lock coupling; it does
not explain loader or filesystem stalls and does not isolate every repository
Cargo caller. Firmware compilation already has its own `.bazel-firmware-target`.

## Compare build concurrency

The jobs experiment warms one private target directory, then alternates one and
two build jobs over the same two independent, bounded build-script workloads.
Each trial preserves its Cargo timing HTML and actual build-script intervals.

```sh
DIAGNOSTIC_PARENT="$(mktemp -d)"
DIAGNOSTIC_PARENT="$(cd "$DIAGNOSTIC_PARENT" && pwd -P)"
node scripts/host-stalls/cargo-jobs-experiment.mjs --root "$DIAGNOSTIC_PARENT/experiment" --cargo "$(rustup which --toolchain stable cargo)" --rustc "$(rustup which --toolchain stable rustc)"
```

Expected serial execution with one job and overlapping execution with two jobs
demonstrate that this control is working. They do not establish a connection to
intermittent loader stalls or justify reducing build concurrency globally.

## Interpret and escalate

| Observation                                                             | Supported interpretation                                                              |
| ----------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| `_dyld_start` before an entry marker or test frame                      | Executable startup has not reached the observed application boundary.                 |
| `dlopen` / `fcntl` during a proc-macro load                             | The compiler is waiting in the loader; the responsible OS mechanism remains unproven. |
| `__getdirentries64` during cleanup                                      | Directory enumeration is taking time.                                                 |
| Cargo reports waiting for a build-directory lock                        | Another Cargo owner is delaying this command.                                         |
| Child timestamps show prompt completion but the tool returns much later | Investigate result delivery separately from child execution.                          |

Change one variable per experiment. Use `--timings` for Cargo build scheduling;
it does not replace process traces or measure test-body execution. If repeated
samples identify file operations, capture a short, process-filtered `fs_usage`
trace while the same verified process remains alive. That escalation may require
local privileges; the recorder does not elevate, disable security services,
change signing, clear caches, or reboot the host.

Process discovery is polled, so very short-lived descendants or deliberate
daemonization may escape observation. PID/start-time/group checks on macOS are
not atomic. The recorder reports unanchored groups or incomplete cleanup rather
than guessing which processes to signal. It is for supervised foreground host
commands, not hardware controllers or commands intended to leave services alive.
An already-running Bazel server and its subprocesses are not descendants of its
new CLI client. The recorder does not adopt or terminate that shared server.
Use direct Cargo reproductions for compiler capture, and separately attribute
any diagnostic-only server sampling before investigating server-owned work.

Sources: [Apple's slow-operation diagnostics](https://developer.apple.com/library/archive/documentation/Performance/Conceptual/CodeSpeed/Articles/DiagnosingSlowness.html),
[filesystem tracing](https://developer.apple.com/library/archive/documentation/Performance/Conceptual/FileSystem/Articles/FileSystemCalls.html),
[Cargo build timings](https://doc.rust-lang.org/stable/cargo/reference/timings.html),
and [Cargo build-cache configuration](https://doc.rust-lang.org/cargo/reference/build-cache.html).
The command surface follows [Just's positional-argument contract](https://just.systems/man/en/positional-arguments.html).
