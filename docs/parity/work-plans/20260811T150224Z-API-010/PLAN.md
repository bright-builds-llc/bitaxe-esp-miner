# Ultra 205 boot recovery plan

- Run ID: `20260811T150224Z-API-010`
- Parity row: `API-010`
- Initial status: `implemented`
- Source commit: `1bb26b4de1a552b129b5f2cf6bf5e93305ccae80`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-ultra205-boot-recovery-attempt-010`
- Continues plan: `docs/parity/work-plans/20260805T005320Z-API-010/PLAN.md`
- Prior closure: `docs/parity/work-plans/20260805T005320Z-API-010/CLOSURE.md`

## Problem statement

The connected Ultra 205 again blinks roughly once per second, matching the
previously captured panic-reset loop. Attempt 007 proved the then-installed
package overflowed the operator-sensor task through the physical-screen
snapshot path. Commit `50287f62` replaced that oversized path with a narrow
projection, but attempts 008 and 009 could not establish an observable ROM
download session and therefore never installed the fix.

Physical access has returned and the user has freshly connected the device.
That creates a new USB enumeration context rather than an unchanged retry.
Later CFG-005 work is software-only and does not change the last confirmed
on-device package, although the new recovery package will include those
verified settings changes.

## Feedback loop and hypotheses

The tight red-capable loop is one protected `just detect-ultra205` invocation.
It exercises the exact automatic ROM-entry and synchronization boundary that
stopped attempt 009 and returns a typed terminal category without a flash
write.

Ranked hypotheses:

1. Fresh enumeration restores automatic ROM entry, so detection passes and one
   exact-package observation flash can install the screen-stack fix.
2. Automatic USB reset still cannot establish ROM download mode, so detection
   repeats the prior closed connection signature and stops before write.
3. Port ownership, accessibility, or enumeration stability has changed, so
   device-session admission reports a distinct pre-sync typed boundary.
4. Flashing succeeds but a different runtime panic remains; exact-package
   runtime observation distinguishes it from the historical screen overflow.

## Exact command and effect contract

Run only these commands, in order:

1. `just package`
2. `test ! -e scratch/ultra205-boot-recovery/wrapper-010 && (umask 077; mkdir -m 700 -p scratch/ultra205-boot-recovery/wrapper-010 && just detect-ultra205 > scratch/ultra205-boot-recovery/wrapper-010/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `just mining-campaign stage=observation board=205 port=<detector-port> manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=scratch/ultra205-boot-recovery/attempt-010 duration-seconds=360 redact-evidence=true`

The package must come from the clean pushed plan commit, bind board 205, the
pinned reference commit, exact artifact digests, and one non-dirty source
identity. The detector must find exactly one likely ESP USB serial device and
pass `espflash board-info --chip esp32s3` under the repository's typed USB
session policy. The ignored credential file is admitted only as an opaque
local input and is never printed, summarized, or committed.

The observation campaign may perform one factory-package write and one NVS
partition write containing only the local Wi-Fi settings, `mineonboot=false`,
and the observation-stage marker. This replacement can remove prior device
settings, including hostname and pool configuration. It may then observe the
same physical device through the qualified receive-only and same-origin
runtime paths for at most 360 seconds.

Mining, pool access, ASIC initialization or work, voltage, frequency, fan,
thermal, power control, OTA, erase-flash, raw writes, discovery, foreign-process
termination, fault injection, direct UART, pins, pads, headers, GPIO, probes,
jumpers, soldering, injected signals, and parity promotion are prohibited.

## Evidence, recovery, and stop rules

The wrapper and campaign roots must be absent before use, mode 0700, ignored,
and contain only mode-0600 artifacts. Raw port, USB, network, origin, command,
credential, serial, and process values stay private. Public conclusions may
contain only source/reference identities, closed categories, bounded counts,
and safe booleans.

Detector failure stops before the campaign. A completed flash remains a
completed device effect even if monitor or runtime proof later fails. Preserve
the earliest typed failure through cleanup and record recovery only as a
secondary safe result. Do not automatically reflash, erase, retry, or request
manual boot-mode intervention. Release every owned USB and process resource.
This plan consumes at most one detector and one conditional campaign.

## Verification and acceptance

Before hardware, run the repository-required Rust and Bright Builds checks,
package the canonical ESP32-S3 firmware, validate parity, progress, redaction,
reference cleanliness, plan selection, immutable plan bytes, and the final
diff. Commit and push the plan checkpoint, then rebuild the exact package so
its manifest names that clean source commit.

Success requires detector admission, exact-package flash completion, trusted
runtime identity for the same package, stable observation, `mineonboot=false`,
no mining or hardware-control effect, complete USB cleanup, correct private
modes, and redaction. This proves boot recovery only. It does not exercise
theme mutation or restart durability and therefore cannot promote `API-010`.

On any other outcome, append the closed terminal category to `WORKLOG.md`,
update only the active task block, withhold `RESULT.md` and parity evidence,
leave `API-010` at `implemented`, and stop without retry.
