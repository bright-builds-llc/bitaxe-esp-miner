# ADR-0021: Fixed USB Serial/JTAG and browser Web Serial

## Status

Accepted on 2026-09-04 by the owner-approved migration plan.

## Decision

ESP32-S3 USB Serial/JTAG is the sole USB controller in normal, diagnostic,
safe-baseline, and Worker operation. Worker activity is an application state.
The browser connects directly with Web Serial; no installed helper or active
WebUSB compatibility path remains. Initial hardware qualification is Ultra 205,
macOS, and desktop Chrome.

This supersedes ADR-0020's PHY switching and the USB-topology portions of
ADR-0018/0019. ADR-0015's separation of bootloader, observation, and control
responsibilities remains, but those responsibilities may share one serial
channel. Historical plans and evidence retain their original conclusions.

Gate publishes Controller 0.4, serial transport 0.1, and possession 0.2. The
signed serial application manifest replaces the application USB descriptor
claim. Fresh possession binds exact firmware identity and a logical session.
Device Identity, role-separated signing keys, durable authorization high-water
marks, the restoration journal, secret zeroization, and the sole Production
Mining Session remain mandatory.

Serial envelopes contain `profile`, `kind`, `sessionId`, `sequence`, and
`payload`. Session, control, heartbeat, and diagnostic records share one bounded
incremental stream: 64 KiB control payload, 65 KiB complete wire record including
newline. Gate's published profile is the canonical wire specification. One
firmware writer serializes application output; diagnostic backpressure never
blocks control or liveness, and control payloads never become logs.

The browser allows one connection per origin, coordinates tabs with Web Locks,
and releases streams/port ownership before flashing. A hidden or closing page
attempts restoration, invalidates its session, and releases the port. Returning
requires a fresh explicit connection/start. Session-bound advancing heartbeats
arrive every second and never renew a Work Lease. At 2.8 seconds without a valid
heartbeat, device-local generation revocation closes actual work/submission
admission and begins shutdown within three seconds, using the remaining 200 ms. Blocking commands,
NVS, logging, and hardware preparation cannot own this deadline. Cooling is a
separate bounded postcondition.

Ordinary firmware updates write validated disjoint package segments and
preserve NVS, Device Identity, replay marks, and unrelated partitions. Factory
installation/reset is explicit, never implicit recovery. ROM admission and
exact runtime identity remain separate evidence; a shared descriptor does not
prove which execution owner is running.

## Migration and verification

The migration establishes a new fixed-Serial/JTAG safe recovery baseline,
replacing recovery-006 as this migration's required destination. It does not
verify any unresolved historical recovery, selected-app, or parity evidence.
Obsolete mechanisms are superseded administratively with obligations transferred
to the active migration/qualification tasks.

Hardware requires clean pushed sources, exact Gate/package identity, protected
evidence, fresh same-device admission, and completed prior USB cleanup. No
session may be layered over the currently blocked detector. Built-in BOOT/RESET
may bootstrap or recover the device; no direct UART, electrical pin access,
erase-flash, or electrical fault injection is authorized.

Qualification requires a no-mining baseline, browser identity/framing tests,
and 20 browser-release/flash/reconnect cycles preserving identity and settings.
The owner separately approved real mining through the existing private owner
pool: 180 active seconds normal, 30 foreground-loss, and 30 heartbeat-loss,
with a device-enforced cumulative 240-second ceiling. Active time includes the
period from first work dispatch through confirmed ASIC reset/power-off; reserve
the bounded shutdown tail and never assume preparation time covers it.
Use the existing
Conservative profile (400 MHz, 1100 mV, fan 100%), fresh safety observations,
4.5–5.5 V input, at most 15 W, below 75 C, and nonzero fresh RPM. Preserve the
existing ordered shutdown and bounded cooling to 45 C before fan 30%.

No accepted share within the bound leaves that criterion unverified. Any
failure preserves its earliest signature; a retry needs verified progress.
Finish with the new firmware installed, leases/secrets cleared, mining disabled,
and resources released. Hardware success does not automatically promote parity.
