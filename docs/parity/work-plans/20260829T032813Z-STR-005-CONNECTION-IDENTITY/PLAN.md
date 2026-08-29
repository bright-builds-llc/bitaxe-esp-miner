# Prove STR-005 TCP connection identity and send path

- Run ID: `20260829T032813Z-STR-005-CONNECTION-IDENTITY`
- Parity row: `STR-005`
- Initial status: `implemented`
- Source commit: `615b81cd78c7fdee00956f2e1a23eddd6c30b4e7`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-str005-tcp-payload-205`
- Parent plan: `docs/parity/work-plans/20260828T185251Z-STR-005/PLAN.md`

## Goal and scope

Resolve the remaining zero-byte boundary without entering Noise, Stratum V2
messages, mining, ASIC, fan, voltage, thermal/power control, direct UART/pins,
external pools, OTA, erase, fault injection, or parity promotion.

Diagnostic-009 must first prove whether the firmware socket that reports the
64-byte write is the same exact TCP connection observed by the same-subnet
fixture. Diagnostic-010 is eligible only when diagnostic-009 proves exactly one
tuple-matched connection and its correlated candidate still receives zero
bytes. More than one exact-peer connection is a failure even if one delivers
the canary.

## Recovery readiness

Before any device effect, the protected supervisor must prove:

- the public redacted recovery projection is a regular tracked mode-`0644`
  file;
- private restore bundle/root/receipt/credentials retain mode `0600`/`0700`;
- source lineage, plan, bundle, projection, and validator receipt are valid;
- one allowed non-symlink `esptool.py` canonicalizes inside the workspace;
- the exact managed NVS Python is executable and imports
  `esp_idf_nvs_partition_gen`;
- `restore-installed --admission-only` accepts the exact action, plan, bundle,
  authorization, and private root.

Typed failures are `recovery_projection_mode`, `restore_esptool`,
`restore_nvs_python`, and `restore_admission`. They stop before the diagnostic
private root, fixture, credential-derived seed, flash, or network effect.

Fresh recovery-only roots are `scratch/str005-tcp-payload/recovery-003` and
`scratch/str005-tcp-payload/recovery-004`. Each is consume-once and may be used
only after an incomplete inline restore; a recovery result must prove exact
recovery-006 identity/settings, `mineonboot=false`, inactive zero-work/share
state, fresh device admission, and cleanup before later diagnostics.

## Diagnostic-009 contract

Firmware emits and replays a private connection marker containing only its
local ephemeral TCP port. The raw port remains ProtectedOperational in the
mode-`0600` monitor log and is never promoted.

The fixture admits only the detector-derived peer IP, inventories at most three
exact-peer connections in one single-threaded nonblocking loop, retains at most
65 bytes per candidate, and observes candidates for at most ten seconds. A
fourth candidate sets `candidate_overflow` and fails. The fixed `0xa5` receipt
is sent only to a candidate that delivers exactly `0x00..0x3f` followed by EOF
or the admitted no-extra-byte boundary.

Private fixture evidence records candidate remote ports and per-candidate read
facts. The supervisor joins the firmware local port to one fixture remote port.
The public v2 projection contains only:

- `tuple_match`, `exact_peer_connection_count`,
  `other_exact_peer_connection_count`, `candidate_overflow`, and
  `correlated_candidate_found`;
- correlated byte count, closed read category, digest match, extra-byte count,
  and receipt status;
- send adapter, locally reported byte count, closed socket-error families,
  existing timings/safety facts, exact restoration, cleanup, and redaction.

The public fixture object is built from an explicit allowlist. No raw local or
remote port, address, endpoint, credential, worker, lease, command, or log may
enter it. Firmware terminal accounting records the locally reported 64-byte
write even when later receipt validation fails.

After a separate clean implementation commit is pushed and packaged, run fresh
detector admission and exactly once:

`just stratum-v2-tcp-payload start --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json --private-parent scratch/str005-tcp-payload/diagnostic-009 --projection docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-009.json --plan docs/parity/work-plans/20260829T032813Z-STR-005-CONNECTION-IDENTITY/PLAN.md --diagnostic-ordinal 9 --capture-timeout-seconds 360 --redact-evidence`

Accepted diagnostic-009 requires exactly one exact-peer candidate, one matching
tuple, exact 64-byte receipt/digest, zero extras, receipt acknowledgment, exact
restoration, independent v2 validation, cleanup, and redaction.

## Conditional diagnostic-010 contract

Diagnostic-010 is eligible only if diagnostic-009 proves exactly one
tuple-matched candidate with zero received bytes and complete restoration.

Its consume-once NVS case is `direct_64_v1`. It replaces the standard-library
write with a bounded partial-write loop over `esp_idf_svc::sys::lwip_send` on
the same socket. It never sends through both adapters. The private transcript
retains only raw operational connection identity; public evidence records
adapter `lwip_direct`, reported bytes in `0..=64`, closed pre-send/post-send/
post-shutdown `SO_ERROR` families, direct-send category, correlated fixture
facts, restoration, cleanup, and redaction.

After a separate eligible implementation commit is pushed and packaged, run
fresh detector admission and exactly once:

`just stratum-v2-tcp-payload start --board 205 --port <detector-port> --package-manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --wifi-credentials wifi-credentials.json --restore-bundle scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json --private-parent scratch/str005-tcp-payload/diagnostic-010 --projection docs/parity/evidence/str005-tcp-payload/tcp-payload-projection-010.json --plan docs/parity/work-plans/20260829T032813Z-STR-005-CONNECTION-IDENTITY/PLAN.md --diagnostic-ordinal 10 --capture-timeout-seconds 360 --redact-evidence`

Branch decisions are fixed:

| Result | Action |
| --- | --- |
| Diagnostic-009: one matching connection delivers 64 bytes | Complete this child; do not run diagnostic-010 |
| Diagnostic-009: multiple connections or tuple mismatch | Stop and fix connection ownership; do not run diagnostic-010 |
| Diagnostic-009: one matching connection receives zero bytes | Implement, verify, and run diagnostic-010 |
| Any incomplete restoration/cleanup | Run the fresh recovery-only contract and stop diagnostics |
| Diagnostic-010 delivers bytes | Classify the standard `TcpStream` send path as the cause |
| Diagnostic-010 reports bytes but fixture receives zero | Stop at the ESP-IDF/lwIP or network-interface boundary |

## Verification and evidence

Write red tests before each production change. Cover recovery projection mode,
missing/out-of-workspace esptool, missing NVS Python/import, restore admission,
one/multiple/overflow fixture candidates, silent plus payload candidates,
unexpected peer, partial/EOF/timeout/mismatch/extra bytes, tuple joins,
duplicate replay markers, raw-port exclusion, projection-v2 acceptance, direct
partial/interrupted/zero writes, closed errno mapping, exact 64-byte bound, and
mutual exclusion of send adapters.

Before every commit or hardware effect run formatting, strict Clippy,
all-target/all-feature build, full Cargo tests, Bright Builds, full Bazel tests,
canonical package, parity, parity progress, redaction, reference cleanliness,
whitespace, sensitive-value review, and final diff review.

Accepted evidence completes only this decomposed child. STR-005 remains
`implemented | unit,golden,workflow` until later cumulative authenticated-share
evidence supports promotion.
