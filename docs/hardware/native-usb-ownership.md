# Fixed native USB ownership

ADR-0021 is authoritative. This guide replaces the TinyUSB/Serial-JTAG switching
procedure. Historical work plans, archived task records and consumed attempts
retain their original facts; they are not active hardware instructions.

## Controller and execution identity

ESP32-S3 USB Serial/JTAG stays enabled in normal, diagnostic, safe-baseline and
Worker operation. The hardware controller provides the single serial channel
used by the browser and maintenance tools. Worker activity changes application
state. No TinyUSB installation, vendor interface, 1200-baud maintenance gesture,
PHY handoff, or alternate application USB profile remains.

Physical identity, OS enumeration, execution owner and logical control session
are separate. The shared Serial/JTAG descriptor can belong to ROM or firmware.
Only current board-info admits ROM; application proof requires fresh exact
source/ELF evidence. Browser possession additionally binds Device Identity,
the signed serial application manifest and the current unpredictable session.
VID/PID, device paths and serial strings are discovery hints, never authority.

## Cryptographic randomness before network ownership

Startup seeds the existing cryptographic `StdRng` once using a qualified ESP32-S3
entropy source, then disables that source before ADC or Wi-Fi initialization.
Session nonces and newly created Device Identity seeds use the seeded owner;
missing or contended ownership fails closed. Stored Device Identity is preserved.
There is no per-connection ADC reconfiguration or fallback to an unqualified
hardware random stream after network failure. This follows the startup ordering
in [ESP-IDF's RNG guidance](https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-reference/system/random.html)
and uses the [pinned Rand cryptographic generator](https://github.com/rust-random/rand/blob/0.8.5/src/rngs/std.rs).

## Serial protocol and ownership

Gate's published Controller 0.4, serial transport 0.1 and possession 0.2 are the
wire authority. Messages use bounded JSON-line envelopes with profile, kind,
sessionId, sequence and payload. Control payloads are at most 65536 bytes;
complete wire records are at most 66560 bytes including newline. Decoders
handle arbitrary fragmentation, coalescing and boot-log resynchronization.
Unknown, malformed or stale control traffic cannot authorize work.

One firmware owner serializes application output. Heartbeat/control traffic has
priority over bounded diagnostics. Raw credentials, signing input, proofs and
control payloads never become logs. Allowlisted boot diagnostics before hello
are observation, not control authority; ROM output is not a Worker message.

The browser uses Web Serial directly, with a user gesture for the first port
grant. Web Locks coordinate one active Worker per origin; OS exclusivity protects
the port. Fully release streams and locks before CLI flashing or another reader.
Hidden, navigating or closing pages attempt restoration and invalidate/release
the session. Foreground return requires explicit fresh connection/start.

## Liveness and safety

Only advancing authenticated heartbeats for the admitted session refresh link
authority. They arrive every second and never renew a signed Work Lease. At
2.8 seconds without one, a generation-bound atomic latch revokes work/submission
admission. Bounded cancellation and scheduling must begin the existing ordered
shutdown within three seconds. Blocking Start/Restore, NVS, diagnostics, queued
actuation and late preparation completion cannot bypass this deadline.

Cooling remains a separate bounded postcondition. Live acceptance additionally
has a durable 240000-ms cumulative budget; reconnect, renewal, interruption and
retry cannot reset it. Missing or exhausted acceptance admission fails closed.

## Flashing and recovery

Use repo-owned just commands. Freeze a clean pushed package before effects,
retain the physical lease, admit ESP32-S3 ROM before writes, and use validated
non-overlapping package segments. Ordinary updates exclude NVS and unrelated
data partitions, preserving Device Identity and durable replay marks. Factory
reset is never a fallback for missing runtime evidence.

ROM entry/application exit remain explicit reset operations. The fixed
controller eliminates custom PHY handoff, not ROM execution or every USB
re-enumeration. A manual BOOT/RESET bootstrap can need physical RESET to resample
its boot strap. Do not repeat an unchanged ineffective reset. Software reset
recipes require their own verified bounded contract.

The standard application-return helper retains ROM admission and the physical
lease, checks RTC FORCE_DOWNLOAD, conditionally clears only that bit with verified
readback, and invokes the validated espflash native Serial/JTAG reset once. The
native sequence clears virtual BOOT before reset/release. The exact command,
register mask, 30-second child bounds, cleanup, and stop conditions are recorded
in the active qualification task. There is no esptool `run` plus generic RTS-only
reset. Hardware success still requires fresh application evidence.

Completed writes, application return and cleanup are separate outcomes. Missing
boot transcripts do not authorize repeat flashing. Reacquire only the admitted
physical device, never through network discovery. Preserve the earliest failure
and require complete owned-process/port cleanup before reuse. An uninterruptible
host open is an unresolved cleanup boundary; do not layer another session over
it or infer machine-wide failure from agent-only timing evidence.

The migration's recovery destination is an exact fixed-Serial/JTAG safe baseline
with mining disabled. Recovery-006 and old native-USB discriminator plans remain
historical; their missing evidence is not promoted by architectural replacement.

## Verification and privacy

Run just verify-native-usb-ownership after transport/startup/host changes.
Qualification requires exact no-mining identity/framing checks and 20 complete
browser-release/flash/reconnect cycles preserving Device Identity/settings.
Only afterward run the active live-acceptance task's 180000/30000/30000-ms windows
and Conservative hardware limits.

Use a fresh mode-0700 parent, absent supervisor child, distinct mode-0600 logs,
immutable attempt artifacts and closed outcomes. Raw operational material stays
protected. Committed evidence excludes device, network, pool, credential, proof,
token and NVS-secret values. Software completion or one stable boot never causes
automatic parity promotion.

## Qualification command sequence

The active `TASKS.md` contracts define the allowed effects and stop conditions.
All placeholders below bind to the exact clean pushed sources, canonical
package, admitted device, and protected ignored attempt root. A preflight is
read-only with respect to the device; it creates one local campaign context.
Do not replace that context to obtain another mining budget.

1. Resolve the existing detector's pending cleanup before opening USB. If its
   driver open remains uninterruptible, disconnect only the supplied USB cable,
   keep barrel power attached, and wait for the known owner/child to exit.
   Confirm their exit and lease cleanup before reconnecting. A built-in
   BOOT/RESET bootstrap may be used only when standard ROM admission fails;
   record the manual-bootstrap reset case separately from buttonless cycles.
1. Run `just detect-ultra205` into a protected local log. Require one physical
   device and explicit ROM admission for writes. Create each evidence directory
   once; do not reuse a consumed failed attempt.
1. Install with `just flash-monitor --board 205 --port <admitted-port> --manifest <canonical-package-json> --evidence-dir <new-private-directory> --redact-evidence`. Do not pass `--factory-reset`, image overrides, or NVS
   provisioning inputs. The browser must have released the port first.
1. Run `just fixed-usb-qualification preflight --firmware-root <firmware-repo> --gate-root <gate-repo> --firmware-commit <clean-firmware-commit> --gate-commit <exact-pinned-gate-commit> --manifest <canonical-package-json> --private-root <new-ignored-private-root> --authority-directory <protected-authority-directory>`. Its parent must be mode 0700.
1. Run `just fixed-usb-qualification serve --private-root <that-private-root> --authority-directory <protected-authority-directory> --pool-credentials <ignored-owner-pool-file>` with separate protected stdout/stderr. Open the
   emitted loopback page in desktop Chrome. The browser owns USB directly;
   the local test server signs bounded grants and records closed facts.
1. Configure and explicitly connect without starting work. Verify exact source,
   ELF, and the browser's private Device Identity/settings/replay comparison,
   plus the maximum transport probe. Only continuity booleans and an unrelated
   random page-baseline identifier enter cycle reports; never persist those
   fingerprints. Retain the same page for all 20 cycles. Close the browser port,
   flash the same package,
   reconnect explicitly and compare evidence for each of 20 cycles. Seal each
   real observation using `just fixed-usb-qualification record-cycle --private-root <that-private-root> --input <protected-cycle-report>`.
   A report validator does not itself perform or prove a hardware operation.
1. Only after all 20 cycles pass, prepare the three signed acceptance windows
   through the page. Use the task's conservative profile and cumulative device
   budget. The normal window requires a correlated accepted share; hide the
   page in window 1 and suppress heartbeats in window 2. Reconnect explicitly
   to collect retained device stop evidence. Missing evidence remains a failure.
1. Validate a completed window with the page's validation button or
   `just fixed-usb-qualification judge --private-root <that-private-root> --window <0-or-1-or-2>`. Finish with qualified cooling, persisted
   `mineonboot=false`, cleared leases/volatile credentials, browser closure,
   supervisor termination and proved USB cleanup. Promote only independently
   redacted evidence; leave unverified criteria and unrelated tasks open.

Human checkpoints have no deadline. Device leases, serial operations, signer
children, active-mining windows and cooling retain their bounded deadlines.
