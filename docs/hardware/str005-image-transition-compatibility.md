# Planned-image history compatibility

Gate `9643e87664397a321c715a3a1b1bb6c1183b83ea` replaces
`bd26128788dd3f04842984e545fa2382c0b50561` for prospective qualification. The
[exact diff and unchanged blob record](str005-image-transition-compatibility.json)
binds its archive and clean bundle. Historical installation001 and startup003
keep their actual Gate/source identities and immutable outcomes.

Only three production files changed: `worker-v2-image-history.ts`,
`worker-v2-page.ts` and `worker-serial-acceptance.ts`. The page retains separate
histories across an explicit before-to-candidate image transition. Both configured
source and full ELF must change, the controller must admit that exact candidate,
and original preservation must match. An old retained record must be terminal
with released socket/worker/fence. An old idle state is also supported. The first
candidate status must be idle on a strictly newer boot. Subsequent idle boots may
advance monotonically for controlled core self-tests; once a candidate record
exists, its immutable boot/history and record-loss checks apply. Old history and
one-use work claims remain retained; no public reset-history API was introduced.

The remaining seven changed paths are tests, fixture support and the Gate issue
record. Controller0.4 and possession0.2 fixtures, serial0.2 specification, serial
controller runtime, V2 serial-control transport, serial framing, Cargo manifests
and lockfile, and package manifest have identical Git blobs across the two pins.
There is no wire/configuration API, signing, budget, NVS, flash, firmware USB or
mining-protocol change in this Gate correction.

A production page/controller/WebCrypto regression reproduced the old failure.
Independent review found and corrected idle-before-update and post-update
self-test-reboot cases before publication. The resulting 24 focused tests passed
independent review; the complete Gate verification passed 864 web tests, Rust
checks, actual browser checks, build/package and standards. The new firmware
collector separately tests pre-begin rejection/timeouts, independent Stop/Close,
first-cause persistence and non-promotion of partial installation receipts.

Under ADR-0029, unchanged protocol/transport evidence remains applicable to those
mechanisms. This review admits fresh read-only recovery on installed firmware
`8d4e470e4cd3e18be510d3912586aeda65960275`, full ELF
`d0dd775335e55e9d009a0a779bc938bebf64e76fe78d8d7c8144a06ef1c9d88e`,
through the separately published task contract. It does not qualify new-image
preservation, capture, mining, renewals, accepted shares, heartbeat shutdown or
four-cycle durability. Later effects need their own verified scope and fresh
observations. Installation001 remains partial and parity remains90/95.
