use bitaxe_device_session::{
    BaselineApplication, DevicePhase, ExpectedPostcondition, PhysicalMatch, PlatformCategory,
    PrivateBootB, SerialPhase, SessionEvent, SessionRequest, SessionState, REQUEST_SCHEMA,
};
use serde_json::Value;

use super::super::runtime_identity::{
    validate_observed_runtime_identity_documents, ObservedRuntimeIdentityAdmission,
    RuntimeIdentityEvidenceError, RuntimeIdentityObservationSource,
};
use super::*;

const PACKAGE: &str = include_str!("../../../fixtures/phase36/runtime-identity-package.json");
const SOURCE_COMMIT: &str = "1111111111111111111111111111111111111111";
const REFERENCE_COMMIT: &str = "2222222222222222222222222222222222222222";
const APP_ELF: &str = "6666666666666666666666666666666666666666666666666666666666666666";
const PHYSICAL_IDENTITY: &str = "8888888888888888888888888888888888888888888888888888888888888888";
const HOSTNAME_DIGEST: &str = "9999999999999999999999999999999999999999999999999999999999999999";
const PROTECTED_CANARY: &str = "phase36-runtime-identity-protected-canary";

struct Documents {
    package: String,
    request: String,
    ledger: String,
    private_result: String,
    public_projection: String,
}

impl Documents {
    fn validate(
        &self,
        include_ledger: bool,
    ) -> Result<ObservedRuntimeIdentityAdmission, RuntimeIdentityEvidenceError> {
        validate_observed_runtime_identity_documents(
            &self.package,
            Some(&self.request),
            include_ledger.then_some(self.ledger.as_str()),
            Some(&self.private_result),
            Some(&self.public_projection),
        )
    }
}

fn request() -> SessionRequest {
    SessionRequest {
        schema_version: REQUEST_SCHEMA.to_owned(),
        board_category: "205".to_owned(),
        admitted_port: PROTECTED_CANARY.to_owned(),
        physical_identity_digest: PHYSICAL_IDENTITY.to_owned(),
        trusted_origin: format!("https://{PROTECTED_CANARY}.invalid"),
        baseline: BaselineApplication {
            boot_session: "boot-a".to_owned(),
            boot_ordinal: 10,
            source_commit: SOURCE_COMMIT.to_owned(),
            reference_commit: REFERENCE_COMMIT.to_owned(),
            app_elf_sha256: APP_ELF.to_owned(),
        },
        expected_postcondition: ExpectedPostcondition {
            hostname_sha256: HOSTNAME_DIGEST.to_owned(),
        },
    }
}

fn boot_b(request: &SessionRequest) -> PrivateBootB {
    PrivateBootB {
        boot_session: "boot-b".to_owned(),
        boot_ordinal: 11,
        reset_reason_category: "software_cpu".to_owned(),
        trusted_origin: request.trusted_origin.clone(),
        source_commit: SOURCE_COMMIT.to_owned(),
        reference_commit: REFERENCE_COMMIT.to_owned(),
        app_elf_sha256: APP_ELF.to_owned(),
        hostname_sha256: HOSTNAME_DIGEST.to_owned(),
    }
}

fn eligible_events(request: &SessionRequest) -> Vec<SessionEvent> {
    let sample = |phase| SessionEvent::DeviceSample {
        phase,
        physical_match: PhysicalMatch::UniqueSame,
        enumeration_token: PROTECTED_CANARY.to_owned(),
        accessible: true,
        holder_count: 0,
    };
    vec![
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
        SessionEvent::BootBObserved {
            boot_b: boot_b(request),
        },
        SessionEvent::ObservationWindowExpired {
            duration_millis: 1_000,
        },
        SessionEvent::CleanupComplete,
    ]
}

fn documents() -> Documents {
    let request = request();
    let events = eligible_events(&request);
    let mut state = SessionState::new(
        request.baseline.clone(),
        request.expected_postcondition.clone(),
        request.trusted_origin.clone(),
    );
    for event in events.iter().cloned() {
        state.apply(event);
    }
    let ledger = events
        .iter()
        .map(|event| serde_json::to_string(event).expect("event should serialize"))
        .collect::<Vec<_>>()
        .join("\n");
    Documents {
        package: PACKAGE.to_owned(),
        request: serde_json::to_string(&request).expect("request should serialize"),
        ledger,
        private_result: serde_json::to_string(&state.private_result())
            .expect("private result should serialize"),
        public_projection: serde_json::to_string(&state.projection())
            .expect("public projection should serialize"),
    }
}

fn mutate_json(document: &str, mutate: impl FnOnce(&mut Value)) -> String {
    let mut value = serde_json::from_str(document).expect("document should be JSON");
    mutate(&mut value);
    serde_json::to_string(&value).expect("mutated document should serialize")
}

fn assert_package_drift_is_rejected(field: &str, replacement: &str) {
    let mut changed = documents();
    changed.package = mutate_json(&changed.package, |value| {
        value[field] = Value::String(replacement.to_owned());
    });
    assert_eq!(
        changed.validate(true),
        Err(RuntimeIdentityEvidenceError::ExactPackageMismatch)
    );
}

#[test]
fn phase36_runtime_identity_replays_observed_boot_b_and_exact_package() {
    // Arrange
    let documents = documents();

    // Act
    let result = documents
        .validate(true)
        .expect("eligible replay should validate");

    // Assert
    let ObservedRuntimeIdentityAdmission::Validated { identity } = result else {
        panic!("eligible replay should produce validated identity");
    };
    assert_eq!(
        identity.observation_source,
        RuntimeIdentityObservationSource::DeviceSessionReplay
    );
    assert!(identity.same_physical_device);
    assert_eq!(identity.application_elf_digest, APP_ELF);
    assert_eq!(identity.exact_package.firmware_elf_digest, APP_ELF);
    assert_eq!(identity.claim_fact_digest.len(), 64);
}

#[test]
fn phase36_runtime_identity_package_fields_alone_are_typed_insufficient() {
    // Arrange
    let package = PACKAGE;

    // Act
    let result = validate_observed_runtime_identity_documents(package, None, None, None, None)
        .expect("package-only input should classify");

    // Assert
    assert_eq!(
        result,
        ObservedRuntimeIdentityAdmission::Insufficient {
            component_insufficiencies: vec![ComponentInsufficiency::RuntimeIdentityObservation],
        }
    );
}

#[test]
fn phase36_runtime_identity_accepts_independently_complete_terminal_pair() {
    // Arrange
    let documents = documents();

    // Act
    let result = documents
        .validate(false)
        .expect("complete terminal pair should validate");

    // Assert
    let ObservedRuntimeIdentityAdmission::Validated { identity } = result else {
        panic!("terminal pair should produce validated identity");
    };
    assert_eq!(
        identity.observation_source,
        RuntimeIdentityObservationSource::TerminalResultProjection
    );
}

#[test]
fn phase36_runtime_identity_rejects_source_commit_drift() {
    // Arrange
    let replacement = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    // Act and Assert
    assert_package_drift_is_rejected("source_commit", replacement);
}

#[test]
fn phase36_runtime_identity_rejects_reference_commit_drift() {
    // Arrange
    let replacement = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    // Act and Assert
    assert_package_drift_is_rejected("reference_commit", replacement);
}

#[test]
fn phase36_runtime_identity_rejects_firmware_elf_drift() {
    // Arrange
    let replacement = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

    // Act and Assert
    assert_package_drift_is_rejected("firmware_elf_digest", replacement);
}

#[test]
fn phase36_runtime_identity_rejects_different_physical_device_observation() {
    // Arrange
    let mut documents = documents();
    documents.ledger = documents.ledger.replacen(
        "\"physical_match\":\"unique_same\"",
        "\"physical_match\":\"unique_different\"",
        1,
    );

    // Act
    let result = documents.validate(true);

    // Assert
    assert_eq!(result, Err(RuntimeIdentityEvidenceError::ReplayMismatch));
}

#[test]
fn phase36_runtime_identity_rejects_boot_session_not_advanced() {
    // Arrange
    let mut documents = documents();
    documents.private_result = mutate_json(&documents.private_result, |value| {
        value["boot_b"]["boot_session"] = Value::String("boot-a".to_owned());
    });

    // Act
    let result = documents.validate(false);

    // Assert
    assert_eq!(
        result,
        Err(RuntimeIdentityEvidenceError::BootSessionMismatch)
    );
}

#[test]
fn phase36_runtime_identity_rejects_public_private_disagreement() {
    // Arrange
    let mut documents = documents();
    documents.public_projection = mutate_json(&documents.public_projection, |value| {
        value["same_physical_device"] = Value::Bool(false);
    });

    // Act
    let result = documents.validate(true);

    // Assert
    assert_eq!(result, Err(RuntimeIdentityEvidenceError::ReplayMismatch));
}

#[test]
fn phase36_runtime_identity_rejects_missing_ledger_step() {
    // Arrange
    let mut documents = documents();
    documents.ledger = documents
        .ledger
        .lines()
        .filter(|line| !line.contains("\"event\":\"restart_response_received\""))
        .collect::<Vec<_>>()
        .join("\n");

    // Act
    let result = documents.validate(true);

    // Assert
    assert_eq!(result, Err(RuntimeIdentityEvidenceError::MissingLedgerStep));
}

#[test]
fn phase36_runtime_identity_errors_never_render_protected_values() {
    // Arrange
    let mut documents = documents();
    documents.private_result.push_str(PROTECTED_CANARY);

    // Act
    let error = documents
        .validate(true)
        .expect_err("malformed protected result should fail");
    let rendered = format!("{error:?} {error}");

    // Assert
    assert!(!rendered.contains(PROTECTED_CANARY));
}
