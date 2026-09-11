# Host stall investigation: 2026-09-10

The investigation reproduced Cargo build-lock amplification and removed that
coupling from Bazel workspace status. It did not reproduce or identify the cause
of the earlier intermittent native-loader and filesystem stalls.

## Identical executable across execution paths

A dependency-free Rust program emits a fixed marker at main entry. Its executable
SHA-256 was `c08cf3091085fefcf0a3b7ef39d12f2d4efe222e3774749171dee246580b1479`.
All series used the same bytes, executable path, working directory, 15-second
overall deadline and two-second quiet trigger. Each had three successful runs
and complete cleanup.

| Execution path, in observation order     | Entry marker observed after spawn request (ms) |
| ---------------------------------------- | ---------------------------------------------- |
| Agent, initial series                    | 136.820, 1.868, 1.891                          |
| User interactive shell hosted by ChatGPT | 2.030, 1.890, 1.723                            |
| Agent after the interactive shell        | 2.226, 2.004, 1.983                            |
| External Terminal.app                    | 2.113, 1.766, 1.846                            |
| Agent after Terminal.app                 | 2.118, 1.973, 1.788                            |

The user performed the interactive and external shell runs. Recorded ancestry
distinguishes `zsh → ChatGPT` from `login → Terminal → launchd`, while agent runs
include the `codex` process. The external control was not inferred from its label.

All 15 launches passed. The first observation took 137 ms; later observations
were approximately two milliseconds. This is non-reproduction during the sampled
period, not proof that the original stall is fixed or that one execution path
is always faster. The marker measures observation by the recorder, including
pipe delivery and scheduling, rather than the exact instant inside `main`.

## Real Cargo lock experiment

A bounded build script held a real Cargo build-directory lock for five seconds.
Both status target directories were warmed before trials. Four alternating
trials changed only the status command's target directory, retaining explicit
installed compilers, offline dependency-free fixtures and the same workload.

| Condition       | Initial experiment entry delays | Final validation entry delays | Evidence                                                                          |
| --------------- | ------------------------------- | ----------------------------- | --------------------------------------------------------------------------------- |
| Shared target   | 5,216 / 5,181 ms                | 5,229 / 5,190 ms              | Cargo reported waiting for the build-directory lock.                              |
| Separate target | 181 / 150 ms                    | 190 / 152 ms                  | Status entered while the holder still ran; no lock message or unexpected rebuild. |

All commands completed and cleaned up. Actual repository workspace-status output
was byte-identical with default and dedicated target directories against the
same dirty checkout: SHA-256
`3bc85a1ac1d39b22a506b869bac72bda9fc66c858b6de7e3eb1050d0b69ee23f`.
The cold dedicated-target build took 8.235 seconds versus 1.118 seconds for the
existing target; that comparison includes compilation and is not a warm-start
performance claim.

Bazel now uses `.bazel-workspace-status-target`, which is ignored by Git. The
Git/reference-derived identity logic is unchanged. This isolates that status
subprocess from normal host Cargo checks. Other Cargo callers may still share
targets, and actual firmware compilation already uses `.bazel-firmware-target`.

## Build concurrency control

Two independent half-second build-script workloads were run with one and two
jobs, alternating twice over the same warmed private target directory. Four
Cargo timing HTML reports were retained.

| Setting  | Observed intervals             | Total command duration |
| -------- | ------------------------------ | ---------------------- |
| One job  | Serial, with 135 / 153 ms gaps | 1.762 / 1.833 s        |
| Two jobs | 412 / 414 ms overlap           | 1.166 / 1.168 s        |

All trials passed and cleaned up. No native stall appeared. The control confirms
normal scheduling behavior and gives no basis for globally forcing one job.
It used one and two jobs over small warmed fixtures; it did not recreate a cold,
fully parallel build of the entire project or rule out load-dependent stalls.

## Recorder verification and limits

Actual subprocess tests cover quiet success, delayed and fragmented markers,
output caps, nonzero and spawn failures, cancellation, descendant cleanup,
private files, diagnostic failure and reused process-group rejection. Synthetic
ownership tests exposed and fixed a case where an escaped descendant could
otherwise cause a reused group to be adopted on a later cleanup pass.

A real three-second `/bin/sleep` capture succeeded with one quiet-triggered
snapshot, successful `lsof` and `sample`, no recorder failure and complete cleanup.
Quietness did not turn the successful command into a timeout.

The first full canonical test run exposed a recorder defect under parallel load:
long executable paths made the global process snapshot exceed its output cap.
Cleanup failed closed, but retained process/pipe handles kept the test runner
alive after its failure summary was written. The original failure and attributed
external cleanup are preserved. The collector now takes bounded compact global
metadata and obtains names only for selected processes. Unproved cleanup releases
recorder-owned handles without expanding authority to signal unknown processes.
This failure was in the new recorder, not evidence of the original loader stall.

Final review also added guards against continuing a cancelled probe series,
accepting missing comparison conditions, and exposing private executable names
in public spawn-error events. The paired measurements above preceded these
diagnostic-path corrections; their original records remain unchanged.

The recorder does not adopt a preexisting shared Bazel server. Process identity
checks are not atomic on macOS, and short-lived or deliberately daemonized
descendants may evade polling. Unanchored groups and incomplete cleanup are
reported instead of authorizing speculative signals. No security configuration,
test deadline, device deadline or host service was changed. No hardware or
credential operations occurred.

Private evidence is retained under `scratch/host-stalls-20260910`, including
original series, comparisons, quiet capture, experiment logs, fixture markers
and timing HTMLs. The copied Cargo evidence includes a digest manifest. Key
experiment report digests are:

- Final lock validation: `d22c5406ab90dd56af4edd80c041fa7f42231bfebe6293432e2c73924b241820`.
- Jobs comparison: `fe6e0122b3cd89503457ffd65d9c83cfaf7b27a3fc3e25559cba264b85f96a1c`.

Use the [diagnostic commands](host-stall-diagnostics.md) during the next naturally
occurring stall. Escalate to a short, attributed filesystem trace only if the
captured process samples justify it; do not infer a host-wide failure from
agent-only timing.
