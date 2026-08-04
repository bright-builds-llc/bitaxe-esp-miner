const STARTUP_SOURCE: &str = include_str!("startup.rs");
const RUNTIME_SOURCE: &str = include_str!("statistics_runtime.rs");
const SNAPSHOT_SOURCE: &str = include_str!("runtime_snapshot.rs");
const SETTINGS_SOURCE: &str = include_str!("settings_adapter.rs");
const HTTP_HANDLER_SOURCE: &str = include_str!("http_api/handlers.rs");

#[test]
fn startup_creates_exactly_one_statistics_producer() {
    // Arrange
    let start_call = "statistics_runtime::start()";

    // Act
    let startup_count = STARTUP_SOURCE.matches(start_call).count();

    // Assert
    assert_eq!(startup_count, 1);
    assert_eq!(RUNTIME_SOURCE.matches("thread::Builder::new()").count(), 1);
    assert!(RUNTIME_SOURCE.contains("PRODUCER_THREAD_NAME: &str = \"statistics\""));
}

#[test]
fn producer_uses_absolute_one_second_deadlines_and_confirmed_frequency() {
    // Arrange
    let cadence = "pub const STATISTICS_CADENCE_MS: u64 = 1_000";

    // Act
    let cadence_count = RUNTIME_SOURCE.matches(cadence).count();

    // Assert
    assert_eq!(cadence_count, 1);
    assert!(RUNTIME_SOURCE.contains("PeriodicDeadline::new"));
    assert!(RUNTIME_SOURCE.contains("schedule.advance_past"));
    assert!(RUNTIME_SOURCE.contains("statistics_frequency_seconds()"));
    assert!(RUNTIME_SOURCE.contains("record_statistics_sample(now_ms, frequency_seconds)"));
}

#[test]
fn frequency_reader_uses_only_confirmed_settings_snapshot() {
    // Arrange
    let function = "pub fn statistics_frequency_seconds() -> u16";
    let start = SETTINGS_SOURCE.find(function).expect("frequency reader must exist");
    let source = &SETTINGS_SOURCE[start..];

    // Act
    let confirmed_read = source.find("current_settings_snapshot()");
    let exact_key = source.find("\"statsFrequency\"");

    // Assert
    assert!(confirmed_read.is_some());
    assert!(exact_key.is_some());
    assert!(confirmed_read < exact_key);
}

#[test]
fn http_statistics_reads_history_without_recording_or_draining() {
    // Arrange
    let function = "pub fn projected_statistics(timestamp_ms: u64) -> StatisticsWire";
    let start = SNAPSHOT_SOURCE.find(function).expect("projection must exist");
    let end = SNAPSHOT_SOURCE[start..]
        .find("/// Records one producer-cadence statistics sample")
        .map(|offset| start + offset)
        .expect("projection boundary must exist");
    let source = &SNAPSHOT_SOURCE[start..end];

    // Act
    let history_read = source.matches("statistics_samples()").count();

    // Assert
    assert_eq!(history_read, 1);
    assert!(!source.contains("record_statistics_sample"));
    assert!(!source.contains("collect_projected_api_views"));
    assert!(!source.contains("maybe_drain"));
    assert!(HTTP_HANDLER_SOURCE.contains("projected_statistics(timestamp_ms)"));
}

#[test]
fn history_storage_is_separate_from_command_visible_state() {
    // Arrange
    let storage = "static STATISTICS_HISTORY: OnceLock<Mutex<StatisticsHistory>>";

    // Act
    let storage_count = SNAPSHOT_SOURCE.matches(storage).count();

    // Assert
    assert_eq!(storage_count, 1);
    assert!(SNAPSHOT_SOURCE.contains("collect_operator_snapshot_candidate(false)"));
    assert!(SNAPSHOT_SOURCE.contains("StatisticsSample::from_snapshot"));
    assert!(!SNAPSHOT_SOURCE.contains("statistics_history: StatisticsHistory"));
}
