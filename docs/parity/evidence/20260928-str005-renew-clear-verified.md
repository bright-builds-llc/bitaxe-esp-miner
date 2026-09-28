# Archive-bound clear and post-clear recovery verified

Clear001 passed against the captured dump with SHA-256
`bc8a8079d97fa2395f63bda78286be6d7c83aaeb39774199e643b8226e11288b`.
The official owner compared the actual pre-erase bytes, erased only the admitted
core region, verified all 974,848 readback bytes as0xFF, returned to the exact
f000872f/a3e25741 image and released its lease. The all-FF readback SHA-256 is
`94a21164829c644f15d62317c52d9f42a0ef66bd084d5ffdeb007b375e210951`.

The clear receipt deliberately does not claim an authenticated hardware baseline.
Independent post-clear recovery004 subsequently passed on boot15, with idle V2
status, next20 / last19 / 1,920,000 ms / pending=false, unchanged exhausted original
budget, current preservation/restoration and browser/server/listener/serial release.

Sealed private roots under `scratch/str005-renew/`:

- `current-recovery003/attempt`: `bc28e576f2ab81a1f4948ef68aeeb8734e52aa95585d725229c90396f77bd98f`
- `clear001/attempt`: `ee021f8eb30c124e332ef18aad71025645c7bea9ed369dcf4503acf24495fb51`
- `current-recovery004/attempt`: `db2117d05e64e76045701d1009b2f5ac4f55612f682d6e4e09264539b46b43ee`

The share admission reader independently rejudged installation, actual capture/
cutoff, clear and successful post-clear recovery. Its private manifest SHA-256 is
`a56bb4b89f3d002a115562daa516f51c262dae5a603d066c187afc4e568b9cd6`.
No grant, Start, mining or renewal occurred in these stages. The retained dump and
all earlier seals remain intact. Mining and heartbeat claims remain outstanding;
parity remains90/95.
