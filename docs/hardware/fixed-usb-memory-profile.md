# Fixed USB service memory profile

Ultra 205 must initialize Wi-Fi, HTTP and its telemetry worker, the production mining owner, fixed Serial/JTAG supervision, and statistics together. A visible serial device or a successful individual owner does not establish healthy startup.

The exact-package attempt-006 (`0bdcbe29`) observed stable execution and a safe no-mining baseline but failed `http_telemetry_worker/no_memory`. SPIFFS and the HTTP server initialized; the failed telemetry spawn dropped the server. Later successful owners therefore did not prove that the complete service set fit in memory. The failure remains unqualified evidence in the active task.

## Selected buffer profile

The pinned ESP-IDF 5.5.4 Wi-Fi performance guide lists a memory-saving PSRAM buffer profile with six static RX buffers, six static TX buffers and an AMPDU receive window of twelve. Dynamic RX remains 32. Compared with the observed 16/16 static pools, this releases approximately 32,000 bytes of DMA-capable internal memory. Static TX is retained because PSRAM operation requires DMA-capable TX storage. This trades burst throughput for application memory; the vendor measurements are not hardware qualification of this firmware. See the [ESP32-S3 Wi-Fi guide](https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/wifi.html#how-to-configure-parameters).

The repository selects only those buffer counts and receive window. The buffer-profile change leaves other tuning-table settings, service stacks, the 98,304-byte internal reserve, ownership, scheduling, safety thresholds and USB controller selection unchanged. The pinned Kconfig ranges admit these values. Its recommendations favor larger buffers for throughput; the documented memory-saving profile is selected deliberately for the measured service-allocation failure.

Canonical packaging validates the generated SDK configuration, including static TX mode, AMPDU enablement, PSRAM preference and the reserve. Missing, duplicated or stale values fail the build before artifact publication; requested defaults alone are insufficient.

## Main-task telemetry handoff

Attempt-007 (`4431b4dd`) confirmed the buffer-profile improvement: HTTP became ready and Wi-Fi construction retained 195,447 internal/DMA bytes. Statistics thread creation then failed, with 14,535 total internal/DMA bytes free and a largest block of 6,656 bytes. The checkpoint is taken after the attempt; its name does not prove successful thread creation.

The startup main task already has a 16 KiB stack. It becomes the 16 KiB telemetry owner after startup frames unwind, avoiding simultaneous main and telemetry stacks during initialization. Statistics retains its existing 8 KiB owner. A prepared HTTP runtime owns its server until activation or failure cleanup. Activation sets priority 5 before publishing HTTP and runtime readiness and then enters the existing 500 ms cadence loop; it does not allocate a new task or change the per-frame HTTP queue model.

Main retains its existing CPU0 affinity. Telemetry therefore changes from an unpinned pthread to CPU0, while its priority and cadence remain the same. This is a deliberate scheduling difference requiring hardware qualification. Independent USB and safety tasks retain their existing configuration. Canonical builds verify main's 16 KiB stack, CPU0 affinity and priority 5 contract; no stack is reduced to force startup success.

## Required evidence

Require complete healthy startup and later-owner readiness on the exact clean package. Preserve fresh heap checkpoints, serial identity, settings and authorization continuity, maximum-size exchanges and cleanup. Wi-Fi stability and production pool behavior still require the planned bounded acceptance; no network or hardware parity is promoted by the configuration change or host tests.

## Required thread reservations

September 15's [failed controlled-restart preparation](../parity/evidence/20260915-reconnect-startup-failure.md)
shows why late allocation must be reviewed across the complete required service
set. Reserving the statistics thread early allowed it to become active, but the
later 8192-byte Wi-Fi reconnect thread returned an out-of-memory error. The USB
receive thread was another required 8192-byte allocation after association.

The successor reserves reconnect together with the prepared Wi-Fi owner and
receive together with the prepared Worker runtime. The existing statistics
activation gate is shared by these preparations. Reservation allocates the
unchanged thread while internal memory is available; activation permits its
existing loop to run only at the original owner/subscription or USB-install
boundary. Dropped or failed preparations cancel the parked thread. Preparing a
receiver does not permit USB reads or application authority, and preparing
reconnect does not process queued network events.

Wi-Fi preparation captures the already initialized credential state once and
reserves reconnect only for valid station credentials. Startup consumes that
same state. Missing or invalid credentials must reach AP-only provisioning
without attempting an unused reconnect allocation. Current startup has no
intervening credential writer: RF and the USB command reader are still inactive.

This preserves the selected buffer counts, internal reserve, stacks, priorities,
CPU affinity, statistics sampling and heartbeat deadlines. It changes allocation
order rather than reducing the required resource budget. Conditional captive-DNS
and explicit reconnect-probe threads remain separate allocations. Deterministic
lifecycle tests cannot prove available device heap; the full service set must
still pass exact-package startup and the subsequent hardware qualification.

## Firmware package size profile

The application crate's release profile uses `opt-level = "z"`; dependency
versions and their profile settings, the global release profile, and ESP-IDF/C
options remain unchanged. The successor003 preview measured 4118992 bytes,
leaving 75312 bytes in the existing 4194304-byte OTA slot. This is an observed
build result, not a guarantee for later revisions.

Changing optimization can change stack layout and instruction timing. The
native telemetry audit therefore recognizes the exact outlined
`OperatorSnapshotPublisher::publish_profiled` call hop and includes its full
7392-byte frame. The measured projection path totals12448/16384 bytes; the
recognizer still rejects unknown wrappers, clobbered call targets and dynamic
stack adjustments, and reports oversized paths as budget failures. Complete
callgraph and hardware safety claims remain false. Exact clean publication,
fresh native audits and the unchanged hardware cadence/shutdown criteria remain
required before qualification can close.
