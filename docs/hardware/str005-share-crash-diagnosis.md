# Share001 failure recovery and crash acquisition

Owner: `task-str005-v2-accepted-share-probe`. ADR-0029, ADR-0030 and ADR-0031
apply. This contract preserves Share001's immutable failed result and admits
separate current-state evidence only. Parity remains90/95.

## Published identity and predecessor

The predecessor is `scratch/str005-share/share001/attempt`, inventory SHA-256
`1125f5d0dea1fa310ad3001cfa3c2aaa775924aacd1240a68555021ce559261e`.
Its installed source is `f000872f2e436aa7cdaa8cbfa41eee965a27731e`, full ELF
`a3e257418d625aef5e9423fe28f598377f92b092b634264d3cc1e86b4e83e3c2`,
and Gate `9643e87664397a321c715a3a1b1bb6c1183b83ea`.
The pinned exact private ELF and package remain under
`scratch/development-core-dumps/build-f000872f/`.

Host source and command contract must be clean, software-verified and pushed
before access. Gate source/assets, predecessor inventory and same physical
identity are revalidated. Later stages need their own explicit task activation;
this document alone enables no effect. Disabled share/heartbeat/mining, renewal,
installation, self-test and core-clearing gates stay disabled during diagnosis.

## Stage 1: failure-only current recovery

Use `just str005-share-recovery preflight --private-root ROOT --share-root SHARE001 --gate-root GATE`,
then `serve --private-root ROOT` and `finish --private-root ROOT`.
The detailed producer contract lives in `scripts/str005-share-recovery/CONTRACT.md`.
Only its exact active task/source gate admits execution. Each root is an absent
child of a new ignored0700 parent; wrapper stdout/stderr and detector logs are
separate0600 files. No credential or authority directory is read.

Fresh `just detect-ultra205` must admit exactly one physical device with a known
native-USB profile. Require detector age\<=60s, exclusive ownership and fresh
native browser permission, Hello, exact application identity and possession.
The recovery page has no Start, signer, fixture, restart, flash or fault control.

Save actual qualification and original campaign accounting, diagnostics,
restoration, current/retained status and closure independently. No expected next
ordinal, completed ordinal or charged total is substituted for measurement.
Pending accounting blocks acquisition. Preserve the actual old-attempt loss
without relabeling current idle as historical release. Each read\<=30s; Stop and
Close each\<=150s. Late stage results cannot write after expiry. Every failure
attempts restoration/Close and exact process/listener/serial release.

Finalization rejudges the producer files and seals partial results. A safe current
baseline may qualify while historical resources remain unavailable. Its generated
current recovery proof uses the original collection-begin timestamp and does not
claim the missing original authorization checkpoint or historical preservation.
No retry is implied: only a targeted regression-backed correction or an explicitly
published next stage with fresh evidence permits further access.

## Stage 2: preserve the untouched crash partition

Stage 2 is disabled until Stage 1 passes and the active task explicitly publishes
acquisition. Enable only the existing accepted-share-owned acquisition marker
`Renew-image core-dump acquisition: enabled (fresh recovery required).`
The legacy marker names the installed Renew-corrected image; it grants only this
published read operation. Its core-clearing marker remains disabled.

Build `//tools/flash:flash` from the clean published acquisition revision before
collecting the fresh proof. Use a new Stage1 collection from that same revision;
proof age\<=120s at admission, actual current safe state, no pending reservation,
exact image/physical identity, inactive authority, matched current settings/Device
Identity and proven serial release. The durable semantic proof claim is one-use,
including after failure. Never update proof timestamps or reuse an old proof.

Run the existing repo command once with absolute paths and a fresh detector port:

```text
just core-dump-read --board 205 --port PORT \
  --expected-physical-sha256 PHYSICAL \
  --expected-installed-source f000872f2e436aa7cdaa8cbfa41eee965a27731e \
  --expected-installed-elf a3e257418d625aef5e9423fe28f598377f92b092b634264d3cc1e86b4e83e3c2 \
  --private-root NEW_ACQUISITION_ROOT --recovery-proof FRESH_PROOF
```

Derive PHYSICAL and PORT from the verified private producer and fresh detector;
never publish their raw values. Keep command/vendor outputs private. Supervise
the finite operation for\<=1200s using the existing private process owner and its
component bounds. Retain the same physical USB lease, require board-info/ROM
admission, validate partition geometry and read the complete974848-byte core
region plus partition table. No erase, write, provisioning or NVS change.

Acquisition is reset-capable. Its owner must return to the exact installed image
and release resources even after read failure. Independently collect a new Stage1
post-return recovery; do not infer safe baseline from the ROM return receipt.
Preserve partial raw bytes, failure stage and release outcome. No second read on
an unchanged failed boundary. No self-test may overwrite the original crash.

Before interpreting the dump, run the offline
`just str005-share-crash verify-acquisition --private-root NEW_ACQUISITION_ROOT`.
It independently requires the current table's974848-byte core region, exact raw
length, installed identity and actual acquisition/return/cleanup receipts, then
seals the protected original files. Generic read success alone is insufficient
because the shared backend also supports a legacy64-KiB partition. This archive
verification does not claim a valid decoded crash or historical resource release.

## Stage 3: offline diagnosis

Use `just core-dump inspect` and `just core-dump analyze` with `--dump`, `--elf`,
`--elf-sha256` and new `--private-root` arguments. These are offline commands;
there is no serial fallback. Require the pinned managed ESP-IDF5.5.4 decoder/GDB,
checksum and full ELF identity. Keep dumps, extracted memory/ELF, debug output
and temporary files0600 inside ignored0700 roots.

Inspect the faulting task, registers, stack bounds and symbolized frames against
the exact installed source and native code. Report only reviewed redacted facts,
separate hypotheses and reproducible code boundaries; raw memory never enters
tracked evidence. Missing/truncated/wrong-image dump or insufficient diagnosis is
a precise blocker, not permission to invent a cause or retry Start.

A demonstrated repair, canonical package/native checks and a new published
installation contract are required before any flash. Archive-bound clearing,
startup, accepted-share and heartbeat-loss each retain their independent later
contracts and evidence requirements. No external pool, refund, factory reset,
direct pins, historical replay or parity promotion is admitted here.
