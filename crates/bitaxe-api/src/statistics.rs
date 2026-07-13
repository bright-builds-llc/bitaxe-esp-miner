//! Pure `/api/system/statistics` response mapping.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/tasks/statistics_task.h`
//! - `reference/esp-miner/main/http_server/http_server.c:GET_system_statistics`
//! - `reference/esp-miner/main/http_server/axe-os/src/models/enum/eChartLabel.ts`

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::mining::mining_state_from_runtime;
use crate::ApiSnapshot;

const TIMESTAMP_LABEL: &str = "timestamp";

const ALL_COLUMNS: [StatisticsColumn; 18] = [
    StatisticsColumn::Hashrate,
    StatisticsColumn::Hashrate1m,
    StatisticsColumn::Hashrate10m,
    StatisticsColumn::Hashrate1h,
    StatisticsColumn::ErrorPercentage,
    StatisticsColumn::AsicTemp,
    StatisticsColumn::AsicTemp2,
    StatisticsColumn::VrTemp,
    StatisticsColumn::AsicVoltage,
    StatisticsColumn::Voltage,
    StatisticsColumn::Power,
    StatisticsColumn::Current,
    StatisticsColumn::FanSpeed,
    StatisticsColumn::FanRpm,
    StatisticsColumn::Fan2Rpm,
    StatisticsColumn::WifiRssi,
    StatisticsColumn::FreeHeap,
    StatisticsColumn::ResponseTime,
];

/// Upstream-compatible statistics response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatisticsWire {
    #[serde(rename = "currentTimestamp")]
    pub current_timestamp: u64,
    pub labels: Vec<String>,
    pub statistics: Vec<Vec<Value>>,
}

/// One historical statistics sample before column projection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatisticsSample {
    pub timestamp: u64,
    pub hashrate: f64,
    pub hashrate_1m: f64,
    pub hashrate_10m: f64,
    pub hashrate_1h: f64,
    pub error_percentage: f64,
    pub asic_temp: f64,
    pub asic_temp2: f64,
    pub vr_temp: f64,
    pub asic_voltage: i16,
    pub voltage: f64,
    pub power: f64,
    pub current: f64,
    pub fan_speed: f64,
    pub fan_rpm: u16,
    pub fan2_rpm: u16,
    pub wifi_rssi: i8,
    pub free_heap: u32,
    pub response_time: f64,
}

impl Default for StatisticsSample {
    fn default() -> Self {
        Self {
            timestamp: 0,
            hashrate: 0.0,
            hashrate_1m: 0.0,
            hashrate_10m: 0.0,
            hashrate_1h: 0.0,
            error_percentage: 0.0,
            asic_temp: 0.0,
            asic_temp2: 0.0,
            vr_temp: 0.0,
            asic_voltage: 0,
            voltage: 0.0,
            power: 0.0,
            current: 0.0,
            fan_speed: 0.0,
            fan_rpm: 0,
            fan2_rpm: 0,
            wifi_rssi: 0,
            free_heap: 0,
            response_time: 0.0,
        }
    }
}

impl StatisticsSample {
    #[must_use]
    pub fn from_snapshot(snapshot: &ApiSnapshot, timestamp: u64, response_time: f64) -> Self {
        let mining = mining_state_from_runtime(&snapshot.mining);
        let safe_telemetry = snapshot.safe_telemetry.operator_projection();

        Self {
            timestamp,
            hashrate: mining.hash_rate,
            hashrate_1m: mining.hash_rate_1m,
            hashrate_10m: mining.hash_rate_10m,
            hashrate_1h: mining.hash_rate_1h,
            error_percentage: 0.0,
            asic_temp: safe_telemetry.chip_temp_celsius,
            asic_temp2: safe_telemetry.chip_temp2_celsius,
            vr_temp: safe_telemetry.vr_temp_celsius,
            asic_voltage: safe_telemetry.core_voltage_actual_mv.round() as i16,
            voltage: safe_telemetry.voltage_volts,
            power: safe_telemetry.power_watts,
            current: safe_telemetry.current_amps,
            fan_speed: f64::from(safe_telemetry.fan_speed_percent),
            fan_rpm: safe_telemetry.fan_rpm,
            fan2_rpm: safe_telemetry.fan2_rpm,
            wifi_rssi: clamp_wifi_rssi(safe_telemetry.wifi_rssi_dbm),
            free_heap: snapshot.platform.free_heap.min(u64::from(u32::MAX)) as u32,
            response_time,
        }
    }
}

/// Returns a statistics response with no historical samples.
#[must_use]
pub fn empty_statistics_response(
    current_timestamp: u64,
    maybe_columns: Option<&str>,
) -> StatisticsWire {
    statistics_response(current_timestamp, maybe_columns, &[])
}

/// Projects optional statistics columns and appends `timestamp` to every row.
#[must_use]
pub fn statistics_response(
    current_timestamp: u64,
    maybe_columns: Option<&str>,
    samples: &[StatisticsSample],
) -> StatisticsWire {
    let columns = selected_columns(maybe_columns);
    let mut labels = columns
        .iter()
        .map(|column| column.label().to_owned())
        .collect::<Vec<_>>();
    labels.push(TIMESTAMP_LABEL.to_owned());

    let statistics = samples
        .iter()
        .map(|sample| sample_row(sample, &columns))
        .collect();

    StatisticsWire {
        current_timestamp,
        labels,
        statistics,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatisticsColumn {
    Hashrate,
    Hashrate1m,
    Hashrate10m,
    Hashrate1h,
    ErrorPercentage,
    AsicTemp,
    AsicTemp2,
    VrTemp,
    AsicVoltage,
    Voltage,
    Power,
    Current,
    FanSpeed,
    FanRpm,
    Fan2Rpm,
    WifiRssi,
    FreeHeap,
    ResponseTime,
}

impl StatisticsColumn {
    const fn label(self) -> &'static str {
        match self {
            Self::Hashrate => "hashrate",
            Self::Hashrate1m => "hashrate_1m",
            Self::Hashrate10m => "hashrate_10m",
            Self::Hashrate1h => "hashrate_1h",
            Self::ErrorPercentage => "errorPercentage",
            Self::AsicTemp => "asicTemp",
            Self::AsicTemp2 => "asicTemp2",
            Self::VrTemp => "vrTemp",
            Self::AsicVoltage => "asicVoltage",
            Self::Voltage => "voltage",
            Self::Power => "power",
            Self::Current => "current",
            Self::FanSpeed => "fanSpeed",
            Self::FanRpm => "fanRpm",
            Self::Fan2Rpm => "fan2Rpm",
            Self::WifiRssi => "wifiRssi",
            Self::FreeHeap => "freeHeap",
            Self::ResponseTime => "responseTime",
        }
    }

    fn value(self, sample: &StatisticsSample) -> Value {
        match self {
            Self::Hashrate => json!(sample.hashrate),
            Self::Hashrate1m => json!(sample.hashrate_1m),
            Self::Hashrate10m => json!(sample.hashrate_10m),
            Self::Hashrate1h => json!(sample.hashrate_1h),
            Self::ErrorPercentage => json!(sample.error_percentage),
            Self::AsicTemp => json!(sample.asic_temp),
            Self::AsicTemp2 => json!(sample.asic_temp2),
            Self::VrTemp => json!(sample.vr_temp),
            Self::AsicVoltage => json!(sample.asic_voltage),
            Self::Voltage => json!(sample.voltage),
            Self::Power => json!(sample.power),
            Self::Current => json!(sample.current),
            Self::FanSpeed => json!(sample.fan_speed),
            Self::FanRpm => json!(sample.fan_rpm),
            Self::Fan2Rpm => json!(sample.fan2_rpm),
            Self::WifiRssi => json!(sample.wifi_rssi),
            Self::FreeHeap => json!(sample.free_heap),
            Self::ResponseTime => json!(sample.response_time),
        }
    }
}

fn selected_columns(maybe_columns: Option<&str>) -> Vec<StatisticsColumn> {
    let Some(columns) = maybe_columns else {
        return ALL_COLUMNS.to_vec();
    };

    let mut selected = Vec::new();
    for column in columns.split(',').filter_map(column_from_label) {
        if !selected.contains(&column) {
            selected.push(column);
        }
    }

    if selected.is_empty() {
        return ALL_COLUMNS.to_vec();
    }

    selected
}

fn column_from_label(label: &str) -> Option<StatisticsColumn> {
    ALL_COLUMNS
        .iter()
        .copied()
        .find(|column| column.label() == label.trim())
}

fn sample_row(sample: &StatisticsSample, columns: &[StatisticsColumn]) -> Vec<Value> {
    let mut row = columns
        .iter()
        .map(|column| column.value(sample))
        .collect::<Vec<_>>();
    row.push(json!(sample.timestamp));
    row
}

fn clamp_wifi_rssi(value: i16) -> i8 {
    value.clamp(i16::from(i8::MIN), i16::from(i8::MAX)) as i8
}

#[cfg(test)]
mod tests {
    use bitaxe_safety::observation::{
        BootSessionId, MonotonicMillis, Observation, ObservationSequence,
    };
    use serde_json::{json, Value};

    use crate::statistics::{empty_statistics_response, statistics_response, StatisticsSample};
    use crate::{ApiSnapshot, SafeTelemetrySnapshot, TelemetryObservations};

    #[test]
    fn statistics_response_applies_optional_columns_and_keeps_timestamp_last() {
        // Arrange
        let sample = StatisticsSample {
            timestamp: 10,
            hashrate: 1.5,
            power: 2.5,
            fan_rpm: 3,
            ..StatisticsSample::default()
        };

        // Act
        let response = statistics_response(20, Some("hashrate,power,fanRpm"), &[sample]);

        // Assert
        assert_eq!(response.current_timestamp, 20);
        assert_eq!(
            response.labels,
            vec!["hashrate", "power", "fanRpm", "timestamp"]
        );
        assert_eq!(
            response.statistics,
            vec![vec![json!(1.5), json!(2.5), json!(3), json!(10)]]
        );
    }

    #[test]
    fn empty_statistics_history_uses_fixture_shape_without_fake_history() {
        // Arrange
        let fixture = include_str!("../fixtures/api/statistics-empty-compatible.json");
        let expected: Value =
            serde_json::from_str(fixture).expect("statistics fixture should be valid JSON");

        // Act
        let response = empty_statistics_response(0, None);
        let actual = serde_json::to_value(response).expect("statistics should serialize");

        // Assert
        assert_eq!(actual, expected);
        assert_eq!(actual["statistics"], json!([]));
    }

    #[test]
    fn safety_telemetry_projection_statistics_sample_uses_safe_telemetry_values() {
        // Arrange
        let mut snapshot = ApiSnapshot::safe_ultra_205();
        snapshot.platform.free_heap = 4_096;
        snapshot.safe_telemetry =
            SafeTelemetrySnapshot::from_observations(&fresh_telemetry_observations());

        // Act
        let sample = StatisticsSample::from_snapshot(&snapshot, 123, 7.5);

        // Assert
        assert_eq!(sample.timestamp, 123);
        assert_eq!(sample.asic_temp, 56.0);
        assert_eq!(sample.vr_temp, 45.0);
        assert_eq!(sample.asic_voltage, 0);
        assert_eq!(sample.voltage, 5.1);
        assert_eq!(sample.power, 11.5);
        assert_eq!(sample.current, 2.25);
        assert_eq!(sample.fan_speed, 0.0);
        assert_eq!(sample.fan_rpm, 3_200);
        assert_eq!(sample.wifi_rssi, -90);
        assert_eq!(sample.free_heap, 4_096);
        assert_eq!(sample.response_time, 7.5);
    }

    #[test]
    fn safety_telemetry_projection_statistics_sample_keeps_unavailable_numeric_compatibility() {
        // Arrange
        let snapshot = ApiSnapshot::safe_ultra_205();

        // Act
        let sample = StatisticsSample::from_snapshot(&snapshot, 123, 7.5);

        // Assert
        assert_eq!(snapshot.safe_telemetry.power_watts, 0.0);
        assert_eq!(sample.asic_temp, 0.0);
        assert_eq!(sample.power, 0.0);
        assert_eq!(sample.voltage, 0.0);
        assert_eq!(sample.current, 0.0);
        assert_eq!(sample.fan_rpm, 0);
    }

    fn fresh_telemetry_observations() -> TelemetryObservations {
        TelemetryObservations {
            power_watts: fresh_f64(11.5, 1),
            bus_voltage_volts: fresh_f64(5.1, 2),
            current_amps: fresh_f64(2.25, 3),
            chip_temp_celsius: fresh_f64(56.0, 4),
            vr_temp_celsius: fresh_f64(45.0, 5),
            fan_rpm: fresh_u16(3_200, 6),
        }
    }

    fn fresh_f64(value: f64, prior_sequence: u64) -> Observation<f64> {
        Observation::record_success(
            value,
            BootSessionId::new(7),
            ObservationSequence::new(prior_sequence),
            MonotonicMillis::new(250),
        )
        .expect("fixture sequence should advance")
        .0
    }

    fn fresh_u16(value: u16, prior_sequence: u64) -> Observation<u16> {
        Observation::record_success(
            value,
            BootSessionId::new(7),
            ObservationSequence::new(prior_sequence),
            MonotonicMillis::new(250),
        )
        .expect("fixture sequence should advance")
        .0
    }
}
