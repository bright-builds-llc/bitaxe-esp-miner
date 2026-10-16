//! BAP answer and subscription values, projected from the system-info wire in the
//! same units and formats as the pinned reference's `bap_handlers.c` and
//! `bap_subscription.c`, which differ from the reference README in places.
//!
//! Owner-accepted divergence (2026-10-14): the `wifi` subscription never sends the
//! stored Wi-Fi password, which the reference sends in plain text as `wifi_password`.

use bitaxe_config::catalog::board_catalog;
use bitaxe_core::bap::{BapParameter, BapRequestSnapshot};
use bitaxe_core::wifi_rssi;

use crate::best_difficulty::reference_u64;
use crate::wire::SystemInfoWire;

/// The reference `systemInfo` answer: `deviceModel` is the board family
/// (`DEVICE_CONFIG.family.name`), never the board version.
#[must_use]
pub fn bap_request_snapshot(info: SystemInfoWire) -> BapRequestSnapshot {
    BapRequestSnapshot {
        device_model: board_family(&info.board_version).to_owned(),
        asic_model: info.asic_model,
        pool_endpoint: info.stratum_url,
        pool_port: info.stratum_port,
        pool_user: info.stratum_user,
        shares_accepted: info.shares_accepted,
        shares_rejected: info.shares_rejected,
        block_height: info
            .maybe_block_height
            .and_then(|height| i32::try_from(height).ok())
            .unwrap_or_default(),
        found_block: i32::try_from(info.block_found).unwrap_or(i32::MAX),
        show_new_block: info.show_new_block,
    }
}

/// The catalog family for an upstream board version; empty for an unknown board.
#[must_use]
pub fn board_family(board_version: &str) -> &'static str {
    board_catalog()
        .iter()
        .find(|entry| entry.board_version() == board_version)
        .map_or("", |entry| entry.family())
}

/// The `(token, value)` pairs one subscription update publishes, in reference order.
#[must_use]
pub fn bap_subscription_values(
    parameter: BapParameter,
    info: &SystemInfoWire,
) -> Vec<(&'static str, String)> {
    match parameter {
        BapParameter::Hashrate => vec![("hashrate", format!("{:.2}", info.hash_rate))],
        // The reference formats these floats with `%f`: six decimals.
        BapParameter::Temperature => vec![
            ("chipTemp", format!("{:.6}", info.temp)),
            ("vrTemp", format!("{:.6}", info.vr_temp)),
        ],
        BapParameter::Power => vec![("power", format!("{:.2}", info.power))],
        // The reference's power module reports millivolts and milliamps.
        BapParameter::Voltage => vec![("voltage", format!("{:.2}", info.voltage_millivolts))],
        BapParameter::Current => vec![("current", format!("{:.2}", info.current_milliamps))],
        BapParameter::Shares => vec![(
            "shares",
            format!("{}/{}", info.shares_accepted, info.shares_rejected),
        )],
        BapParameter::FanSpeed => vec![("fan_speed", info.fan_rpm.to_string())],
        BapParameter::BestDifficulty => vec![(
            "best_difficulty",
            suffix_string(reference_u64(info.best_diff)),
        )],
        BapParameter::BlockHeight => vec![(
            "block_height",
            info.maybe_block_height.unwrap_or_default().to_string(),
        )],
        BapParameter::Wifi => vec![
            ("wifi_ssid", info.ssid.clone()),
            (
                "wifi_rssi",
                wifi_rssi::bap_rssi_dbm(info.wifi_status == "connected", info.wifi_rssi)
                    .to_string(),
            ),
            ("wifi_ip", info.ipv4.clone()),
        ],
        BapParameter::FoundBlock => vec![("found_block", info.block_found.to_string())],
        _ => Vec::new(),
    }
}

/// The reference `suffixString(value, buf, size, 0)`: integer-divide to three
/// significant thousands, then two decimals and an SI suffix, or the bare integer
/// below 1,000.
#[must_use]
pub fn suffix_string(value: u64) -> String {
    const KILO: u64 = 1_000;
    const STEPS: [(u64, u64, &str); 6] = [
        (1_000_000_000_000_000_000, 1_000_000_000_000_000, "E"),
        (1_000_000_000_000_000, 1_000_000_000_000, "P"),
        (1_000_000_000_000, 1_000_000_000, "T"),
        (1_000_000_000, 1_000_000, "G"),
        (1_000_000, KILO, "M"),
        (KILO, 1, "k"),
    ];
    let Some((_, divisor, suffix)) = STEPS.iter().find(|(floor, _, _)| value >= *floor) else {
        return value.to_string();
    };
    // `dval = (double)(val / divisor) / 1000.0`, printed with `%.2f`.
    format!("{:.2}{suffix}", (value / divisor) as f64 / KILO as f64)
}

#[cfg(test)]
mod tests;
