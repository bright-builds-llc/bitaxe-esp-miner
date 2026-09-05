# Fixed USB service memory profile

Ultra 205 must initialize Wi-Fi, HTTP and its telemetry worker, the production mining owner, fixed Serial/JTAG supervision, and statistics together. A visible serial device or a successful individual owner does not establish healthy startup.

The exact-package attempt-006 (`0bdcbe29`) observed stable execution and a safe no-mining baseline but failed `http_telemetry_worker/no_memory`. SPIFFS and the HTTP server initialized; the failed telemetry spawn dropped the server. Later successful owners therefore did not prove that the complete service set fit in memory. The failure remains unqualified evidence in the active task.

## Selected buffer profile

The pinned ESP-IDF 5.5.4 Wi-Fi performance guide lists a memory-saving PSRAM buffer profile with six static RX buffers, six static TX buffers and an AMPDU receive window of twelve. Dynamic RX remains 32. Compared with the observed 16/16 static pools, this releases approximately 32,000 bytes of DMA-capable internal memory. Static TX is retained because PSRAM operation requires DMA-capable TX storage. This trades burst throughput for application memory; the vendor measurements are not hardware qualification of this firmware. See the [ESP32-S3 Wi-Fi guide](https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/wifi.html#how-to-configure-parameters).

The repository selects only those buffer counts and receive window. Other tuning-table settings, service stacks, the 98,304-byte internal reserve, ownership, scheduling, safety thresholds and USB controller selection remain unchanged. The pinned Kconfig ranges admit these values. Its recommendations favor larger buffers for throughput; the documented memory-saving profile is selected deliberately for the measured service-allocation failure.

Canonical packaging validates the generated SDK configuration, including static TX mode, AMPDU enablement, PSRAM preference and the reserve. Missing, duplicated or stale values fail the build before artifact publication; requested defaults alone are insufficient.

## Required evidence

Require complete healthy startup and later-owner readiness on the exact clean package. Preserve fresh heap checkpoints, serial identity, settings and authorization continuity, maximum-size exchanges and cleanup. Wi-Fi stability and production pool behavior still require the planned bounded acceptance; no network or hardware parity is promoted by the configuration change or host tests.
