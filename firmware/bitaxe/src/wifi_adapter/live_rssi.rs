//! Reads the station RSSI live, as the reference `get_wifi_current_rssi` does for
//! every system-info request, statistics sample, BAP update and screen refresh.
//! Only the RSSI leaves this function; the record's SSID and BSSID are dropped.

use esp_idf_svc::sys;

/// The associated access point's RSSI, or `None` while the station is not associated.
pub(super) fn station_rssi_dbm() -> Option<i16> {
    let mut record = sys::wifi_ap_record_t::default();
    // SAFETY: `record` is a valid, writable `wifi_ap_record_t` for the whole call.
    let result = unsafe { sys::esp_wifi_sta_get_ap_info(&mut record) };
    (result == sys::ESP_OK).then_some(i16::from(record.rssi))
}
