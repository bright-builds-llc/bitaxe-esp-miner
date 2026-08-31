# Native USB configuration-AP recovery — NVS-first successor

- Run ID: `20260831T033840Z-NATIVE-USB-CONFIG-AP-RECOVERY-NVS-FIRST`
- Source base: `ad2120a5`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-native-usb-config-ap-recovery-205`
- Supersedes plan: `20260830T184150Z-NATIVE-USB-CONFIG-AP-RECOVERY`

## Objective

Separate recovery into two independently committed and hardware-gated stages.
First read and semantically validate installed NVS without changing the Mac's
network. Only an accepted `nvs_match` may authorize configuration-AP recovery.
Recovery-006 remains installed throughout.

Firmware/NVS writes, erase, mining, ASIC, fan/voltage, transition diagnostics,
OTA, manual board buttons, broad discovery, direct UART/pins/pads/probes, other
devices/boards, durability, and parity promotion remain excluded.

## Interface and state

Expose `just native-usb-config-ap-recovery preflight|read-nvs|recover|resume|finalize`
over `scratch/native-usb-config-ap-recovery/attempt-001`.

The consume-once state machine is `prepared → nvs_read_started → nvs_match |
nvs_mismatch`; only `nvs_match` can continue through `ap_recovery_started →
settings_authenticated → restart_sent → host_network_restored →
station_authenticated → complete`.

`resume` consumes only a sealed post-effect state. It never repeats NVS
readback, association, settings/theme requests, restart, or host restoration.
Preserve earliest failure. A repeated authoritative post-fix signature is
terminal.

## Stage 1 — read-only NVS discriminator

Implement and commit this stage before AP code or host-network effects.

- Through one retained `UsbSession`, admit the same Ultra 205 and ROM
  downloader.
- Canonicalize a contained non-symlinked managed `esptool.py` and run exactly
  `read_flash 0x9000 0x6000`; reject every other command, range, size, and
  output location.
- Store the exact 24 KiB dump privately, return to recovery-006, and prove USB
  and process cleanup.
- Parse it with contained managed `nvs_tool.py --integrity-check --dump minimal
  --format json`.
- Independently generate and decode the expected ordinary seed from the
  protected Wi-Fi input.
- Compare typed namespace, key, encoding, and value digests for
  `main:wifissid`, `main:wifipass`, `main:mineonboot=0`, and every ordinary
  Ultra 205 default.

Publish no public projection. Seal only `nvs_match` or `nvs_mismatch`.
`nvs_mismatch` is terminal and requires a separately authorized NVS-write
recovery contract. AP association remains prohibited.

Run every required gate, commit/push Stage 1, rebuild the exact clean package,
run one effect-free preflight, and consume exactly one readback ordinal.

## Stage 2 — conditional configuration-AP recovery

Implement only after accepted `nvs_match`, in a separate verified commit.

- Derive the exact `Bitaxe_XXXX` AP from private USB base-MAC evidence.
- Snapshot Mac Wi-Fi power and default-route interface/gateway.
- Perform one directed CoreWLAN scan and association only to that candidate;
  require one DHCP lease and RFC1918 gateway and make no external call while
  associated.
- Authenticate strict local HTTP by AP MAC offset, exact recovery-006 identity,
  `mineonboot=false`, and a closed Wi-Fi reason.
- Send at most one settings PATCH and theme POST; reconcile uncertain responses
  by reads without repetition; request one restart after exact persistence.
- Restore Mac Wi-Fi once and require the original route interface/gateway.
- Collect one no-timeout displayed station IP, bind STA MAC to USB base MAC,
  and prove exact settings/theme, inactive zero-work/share state, final USB
  admission, and cleanup.

Host restoration failure stops device work. Absent station recovery after the
verified repair is terminal.

## Evidence and verification

Only `finalize` after `complete` publishes
`bitaxe-native-usb-config-ap-recovery-projection-v1`. Public evidence contains
safe digests, NVS match facts, closed Wi-Fi reason, AP/STA MAC bindings,
bounded counts, restoration/safe-state facts, host restoration, cleanup, and
redaction. Raw NVS, credentials, SSIDs, MACs, IPs, interfaces, gateways, HTTP
bodies, USB identities, and process data stay private; local UI may show IPs.

Use red-to-green tests for exact read range and no-write ownership; contained
tools/runfiles; corrupt/truncated/oversized and wrong namespace/key/type/value
NVS cases; semantic seed comparison; raw-value exclusion; state non-repetition;
AP association/DHCP/route restoration; MAC offsets; strict HTTP request-once
reconciliation; restart/resume; final station binding; allowlisting; modes;
process groups; and cleanup.

Before every commit or hardware stage run ordered Cargo formatting, strict
Clippy, all-target/all-feature build, all-feature tests, Bright Builds, all
Bazel tests, normal/rollback firmware links, canonical package, native-USB
ownership, parity/progress, redaction, reference cleanliness, whitespace,
sensitive-value scan, and final diff review.

On success create `RESULT.md`, archive only
`task-native-usb-config-ap-recovery-205`, and leave recovery-006 installed.

## Assumptions

- The successor controls future work; `ad2120a5` remains immutable history.
- `Bitaxe_ABCD` is the current USB-derived recovery-006 fallback AP.
- Mac Wi-Fi interruption is authorized only after `nvs_match`.
- Recovery firmware uses base MAC for STA and the documented increment for AP.
- Repo-local native-USB/hardware/evidence guidance and Bright Builds standards
  govern implementation.
