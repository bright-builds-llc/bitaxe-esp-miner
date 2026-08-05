# Parity work plan

- Run ID: `20260805T005320Z-API-010`
- Parity row: `API-010`
- Initial status: `implemented`
- Source commit: `db0bef8fbdbf7cbffe64a4c015de1c7ea7b080fc`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api010-bootloader-diagnostic-attempt-009`

## Selection and failure signal

The clean synchronized `main` branch and deterministic selector resume only
the open `API-010` lineage. This plan directly continues
`docs/parity/work-plans/20260804T230534Z-API-010/PLAN.md` after attempt-008
packaged the fixed commit `50287f62` but stopped before flashing as
`bootloader_connect_failed`.

Protected reduction proves the same physical device remained accessible and
holder-free, cleanup completed, espflash failed to synchronize with the
bootloader, and USB enumeration did not change. The protected child stderr is
only the generic espflash `connection_failed` diagnostic. Pinned espflash 4.5.0
source shows that the CLI collapses the underlying reset/synchronization error
after its reset-strategy loop unless debug logging is enabled.

The ranked hypotheses are:

1. the USB-JTAG/Serial reset strategy fails before ROM synchronization;
2. reset succeeds but the synchronization exchange receives no valid reply;
3. the installed panic-loop application races the automatic reset boundary;
4. a host transport, holder, or identity failure occurs before reset.

NVS corruption is not eligible: the previous failure was proved as a firmware
stack overflow, erasing settings cannot install the fixed image, and factory
reset would destroy recovery inputs without addressing this boundary.

## Software diagnosis and fix

At the real device-session policy seam, add a closed espflash connection
signature that distinguishes process timeout/interruption, missing device,
serial/reset I/O, wrong boot mode, no sync reply, SLIP framing, read mismatch,
command timeout, generic connection failure, and unavailable diagnostics.
Drive it with production-shaped protected debug transcripts.

For `UsbOperation::Detect` board-info children only, set a child-local
`RUST_LOG=debug` filter so pinned espflash records the otherwise-hidden reset
error in its already private mode-0600 stderr. Do not change flash, monitor,
capture, or campaign child environments. Add a real-child-process test proving
the environment reaches the child and remains within private output. Return
only `bootloader_connect_failed` plus the closed signature; never return raw
debug text, ports, USB identities, process data, commands, or paths.

The tight red/green loop is:

`cargo test -p bitaxe-device-session bootloader_diagnostic --all-features`

It must fail against the current generic classifier, pass with the typed
classifier, cover every closed variant, and prove sensitive fixture tokens are
absent from public error details.

## Hardware contract

After the diagnostic implementation is clean, completely verified, committed,
and pushed, build the exact package and run one protected detector. The
detector uses only pinned `espflash board-info --chip esp32s3 --non-interactive
--before usb-reset --after hard-reset` under the existing durable USB
supervisor. It may reset and probe the provided USB interface but performs no
flash, NVS, settings, network, or other write.

The wrapper root is
`scratch/api010-theme-durability/wrapper-009`, absent before launch and mode
0700 with mode-0600 streams. Device-session traces remain beneath their
existing ignored mode-0700 roots with mode-0600 children. Public records may
contain only source/reference commits, closed category/signature, safe
booleans, bounded counts/durations, file modes, and conclusion.

If the diagnostic detector succeeds, its same protected stdout becomes the
sole detector handoff for one conditional capture beneath
`scratch/api010-theme-durability/attempt-009`. That capture may perform one
exact-package flash-monitor transaction, read the original theme, POST one
generated non-secret alternate theme, confirm immediate readback, request one
normal software restart, prove the same physical device and exact build at
ordinal `N+1`, confirm persistence, restore the original theme, confirm
restoration and cleanup, and use at most one built-in exact-package recovery
flash if restoration cannot otherwise be confirmed.

The only eligible public artifact is
`docs/parity/evidence/api010-theme-durability/theme-durability-projection.json`
after complete success and semantic redaction. Do not discover origins; read or
expose credentials; alter Wi-Fi or pool settings; mine; enable ASIC work;
change voltage, frequency, fan, thermal, or power controls; exercise display
input; perform OTA, erase, factory reset, fault injection, or raw partition
writes; terminate foreign processes; or use direct UART, pins, pads, headers,
GPIO, probes, jumpers, soldering, or injected signals.

## Verification and execution

Before hardware, run the focused device-session and flash targets, canonical
firmware build/package, then:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`
9. semantic redaction, pinned-reference cleanliness, immutable-plan,
   sensitive-output, selector, and diff checks

Commit and push the diagnostic implementation. The only authorized hardware
sequence is:

1. `just package`
2. `test ! -e scratch/api010-theme-durability/wrapper-009 && (umask 077; mkdir -m 700 scratch/api010-theme-durability/wrapper-009 && just detect-ultra205 > scratch/api010-theme-durability/wrapper-009/detector.stdout 2>&1)`
3. Only after command 2 succeeds:
   `test ! -e scratch/api010-theme-durability/attempt-009 && test ! -e docs/parity/evidence/api010-theme-durability/theme-durability-projection.json && (umask 077; just verify-theme-durability --private-root scratch/api010-theme-durability/attempt-009 --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --detector-output scratch/api010-theme-durability/wrapper-009/detector.stdout --projection docs/parity/evidence/api010-theme-durability/theme-durability-projection.json --capture-timeout-seconds 360 > scratch/api010-theme-durability/wrapper-009/verify.stdout 2> scratch/api010-theme-durability/wrapper-009/verify.stderr)`

The detector may use only its existing one internally eligible retry after an
objective same-device enumeration change. Any detector failure consumes this
diagnostic probe and forbids capture. If the closed signature is unchanged,
stop as `repeated_boundary` and require an external normal-connector transport
change before another task. A distinct signature returns to diagnosis. Capture
launch failure, timeout, malformed/missing projection, panic loop, non-ready
session, persistence mismatch, restoration uncertainty, cleanup/privacy
failure, or safety-invariant violation ends without retry and preserves the
earliest typed category.

Promotion requires `bitaxe-theme-durability-evidence-v1` to bind the exact
clean package and reference, one admitted board 205, a ready same-device
session, one software restart, exact build recovery, changed boot session,
ordinal `N+1`, persisted theme equality, exact restoration, disabled mining
and hardware control, complete cleanup, and passed redaction. Otherwise
withhold evidence and `RESULT.md`, keep `API-010` at `implemented`, and stop.
