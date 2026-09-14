# Host launch investigation — 2026-09-14

The earlier launch blockage was an asynchronous **AppleSystemPolicy execution
decision wait**, rather than a Rust build-script deadlock. The assessment reply
arrived after the process had been cancelled. Current launch probes and the
original clean package build pass without a reboot, source rewrite, re-signing,
quarantine change or security-policy change. Why the service took roughly ten
minutes to respond remains unknown; this is observed recovery, not a claim of a
permanent host repair.

## Evidence chain

The retained sample shows the dependency build-script child at `_dyld_start`
with no Rust frames; its Cargo parent waits for child output. The static
signature check passes. Those facts alone did not establish the cause.

The additional host logs correlate the exact executable, PID 78435, sampled
thread 29797763 and assessment reference 1305876:

| Event                                                                                                        | Observed local time / elapsed                                                     |
| ------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------- |
| Original launch                                                                                              | September 13, approximately 23:05:18                                              |
| AppleSystemPolicy sleep interrupted by cancellation                                                          | 23:12:29.264                                                                      |
| Late policy response identifies the same process, then reports its reference absent because the process died | 23:15:19.904; 601.405 seconds after launch and 170.640 seconds after cancellation |

The cancellation-time non-allow message is not treated as a separately completed
policy denial. Likewise, the AMFI missing-CMS/CT-signature messages also appear
in the successful launch window, so they do not establish a fatal denial.
No conclusion about XProtect, network failure, signature corruption or cache
corruption is supported by this evidence.

An allowed-cache observation was present in the initial diagnostic tool output.
It was absent in a subsequent export that included info/debug events, so it is
retained separately with tool-transcript provenance. It is not fabricated as a
line in the later exported log. Shared provenance tokens were not used to join
unrelated executables.

Protected evidence is under `scratch/host-launch-root-cause-20260914`:

- Timeline SHA-256: `c899165da4fad5240ba659afe5e8eee5aaa18f3c49bfc8203fe460665017ca06`.
- Base source manifest: `a66fb3360d6f7a8bbfd3887d82ca65e952f3ac3189bd14882d1563929f4ac66c`.
- Provenance supplement: `a6b4ec2681eb3ab6496700ca3daa49b0bd6ea9967db8f1c18276a3a544a762ce`.

## Feedback loop and recovery

The repo-owned host-stall recorder executed the previously stalled build-script
path with an explicit eight-second bound, then repeated it twice. All three
completed normally; the first child exited about 11 ms after spawn. Its SHA-256,
unchanged across today's probes and package build, is
`c1cff78386809498158c73fcb6a49417eb493a6ca53355ff9396c685c607edf1`.
Its file modification time predates the failure. `cargo +stable --version` also
completed. The host boot and application-parent session were unchanged.

The original `just package` command then completed successfully in 60.792 seconds
under the recorder. The manifest was clean and bound to published firmware
`73ad71c6c49df54120b03a243e047255115cfb6a`, ELF
`bba818221df20fe1f0427f69b6cede6bf226eb13c3bb9342c786d758f8c1181f`.
This cleared the concrete package-admission blocker and allowed qualification
to resume. No firmware behavior change was used to mask the host wait.

The timing loop is red-capable for a recurrence and records/cleans owned
processes on its explicit timeout. The transient service delay cannot be
reproduced deterministically now, so no synthetic unit test claims to repair
macOS. Quiet-output captures, exact process identity and ordinary clean-package
admission remain the diagnostic approach if it recurs. Prior failure artifacts
are unchanged; host diagnostics do not establish device cadence or mining safety.
