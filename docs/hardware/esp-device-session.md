# ESP Device Session Contract

This contract applies to repository-owned workflows that cross bootloader,
running-firmware, USB lifecycle, and HTTP application boundaries. It implements
[ADR-0015](../adr/0015-separate-bootloader-runtime-and-control-transports.md)
and remains subordinate to the repository hardware-attempt and evidence
policies.

## Responsibility Matrix

| Responsibility                       | Required backend                                             | Prohibited substitution                                 |
| ------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------- |
| Detect and bind the target           | Phase-owned detector plus pinned espflash                    | Stale node names or unrelated prior evidence            |
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
