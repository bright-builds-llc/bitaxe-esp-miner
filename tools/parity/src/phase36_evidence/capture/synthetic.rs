use bitaxe_device_session::{
    BaselineApplication, DevicePhase, ExpectedPostcondition, PhysicalMatch, PlatformCategory,
    PrivateBootB, SerialPhase, SessionEvent, SessionRequest, SessionState, REQUEST_SCHEMA,
};
use camino::Utf8Path;

use super::filesystem::{write_new_private_file, CaptureFileError};
use super::{
    write_candidate_from_private_file, BrokerCaptureDocument, CaptureObservationSource,
    Phase36CaptureCandidate, Phase36PrivateCaptureBundle, RuntimeIdentityCaptureDocuments,
    SubstantiveCaptureDocuments, PHASE36_PRIVATE_CAPTURE_SCHEMA,
};
use crate::phase36_broker::{
    Phase36AllowedOperation, Phase36LedgerRecord, Phase36LedgerState, Phase36LedgerTransition,
};

const SESSION: &str = "0123456789abcdef0011223344556677";
const SOURCE_COMMIT: &str = "1111111111111111111111111111111111111111";
const REFERENCE_COMMIT: &str = "2222222222222222222222222222222222222222";
const MANIFEST_DIGEST: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const EXECUTABLE_DIGEST: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const FACTORY_DIGEST: &str = "5555555555555555555555555555555555555555555555555555555555555555";
const ELF_DIGEST: &str = "6666666666666666666666666666666666666666666666666666666666666666";
const PACKAGE_DIGEST: &str = "7777777777777777777777777777777777777777777777777777777777777777";
const PHYSICAL_DIGEST: &str = "8888888888888888888888888888888888888888888888888888888888888888";
const HOST_DIGEST: &str = "9999999999999999999999999999999999999999999999999999999999999999";

pub fn write_synthetic_capture(
    private_output: &Utf8Path,
    candidate_output: &Utf8Path,
) -> Result<Phase36CaptureCandidate, CaptureFileError> {
    let bundle = synthetic_bundle()?;
    let bytes = serde_json::to_vec_pretty(&bundle).map_err(|_| CaptureFileError::OutputFailed)?;
    write_new_private_file(private_output, &bytes)?;
    write_candidate_from_private_file(private_output, candidate_output)
}

fn synthetic_bundle() -> Result<Phase36PrivateCaptureBundle, CaptureFileError> {
    let projection = serde_json::json!({
        "bootSession": SESSION,
        "operatorSnapshotRevision": 15,
        "current": 1.25,
        "voltage": 5.1,
        "power": 6.375,
        "temp": 55.25,
        "fanrpm": 4800,
        "currentStatus": {"state":"fresh","stamp":{"bootSession":7,"sequence":11,"acquiredAtMs":1100}},
        "voltageStatus": {"state":"fresh","stamp":{"bootSession":7,"sequence":11,"acquiredAtMs":1100}},
        "powerStatus": {"state":"fresh","stamp":{"bootSession":7,"sequence":11,"acquiredAtMs":1100}},
        "chipTempStatus": {"state":"fresh","stamp":{"bootSession":7,"sequence":12,"acquiredAtMs":1200}},
        "fanRpmStatus": {"state":"fresh","stamp":{"bootSession":7,"sequence":13,"acquiredAtMs":1300}},
        "runtimeHealth": {
            "selfTestState":"idle",
            "supervisorAvailability":"available",
            "checkpointCategory":"telemetry",
            "checkpointSequence":14,
            "checkpointAgeMillis":250,
            "checkpointHealth":"healthy",
            "taskWatchdogParticipation":"unavailable",
            "taskWatchdogReason":"unproved"
        }
    });
    let json = serde_json::to_string(&projection).map_err(|_| CaptureFileError::OutputFailed)?;
    let substantive = SubstantiveCaptureDocuments {
        system_info_document: format!(
            "system_info_json: {json}\noperator_snapshot_boot_session: {SESSION}\noperator_snapshot_revision: 15\n"
        ),
        websocket_document: format!(
            "live_websocket_json: {json}\noperator_snapshot_boot_session: {SESSION}\noperator_snapshot_revision: 15\n"
        ),
        retained_document: format!(
            "operator_snapshot session={SESSION} revision=15 redacted=true\nsubstantive_snapshot_json: {json}\n"
        ),
    };
    Ok(Phase36PrivateCaptureBundle {
        schema_version: PHASE36_PRIVATE_CAPTURE_SCHEMA.to_owned(),
        board_category: "205".to_owned(),
        substantive,
        runtime_identity: synthetic_runtime_identity()?,
        broker: synthetic_broker()?,
    })
}

fn synthetic_runtime_identity() -> Result<RuntimeIdentityCaptureDocuments, CaptureFileError> {
    let request = SessionRequest {
        schema_version: REQUEST_SCHEMA.to_owned(),
        board_category: "205".to_owned(),
        admitted_port: "synthetic-protected-port".to_owned(),
        physical_identity_digest: PHYSICAL_DIGEST.to_owned(),
        trusted_origin: "https://synthetic-protected-origin.invalid".to_owned(),
        baseline: BaselineApplication {
            boot_session: "synthetic-boot-a".to_owned(),
            boot_ordinal: 10,
            source_commit: SOURCE_COMMIT.to_owned(),
            reference_commit: REFERENCE_COMMIT.to_owned(),
            app_elf_sha256: ELF_DIGEST.to_owned(),
        },
        expected_postcondition: ExpectedPostcondition {
            hostname_sha256: HOST_DIGEST.to_owned(),
        },
    };
    let boot_b = PrivateBootB {
        boot_session: SESSION.to_owned(),
        boot_ordinal: 11,
        reset_reason_category: "software_cpu".to_owned(),
        trusted_origin: request.trusted_origin.clone(),
        source_commit: SOURCE_COMMIT.to_owned(),
        reference_commit: REFERENCE_COMMIT.to_owned(),
        app_elf_sha256: ELF_DIGEST.to_owned(),
        hostname_sha256: HOST_DIGEST.to_owned(),
    };
    let sample = |phase| SessionEvent::DeviceSample {
        phase,
        physical_match: PhysicalMatch::UniqueSame,
        enumeration_token: "synthetic-protected-enumeration".to_owned(),
        accessible: true,
        holder_count: 0,
    };
    let events = vec![
        SessionEvent::PlatformObserved {
            category: PlatformCategory::Macos,
        },
        sample(DevicePhase::Initial),
        sample(DevicePhase::Initial),
        sample(DevicePhase::Initial),
        SessionEvent::ReaderArmed,
        SessionEvent::SerialBytes {
            phase: SerialPhase::PreRestart,
            count: 10,
        },
        SessionEvent::BaselineConfirmed,
        SessionEvent::RestartRequestStarted,
        SessionEvent::RestartRequestBytesWritten { count: 10 },
        SessionEvent::RestartRequestWriteComplete,
        SessionEvent::RestartResponseReceived,
        SessionEvent::ServiceLossObserved,
        SessionEvent::DeviceAbsent,
        sample(DevicePhase::Recovery),
        sample(DevicePhase::Recovery),
        sample(DevicePhase::Recovery),
        SessionEvent::ReaderReacquired,
        SessionEvent::SerialBytes {
            phase: SerialPhase::PostRestart,
            count: 20,
        },
        SessionEvent::BootBObserved { boot_b },
        SessionEvent::ObservationWindowExpired {
            duration_millis: 1_000,
        },
        SessionEvent::CleanupComplete,
    ];
    let mut state = SessionState::new(
        request.baseline.clone(),
        request.expected_postcondition.clone(),
        request.trusted_origin.clone(),
    );
    for event in events.iter().cloned() {
        state.apply(event);
    }
    let event_ledger_document = events
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| CaptureFileError::OutputFailed)?
        .join("\n");
    let exact_package_document = serde_json::json!({
        "schema_version":"phase36-runtime-package-v1",
        "source_commit":SOURCE_COMMIT,
        "reference_commit":REFERENCE_COMMIT,
        "manifest_digest":MANIFEST_DIGEST,
        "executable_image_digest":EXECUTABLE_DIGEST,
        "factory_image_digest":FACTORY_DIGEST,
        "firmware_elf_digest":ELF_DIGEST,
        "package_digest":PACKAGE_DIGEST
    });
    Ok(RuntimeIdentityCaptureDocuments {
        exact_package_document: serde_json::to_string(&exact_package_document)
            .map_err(|_| CaptureFileError::OutputFailed)?,
        request_document: serde_json::to_string(&request)
            .map_err(|_| CaptureFileError::OutputFailed)?,
        event_ledger_document,
        private_result_document: serde_json::to_string(&state.private_result())
            .map_err(|_| CaptureFileError::OutputFailed)?,
        public_projection_document: serde_json::to_string(&state.projection())
            .map_err(|_| CaptureFileError::OutputFailed)?,
    })
}

fn synthetic_broker() -> Result<BrokerCaptureDocument, CaptureFileError> {
    let mut state =
        Phase36LedgerState::start(100).map_err(|_| CaptureFileError::ClassificationFailed)?;
    let mut records = Vec::new();
    let mut millis = 100;
    for operation in Phase36AllowedOperation::SUCCESS_ORDER {
        for transition in [
            Phase36LedgerTransition::Authorized,
            Phase36LedgerTransition::Invoked,
            Phase36LedgerTransition::Completed,
            Phase36LedgerTransition::Closed,
        ] {
            millis += 1;
            let record = Phase36LedgerRecord::next(&state, operation, transition, millis)
                .map_err(|_| CaptureFileError::ClassificationFailed)?;
            state
                .apply(&record)
                .map_err(|_| CaptureFileError::ClassificationFailed)?;
            records.push(record);
        }
    }
    Ok(BrokerCaptureDocument {
        observation_source: CaptureObservationSource::IndependentBrokerLedger,
        package_digest: PACKAGE_DIGEST.to_owned(),
        same_physical_device_observed: true,
        interval_start_millis: 100,
        interval_end_millis: millis + 1,
        records,
    })
}
