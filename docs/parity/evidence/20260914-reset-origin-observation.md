# Accepted retained-runtime observation

Observation 005 is independently accepted as `observed_stable`. It establishes
fresh read-only stability of the installed Ultra 205 runtime. It does not identify
the preceding panic, qualify a controlled restart, satisfy CPU0 cadence under
load, or authorize mining.

## Bound identities

| Component                    | Identity                                                           |
| ---------------------------- | ------------------------------------------------------------------ |
| Installed firmware           | `daa69bb03274968e78d131e9b0bf13ff12f64ff0`                         |
| Installed ELF                | `49643f417a27b1842865c92109facf253efc8b3837c3906074ccc20a41ce3545` |
| Gate                         | `82992de904616cbfc313044b91aa376fef3d91c9`                         |
| Published observation driver | `0179a6e5975bdc7ff56f63a6f5ffa273df18c87b`                         |
| Result file SHA256           | `8847afd051a0a17ad3703dd3a5e6986b13e0c6ba395f705160095ad9f1543817` |
| Canonical receipt SHA256     | `f1762ddbe756afde4057791659a29e00f5ee7c8554a3c1602e4ecb3c41378bec` |

The protected result is
`scratch/qualification-implementation-20260907/iterative/reset-origin-observation-005/result.json`.
Its thirteen retained runtime artifacts were not rebuilt or relabelled when the
host observer changed. `reset-origin-review` and a separate read-only audit
validated the receipt, all inventory entries and private artifact permissions.

## Observations and final state

| Measurement                 | Accepted evidence                                                                                                                     |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| Host coverage               | 130039 ms                                                                                                                             |
| Boot progression            | 43 advances over 126710 device ms                                                                                                     |
| Healthy startup progression | 230 advances over 128960 device ms                                                                                                    |
| Maximum sampled gap         | 1009 ms overall/startup; 3368 ms boot                                                                                                 |
| Boot history                | One observed ordinal 2; initial category `panic`; zero transitions                                                                    |
| Capture checks              | No identity conflicts, clock regressions or diagnostic issues                                                                         |
| Journal ordering            | Before/start 3 → end 115 → after 116 → closed 119                                                                                     |
| Qualification ledger        | Next 17, last completed 16, 1380000 ms charged, pending false                                                                         |
| Original campaign           | 240000 ms exhausted; unchanged                                                                                                        |
| Preservation and cleanup    | Identity/settings/authorization high-water match; mine-on-boot false; leases inactive; browser, supervisor and USB ownership released |

Host and device spans are measured independently. These are sampled diagnostics,
not an uninterrupted byte capture or fresh idle sensor/owner-stack measurements.
The payload-free `previous_boot/wrong_firmware` preparation marker remains
unattributable retained history. It is neither current failure nor health proof.
No installation, restart, allowance issuance, reservation or mining occurred.

## Preserved failed observations

Observations 001–004 remain sealed and unverified:

- **001:** JSON-encoded HTML prevented configuration and USB control.
- **002:** the host classifier rejected the payload-free previous-firmware
  preparation marker before capture.
- **003:** the state journal failed after 83 records. A production browser replay
  reproduced exhausted keepalive request capacity when response bodies were not
  consumed; consuming each body resolved that replay and the later live journal.
  The original failed request's transport cause was not retained.
- **004:** complete collection lacked a fresh status row between end and
  after-accounting. The strict independent ordering predicate rejected it.

Corrections were tested and published before each guarded fresh observation.
The finalization regression runs both production clients against the actual
supervisor and independent judge with a fake device clock; it reproduced the
ordering rejection before correction and passed afterward. All 122 canonical
targets, ordered Cargo gates, ownership/reference/redaction/standards and parity
checks passed before observation 005. No acceptance criterion or prior result
was relaxed or rewritten.

## Remaining qualification

The [prospective reset-origin contract](../../hardware/reset-origin-recovery.md)
requires a new verified firmware/Gate pair for the missing authenticated restart
interface, a fresh observation on that pair, and one bounded controlled restart
with independent identity, startup, accounting and cleanup proof. Only accepted
recovery can support fresh complete cadence qualification. USB qualification,
USB migration and cadence tasks remain active; parity remains 90/95.
