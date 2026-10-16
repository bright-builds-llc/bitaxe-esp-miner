//! Per-surface Wi-Fi RSSI, as the pinned reference reports it. Every surface reads
//! the station RSSI live (`get_wifi_current_rssi`); they differ only when there is
//! no reading. System info and statistics keep their `-90` default after a failed
//! read. BAP and the screen read only while the station is connected and otherwise
//! report `-128`, which the screen shows as `RSSI: -- dBm`.

/// The reference BAP and screen value when the station is not connected.
pub const DISCONNECTED_RSSI_DBM: i16 = -128;

/// The RSSI the screen shows: the live reading, only while connected.
#[must_use]
pub fn connected_rssi_dbm(connected: bool, maybe_live_dbm: Option<i16>) -> Option<i16> {
    maybe_live_dbm.filter(|_| connected)
}

/// The BAP `wifi_rssi` value: the reported RSSI while connected, else `-128`.
#[must_use]
pub fn bap_rssi_dbm(connected: bool, reported_dbm: i16) -> i16 {
    if connected {
        reported_dbm
    } else {
        DISCONNECTED_RSSI_DBM
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_screen_shows_the_live_reading_while_connected() {
        assert_eq!(connected_rssi_dbm(true, Some(-61)), Some(-61));
    }

    #[test]
    fn the_screen_shows_no_value_when_a_connected_read_fails() {
        assert_eq!(connected_rssi_dbm(true, None), None);
    }

    #[test]
    fn the_screen_hides_a_reading_taken_before_the_station_is_connected() {
        assert_eq!(connected_rssi_dbm(false, Some(-61)), None);
    }

    #[test]
    fn bap_reports_the_reading_while_connected() {
        assert_eq!(bap_rssi_dbm(true, -61), -61);
    }

    #[test]
    fn bap_reports_minus_128_while_disconnected() {
        assert_eq!(bap_rssi_dbm(false, -90), DISCONNECTED_RSSI_DBM);
    }
}
