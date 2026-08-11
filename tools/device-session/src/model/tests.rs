use serde_json::Value;

use super::*;

fn digest(character: char) -> String {
    std::iter::repeat_n(character, 64).collect()
}

fn baseline() -> BaselineApplication {
    BaselineApplication {
        boot_session: "boot-a".to_owned(),
        boot_ordinal: 7,
        source_commit: "source".to_owned(),
        reference_commit: "reference".to_owned(),
        app_elf_sha256: digest('a'),
        running_partition: None,
    }
}

fn postcondition() -> ExpectedPostcondition {
    ExpectedPostcondition {
        hostname_sha256: digest('b'),
        running_partition: None,
    }
}

fn reboot_intent() -> RebootIntent {
    RebootIntent {
        schema_version: REBOOT_INTENT_SCHEMA.to_owned(),
        board_category: "205".to_owned(),
        trusted_origin: "http://trusted-device".to_owned(),
        baseline: baseline(),
        expected_postcondition: postcondition(),
    }
}

fn ota_intent() -> OtaIntent {
    let mut baseline = baseline();
    baseline.running_partition = Some("factory".to_owned());
    let mut expected_postcondition = postcondition();
    expected_postcondition.running_partition = Some("ota_0".to_owned());
    OtaIntent {
        schema_version: OTA_INTENT_SCHEMA.to_owned(),
        board_category: "205".to_owned(),
        trusted_origin: "http://trusted-device".to_owned(),
        baseline,
        expected_postcondition,
        ota_image_sha256: digest('d'),
    }
}

#[test]
fn reboot_intent_binds_the_internally_admitted_device() {
    // Arrange
    let intent = reboot_intent();
    let physical_identity_digest = digest('c');

    // Act
    let request = intent.bind_device("/dev/cu.usbmodem-test".to_owned(), physical_identity_digest);

    // Assert
    assert!(request.schema_is_valid());
    assert_eq!(request.schema_version, REQUEST_SCHEMA);
    assert_eq!(request.admitted_port, "/dev/cu.usbmodem-test");
    assert_eq!(request.physical_identity_digest, digest('c'));
}

#[test]
fn reboot_intent_rejects_an_external_device_identity_field() {
    // Arrange
    let mut value = serde_json::to_value(reboot_intent()).expect("intent must serialize");
    value["physical_identity_digest"] = Value::String(digest('c'));

    // Act
    let result = serde_json::from_value::<RebootIntent>(value);

    // Assert
    assert!(result.is_err());
}

#[test]
fn reboot_intent_rejects_an_invalid_contract() {
    // Arrange
    let mut intent = reboot_intent();
    intent.board_category = "other".to_owned();

    // Act
    let valid = intent.schema_is_valid();

    // Assert
    assert!(!valid);
}

#[test]
fn ota_intent_requires_the_factory_to_ota_zero_contract() {
    // Arrange
    let valid = ota_intent();
    let mut wrong_target = valid.clone();
    wrong_target.expected_postcondition.running_partition = Some("ota_1".to_owned());

    // Act
    let request = valid.bind_device("/dev/cu.test".to_owned(), digest('c'));

    // Assert
    assert!(request.schema_is_valid());
    assert_eq!(
        request.baseline.running_partition.as_deref(),
        Some("factory")
    );
    assert_eq!(
        request.expected_postcondition.running_partition.as_deref(),
        Some("ota_0")
    );
    assert!(!wrong_target.schema_is_valid());
}

fn boot_b() -> PrivateBootB {
    PrivateBootB {
        boot_session: "boot-b".to_owned(),
        boot_ordinal: 8,
        reset_reason_category: "software_cpu".to_owned(),
        trusted_origin: "http://trusted-device".to_owned(),
        source_commit: "source".to_owned(),
        reference_commit: "reference".to_owned(),
        app_elf_sha256: digest('a'),
        hostname_sha256: digest('b'),
        running_partition: "factory".to_owned(),
    }
}

fn stable_samples(state: &mut SessionState, phase: DevicePhase, token: &str) {
    for _ in 0..3 {
        state.apply(SessionEvent::DeviceSample {
            phase,
            physical_match: PhysicalMatch::UniqueSame,
            enumeration_token: token.to_owned(),
            accessible: true,
            holder_count: 0,
        });
    }
}

fn ready_through_request(state: &mut SessionState) {
    state.apply(SessionEvent::PlatformObserved {
        category: PlatformCategory::Macos,
    });
    stable_samples(state, DevicePhase::Initial, "enumeration-a");
    state.apply(SessionEvent::ReaderArmed);
    state.apply(SessionEvent::SerialBytes {
        phase: SerialPhase::PreRestart,
        count: 16,
    });
    state.apply(SessionEvent::BaselineConfirmed);
    state.apply(SessionEvent::RestartRequestStarted);
    state.apply(SessionEvent::RestartRequestBytesWritten { count: 128 });
    state.apply(SessionEvent::RestartRequestWriteComplete);
}

fn qualify_stable_post_restart_device(state: &mut SessionState) {
    stable_samples(state, DevicePhase::Recovery, "enumeration-a");
}

fn expire(state: &mut SessionState) {
    state.apply(SessionEvent::ObservationWindowExpired {
        duration_millis: 360_000,
    });
    state.apply(SessionEvent::CleanupComplete);
}

#[test]
fn response_missing_can_still_finish_ready_from_authoritative_quorum() {
    // Arrange
    let mut state = SessionState::new(
        baseline(),
        postcondition(),
        "http://trusted-device".to_owned(),
    );
    ready_through_request(&mut state);
    qualify_stable_post_restart_device(&mut state);

    // Act
    state.apply(SessionEvent::BootBObserved { boot_b: boot_b() });
    state.apply(SessionEvent::CleanupComplete);

    // Assert
    assert_eq!(state.terminal_category(), TerminalCategory::Ready);
    assert_eq!(
        state.projection().request_outcome,
        RequestOutcome::ResponseMissing
    );
}

#[test]
fn partial_request_cannot_claim_unrelated_matching_boot_transition() {
    // Arrange
    let mut state = SessionState::new(
        baseline(),
        postcondition(),
        "http://trusted-device".to_owned(),
    );
    state.apply(SessionEvent::PlatformObserved {
        category: PlatformCategory::Macos,
    });
    stable_samples(&mut state, DevicePhase::Initial, "enumeration-a");
    state.apply(SessionEvent::ReaderArmed);
    state.apply(SessionEvent::SerialBytes {
        phase: SerialPhase::PreRestart,
        count: 1,
    });
    state.apply(SessionEvent::BaselineConfirmed);
    state.apply(SessionEvent::RestartRequestStarted);
    state.apply(SessionEvent::RestartRequestBytesWritten { count: 1 });

    // Act
    state.apply(SessionEvent::BootBObserved { boot_b: boot_b() });
    state.apply(SessionEvent::ObservationWindowExpired {
        duration_millis: 360_000,
    });
    state.apply(SessionEvent::CleanupComplete);

    // Assert
    assert_eq!(
        state.terminal_category(),
        TerminalCategory::RestartAttributionAmbiguous
    );
    assert_eq!(
        state.projection().request_outcome,
        RequestOutcome::TransmissionAmbiguous
    );
    assert_eq!(state.projection().request_attempt_count, 1);
}

#[test]
fn disappearance_allows_three_sample_same_device_reacquisition() {
    // Arrange
    let mut state = SessionState::new(
        baseline(),
        postcondition(),
        "http://trusted-device".to_owned(),
    );
    ready_through_request(&mut state);

    // Act
    state.apply(SessionEvent::DeviceAbsent);
    stable_samples(&mut state, DevicePhase::Recovery, "enumeration-b");
    state.apply(SessionEvent::BootBObserved { boot_b: boot_b() });
    state.apply(SessionEvent::CleanupComplete);

    // Assert
    let projection = state.projection();
    assert_eq!(projection.terminal_category, TerminalCategory::Ready);
    assert!(projection.same_physical_device);
    assert!(projection.reenumerated);
    assert_eq!(projection.enumeration_change_count, 1);
}

#[test]
fn multiple_physical_matches_fail_closed_and_cleanup_cannot_replace_terminal() {
    // Arrange
    let mut state = SessionState::new(
        baseline(),
        postcondition(),
        "http://trusted-device".to_owned(),
    );
    state.apply(SessionEvent::PlatformObserved {
        category: PlatformCategory::Macos,
    });

    // Act
    state.apply(SessionEvent::DeviceSample {
        phase: DevicePhase::Initial,
        physical_match: PhysicalMatch::Multiple,
        enumeration_token: String::new(),
        accessible: false,
        holder_count: 0,
    });
    state.apply(SessionEvent::CleanupFailed);

    // Assert
    assert_eq!(
        state.terminal_category(),
        TerminalCategory::UsbIdentityDrift
    );
    assert!(state.private_result().maybe_secondary_cleanup_failure);
}

#[test]
fn second_restart_attempt_is_the_first_terminal_failure() {
    // Arrange
    let mut state = SessionState::new(
        baseline(),
        postcondition(),
        "http://trusted-device".to_owned(),
    );
    ready_through_request(&mut state);

    // Act
    state.apply(SessionEvent::RestartRequestStarted);
    state.apply(SessionEvent::CleanupFailed);

    // Assert
    assert_eq!(
        state.terminal_category(),
        TerminalCategory::RestartAttributionAmbiguous
    );
    assert_eq!(state.projection().request_attempt_count, 2);
}

#[test]
fn restart_request_requires_an_armed_reader_with_pre_restart_delivery() {
    // Arrange
    let mut state = SessionState::new(
        baseline(),
        postcondition(),
        "http://trusted-device".to_owned(),
    );
    state.apply(SessionEvent::PlatformObserved {
        category: PlatformCategory::Macos,
    });
    stable_samples(&mut state, DevicePhase::Initial, "enumeration-a");
    state.apply(SessionEvent::BaselineConfirmed);

    // Act
    state.apply(SessionEvent::RestartRequestStarted);

    // Assert
    assert_eq!(
        state.terminal_category(),
        TerminalCategory::ObserverUnqualified
    );
    assert_eq!(state.projection().request_attempt_count, 1);
    assert!(!state.projection().reader_armed);
}

#[test]
fn non_macos_platforms_are_explicitly_unsupported() {
    for category in [
        PlatformCategory::Linux,
        PlatformCategory::Windows,
        PlatformCategory::Other,
    ] {
        // Arrange
        let mut state = SessionState::new(
            baseline(),
            postcondition(),
            "http://trusted-device".to_owned(),
        );

        // Act
        state.apply(SessionEvent::PlatformObserved { category });

        // Assert
        assert_eq!(
            state.terminal_category(),
            TerminalCategory::ObserverUnqualified
        );
    }
}

#[test]
fn strict_projection_excludes_private_identity_and_application_values() {
    // Arrange
    let mut state = SessionState::new(
        baseline(),
        postcondition(),
        "http://trusted-device".to_owned(),
    );
    ready_through_request(&mut state);
    qualify_stable_post_restart_device(&mut state);
    state.apply(SessionEvent::BootBObserved { boot_b: boot_b() });
    state.apply(SessionEvent::CleanupComplete);

    // Act
    let value = serde_json::to_value(state.projection()).expect("projection must serialize");
    let text = serde_json::to_string(&value).expect("projection must serialize to text");
    let keys = value
        .as_object()
        .expect("projection must be an object")
        .keys()
        .cloned()
        .collect::<Vec<_>>();

    // Assert
    assert!(!text.contains("trusted-device"));
    assert!(!text.contains("boot-a"));
    assert!(!text.contains("boot-b"));
    for private_key in [
        "boot_session",
        "hostname",
        "trusted_origin",
        "physical_identity_digest",
        "admitted_port",
    ] {
        assert!(!keys.iter().any(|key| key == private_key));
    }
    assert_eq!(
        value["terminal_category"],
        Value::String("ready".to_owned())
    );
}

#[test]
fn terminal_precedence_classifies_every_post_request_boundary() {
    struct Case {
        expected: TerminalCategory,
        mutate: fn(&mut SessionState, &mut PrivateBootB),
    }

    fn unchanged(_: &mut SessionState, _: &mut PrivateBootB) {}
    fn missing_usb(state: &mut SessionState, _: &mut PrivateBootB) {
        state.apply(SessionEvent::DeviceAbsent);
    }
    fn missing_service(state: &mut SessionState, _: &mut PrivateBootB) {
        state.apply(SessionEvent::RestartResponseReceived);
    }
    fn wrong_origin(_: &mut SessionState, boot: &mut PrivateBootB) {
        boot.trusted_origin = "http://other-device".to_owned();
    }
    fn wrong_build(_: &mut SessionState, boot: &mut PrivateBootB) {
        boot.source_commit = "other-source".to_owned();
    }
    fn same_session(_: &mut SessionState, boot: &mut PrivateBootB) {
        boot.boot_session = "boot-a".to_owned();
    }
    fn wrong_reset(_: &mut SessionState, boot: &mut PrivateBootB) {
        boot.reset_reason_category = "power_on".to_owned();
    }
    fn wrong_ordinal(_: &mut SessionState, boot: &mut PrivateBootB) {
        boot.boot_ordinal = 9;
    }
    fn wrong_postcondition(_: &mut SessionState, boot: &mut PrivateBootB) {
        boot.hostname_sha256 = digest('c');
    }

    let cases = [
        Case {
            expected: TerminalCategory::UsbIdentityUnavailable,
            mutate: missing_usb,
        },
        Case {
            expected: TerminalCategory::ServiceRecoveryTimeout,
            mutate: missing_service,
        },
        Case {
            expected: TerminalCategory::BootIdentityInvalid,
            mutate: wrong_origin,
        },
        Case {
            expected: TerminalCategory::BuildIdentityMismatch,
            mutate: wrong_build,
        },
        Case {
            expected: TerminalCategory::SessionNotAdvanced,
            mutate: same_session,
        },
        Case {
            expected: TerminalCategory::ResetReasonWrong,
            mutate: wrong_reset,
        },
        Case {
            expected: TerminalCategory::OrdinalNotNext,
            mutate: wrong_ordinal,
        },
        Case {
            expected: TerminalCategory::PostconditionMismatch,
            mutate: wrong_postcondition,
        },
        Case {
            expected: TerminalCategory::Ready,
            mutate: unchanged,
        },
    ];

    for case in cases {
        // Arrange
        let mut state = SessionState::new(
            baseline(),
            postcondition(),
            "http://trusted-device".to_owned(),
        );
        ready_through_request(&mut state);
        qualify_stable_post_restart_device(&mut state);
        let mut observed_boot = boot_b();

        // Act
        (case.mutate)(&mut state, &mut observed_boot);
        if case.expected != TerminalCategory::ServiceRecoveryTimeout {
            state.apply(SessionEvent::BootBObserved {
                boot_b: observed_boot,
            });
        }
        expire(&mut state);

        // Assert
        assert_eq!(state.terminal_category(), case.expected);
    }
}

#[test]
fn ota_partition_mismatch_is_a_postcondition_failure() {
    // Arrange
    let intent = ota_intent();
    let mut state = SessionState::new(
        intent.baseline,
        intent.expected_postcondition,
        intent.trusted_origin,
    );
    ready_through_request(&mut state);
    qualify_stable_post_restart_device(&mut state);
    let mut observed_boot = boot_b();
    observed_boot.running_partition = "ota_1".to_owned();

    // Act
    state.apply(SessionEvent::BootBObserved {
        boot_b: observed_boot,
    });
    expire(&mut state);

    // Assert
    assert_eq!(
        state.terminal_category(),
        TerminalCategory::PostconditionMismatch
    );
}

#[test]
fn missing_restart_attempt_is_classified_before_usb_and_service_facts() {
    // Arrange
    let mut state = SessionState::new(
        baseline(),
        postcondition(),
        "http://trusted-device".to_owned(),
    );
    state.apply(SessionEvent::PlatformObserved {
        category: PlatformCategory::Macos,
    });
    stable_samples(&mut state, DevicePhase::Initial, "enumeration-a");
    state.apply(SessionEvent::ReaderArmed);
    state.apply(SessionEvent::SerialBytes {
        phase: SerialPhase::PreRestart,
        count: 1,
    });

    // Act
    expire(&mut state);

    // Assert
    assert_eq!(
        state.terminal_category(),
        TerminalCategory::RestartRequestNotSent
    );
}

#[test]
fn reader_failure_can_reacquire_same_device_without_claiming_usb_disappearance() {
    // Arrange
    let mut state = SessionState::new(
        baseline(),
        postcondition(),
        "http://trusted-device".to_owned(),
    );
    ready_through_request(&mut state);

    // Act
    state.apply(SessionEvent::ReaderLost);
    stable_samples(&mut state, DevicePhase::Recovery, "enumeration-a");
    state.apply(SessionEvent::ReaderReacquired);
    state.apply(SessionEvent::SerialBytes {
        phase: SerialPhase::PostRestart,
        count: 8,
    });
    state.apply(SessionEvent::BootBObserved { boot_b: boot_b() });
    state.apply(SessionEvent::CleanupComplete);

    // Assert
    let projection = state.projection();
    assert_eq!(projection.terminal_category, TerminalCategory::Ready);
    assert_eq!(projection.serial_delivery, SerialDelivery::Reacquired);
    assert_eq!(projection.usb_disappearance_count, 0);
}

#[test]
fn serial_delivery_cannot_override_missing_http_boot_quorum() {
    // Arrange
    let mut state = SessionState::new(
        baseline(),
        postcondition(),
        "http://trusted-device".to_owned(),
    );
    ready_through_request(&mut state);
    qualify_stable_post_restart_device(&mut state);
    state.apply(SessionEvent::SerialBytes {
        phase: SerialPhase::PostRestart,
        count: 32,
    });
    state.apply(SessionEvent::RestartResponseReceived);

    // Act
    expire(&mut state);

    // Assert
    assert_eq!(
        state.terminal_category(),
        TerminalCategory::ServiceRecoveryTimeout
    );
}
