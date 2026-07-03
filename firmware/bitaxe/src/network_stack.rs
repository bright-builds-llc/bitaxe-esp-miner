//! Shared ESP-IDF network stack initialization.

use esp_idf_svc::sys;

/// Initializes ESP-IDF network services required by Wi-Fi and HTTP.
///
/// Callers that need the default event loop should use
/// `EspSystemEventLoop::take()`. Creating it here through raw ESP-IDF would
/// bypass esp-idf-svc's ownership tracker and make a later take fail with
/// `ESP_ERR_INVALID_STATE`.
pub fn initialize() -> anyhow::Result<()> {
    let netif_result = unsafe { sys::esp_netif_init() };
    if !matches!(netif_result, sys::ESP_OK | sys::ESP_ERR_INVALID_STATE) {
        anyhow::bail!("esp_netif_init failed: esp_err={netif_result}");
    }

    Ok(())
}
