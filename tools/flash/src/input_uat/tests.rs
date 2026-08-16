use super::*;

const TEST_SOURCE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const TEST_REFERENCE_COMMIT: &str = "abcdef0123456789abcdef0123456789abcdef01";
const TEST_APP_ELF_SHA256: &str =
    "ca16ef5bd57d7e4b2f2f016ffb9236c426e68f16072bc1c5a53ef0e515f1d063";

fn observer() -> InputUatObserver {
    InputUatObserver::new(
        ExpectedRuntimeAttestationIdentity {
            firmware_commit: TEST_SOURCE_COMMIT.to_owned(),
            reference_commit: TEST_REFERENCE_COMMIT.to_owned(),
            app_elf_sha256: TEST_APP_ELF_SHA256.to_owned(),
        },
        true,
        true,
    )
}

fn runtime_attestation_log() -> String {
    let lines = [10_000_u64, 20_000]
        .into_iter()
        .map(|uptime_ms| {
            format!(
                "runtime_boot_attestation schema_version=1 session=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa boot_ordinal=7 reset_reason=other uptime_ms={uptime_ms} board=205 asic=BM1366 mining=disabled work_submission=disabled hardware_control=disabled firmware_commit={TEST_SOURCE_COMMIT} reference_commit={TEST_REFERENCE_COMMIT} app_elf_sha256={TEST_APP_ELF_SHA256} esp_idf_version=v5.5.4 ota_boot_validation=complete spiffs_mount=available api_route_shell=started redacted=true"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("{lines}\n")
}

#[test]
fn fragmented_runtime_attestation_reaches_checkpoint_without_parse_failure() {
    // Arrange
    let mut observer = observer();
    let log = runtime_attestation_log();
    let marker_split = log
        .find("firmware_commit")
        .expect("attestation field")
        .saturating_add(7);
    let newline_split = log.find('\n').expect("first newline").saturating_add(1);

    // Act
    let first = observer.observe_chunk(&log.as_bytes()[..marker_split]);
    let second = observer.observe_chunk(&log.as_bytes()[marker_split..newline_split]);
    let ready = observer.observe_chunk(&log.as_bytes()[newline_split..]);

    // Assert
    assert_eq!(first, InputUatAction::Continue);
    assert_eq!(second, InputUatAction::Continue);
    assert_eq!(ready, InputUatAction::PublishCheckpoint);
    assert_eq!(observer.maybe_failure, None);
}

#[test]
fn precheckpoint_click_is_ignored_and_later_short_click_completes() {
    // Arrange
    let mut observer = observer();
    let identity = runtime_attestation_log();
    let first = format!("{identity}\ninput_event=short_click effect=screen_advance\n");

    // Act
    let ready = observer.observe_chunk(first.as_bytes());
    observer.publish_checkpoint();
    let complete =
        observer.observe_chunk(b"I (123) input: input_event=short_click effect=screen_advance\n");

    // Assert
    assert_eq!(ready, InputUatAction::PublishCheckpoint);
    assert_eq!(complete, InputUatAction::Stop);
    assert_eq!(observer.short_click_count, 1);
    assert!(observer.complete());
}

#[test]
fn duplicate_and_long_press_markers_fail_closed() {
    // Arrange
    let mut duplicate = observer();
    duplicate.observe_chunk(runtime_attestation_log().as_bytes());
    duplicate.publish_checkpoint();
    let mut long = observer();
    long.observe_chunk(runtime_attestation_log().as_bytes());
    long.publish_checkpoint();

    // Act
    duplicate.observe_chunk(
        b"input_event=short_click effect=screen_advance\ninput_event=short_click effect=screen_advance\n",
    );
    long.observe_chunk(b"input_event=long_press effect=configuration_ap_toggle ap_enabled=true\n");

    // Assert
    assert_eq!(
        duplicate.maybe_failure,
        Some(InputUatFailure::DuplicateShortClick)
    );
    assert_eq!(long.maybe_failure, Some(InputUatFailure::LongPressObserved));
}

#[test]
fn stale_or_malformed_attestation_blocks_checkpoint() {
    // Arrange
    let mut observer = observer();
    let malformed = runtime_attestation_log().replace("board=205", "board=999");

    // Act
    let action = observer.observe_chunk(malformed.as_bytes());

    // Assert
    assert_eq!(action, InputUatAction::Stop);
    assert_eq!(
        observer.maybe_failure,
        Some(InputUatFailure::RuntimeAttestationInvalid)
    );
}
