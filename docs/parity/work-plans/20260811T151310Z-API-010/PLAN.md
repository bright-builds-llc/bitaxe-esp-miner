# Ultra 205 canonical observation recovery plan

- Run ID: `20260811T151310Z-API-010`
- Parity row: `API-010`
- Initial status: `implemented`
- Source commit: `0c25b17ea50ec3f922a19b15d1e5917b89dfe13a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-ultra205-boot-recovery-attempt-011`
- Continues plan: `docs/parity/work-plans/20260811T150224Z-API-010/PLAN.md`
- Prior closure: `docs/parity/work-plans/20260811T150224Z-API-010/CLOSURE.md`

## Diagnosed boundary

Attempt 010 passed its one detector, proving the previous automatic ROM-entry
blocker had changed. Its conditional command never reached the campaign
process: the Justfile variadic recipe passes tokens through unchanged, while
the plan supplied assignment-style `stage=observation` and the Clap interface
requires canonical `--stage observation`. No attempt root, USB session, flash,
NVS write, credential read, or runtime observation occurred.

This plan changes the measured failing boundary. It adds a focused parser
regression for the exact observation command and uses canonical long flags in
the immutable hardware contract. It does not alter flash, firmware runtime, or
device-session behavior.

## Regression and software gate

Add one unit test beside the existing campaign CLI tests. It must parse:

`bitaxe-flash mining-campaign --stage observation --board 205 --port <port> --manifest <manifest> --wifi-credentials <credentials> --evidence-dir <root> --duration-seconds 360 --redact-evidence`

The assertion must prove stage `observation`, board 205, the expected port and
manifest, `profile=None`, `pool_credentials=None`, duration 360, and redaction
enabled. The exact assignment-style token must remain rejected by Clap.

Run the focused Cargo and Bazel flash tests, then the ordered Cargo format,
strict Clippy, all-target build, and all-feature tests; Bright Builds; all
Bazel tests; package; parity and progress; redaction; reference cleanliness;
selector; immutable plan; and diff checks. Commit and push the test before
hardware, then rebuild the exact package from that clean source identity.

## Exact hardware contract

Run only these effect-capable commands:

1. `just package`
2. `test ! -e scratch/ultra205-boot-recovery/wrapper-011 && (umask 077; mkdir -m 700 -p scratch/ultra205-boot-recovery/wrapper-011 && just detect-ultra205 > scratch/ultra205-boot-recovery/wrapper-011/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `just mining-campaign --stage observation --board 205 --port <detector-port> --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --evidence-dir scratch/ultra205-boot-recovery/attempt-011 --duration-seconds 360 --redact-evidence`

Admit exactly one Ultra 205. The manifest must bind the clean pushed source,
pinned reference, schema v3, exact artifact digests, and `source_dirty=false`.
The ignored Wi-Fi credential file is an opaque local input and no pool
credential file is read.

Allowed effects are repo-owned USB reset/re-enumeration, one factory-package
write, one NVS replacement containing only local Wi-Fi settings,
`mineonboot=false`, and the observation marker, followed by bounded receive-only
and same-origin observation. NVS replacement may remove prior hostname, pool,
and other settings.

Mining, pool traffic, ASIC initialization or work, voltage, frequency, fan,
thermal, power control, OTA, erase-flash, ad hoc/raw writes, discovery,
foreign-process termination, fault injection, parity promotion, direct UART,
pins, pads, headers, GPIO, probes, jumpers, soldering, and injected signals are
prohibited.

## Evidence, recovery, and terminal outcomes

The new wrapper and attempt roots must be absent, ignored, mode 0700, and hold
only mode-0600 artifacts. Raw credentials, ports, USB/network identities,
origins, commands, serial, and process traces remain private. Public reporting
is restricted to safe provenance, closed categories, bounded counts, and
booleans.

Detector failure stops before campaign launch. Preserve a completed flash as a
completed effect even if NVS or observation later fails. Preserve the earliest
typed failure through cleanup, report recovery only secondarily, and release
all owned USB/process resources. Do not retry, erase, reflash, or enter manual
boot mode. Attempt 011 permits one detector and one conditional campaign.

Success requires detector admission, exact-package flash, safe NVS seed,
trusted exact-package runtime identity, stable observation, no mining or
hardware-control effects, cleanup, correct modes, and redaction. On success,
record boot recovery without promoting `API-010`; theme mutation and restart
durability were not exercised. On failure, withhold `RESULT.md` and parity
evidence, keep `API-010` at `implemented`, and close with the typed boundary.
