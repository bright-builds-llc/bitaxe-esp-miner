use core::fmt;

use super::{Notification, ScreenPage, ScreenSnapshot, SCREEN_LINE_COUNT, SCREEN_MAX_LINE_CHARS};

/// Complete private four-line OLED frame.
#[derive(Clone, PartialEq, Eq)]
pub struct ScreenFrame {
    lines: [String; SCREEN_LINE_COUNT],
}

impl fmt::Debug for ScreenFrame {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ScreenFrame { private_text: redacted }")
    }
}

impl ScreenFrame {
    /// Returns private lines for the physical display adapter only.
    #[must_use]
    pub fn private_lines(&self) -> [&str; SCREEN_LINE_COUNT] {
        self.lines.each_ref().map(String::as_str)
    }

    /// Confirms the exact bounded text geometry.
    #[must_use]
    pub fn fits_ultra205(&self) -> bool {
        self.lines
            .iter()
            .all(|line| line.chars().count() <= SCREEN_MAX_LINE_CHARS)
    }
}

pub(super) fn page_frame(
    page: ScreenPage,
    snapshot: &ScreenSnapshot,
    notification: Notification,
) -> ScreenFrame {
    let lines = match page {
        ScreenPage::SelfTest => {
            let values = snapshot.maybe_self_test.as_ref();
            [
                "BITAXE SELF-TEST".to_owned(),
                values.map_or_else(String::new, |values| values[0].clone()),
                values.map_or_else(String::new, |values| values[1].clone()),
                values.map_or_else(String::new, |values| values[2].clone()),
            ]
        }
        ScreenPage::FirmwareUpdate => {
            let values = snapshot.maybe_firmware_update.as_ref();
            [
                "Firmware update".to_owned(),
                values.map_or_else(String::new, |values| values[0].clone()),
                values.map_or_else(String::new, |values| values[1].clone()),
                String::new(),
            ]
        }
        ScreenPage::AsicStatus => [
            "ASIC STATUS:".to_owned(),
            snapshot.maybe_asic_status.clone().unwrap_or_default(),
            String::new(),
            String::new(),
        ],
        ScreenPage::Overheat => [
            "DEVICE OVERHEAT!".to_owned(),
            "Configuration reset".to_owned(),
            "IP Address:".to_owned(),
            available(&snapshot.ipv4),
        ],
        ScreenPage::Welcome => [
            "Welcome to Bitaxe".to_owned(),
            "Setup Wi-Fi:".to_owned(),
            available(&snapshot.ap_ssid),
            String::new(),
        ],
        ScreenPage::Connection => [
            format!("Wi-Fi: {}", available(&snapshot.ssid)),
            available(&snapshot.wifi_status),
            "Setup Wi-Fi:".to_owned(),
            available(&snapshot.ap_ssid),
        ],
        ScreenPage::BitaxeIntro => [
            "Bitaxe".to_owned(),
            available(&snapshot.model),
            available(&snapshot.board),
            available(&snapshot.version),
        ],
        ScreenPage::OpenSourceIntro => [
            "Open Source Miners".to_owned(),
            "Open source firmware".to_owned(),
            String::new(),
            String::new(),
        ],
        ScreenPage::Urls => [
            "Stratum Host:".to_owned(),
            available(&snapshot.pool_host),
            "IP Address:".to_owned(),
            available(&snapshot.ipv4),
        ],
        ScreenPage::Statistics => [
            numeric_line("Gh/s", snapshot.maybe_hashrate_ghs, 2),
            efficiency_line(snapshot.maybe_power_watts, snapshot.maybe_hashrate_ghs),
            best_line(snapshot.maybe_best_difficulty, snapshot.show_new_block),
            numeric_line("Temp", snapshot.maybe_temperature_celsius, 1),
        ],
        ScreenPage::Mining => [
            integer_line("Block", snapshot.maybe_block_height.map(u64::from)),
            numeric_line("Difficulty", snapshot.maybe_network_difficulty, 0),
            "Scriptsig:".to_owned(),
            snapshot
                .maybe_scriptsig
                .as_deref()
                .map_or_else(|| "--".to_owned(), str::to_owned),
        ],
        ScreenPage::Wifi => [
            "Wi-Fi Signal".to_owned(),
            snapshot.maybe_rssi_dbm.map_or_else(
                || "RSSI: -- dBm".to_owned(),
                |rssi| format!("RSSI: {rssi} dBm"),
            ),
            signal_line(snapshot.maybe_rssi_dbm),
            uptime_line(snapshot.uptime_seconds),
        ],
    };
    frame_with_notification(lines, notification)
}

pub(super) fn frame(lines: [&str; SCREEN_LINE_COUNT]) -> ScreenFrame {
    ScreenFrame {
        lines: lines.map(clean_line),
    }
}

fn frame_with_notification(
    mut lines: [String; SCREEN_LINE_COUNT],
    notification: Notification,
) -> ScreenFrame {
    let label = notification.label();
    if !label.is_empty() {
        let keep = SCREEN_MAX_LINE_CHARS.saturating_sub(label.chars().count() + 1);
        lines[0] = format!("{} {label}", truncate(&lines[0], keep));
    }
    ScreenFrame {
        lines: lines.map(|line| clean_line(&line)),
    }
}

fn clean_line(value: &str) -> String {
    let normalized: String = value
        .chars()
        .map(|character| {
            if character.is_ascii_control() || character == '\n' || character == '\r' {
                ' '
            } else if character.is_ascii() {
                character
            } else {
                '?'
            }
        })
        .collect();
    truncate(normalized.trim(), SCREEN_MAX_LINE_CHARS)
}

fn truncate(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

fn available(value: &str) -> String {
    if value.trim().is_empty() {
        "--".to_owned()
    } else {
        value.to_owned()
    }
}

fn numeric_line(label: &str, maybe_value: Option<f64>, decimals: usize) -> String {
    maybe_value.map_or_else(
        || format!("{label}: --"),
        |value| format!("{label}: {value:.decimals$}"),
    )
}

fn integer_line(label: &str, maybe_value: Option<u64>) -> String {
    maybe_value.map_or_else(
        || format!("{label}: --"),
        |value| format!("{label}: {value}"),
    )
}

fn efficiency_line(maybe_power_watts: Option<f64>, maybe_hashrate_ghs: Option<f64>) -> String {
    match (maybe_power_watts, maybe_hashrate_ghs) {
        (Some(power), Some(hashrate)) if power > 0.0 && hashrate > 0.0 => {
            format!("J/Th: {:.2}", power / (hashrate / 1_000.0))
        }
        _ => "J/Th: --".to_owned(),
    }
}

fn best_line(maybe_difficulty: Option<f64>, show_new_block: bool) -> String {
    let mut line = numeric_line("Best", maybe_difficulty, 0);
    if show_new_block {
        line.push_str(" BLOCK FOUND");
    }
    line
}

fn signal_line(maybe_rssi_dbm: Option<i16>) -> String {
    let quality = match maybe_rssi_dbm {
        Some(rssi) if rssi > -50 => "Excellent",
        Some(rssi) if rssi > -60 => "Good",
        Some(rssi) if rssi > -70 => "Fair",
        Some(_) => "Weak",
        None => "--",
    };
    format!("Signal: {quality}")
}

fn uptime_line(total_seconds: u64) -> String {
    let days = total_seconds / 86_400;
    let hours = (total_seconds % 86_400) / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;
    if days > 0 {
        format!("Uptime: {days}d {hours}h {minutes}m {seconds}s")
    } else if hours > 0 {
        format!("Uptime: {hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("Uptime: {minutes}m {seconds}s")
    } else {
        format!("Uptime: {seconds}s")
    }
}
