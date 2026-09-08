# Preparation diagnostics and bounded live acceptance

The separately authorized diagnostic and final acceptance sequence passed on
Ultra 205 with macOS Chrome and fixed USB Serial/JTAG. The installed device was
left restored, mining disabled and all owned resources closed. This closes
`task-worker-preparation-panic-qualification`; it does not promote unrelated
parity or retrospectively change earlier failed or exhausted campaigns.

## Exact execution identities

| Component | Identity |
| -- | -- |
| Installed firmware | `f3bbfd6a05abcffa3735a736c87ff0c250ab8e2a` |
| Firmware ELF SHA-256 | `77ccb176a50f5d973fe99bb3c277ff456a70852f3eb4a0d8d64d02bb23ce9433` |
| Gate browser runtime | `2106f1c1587025d0570e058647a29492159e5d20` |
| Pinned Gate archive SHA-256 | `565a1879ae9b192dd31b1d6992abc64724d4a5cf2b635f5fb1e717a85c5ebb8f` |
| Qualification host driver | `88ddb8a507477faa9220588d6ecf1f6de27fd754` |

ADR-0028 admits the strictly bounded host-only continuation driver against the
retained runtime. All thirteen artifact copies and their contexts revalidate.
Later report commits and local preview builds are not the installed runtime.
The exact pair passed installation, four current-image no-mining update/reconnect
cycles, 65536-byte exchanges, preservation checks, fan-only restoration and
diagnostic ordinal 10 before final acceptance. That diagnostic dispatched four
work items in 12310 active ms, with three expected-filter matches and zero misses.

## Final acceptance

| Window | Ordinal | Reserved ms | Active ms | Work items | Accepted shares | Renewals | Gate closure ms | Shutdown initiation ms |
| -- | -- | -- | -- | -- | -- | -- | -- | -- |
| Normal | 11 | 180000 | 59069 | 26 | 5 | 3 | 734 | 784 |
| Foreground loss | 12, continuation 01 | 30000 | 10510 | 3 | 0 | 0 | 2807 | 2821 |
| Heartbeat loss | 13 | 30000 | 14020 | 4 | 1 | 1 | 2805 | 2826 |

Normal acceptance included five correlated qualified/submitted/accepted shares,
19 expected-filter matches and zero misses. The browser stopped after acceptance
criteria were satisfied. Heartbeat-loss acceptance included one additional
qualified/submitted/accepted share and one renewal. Final active time totals **83599 ms**;
the full **240000 ms** of reservations was charged without refunds.

Both fault windows established same-generation real work before the fault.
Foreground loss used an actual hidden-page event after disabling debugger focus
emulation and verifying real visible/hidden/visible transitions. No synthetic
visibility event or document-property override was used. The heartbeat-loss page
remained visible while the harness suppressed only heartbeat transmission.
Both produced the device's `heartbeat_timeout` reason. Timings above are device
timestamps relative to the last valid advancing heartbeat, measured separately
for generation revocation and shutdown initiation; both fault windows meet 3 s.

Terminal work/submission counters stayed stable across available post-stop
samples. Generation-bound dispatch/submission/queued-actuation guards and
adversarial software tests support the tested revocation boundary. Sparse serial
telemetry does not independently timestamp every physical ASIC or network event.
Ordered shutdown and cooling were checked separately from admission closure.

Active observations across the three windows were 5.34–5.3625 V, at most 8.83 W,
at most 46 C and at least 7510 RPM, with 400 MHz, 1100 mV and fixed 100% fan.
The minimum observed owner-stack headroom was 7800 bytes, above the 4096-byte
floor. These are sampled observations, not claims about unobserved transients.

## Accounting and preserved failures

The original campaign remains exhausted and immutable: 240000 ms charged,
reserved/completed masks 7, no pending reservation. The separate attempt ledger
advanced from 900000 before normal acceptance to 1080000, 1110000 and finally
**1140000 ms**, next ordinal **14**, last completed **13**, pending **false**.
Reconnection did not reset accounting. Device Identity and ordinary settings
were preserved; durable authorization high-water marks advanced as required.

The first ordinal-12 delivery expired its possession context before Start and
returned `admission_required` before reservation or authorization persistence.
Its original `start_failed/command_rejected` evidence remains unverified and
unchanged. ADR-0028's exclusive continuation reused only the unused allowance
identity/ordinal/limits, with a fresh possession and signed delivery after
unchanged-ledger and cleanup proofs. No ordinal was skipped or reservation
refunded. The child retains its earliest `connect_failed` setup/recovery failure
even though the subsequent actual fault window passed.

Earlier preparation and observation failures remain in their original sealed
records. The added retained breadcrumbs exposed a 28-byte owner-stack margin
in diagnostic 001. Increasing the stack to 24576 bytes and auditing the final
4512-byte native entry frame addressed that measured pressure. Later targeted
changes fixed second-Start resource state, resource-observation races, lifetime
share counters and delivery of the signed advisory difficulty hint. The pool
alone remains authoritative for share targets. The original panic's exact cause
is still unproven; no panic occurred in the successful final sequence.

## Evidence and verification

Protected receipts live under the ignored owner-only qualification directory,
in `iterative/attempt-011`, `iterative/attempt-012-continuation-01` and
`iterative/attempt-013`. Repo-owned result validators and artifact verification
passed for all three, including an independent review. The report contains only
redacted result categories, measurements and public source/artifact identities.
No credentials, pool identifiers, signed grants or private possession bindings
are exported here.

| Protected artifact | SHA-256 |
| -- | -- |
| Final acceptance index | `dd3396c013312ddf944eefd2013986fc9efd042c75b27511138310e6c817ad69` |
| Normal result | `4327f299aab11be8018e06b8e83cf0d411db310b60bfe6ae3d10aa470c6a2223` |
| Foreground continuation result | `bc4119d47dfac2da45ee92bbd27e3721cb42f7c77900766637a19291517efbd6` |
| Heartbeat-loss result | `fd14469abd9f2f1e339dfd108dc3f61884657e08496408a282e3c8a94cb15266` |

The published task contract records exact repo-owned bootstrap, preflight,
signing supervisor, flash, judge, stack-audit and receive-only recovery commands.
Firmware verification passed ordered formatting/lint/build/tests (2148 passed,
one existing ignored), 91 canonical runtime targets, real ESP32-S3 packaging,
ownership, reference, provenance/redaction and standards checks. Host continuation
verification passed 114 harness tests, all 93 canonical targets and the ordered
Rust gates. Gate passed 352 Rust tests (two existing opt-in tests ignored),
417 JavaScript tests, format/lint/type/build, browser conformance, lookup,
package and standards checks. Preserved transient host test/launch failures
passed unchanged reruns; no runtime or test deadline was widened.

## Final state and remaining limits

Fresh final evidence reports **28 C**, **3178 RPM**, **0.44 W**, **5.4775 V**,
`safe_stop_complete`, qualified `fan_paused` at 30%, `mine_on_boot=false`,
confirmed restoration and inactive lease. Full cooling remained enabled until
fresh temperature was at most 45 C. Browser streams/locks and both owned tabs
closed; both serial nodes and the listener were holder-free; the supervisor
was reaped with exit 0. No additional reset or reflash followed acceptance.

Abrupt-loss recovery still encountered stale complete device-to-host replies
before fresh Hello. The published receive-only drain (2 s, at most 66560 bytes,
no TX/reset/DTR/RTS changes or payload retention) restored fresh admission.
Observed drains discarded 5427 bytes for foreground recovery and 5399 bytes
for heartbeat recovery. This is an open recovery usability limitation tracked
as `task-fixed-usb-hello-resynchronization`, not seamless browser recovery.
Previous-boot corrupt receipts do not establish physical retention success;
native-abort/interrupted-write behavior is covered offline. Normal-run focus
emulation means that run supplies work/share/renewal evidence only; the separate
un-emulated foreground window supplies visibility-loss evidence. Broader BWG,
Stratum V2, thermal soak and hardware parity obligations remain unresolved.
