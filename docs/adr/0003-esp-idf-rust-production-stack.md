# Use ESP-IDF Rust for the First Production Stack

The first production Rust firmware target uses ESP-IDF Rust bindings rather than a bare-metal `no_std` stack. Upstream ESP-Miner relies on ESP-IDF facilities such as Wi-Fi, HTTP serving, NVS, SPIFFS, OTA, FreeRTOS tasks, PSRAM-aware allocation, logging, partition images, and ESP flashing conventions, so using ESP-IDF Rust keeps the parity effort focused on device-user behavior instead of rebuilding platform services.
