# ESP Device Session Contract

This contract applies to repository-owned workflows that cross bootloader,
running-firmware, USB lifecycle, and HTTP application boundaries. It implements
[ADR-0015](../adr/0015-separate-bootloader-runtime-and-control-transports.md)
and remains subordinate to the repository hardware-attempt and evidence
policies.

## Responsibility Matrix

| Responsibility                       | Required backend                                             | Prohibited substitution                                 |
| ------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------- |
| Detect and bind the target           | Task-gated repo-owned detector plus pinned espflash          | Stale node names or unrelated prior evidence            |
| Probe or write flash                 | `espflash 4.5.0` with the phase's explicit reset flags       | Application HTTP or ad hoc raw writes                   |
| Observe running firmware             | Receive-only OS-native reader                                | `espflash monitor` as an authoritative runtime observer |
| Request a normal application restart | Existing HTTP restart route                                  | DTR/RTS, bootloader reset, or a repeated POST           |
| Prove the restarted application      | Typed USB identity plus HTTP boot/build/postcondition quorum | Serial bytes, sampled downtime, or node change alone    |

## Session State

A device session distinguishes:

- **physical identity**: stable USB facts that bind the admitted device;
- **enumeration identity**: the current OS node and enumeration epoch;
- **transport ownership**: the one allowed reader and its process lifecycle;
- **application identity**: boot session, reset category, boot ordinal, source
  and application identity; and
- **postcondition**: the exact state that the effect was intended to produce or
  preserve.

The session accepts either a stable enumeration or a disappearance followed by
one unique node with the original physical identity. Three stable samples and
holder checks are required before initial use and after any re-acquisition.
Zero or multiple physical matches, identity drift, an inaccessible node, or an
unexpected holder fail closed.

On macOS, derive physical identity from the same canonical `ioreg` USB fields
used by detector admission and keep the device-node metadata in a separate
enumeration digest. Open the runtime port only with receive-only,
non-controlling, nonblocking semantics (`O_RDONLY | O_NOCTTY | O_NONBLOCK`). Do
not write to the serial descriptor or issue modem-control ioctls.

## Reboot Transaction

1. Validate the protected root, exact current source/package, selected physical
   identity, and baseline application facts.
1. Arm the receive-only reader and prove pre-reboot application-byte delivery.
1. Send the restart request exactly once and separately record request-write
   completion and response receipt.
1. Observe USB, serial, and HTTP recovery within one monotonic bound. Do not
   resend the restart if its response is missing or incomplete.
1. Poll only the previously trusted origin. Connection failures during recovery
   are transitional observations, not permission to discover another origin.
1. Classify `ready` only when the same physical device exposes the exact build,
   a changed boot session, software-reset category, ordinal `N + 1`, and the
   required postcondition.
1. Record serial delivery, sampled service loss, and enumeration behavior as
   corroboration. None may override a failed authoritative quorum.
1. Close the reader, verify no unexpected holder remains, finalize private
   artifacts, and emit only the redacted projection.

The restart and recovery reads use the same strict, bounded TCP/TLS primitive
as the Phase 35 HTTP boundary probe: direct HTTP/1.1, no proxy, no redirect,
explicit request-write completion, verified TLS for HTTPS, bounded headers and
body, and one absolute session deadline.

Terminal classification preserves the first authoritative failure in this
order: `observer_unqualified`, `restart_request_not_sent`,
`restart_attribution_ambiguous`, `usb_identity_unavailable`,
`usb_identity_drift`, `service_recovery_timeout`, `boot_identity_invalid`,
`build_identity_mismatch`, `session_not_advanced`, `reset_reason_wrong`,
`ordinal_not_next`, `postcondition_mismatch`, then `ready`. Cleanup outcomes are
secondary and never replace that category. Serial delivery is recorded as one
of `correlated`, `silent`, `reacquired`, or `failed`.

## Reset Strategy

- Flash/probe stages retain `--before usb-reset --after hard-reset` because the
  current Ultra 205 workflow has crossed those typed boundaries successfully.
- Running-firmware restart uses the existing HTTP route and `esp_restart()`.
- `watchdog-reset` is not an implicit fallback. It may be considered only after
  a distinct typed bootloader-exit diagnosis, a repository regression, and an
  explicit phase-plan change.
- No device-session command may toggle modem-control lines, write serial input,
  or use direct UART, pins, pads, probes, or other electrical interfaces.

## Evidence And Platform Support

Private events, HTTP material, serial bytes, origins, paths, process data, and
USB identity material are protected operational data. Store them only as
mode-`0600` regular files beneath a mode-`0700` ignored root. Console and Git
may contain only closed categories, booleans, bounded counts/durations, and
safe public provenance.

The initial production adapter supports macOS. Linux and Windows fail closed as
`observer_unqualified` until separately implemented and qualified.

## Flash And Reflash Session

The supported `just detect-ultra205`, `just flash`, `just monitor`, and
`just flash-monitor` entrypoints share one macOS USB supervisor. Package
construction and immutable image admission complete before device ownership is
taken. The supervisor then binds the physical-device digest, takes a host-wide
advisory lease, and retains it across flash, valid same-device re-enumeration,
receive-only observation, and cleanup.

Every device-affecting `espflash` child runs in an isolated process group. A
timeout, error, `SIGINT`, or `SIGTERM` causes bounded group termination,
escalation, reaping, and holder verification. A private crash journal permits
stale cleanup only when the owner start time, child process group, executable
path, and executable digest still prove repository ownership. Unknown
applications, browsers, and unmanaged `espflash` processes are never
terminated.

Routine monitoring is the macOS receive-only reader, not `espflash monitor`.
Success is returned only after three stable samples show the admitted physical
device is accessible and holder-free. No postcondition probe reconnects to the
bootloader. Public outcomes use the closed flash vocabulary:
`ready`, `concurrent_repo_session`, `foreign_holder`, `transport_absent`,
`identity_drift`, `bootloader_connect_failed`,
`flash_failed_before_transfer`, `flash_failed_after_transfer`,
`monitor_failed`, `cleanup_failed`, `recovery_not_observed`, and
`repeated_boundary`.

One retry is possible only for a software/transport-recoverable boundary after
cleanup and an objective enumeration change of the same physical device. The
operation and admitted image remain unchanged. Identity drift, absence,
foreign ownership, admission failure, a write/verify failure, or recurrence
stops immediately.

After a successful factory or NVS write, the supervisor observes one continuous
60-second same-device recovery window before classifying
`recovery_not_observed`; it never repeats the successful write merely because
recovery is delayed. Post-flash recovery, monitor admission, and final cleanup
use 60-second bounds. Post-probe recovery and retry admission use 30-second
bounds. The phase policy does not change the same-device identity,
receive-accessibility, foreign-holder, or three-identical-sample admission
requirements.
Each recovery writes a protected bounded summary containing only its phase,
deadline, booleans, maximum stable-sample count, enumeration-change
observation, and final state. A public recovery failure may expose only that
closed signature, never device paths, USB identity material, process data, or
credentials.

The task-gated durability acceptance command is:

```text
just verify-flash-durability board=205 cycles=20 port=<detector-port> manifest=<package-manifest> wifi-credentials=wifi-credentials.json protected-root=scratch/flash-durability/<attempt>
```

It runs the four five-cycle sequences historically recorded by
[`task-durable-ultra205-device-sessions`](../../TASKS.archive.md#task-durable-ultra205-device-sessions--2026-07-25--make-usb-flash-cycles-self-cleaning),
stops at the first failed boundary, and stores private mode-`0600` logs beneath
a new ignored mode-`0700` root. That archived record is context only and does
not authorize a rerun; a fresh active hardware task must satisfy the current
task gate first.
