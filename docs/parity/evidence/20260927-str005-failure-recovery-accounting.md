# STR-005 failure recovery and accounting

Owner: `task-str005-failure-recovery-accounting`. Date: 2026-09-27.
Outcome: **blocked / `stop_hardware_blocker`**.
Boundary: `recovery_retained_record_unavailable`.
Parity remains **90/95**; the task remains active and unarchived.

The [recovery-only contract](../../hardware/str005-failure-recovery-accounting.md)
and implementation were committed, pushed and preflight-verified before device
access. Final collection used host `ea11eec9a66d0f262ee89f4d3dc642d4d5d3fb7c`,
installed firmware `cf7a3f038dabdd5383083734336aeb64a22f837c`, ELF
`66a77d2cb699e2064469c7b124f482a57a7fd6f6b13ef737554f53f721e78590`, and Gate
`e20c0fd52d2216596f904992ffa54fda33be9025`. No firmware update occurred.

## Fresh observations

| Observation                                            | Measured result                                                                          |
| ------------------------------------------------------ | ---------------------------------------------------------------------------------------- |
| Qualification next ordinal                             | 18                                                                                       |
| Last completed ordinal                                 | 17                                                                                       |
| Qualification charged time                             | 1,560,000 ms                                                                             |
| Pending qualification reservation                      | false                                                                                    |
| Original campaign reserved/completed masks             | 7 / 7                                                                                    |
| Original campaign charged time / pending               | 240,000 ms / false                                                                       |
| Authenticated current V2 status                        | idle; retained record absent                                                             |
| Restoration and safe baseline                          | confirmed                                                                                |
| Device lease                                           | inactive                                                                                 |
| Mining and mine-on-boot                                | false                                                                                    |
| Same-page identity/settings/authorization preservation | matched                                                                                  |
| Native Close                                           | closed, disconnected, serial ownership released                                          |
| Collection failures                                    | none                                                                                     |
| Host cleanup                                           | owned page closed; supervisor absent; listener and both serial-node holder checks passed |

Both ledgers were freshly read through Gate's authenticated controller. These
values resolve the current durable accounting uncertainty. They do not prove
that no transient preparation occurred before Share002's panic, authorize reuse
of ordinal18, or select a next mining attempt.

The private diagnostic projection retains a non-authoritative boot observation
and omission counts. Diagnostics do not replace authenticated status or establish
a panic cause. The collector obtained a fresh idle response with `record:null`.
There is therefore no retained Share002 record proving its socket closure, worker
quiescence and fence release. Confirmed current restoration and actual host
cleanup do not reconstruct that historical proof. This is the sole remaining
completion blocker. No further unchanged retry is justified by this result.

## Independent evidence and preserved failures

Final collection004 result digest:
`cf5fbd71bb377e1c540605d8ebea8a7ea5d5bd0f07b582c351dfe97a525d38dd`.
Seal digest:
`61a06aac4a2a1782b01471ae84da911d2723b7066e69b8784981bdda9a4ef01b`.
The private result marks accounting, restoration, serial release and host release
true; retained device-resource proof and overall completion remain false.

Collection001 stopped before browser access when detector-format inspection
exposed a parser mismatch. Collection002 rejected configuration before Connect;
its unverified seal is
`185dc66470448e0b75eef0b22bc9fb7dc8c28795282fa88080a3a750365bd166`.
Collection003 measured the same ledgers and confirmed restoration, but failed
diagnostic/status/Close collection; its seal remains
`41532af9328f103b63969012c87be59f5dffd1a3044be1aad049089c82fc4c09`.
Each continuation followed a published, regression-tested correction to the
specific collection boundary. Earlier failures were not rewritten.

Share002's original seal remains
`14d2122208b2040f1482074c77328cd3a59c651e45bc17e7be7af2648d8f5950`.
The [original failure report](20260927-str005-v2-share-start-unverified.md),
Channel006 publication restrictions, parity checklist and historical evidence
remain unchanged. No Start, mining, grant issuance, pool connection, flashing,
reset, factory operation or Share002 replay was performed by this probe.

## Verification and limitations

All 35 focused tests and five recovery Bazel targets pass, including actual Gate
configuration, diagnostic-parser and serial-control boundaries, with simulated
device responses explicitly distinguished from hardware. Tests cover stale
possession, idle/retained correlation, partial persistence, privacy, failure-latched
controls, failed Close capture and actual loopback-listener release. Gate's
18 controller/page tests and type check passed. Ordered Cargo format, Clippy,
build and tests passed (2,366 tests, three existing ignores); native USB ownership
and symbol checks, reference, redaction, Bright Builds, Markdown and diff checks
passed. Final inventories were independently rechecked without mutation.

The host probe preserves the existing firmware, transport, accounting and safety
implementations. It reuses the qualified Gate instead of creating a signer or
new campaign. Changes to possession, accounting, projection or cleanup invalidate
the affected checks. New proof must remain in separate successor evidence; none
of these observations upgrades Share002, grants mining authority or promotes parity.
