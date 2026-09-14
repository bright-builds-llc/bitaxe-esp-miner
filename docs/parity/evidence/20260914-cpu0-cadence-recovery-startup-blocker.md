# CPU0 cadence recovery — panic-origin startup admission blocker

The single guarded recovery installation remains unqualified. Its write
completed and the observed firmware reached healthy runtime startup, but the
existing assessor rejected the initial panic reset category. No browser,
supervisor, cadence phase, signing operation or mining work began in this
recovery context.

## Exact pair and observed boundary

Firmware: `daa69bb03274968e78d131e9b0bf13ff12f64ff0`.
ELF SHA-256: `49643f417a27b1842865c92109facf253efc8b3837c3906074ccc20a41ce3545`.
Gate: `82992de904616cbfc313044b91aa376fef3d91c9`.
Protected root:
`scratch/qualification-implementation-20260907/iterative/cadence-017-startup-recovery-1`.

The independently reverified failure-only inventory is
`0ed37434d6bb8587ea51dec6b1e3cf41128ee555635058bd74fd2c84b702e834`.
It preserves thirty original files and thirteen package artifacts. Its auditor
SHA-256 is `ab4b08c421e919ef985d96ee808d98179aa6d3f00a5bfdb3fac11ce6a52912c8`.
The preceding ordinal-17 failure seal `d65b506e` remains unchanged.

The clean package and native audits passed before effects. The OTA image is
4187024 bytes, leaving 7280 bytes in the unchanged partition. The targeted
main-task path is 12560 bytes against 16384; the production-owner entry is
4512 bytes. These checks do not claim a complete callgraph bound or hardware
qualification.

The structured installation assessment records:

| Field                            | Observed value                                      |
| -------------------------------- | --------------------------------------------------- |
| Write                            | Completed                                           |
| Startup complete                 | True                                                |
| Startup failed                   | False                                               |
| Stable boot                      | False                                               |
| Retained failure receipt history | False                                               |
| Issues                           | `reboot_observed`, `insufficient_advancing_samples` |
| Evidence trust                   | Untrusted                                           |

The capture contains eleven boot markers, all for boot ordinal 2, with ten
advancing pairs from 266 to 27621 ms. Forty-seven startup-complete markers
advance from 5082 to 28201 ms. No transition to another boot was observed in
that capture. The repeated `panic` reason is a boot-time value repeated by
periodic diagnostics; it is not a series of new panic events.

The preceding reset's timing and firmware attribution remain unknown. Neither
the improved stack audit nor this continuing boot progression supplies that
missing proof.

## Assessor interpretation and disposition

The existing `fixed_serial::boot()` assessor adds `reboot_observed` for a panic,
watchdog or brownout reason even on the first marker. Its `finish()` method
then adds `insufficient_advancing_samples` whenever stable-boot admission fails.
Consequently that second label does not independently establish a lack of
advancing timestamps. No assessor condition or historical receipt was changed.

The recovery's exclusive installation claim remains consumed. Its flash
wrapper exited 1, and the prearmed process observer exited 0 with 308
observations, no recorded failures and no remaining owned children. Actual
host inspection confirmed release of the recorded processes and USB holders.
No browser or supervisor was started, and no authenticated recovery/accounting
receipt exists. Host cleanup is not device-restoration proof.

The policy disposition is `stop_impossible_contract` for this attempt: the
current recovery contract requires its original installation receipt to pass,
and that immutable receipt did not pass. No recovered result, preparation-2
context or mining allowance may be manufactured from it. This disposition does
not mean CPU0 qualification is permanently impossible.

The last authenticated ledger remains the predecessor's next ordinal 17,
last completed 16, total charged 1380000 ms, pending false. These are expected
continuation values, not a fresh device observation. No reservation was issued
by this recovery workflow.

Cadence, USB qualification and migration remain active; parity remains 90/95.
The next bounded step is prospective reset-origin diagnostics and a separate
read-only observation contract that distinguishes an initial reset category
from a reset occurring within a captured epoch. Any continuation must preserve
this failed assessment and establish fresh stable identity/accounting before
the full qualification cycles and measured phases.
