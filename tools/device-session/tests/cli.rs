use std::fs;
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::process::Command;

use bitaxe_device_session::{
    BaselineApplication, DevicePhase, ExpectedPostcondition, FixtureTranscript, PhysicalMatch,
    PlatformCategory, PrivateBootB, SerialPhase, SessionEvent, SessionRequest, FIXTURE_SCHEMA,
    REQUEST_SCHEMA,
};
use camino::Utf8Path;
use serde::Serialize;

fn digest(character: char) -> String {
    std::iter::repeat_n(character, 64).collect()
}

fn request() -> SessionRequest {
    SessionRequest {
        schema_version: REQUEST_SCHEMA.to_owned(),
        board_category: "205".to_owned(),
        admitted_port: "/private/device-node".to_owned(),
        physical_identity_digest: digest('c'),
        trusted_origin: "http://private-device".to_owned(),
        baseline: BaselineApplication {
            boot_session: "private-boot-a".to_owned(),
            boot_ordinal: 41,
            source_commit: "source-commit".to_owned(),
            reference_commit: "reference-commit".to_owned(),
            app_elf_sha256: digest('a'),
        },
        expected_postcondition: ExpectedPostcondition {
            hostname_sha256: digest('b'),
        },
    }
}

fn fixture() -> FixtureTranscript {
    let mut events = vec![SessionEvent::PlatformObserved {
        category: PlatformCategory::Macos,
    }];
    for _ in 0..3 {
        events.push(SessionEvent::DeviceSample {
            phase: DevicePhase::Initial,
            physical_match: PhysicalMatch::UniqueSame,
            enumeration_token: "private-enumeration-a".to_owned(),
            accessible: true,
            holder_count: 0,
        });
    }
    events.extend([
        SessionEvent::ReaderArmed,
        SessionEvent::SerialBytes {
            phase: SerialPhase::PreRestart,
            count: 16,
        },
        SessionEvent::BaselineConfirmed,
        SessionEvent::RestartRequestStarted,
        SessionEvent::RestartRequestBytesWritten { count: 128 },
        SessionEvent::RestartRequestWriteComplete,
        SessionEvent::DeviceAbsent,
    ]);
    for _ in 0..3 {
        events.push(SessionEvent::DeviceSample {
            phase: DevicePhase::Recovery,
            physical_match: PhysicalMatch::UniqueSame,
            enumeration_token: "private-enumeration-b".to_owned(),
            accessible: true,
            holder_count: 0,
        });
    }
    events.extend([
        SessionEvent::BootBObserved {
            boot_b: PrivateBootB {
                boot_session: "private-boot-b".to_owned(),
                boot_ordinal: 42,
                reset_reason_category: "software_cpu".to_owned(),
                trusted_origin: "http://private-device".to_owned(),
                source_commit: "source-commit".to_owned(),
                reference_commit: "reference-commit".to_owned(),
                app_elf_sha256: digest('a'),
                hostname_sha256: digest('b'),
            },
        },
        SessionEvent::CleanupComplete,
    ]);
    FixtureTranscript {
        schema_version: FIXTURE_SCHEMA.to_owned(),
        events,
    }
}

fn stable_events(phase: DevicePhase, token: &str) -> Vec<SessionEvent> {
    (0..3)
        .map(|_| SessionEvent::DeviceSample {
            phase,
            physical_match: PhysicalMatch::UniqueSame,
            enumeration_token: token.to_owned(),
            accessible: true,
            holder_count: 0,
        })
        .collect()
}

fn request_complete_events() -> Vec<SessionEvent> {
    let mut events = vec![SessionEvent::PlatformObserved {
        category: PlatformCategory::Macos,
    }];
    events.extend(stable_events(DevicePhase::Initial, "private-enumeration-a"));
    events.extend([
        SessionEvent::ReaderArmed,
        SessionEvent::SerialBytes {
            phase: SerialPhase::PreRestart,
            count: 16,
        },
        SessionEvent::BaselineConfirmed,
        SessionEvent::RestartRequestStarted,
        SessionEvent::RestartRequestBytesWritten { count: 128 },
        SessionEvent::RestartRequestWriteComplete,
    ]);
    events
}

fn private_boot_b() -> PrivateBootB {
    PrivateBootB {
        boot_session: "private-boot-b".to_owned(),
        boot_ordinal: 42,
        reset_reason_category: "software_cpu".to_owned(),
        trusted_origin: "http://private-device".to_owned(),
        source_commit: "source-commit".to_owned(),
        reference_commit: "reference-commit".to_owned(),
        app_elf_sha256: digest('a'),
        hostname_sha256: digest('b'),
    }
}

fn write_private_json(path: &Utf8Path, value: &impl Serialize) {
    fs::write(
        path.as_std_path(),
        serde_json::to_vec(value).expect("fixture must serialize"),
    )
    .expect("private fixture must be writable");
    #[cfg(unix)]
    fs::set_permissions(path.as_std_path(), fs::Permissions::from_mode(0o600))
        .expect("private fixture mode must be set");
}

#[test]
fn built_cli_applies_fixture_through_private_and_public_evidence_boundaries() {
    // Arrange
    let temporary = tempfile::tempdir().expect("temporary directory must be available");
    let temporary = Utf8Path::from_path(temporary.path()).expect("temporary path must be UTF-8");
    let request_path = temporary.join("request.json");
    let fixture_path = temporary.join("fixture.json");
    let private_root = temporary.join("private-root");
    let projection_path = temporary.join("projection.json");
    write_private_json(&request_path, &request());
    write_private_json(&fixture_path, &fixture());
    fs::create_dir(private_root.as_std_path()).expect("private root must be created");
    #[cfg(unix)]
    fs::set_permissions(
        private_root.as_std_path(),
        fs::Permissions::from_mode(0o700),
    )
    .expect("private root mode must be set");

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_device-session"))
        .args([
            "reboot",
            "--private-root",
            private_root.as_str(),
            "--request-input",
            request_path.as_str(),
            "--projection-output",
            projection_path.as_str(),
            "--fixture-input",
            fixture_path.as_str(),
        ])
        .output()
        .expect("device-session CLI must launch");

    // Assert
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    #[cfg(unix)]
    assert_eq!(
        fs::metadata(private_root.as_std_path())
            .expect("root metadata")
            .mode()
            & 0o777,
        0o700
    );
    for name in [
        "events.private.jsonl",
        "http.private.jsonl",
        "serial.private.bin",
        "result.private.json",
    ] {
        #[cfg(unix)]
        assert_eq!(
            fs::metadata(private_root.join(name).as_std_path())
                .expect("private artifact metadata")
                .mode()
                & 0o777,
            0o600
        );
    }
    #[cfg(unix)]
    assert_eq!(
        fs::metadata(projection_path.as_std_path())
            .expect("projection metadata")
            .mode()
            & 0o777,
        0o600
    );

    let projection_text = fs::read_to_string(projection_path.as_std_path()).expect("projection");
    assert!(!projection_text.contains("private-device"));
    assert!(!projection_text.contains("private-boot"));
    assert!(!projection_text.contains("private-enumeration"));
    assert!(!projection_text.contains("source-commit"));
    let projection: serde_json::Value =
        serde_json::from_str(&projection_text).expect("projection JSON");
    assert_eq!(projection["terminal_category"], "ready");
    assert_eq!(projection["request_outcome"], "response_missing");

    let private_result_text =
        fs::read_to_string(private_root.join("result.private.json").as_std_path())
            .expect("private result");
    let private_result: serde_json::Value =
        serde_json::from_str(&private_result_text).expect("private result JSON");
    assert_eq!(private_result["boot_b"]["boot_session"], "private-boot-b");
    assert_eq!(private_result["boot_b"]["boot_ordinal"], 42);
    assert_eq!(
        private_result["boot_b"]["reset_reason_category"],
        "software_cpu"
    );
    assert_eq!(
        private_result["boot_b"]["trusted_origin"],
        "http://private-device"
    );
    assert_eq!(private_result["boot_b"]["source_commit"], "source-commit");
    assert_eq!(
        private_result["boot_b"]["reference_commit"],
        "reference-commit"
    );
    assert_eq!(private_result["boot_b"]["app_elf_sha256"], digest('a'));
    assert_eq!(private_result["boot_b"]["hostname_sha256"], digest('b'));

    // Act: a second invocation targets the already-populated protected root.
    let repeated = Command::new(env!("CARGO_BIN_EXE_device-session"))
        .args([
            "reboot",
            "--private-root",
            private_root.as_str(),
            "--request-input",
            request_path.as_str(),
            "--projection-output",
            projection_path.as_str(),
            "--fixture-input",
            fixture_path.as_str(),
        ])
        .output()
        .expect("repeated device-session CLI must launch");

    // Assert: no-clobber admission fails without echoing private inputs.
    assert!(!repeated.status.success());
    assert!(repeated.stdout.is_empty());
    let repeated_stderr = String::from_utf8(repeated.stderr).expect("stderr must be UTF-8");
    assert_eq!(
        repeated_stderr,
        "device_session_status=failed category=host_error\n"
    );
    assert_eq!(
        fs::read_to_string(private_root.join("result.private.json").as_std_path())
            .expect("private result must remain readable"),
        private_result_text
    );
}

struct RebootBoundaryCase {
    name: &'static str,
    events: Vec<SessionEvent>,
    expected_category: &'static str,
    maybe_serial_delivery: Option<&'static str>,
}

fn reboot_boundary_cases() -> Vec<RebootBoundaryCase> {
    let mut stable_silent = request_complete_events();
    stable_silent.extend(stable_events(
        DevicePhase::Recovery,
        "private-enumeration-a",
    ));
    stable_silent.extend([
        SessionEvent::BootBObserved {
            boot_b: private_boot_b(),
        },
        SessionEvent::CleanupComplete,
    ]);

    let mut reader_reacquired = request_complete_events();
    reader_reacquired.push(SessionEvent::ReaderLost);
    reader_reacquired.extend(stable_events(
        DevicePhase::Recovery,
        "private-enumeration-b",
    ));
    reader_reacquired.extend([
        SessionEvent::ReaderReacquired,
        SessionEvent::SerialBytes {
            phase: SerialPhase::PostRestart,
            count: 8,
        },
        SessionEvent::BootBObserved {
            boot_b: private_boot_b(),
        },
        SessionEvent::CleanupComplete,
    ]);

    let mut partial_request = request_complete_events();
    let removed = partial_request.pop();
    assert!(matches!(
        removed,
        Some(SessionEvent::RestartRequestWriteComplete)
    ));
    partial_request.extend(stable_events(
        DevicePhase::Recovery,
        "private-enumeration-a",
    ));
    partial_request.extend([
        SessionEvent::BootBObserved {
            boot_b: private_boot_b(),
        },
        SessionEvent::ObservationWindowExpired {
            duration_millis: 360_000,
        },
        SessionEvent::CleanupComplete,
    ]);

    let mut identity_drift = request_complete_events();
    identity_drift.extend([
        SessionEvent::DeviceSample {
            phase: DevicePhase::Recovery,
            physical_match: PhysicalMatch::UniqueDifferent,
            enumeration_token: "private-drifted-enumeration".to_owned(),
            accessible: true,
            holder_count: 0,
        },
        SessionEvent::CleanupComplete,
    ]);

    let mut holder_conflict = vec![SessionEvent::PlatformObserved {
        category: PlatformCategory::Macos,
    }];
    holder_conflict.push(SessionEvent::DeviceSample {
        phase: DevicePhase::Initial,
        physical_match: PhysicalMatch::UniqueSame,
        enumeration_token: "private-enumeration-a".to_owned(),
        accessible: true,
        holder_count: 1,
    });
    holder_conflict.push(SessionEvent::CleanupComplete);

    let mut service_timeout = request_complete_events();
    service_timeout.extend(stable_events(
        DevicePhase::Recovery,
        "private-enumeration-a",
    ));
    service_timeout.extend([
        SessionEvent::RestartResponseReceived,
        SessionEvent::ObservationWindowExpired {
            duration_millis: 360_000,
        },
        SessionEvent::CleanupComplete,
    ]);

    vec![
        RebootBoundaryCase {
            name: "stable-silent",
            events: stable_silent,
            expected_category: "ready",
            maybe_serial_delivery: Some("silent"),
        },
        RebootBoundaryCase {
            name: "reader-reacquired",
            events: reader_reacquired,
            expected_category: "ready",
            maybe_serial_delivery: Some("reacquired"),
        },
        RebootBoundaryCase {
            name: "partial-request",
            events: partial_request,
            expected_category: "restart_attribution_ambiguous",
            maybe_serial_delivery: None,
        },
        RebootBoundaryCase {
            name: "identity-drift",
            events: identity_drift,
            expected_category: "usb_identity_drift",
            maybe_serial_delivery: None,
        },
        RebootBoundaryCase {
            name: "holder-conflict",
            events: holder_conflict,
            expected_category: "observer_unqualified",
            maybe_serial_delivery: None,
        },
        RebootBoundaryCase {
            name: "service-timeout",
            events: service_timeout,
            expected_category: "service_recovery_timeout",
            maybe_serial_delivery: None,
        },
    ]
}

fn assert_reboot_boundary_case(case: RebootBoundaryCase) {
    // Arrange
    let temporary = tempfile::tempdir().expect("temporary directory must be available");
    let temporary = Utf8Path::from_path(temporary.path()).expect("temporary path must be UTF-8");
    let request_path = temporary.join(format!("{}-request.json", case.name));
    let fixture_path = temporary.join(format!("{}-fixture.json", case.name));
    let private_root = temporary.join(format!("{}-private", case.name));
    let projection_path = temporary.join(format!("{}-projection.json", case.name));
    write_private_json(&request_path, &request());
    write_private_json(
        &fixture_path,
        &FixtureTranscript {
            schema_version: FIXTURE_SCHEMA.to_owned(),
            events: case.events,
        },
    );
    fs::create_dir(private_root.as_std_path()).expect("private root must be created");
    #[cfg(unix)]
    fs::set_permissions(
        private_root.as_std_path(),
        fs::Permissions::from_mode(0o700),
    )
    .expect("private root mode must be set");

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_device-session"))
        .args([
            "reboot",
            "--private-root",
            private_root.as_str(),
            "--request-input",
            request_path.as_str(),
            "--projection-output",
            projection_path.as_str(),
            "--fixture-input",
            fixture_path.as_str(),
        ])
        .output()
        .expect("device-session CLI must launch");

    // Assert
    assert_eq!(
        output.status.success(),
        case.expected_category == "ready",
        "case={}",
        case.name
    );
    assert!(output.stdout.is_empty(), "case={}", case.name);
    let stderr = String::from_utf8(output.stderr).expect("stderr must remain UTF-8");
    assert!(!stderr.contains("private-"), "case={}", case.name);
    assert!(!stderr.contains("http://"), "case={}", case.name);
    let projection_text =
        fs::read_to_string(projection_path.as_std_path()).expect("projection must be readable");
    assert!(!projection_text.contains("private-"), "case={}", case.name);
    let projection: serde_json::Value =
        serde_json::from_str(&projection_text).expect("projection must be JSON");
    assert_eq!(
        projection["terminal_category"], case.expected_category,
        "case={}",
        case.name
    );
    if let Some(serial_delivery) = case.maybe_serial_delivery {
        assert_eq!(
            projection["serial_delivery"], serial_delivery,
            "case={}",
            case.name
        );
    }
}

#[test]
fn built_cli_preserves_typed_reboot_boundaries_without_private_terminal_leakage() {
    // Arrange
    let cases = reboot_boundary_cases();

    // Act / Assert
    for case in cases {
        assert_reboot_boundary_case(case);
    }
}
